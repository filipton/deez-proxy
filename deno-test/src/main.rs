use crate::structs::Queue;
use crate::structs::V8Request;
use crate::structs::V8Response;
use color_eyre::Result;
use deno_core::JsRuntime;
use deno_core::RuntimeOptions;
use lazy_static::lazy_static;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::net::TcpStream;

mod extensions;
mod structs;
mod utils;

lazy_static! {
    pub static ref JOB_QUEUE: Queue<V8Request, V8Response> = Queue::new();
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
    let rx = JOB_QUEUE.get_rx();

    let extensions = extensions::get_all_extensions();
    let mut runtime = JsRuntime::new(RuntimeOptions {
        extensions,
        ..Default::default()
    });

    while let Ok(job) = rx.recv() {
        println!("Job ({}): {:?}", worker_id, job);

        let req = deno_core::serde_json::to_string(&job.value).unwrap();
        let script = format!(
            r#"
                async function test(req) {{
                    try {{
                    console.log("DSDSADSADSADAS");
                    Deno.core.ops.op_test_console();

                    //let res = await fetch("https://1.1.1.1");
                    console.log("log");
                    console.info("info");
                    console.warn("warn", {{dsa: "dsvcx"}});
                    console.debug("debug", 1234);
                    console.error("error");

                    return {{
                        ip: "localhost:80",
                    }};
                    }} catch (e) {{
                        console.log("Error: " + e);
                        return {{
                            block_connection: true,
                        }};
                    }}
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
                    let mut job_id = 0;
                    if let Err(_e) = handle_client(socket, port, addr, &mut job_id).await {
                        //println!("Handle Client Error: {}", e);
                        _ = JOB_QUEUE.remove_job(job_id).await;
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
    let (res_job_id, mut rx) = JOB_QUEUE
        .enqueue(V8Request {
            ip: addr.ip().to_string(),
            port,
        })
        .await?;

    *job_id = res_job_id;
    let res = rx.recv().await.unwrap();

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
