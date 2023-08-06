use color_eyre::Result;
use deno_core::{JsRuntime, RuntimeOptions};
use lazy_static::lazy_static;
use std::sync::Arc;
use tokio::{net::TcpListener, sync::RwLock};

use crate::{
    handle_client,
    structs::{Queue, V8Request, V8Response},
};

lazy_static! {
    pub static ref JOB_QUEUE: Queue<V8Request, V8Response> = Queue::new();
    pub static ref WORKER_SCRIPT: Arc<RwLock<String>> = {
        let script = std::fs::read_to_string("main.js").unwrap();
        Arc::new(RwLock::new(script))
    };
}

pub fn worker_script_updater() -> Result<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));

        loop {
            interval.tick().await;

            let script = tokio::fs::read_to_string("main.js").await;
            if let Ok(script) = script {
                if script != *WORKER_SCRIPT.read().await {
                    println!("WORKER SCRIPT UPDATED");

                    let mut worker_script = WORKER_SCRIPT.write().await;
                    *worker_script = script;
                }
            }
        }
    });

    Ok(())
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

        let worker_script = WORKER_SCRIPT.read().await;
        let script = format!(
            r#"
                {worker_script}

                async function handler(req) {{
                    try {{
                        return await run(req);
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
