use color_eyre::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use deno_core::op;
use deno_core::Extension;
use deno_core::JsRuntime;
use deno_core::Op;
use deno_core::RuntimeOptions;
use lazy_static::lazy_static;

mod console;

lazy_static! {
    pub static ref JOB_QUEUE: Queue<V8Request> = Queue::new();
    pub static ref CALLBACKS: Arc<RwLock<HashMap<u128, tokio::sync::mpsc::Sender<V8Response>>>> =
        Arc::new(RwLock::new(HashMap::new()));
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

    Ok(())
}

#[op]
async fn op_callback(response: V8Response) -> Result<(), deno_core::error::AnyError> {
    println!("Rust: op_callback {:?}", response);
    /*
    CALLBACKS
        .write()
        .await
        .as_mut()
        .unwrap()
        .send(response)
        .await
        .unwrap();
        */

    Ok(())
}

#[derive(serde::Deserialize, Debug)]
#[allow(dead_code)]
pub struct V8Response {
    pub block_connection: Option<bool>,
    pub hang_connection: Option<bool>,
    pub ip: Option<String>,
    pub no_delay: Option<bool>,

    pub cpu_time: Option<u64>,
}

#[derive(serde::Serialize, Debug)]
#[allow(dead_code)]
pub struct V8Request {
    pub ip: String,
    pub port: u16,
}

pub struct Queue<T> {
    values: Arc<RwLock<Vec<T>>>,
}

impl<T> Queue<T> {
    pub fn new() -> Self {
        Self {
            values: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn enqueue(&self, value: T) {
        self.values.write().await.push(value);
    }

    pub async fn dequeue(&self) -> Option<T> {
        let mut values = self.values.write().await;
        if values.is_empty() {
            return None;
        }

        Some(values.remove(0))
    }

    pub async fn has_values(&self) -> bool {
        let values = self.values.read().await;
        !values.is_empty()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let workers_count = 100;
    let mut workers = vec![];
    for i in 0..workers_count {
        workers.push(tokio::spawn(v8_worker(i)));
    }

    for i in 0..100000 {
        JOB_QUEUE
            .enqueue(V8Request {
                ip: "192.168.1.1".to_string(),
                port: 42069 + i as u16,
            })
            .await;
    }
    
    loop {
        let count = JOB_QUEUE.values.read().await.len();
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        if count == 0 {
            return Ok(());
        }
    }

    //futures::future::join_all(workers).await;

    /*
    let ext = Extension::builder("my_ext")
        .ops(vec![op_sum::DECL, op_sleep::DECL, op_callback::DECL])
        .build();
    let mut runtime = JsRuntime::new(RuntimeOptions {
        extensions: vec![ext],
        ..Default::default()
    });

    let start = std::time::Instant::now();

    let req = V8Request {
        ip: "192.158.1.69".to_string(),
        port: 42069,
    };

    //let mut channel = tokio::sync::mpsc::channel::<V8Response>(1);
    // *CALLBACKS.write().await = Some(channel.0);
    let req = deno_core::serde_json::to_string(&req).unwrap();
    let script = format!(
        r#"
        async function test(req) {{
            Deno.core.print(`DBG: ${{req.ip}} ${{req.port}}\n`);

            let val = Deno.core.ops.op_sum([1,2,3]);
            Deno.core.print(val + "\n");
            Deno.core.print("DBG: LOL\n");
            //await Deno.core.ops.op_sleep(1000);
            Deno.core.print("DBG: LOL2\n");
            return {{
                ip: req.ip,
                cpu_time: 321,
            }};
        }}

        test({}).then(async (res) => {{
            await Deno.core.ops.op_callback(res);
        }});
        "#,
        req
    );
    runtime.execute_script("main.js", script.into()).unwrap();

    while let Err(e) = runtime.run_event_loop(false).await {
        println!("Error: {}", e);
    }

    //let response = channel.1.recv().await.unwrap();
    //println!("Response: {:?}", response);

    println!("Script took {}", start.elapsed().as_micros());
    */

    Ok(())
}

async fn v8_worker(worker_id: u128) -> Result<()> {
    loop {
        if JOB_QUEUE.has_values().await {
            if let Some(job) = JOB_QUEUE.dequeue().await {
                //println!("Job ({}): {:?}", worker_id, job);
                tokio::time::sleep(Duration::from_micros(500)).await;
            }
        }
    }
}
