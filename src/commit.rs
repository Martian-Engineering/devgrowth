use crate::error::AppError;
use crate::job_queue::Job;
use backoff::{Error as BackoffError, ExponentialBackoff};
use chrono::{DateTime, Utc};
use log::info;
use octocrab::models::repos::RepoCommit;
use octocrab::Octocrab;
use octocrab::Page;
use sqlx::PgPool;

pub async fn fetch_and_persist_commits(
    job: &Job,
    octocrab: &Octocrab,
    pool: &PgPool,
) -> Result<(), AppError> {
    let latest_commit_date = get_latest_commit_date(pool, job.repository_id).await?;
    info!(
        "Latest commit date for repository {}/{}: {:?}",
        job.owner, job.name, latest_commit_date
    );

    let mut page: u32 = 1;
    let mut new_commits_found = false;

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

            if let Some(latest_date) = latest_commit_date {
                if date <= latest_date {
                    continue;
                }
            }

            new_commits_found = true;

            sqlx::query!(
                "INSERT INTO commit (repository_id, sha, message, author,
                date) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (repository_id, sha)
                DO NOTHING",
                job.repository_id,
                commit.sha,
                commit.commit.message,
                author,
                date
            )
            .execute(pool)
            .await?;
        }

        if commits.next.is_none() || !new_commits_found {
            break;
        }
        page += 1;
    }

    if !new_commits_found {
        info!(
            "No new commits found for repository: {}/{}",
            job.owner, job.name
        );
    }

    Ok(())
}

async fn get_latest_commit_date(
    pool: &PgPool,
    repository_id: i32,
) -> Result<Option<DateTime<Utc>>, AppError> {
    let result = sqlx::query!(
        "SELECT MAX(date) as latest_date FROM commit WHERE repository_id = $1",
        repository_id
    )
    .fetch_one(pool)
    .await?;

    Ok(result.latest_date)
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
