use color_eyre::Result;
use std::{collections::HashMap, sync::Arc};

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
    queue_tx: crossbeam_channel::Sender<WorkerRequest<S>>,
    queue_rx: crossbeam_channel::Receiver<WorkerRequest<S>>,

    pub returners: Arc<tokio::sync::RwLock<HashMap<u32, tokio::sync::mpsc::Sender<R>>>>,
}

impl<S, R> Queue<S, R>
where
    S: Send + Sync + 'static,
{
    pub fn new() -> Self {
        let (queue_tx, queue_rx) = crossbeam_channel::unbounded::<WorkerRequest<S>>();

        Self {
            queue_tx,
            queue_rx,
            returners: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    pub async fn enqueue(&self, value: S) -> Result<(u32, tokio::sync::mpsc::Receiver<R>)> {
        let channel = tokio::sync::mpsc::channel(1);

        let job_id = rand::random::<u32>();
        self.returners.write().await.insert(job_id, channel.0);
        self.queue_tx.send(WorkerRequest { job_id, value })?;

        Ok((job_id, channel.1))
    }

    pub async fn send_response(&self, job_id: u32, value: R) -> Result<()> {
        let tx = self
            .returners
            .write()
            .await
            .remove(&job_id)
            .ok_or_else(|| {
                color_eyre::eyre::eyre!("Failed to find returner for job_id {:?}", job_id)
            })?;

        tx.send(value).await.map_err(|_| {
            color_eyre::eyre::eyre!("Failed to send value to returner {:?}", job_id)
        })?;
        Ok(())
    }

    pub async fn remove_job(&self, job_id: u32) -> Result<()> {
        self.returners.write().await.remove(&job_id);
        Ok(())
    }

    pub fn get_rx(&self) -> crossbeam_channel::Receiver<WorkerRequest<S>> {
        self.queue_rx.clone()
    }
}
