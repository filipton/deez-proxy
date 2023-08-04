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

pub fn v8_worker(rt: &tokio::runtime::Runtime, worker_id: usize) -> Result<()> {
    let _guard = rt.enter();
    let rx = JOB_QUEUE.get_rx();

    let extensions = crate::extensions::get_all_extensions();
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
                    //await Deno.core.ops.op_test_console();

                    let res = await fetch("http://vps.filipton.space");
                    console.warn("RES: " + await res.text());

                    return {{
                        ip: "localhost:80",
                    }};
                    }} catch (e) {{
                        console.error(e.stack);
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
