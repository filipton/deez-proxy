use std::net::SocketAddr;

use color_eyre::Result;
use deno_core::op;
use deno_core::Extension;
use deno_core::JsRuntimeForSnapshot;
use deno_core::Op;
use deno_core::RuntimeOptions;
use lazy_static::lazy_static;
use tokio::net::TcpListener;
use tokio::net::TcpStream;

use crate::structs::Queue;
use crate::structs::V8Request;
use crate::structs::V8Response;

mod console;
mod structs;
mod utils;

lazy_static! {
    pub static ref JOB_QUEUE: Queue<V8Request, V8Response> = Queue::new();
}

#[op]
fn op_sum(nums: Vec<f64>) -> Result<f64, deno_core::error::AnyError> {
    println!("Rust: op_sum {:?}", nums);
    // Sum inputs
    let sum = nums.iter().fold(0.0, |a, v| a + v);
    // return as a Result<f64, AnyError>
    Ok(sum)
}

#[op]
async fn op_sleep(duration: u64) -> Result<(), deno_core::error::AnyError> {
    println!("Sleeping for {}ms", duration);
    tokio::time::sleep(tokio::time::Duration::from_millis(duration)).await;
    println!("Done sleeping");

    Ok(())
}

#[op]
async fn op_callback(job_id: u32, response: V8Response) -> Result<(), deno_core::error::AnyError> {
    println!("Rust: {} op_callback {:?}", job_id, response);
    JOB_QUEUE
        .send_response(job_id, response)
        .await
        .expect("Failed to send response");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let workers_count = 1usize;
    let mut workers = vec![];
    for i in 0..workers_count {
        workers.push(std::thread::spawn(move || loop {
            let res = v8_worker(i);
            if let Err(e) = res {
                println!("Worker {} failed: {:?}", i, e);
            }

            //JOB_QUEUE.remove_worker(i).await;
        }));
    }

    let args = std::env::args().collect::<Vec<String>>();
    let ports = utils::parse_ports(args.get(1).unwrap_or(&String::from("7070")))?;
    let mut tasks = vec![];

    for port in ports {
        tasks.push(tokio::spawn(port_worker("0.0.0.0", port)));
    }
    futures::future::try_join_all(tasks).await?;

    /*
    loop {
        tokio::time::sleep(tokio::time::Duration::from_micros(10)).await;
        let start = std::time::Instant::now();
        let mut res = JOB_QUEUE
            .enqueue(V8Request {
                ip: format!("192.168.1.1:80"),
                port: 42069,
            })
            .await?;

        let res = res.recv().await.unwrap();
        println!("Got response: {:?}", res);
        println!("Script took {}", start.elapsed().as_micros());
    }
    */

    Ok(())
}

fn v8_worker(worker_id: usize) -> Result<()> {
    println!("Starting worker {}", worker_id);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let _guard = rt.enter();

    let mut rx = rt.block_on(JOB_QUEUE.add_worker());

    let ext = Extension::builder("my_ext")
        .ops(vec![op_sum::DECL, op_sleep::DECL, op_callback::DECL])
        .build();
    let mut runtime = JsRuntimeForSnapshot::new(
        RuntimeOptions {
            extensions: vec![ext],
            ..Default::default()
        },
        Default::default(),
    );

    while let Some(job) = rx.blocking_recv() {
        println!("Job ({}): {:?}", worker_id, job);

        let req = deno_core::serde_json::to_string(&job.value).unwrap();
        let script = format!(
            r#"
        async function test(req) {{
            Deno.core.print(`DBG: ${{req.ip}} ${{req.port}}\n`);

            let val = Deno.core.ops.op_sum([1,2,3]);
            Deno.core.print(val + "\n");
            //await Deno.core.ops.op_sleep(1000);
            Deno.core.print("DBG: LOL\n");

            return {{
                ip: req.ip,
                cpu_time: 321,
            }};
        }}

        test({}).then(async (res) => {{
            await Deno.core.ops.op_callback({}, res);
        }});
        "#,
            req, job.job_id
        );

        runtime.execute_script("main.js", script.into()).unwrap();
        rt.block_on(runtime.run_event_loop(false)).unwrap();
    }

    Ok(())
}

async fn port_worker(bind_ip: &str, port: u16) -> Result<()> {
    let addr = format!("{}:{}", bind_ip, port);
    let listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);

    loop {
        let socket_res = listener.accept().await;

        match socket_res {
            Ok((socket, addr)) => {
                tokio::spawn(async move {
                    if let Err(e) = handle_client(socket, port, addr).await {
                        println!("Handle Client Error: {}", e);
                    }
                });
            }
            Err(e) => {
                println!("Bind Socket Error: {}", e);
            }
        }
    }
}

async fn handle_client(mut socket: TcpStream, port: u16, addr: SocketAddr) -> Result<()> {
    let mut res = JOB_QUEUE
        .enqueue(V8Request {
            ip: format!("192.168.1.1:80"),
            port: 42069,
        })
        .await?;

    let res = res.recv().await.unwrap();

    if res.block_connection.unwrap_or(false) {
        return Ok(());
    } else if res.hang_connection.unwrap_or(false) {
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        return Ok(());
    }

    let mut out_stream = TcpStream::connect(
        res.ip
            .ok_or(color_eyre::eyre::eyre!("Ip is null in V8Response"))?,
    )
    .await?;
    out_stream.set_nodelay(res.no_delay.unwrap_or(false))?;

    tokio::io::copy_bidirectional(&mut socket, &mut out_stream).await?;

    Ok(())
}
