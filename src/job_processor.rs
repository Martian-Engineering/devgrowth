use crate::commit::fetch_and_persist_commits;
use crate::error::AppError;
use crate::job_queue::{Job, JobQueue};
use log::{error, info};
use octocrab::Octocrab;
use sqlx::postgres::PgPool;
use std::sync::Arc;

pub async fn process_jobs(queue: Arc<JobQueue>, octocrab: Arc<Octocrab>, pool: PgPool) {
    loop {
        if let Some(job) = queue.pop().await {
            info!("Processing job for repository: {}/{}", job.owner, job.name);
            let job_clone = job.clone();
            match process_single_job(job, octocrab.clone(), pool.clone()).await {
                Ok(_) => info!(
                    "Job completed successfully for repository: {}/{}",
                    job_clone.owner, job_clone.name
                ),
                Err(e) => {
                    error!(
                        "Job failed for repository: {}/{}: {:?}",
                        job_clone.owner, job_clone.name, e
                    );
                    // Re-queue the job
                    // This is a bad idea, as it can put the queue into an infinite loop
                    // queue.push(job_clone).await;
                }
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

async fn process_single_job(
    job: Job,
    octocrab: Arc<Octocrab>,
    pool: PgPool,
) -> Result<(), AppError> {
    fetch_and_persist_commits(&job, &octocrab, &pool).await
}
