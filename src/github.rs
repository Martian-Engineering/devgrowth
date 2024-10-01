use crate::error::AppError;
use crate::job_queue::Job;
use backoff::{Error as BackoffError, ExponentialBackoff};
use octocrab::models::repos::RepoCommit;
use octocrab::Octocrab;
use octocrab::Page;
use sqlx::PgPool;

pub async fn fetch_and_persist_commits(
    job: &Job,
    octocrab: &Octocrab,
    pool: &PgPool,
) -> Result<(), AppError> {
    let mut page: u32 = 1;
    loop {
        let commits = fetch_commits_with_backoff(octocrab, &job.owner, &job.name, page).await?;

        for commit in commits.items {
            let author = commit
                .commit
                .author
                .as_ref()
                .map(|a| a.name.clone())
                .unwrap_or_else(|| "Unknown".to_string());
            let date = commit
                .commit
                .author
                .and_then(|a| a.date)
                .unwrap_or_else(|| chrono::Utc::now());
            sqlx::query!(
                "INSERT INTO commit (repository_id, sha, message, author, date) VALUES ($1, $2, $3, $4, $5)",
                job.repository_id,
                commit.sha,
                commit.commit.message,
                author,
                date
            )
            .execute(pool)
            .await?;
        }

        if commits.next.is_none() {
            break;
        }
        page += 1;
    }

    Ok(())
}

async fn fetch_commits_with_backoff(
    octocrab: &Octocrab,
    owner: &str,
    repo: &str,
    page: u32,
) -> Result<Page<RepoCommit>, AppError> {
    let operation = || async {
        octocrab
            .repos(owner, repo)
            .list_commits()
            .page(page)
            .per_page(100)
            .send()
            .await
            .map_err(|e| {
                if let octocrab::Error::GitHub { source, .. } = &e {
                    if source.message.contains("API rate limit exceeded") {
                        BackoffError::transient(e)
                    } else {
                        BackoffError::permanent(e)
                    }
                } else {
                    BackoffError::permanent(e)
                }
            })
    };

    backoff::future::retry(ExponentialBackoff::default(), operation)
        .await
        .map_err(AppError::from)
}

pub async fn repository_exists(
    octocrab: &Octocrab,
    repo_owner: &str,
    repo_name: &str,
) -> Result<bool, octocrab::Error> {
    match octocrab.repos(repo_owner, repo_name).get().await {
        Ok(_) => Ok(true),
        Err(octocrab::Error::GitHub { source, .. }) if source.message == "Not Found" => Ok(false),
        Err(e) => Err(e),
    }
}
