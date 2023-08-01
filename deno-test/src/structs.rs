use color_eyre::Result;
use rand::Rng;
use std::{
    collections::HashMap,
    sync::{atomic::AtomicUsize, Arc},
};

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
    pub job_id: u32,
    pub value: T,
}

pub struct Queue<S, R> {
    senders: Arc<tokio::sync::RwLock<Vec<tokio::sync::mpsc::Sender<WorkerRequest<S>>>>>,
    returners: Arc<tokio::sync::RwLock<HashMap<u32, tokio::sync::mpsc::Sender<R>>>>,

    max: AtomicUsize,
    next: AtomicUsize,
}

impl<S, R> Queue<S, R> {
    pub fn new() -> Self {
        Self {
            senders: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            returners: Arc::new(tokio::sync::RwLock::new(HashMap::new())),

            max: AtomicUsize::new(0),
            next: AtomicUsize::new(0),
        }
    }

    pub async fn add_worker(&self) -> tokio::sync::mpsc::Receiver<WorkerRequest<S>> {
        let (tx, rx) = tokio::sync::mpsc::channel::<WorkerRequest<S>>(100);
        self.senders.write().await.push(tx);
        self.max.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        rx
    }

    pub async fn remove_worker(&self, id: usize) {
        self.max.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        self.senders.write().await.remove(id);
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
        let senders = self.senders.read().await;
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
