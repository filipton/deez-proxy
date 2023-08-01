use crate::structs::Queue;
use crate::structs::V8Request;
use crate::structs::V8Response;
use color_eyre::Result;
use deno_core::op;
use deno_core::Extension;
use deno_core::JsRuntimeForSnapshot;
use deno_core::Op;
use deno_core::RuntimeOptions;
use lazy_static::lazy_static;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::net::TcpStream;

mod console;
mod structs;
mod utils;

lazy_static! {
    pub static ref JOB_QUEUE: Queue<V8Request, V8Response> = Queue::new();
}

#[op]
fn op_sum(nums: Vec<f64>) -> Result<f64, deno_core::error::AnyError> {
    println!("Rust: op_sum {:?}", nums);
    let sum = nums.iter().fold(0.0, |a, v| a + v);
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
    /*
    JOB_QUEUE
        .send_response(job_id, response)
        .await
        .expect("Failed to send response");
        */

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let workers_count = 10usize;
    let mut workers = vec![];
    for i in 0..workers_count {
        workers.push(std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            loop {
                let res = v8_worker(&rt, i);
                if let Err(e) = res {
                    println!("Worker {} failed: {:?}", i, e);
                }

                // Remove worker from queue (its rare that this will be called)
                rt.block_on(JOB_QUEUE.remove_worker(i));
            }
        }));
    }

    let args = std::env::args().collect::<Vec<String>>();
    let ports = utils::parse_ports(args.get(1).unwrap_or(&String::from("7070")))?;
    let mut tasks = vec![];

    for port in ports {
        tasks.push(tokio::spawn(port_worker("0.0.0.0", port)));
    }
    futures::future::try_join_all(tasks).await?;

    Ok(())
}

fn v8_worker(rt: &tokio::runtime::Runtime, worker_id: usize) -> Result<()> {
    let _guard = rt.enter();
    //let mut rx = rt.block_on(JOB_QUEUE.add_worker(worker_id));

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

    let rx = JOB_QUEUE.queue_rx.clone();
    while let Ok(job) = rx.recv() {
        //println!("Job ({}): {:?}", worker_id, job);

        let req = deno_core::serde_json::to_string(&job).unwrap();
        let script = format!(
            r#"
        async function test(req) {{
            //Deno.core.print(`DBG: ${{req.ip}} ${{req.port}}\n`);

            //let val = Deno.core.ops.op_sum([1,2,3]);
            //Deno.core.print(val + "\n");
            //await Deno.core.ops.op_sleep(1000);
            //Deno.core.print("DBG: LOL\n");

            return {{
                ip: "localhost:80",
            }};
        }}

        test({}).then(async (res) => {{
            await Deno.core.ops.op_callback({}, res);
        }});
        "#,
            req, worker_id
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
                    let mut job_id = 0;
                    if let Err(_) = handle_client(socket, port, addr, &mut job_id).await {
                        //println!("Handle Client Error ({}): {}", job_id, e);
                        JOB_QUEUE.remove_job(job_id).await;
                    }
                });
            }
            Err(e) => {
                println!("Bind Socket Error: {}", e);
            }
        }
    }
}

async fn handle_client(
    mut socket: TcpStream,
    port: u16,
    addr: SocketAddr,
    job_id: &mut u32,
) -> Result<()> {
    _ = JOB_QUEUE
        .enqueue(V8Request {
            ip: addr.ip().to_string(),
            port,
        })
        .await;

    /*
    *job_id = res_job_id;
    let res = res.recv().await.unwrap();
    */

    let res = V8Response {
        ip: Some("localhost:80".to_string()),
        block_connection: None,
        hang_connection: None,
        no_delay: None,
        cpu_time: None,
    };
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
