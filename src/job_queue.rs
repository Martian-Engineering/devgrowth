use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Job {
    pub repository_id: i32,
    pub owner: String,
    pub name: String,
}

pub struct JobQueue {
    pub queue: Mutex<Vec<Job>>,
}

impl JobQueue {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            queue: Mutex::new(Vec::new()),
        })
    }

    pub async fn push(&self, job: Job) {
        self.queue.lock().await.push(job);
    }

    pub async fn pop(&self) -> Option<Job> {
        self.queue.lock().await.pop()
    }
}
