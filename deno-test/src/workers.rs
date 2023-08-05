use color_eyre::Result;
use deno_core::{JsRuntime, RuntimeOptions};
use lazy_static::lazy_static;
use tokio::net::TcpListener;

use crate::{
    handle_client,
    structs::{Queue, V8Request, V8Response},
};

lazy_static! {
    pub static ref JOB_QUEUE: Queue<V8Request, V8Response> = Queue::new();
}

pub async fn v8_worker(worker_id: usize) -> Result<()> {
    let rx = JOB_QUEUE.get_rx();

    let extensions = crate::extensions::get_all_extensions();
    let mut runtime = JsRuntime::new(RuntimeOptions {
        extensions,
        ..Default::default()
    });

    while let Ok(job) = rx.recv() {
        let req = deno_core::serde_json::to_string(&job.value)?;
        let job_id = job.job_id;

        let script = format!(
            r#"
                async function handler(req) {{
                    try {{
                        //console.log("DSDSADSADSADAS");
                        //await Deno.core.ops.op_test_console();

                        //let res = await fetch("https://echo.filipton.space/r7709629271299675447");
                        //console.warn(await res.text());

                        return {{
                            ip: "localhost:80",
                            no_delay: true,
                        }}
                    }} catch (e) {{
                        console.error("| ERROR | Worker {worker_id} | Job {job_id} |");
                        console.error(e.stack);
                        return {{
                            block_connection: true,
                        }}
                    }}
                }}

                handler({req}).then(async (res) => {{
                    await Deno.core.ops.op_callback({job_id}, res);
                }});
                "#,
        );

        runtime
            .execute_script("main.js", script.into())
            .map_err(|e| color_eyre::eyre::eyre!("Runtime Error (Execute Script): {}", e))?;

        runtime
            .run_event_loop(false)
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Runtime Error (Run Event Loop): {}", e))?;
    }

    Ok(())
}

pub async fn port_listener(bind_ip: &str, port: u16) -> Result<()> {
    let addr = format!("{}:{}", bind_ip, port);
    let listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);

    loop {
        let socket_res = listener.accept().await;

        match socket_res {
            Ok((socket, addr)) => {
                tokio::spawn(async move {
                    let (job_id, mut rx) = JOB_QUEUE
                        .enqueue(V8Request {
                            ip: addr.ip().to_string(),
                            port,
                        })
                        .await?;
                    let res = rx.recv().await.unwrap();

                    if let Err(_e) = handle_client(socket, res).await {
                        //println!("Handle Client Error: {}", _e);
                        JOB_QUEUE.remove_job(job_id).await?;
                    }

                    return Ok::<(), color_eyre::eyre::Error>(());
                });
            }
            Err(e) => {
                println!("Bind Socket Error: {}", e);
            }
        }
    }
}
