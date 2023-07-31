use color_eyre::Result;
use rand::Rng;
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
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

#[derive(serde::Serialize, Debug)]
pub struct WorkerRequest<T> {
    job_id: u32,
    value: T,
}

pub struct Queue<S, R> {
    senders: Arc<std::sync::RwLock<Vec<tokio::sync::mpsc::Sender<WorkerRequest<S>>>>>,
    returners: Arc<RwLock<HashMap<u32, tokio::sync::mpsc::Sender<R>>>>,

    max: AtomicUsize,
    next: AtomicUsize,
}

impl<S, R> Queue<S, R> {
    pub fn new() -> Self {
        Self {
            senders: Arc::new(std::sync::RwLock::new(Vec::new())),
            returners: Arc::new(RwLock::new(HashMap::new())),

            max: AtomicUsize::new(0),
            next: AtomicUsize::new(0),
        }
    }

    pub fn add_worker(&self) -> tokio::sync::mpsc::Receiver<WorkerRequest<S>> {
        let (tx, rx) = tokio::sync::mpsc::channel::<WorkerRequest<S>>(100);
        self.senders.write().unwrap().push(tx);
        self.max.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        rx
    }

    pub async fn enqueue(&self, value: S) -> Result<tokio::sync::mpsc::Receiver<R>> {
        let max = self.max.load(std::sync::atomic::Ordering::SeqCst);
        if max == 0 {
            color_eyre::eyre::bail!("No workers available");
        }

        let (tx, rx) = tokio::sync::mpsc::channel::<R>(1);
        let returner_id = rand::thread_rng().gen::<u32>();
        let w_req = WorkerRequest {
            job_id: returner_id,
            value,
        };

        {
            self.returners.write().await.insert(returner_id, tx);
        }

        let next = self.next.fetch_add(1, std::sync::atomic::Ordering::SeqCst) % max;
        let senders = self.senders.read().unwrap();
        senders[next].send(w_req).await.map_err(|_| {
            color_eyre::eyre::eyre!("Failed to send value to worker {:?}", self.next)
        })?;

        Ok(rx)
    }

    pub async fn send_response(&self, job_id: u32, value: R) -> Result<()> {
        let mut returners = self.returners.write().await;
        let tx = returners.remove(&job_id).ok_or_else(|| {
            color_eyre::eyre::eyre!("Failed to find returner for job_id {:?}", job_id)
        })?;

        tx.send(value).await.map_err(|_| {
            color_eyre::eyre::eyre!("Failed to send value to returner {:?}", job_id)
        })?;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let workers_count = 10;
    let mut workers = vec![];
    for i in 0..workers_count {
        workers.push(tokio::spawn(async move {
            _ = v8_worker(i);
        }));
    }

    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
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
    futures::future::join_all(workers).await;
    Ok(())
}

fn v8_worker(worker_id: u64) -> Result<()> {
    let mut rx = JOB_QUEUE.add_worker();

    let ext = Extension::builder("my_ext")
        .ops(vec![op_sum::DECL, op_sleep::DECL, op_callback::DECL])
        .build();
    let mut runtime = JsRuntime::new(RuntimeOptions {
        extensions: vec![ext],
        ..Default::default()
    });

    loop {
        if let Ok(job) = rx.try_recv() {
            println!("Job ({}): {:?}", worker_id, job);

            let req = deno_core::serde_json::to_string(&job.value).unwrap();
            let script = format!(
                r#"
        async function test(req) {{
            Deno.core.print(`DBG: ${{req.ip}} ${{req.port}}\n`);

            let val = Deno.core.ops.op_sum([1,2,3]);
            Deno.core.print(val + "\n");
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

            /*
            while let Err(e) = runtime.run_event_loop(false).await {
                println!("Error: {}", e);
            }
            */

            // LOAD SIMULATION
            //tokio::time::sleep(tokio::time::Duration::from_millis(3)).await;
        }

        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}
