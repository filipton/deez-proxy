use color_eyre::Result;
use deno_core::op;
use deno_core::Extension;
use deno_core::JsRuntimeForSnapshot;
use deno_core::Op;
use deno_core::RuntimeOptions;
use lazy_static::lazy_static;

use crate::structs::Queue;
use crate::structs::V8Request;
use crate::structs::V8Response;

mod console;
mod structs;

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

            JOB_QUEUE.remove_worker(i);
        }));
    }

    loop {
        tokio::time::sleep(tokio::time::Duration::from_micros(10)).await;
        let start = std::time::Instant::now();
        let mut res = JOB_QUEUE
            .enqueue(V8Request {
                ip: format!("192.168.1.1"),
                port: 42069,
            })
            .await?;

        let res = res.recv().await.unwrap();
        println!("Got response: {:?}", res);
        println!("Script took {}", start.elapsed().as_micros());
    }
}

fn v8_worker(worker_id: usize) -> Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let _guard = rt.enter();

    let mut rx = JOB_QUEUE.add_worker();

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
