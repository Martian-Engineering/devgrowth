use crate::error::AppError;
use crate::job_queue::Job;
use crate::AppState;
use actix_web::{web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use log::{error, info};
use octocrab::Octocrab;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::postgres::PgPool;

#[derive(Debug, Deserialize, Serialize)]
pub struct Repository {
    pub id: i32,
    pub name: String,
    pub owner: String,
    pub indexed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NewRepository {
    pub name: String,
    pub owner: String,
}

#[derive(Serialize)]
pub struct RepositoryMetadata {
    id: i32,
    owner: String,
    name: String,
    commit_count: i64,
    latest_commit_date: Option<DateTime<Utc>>,
    latest_commit_author: Option<String>,
    indexed_at: Option<DateTime<Utc>>,
    github_url: String,
}

pub async fn get_repository_metadata(
    state: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (owner, name) = path.into_inner();

    match fetch_repository_metadata(&state.db_pool, &owner, &name).await {
        Ok(Some(metadata)) => HttpResponse::Ok().json(metadata),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Repository not found"
        })),
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "An error occurred while fetching repository metadata"
        })),
    }
}

async fn fetch_repository_metadata(
    pool: &PgPool,
    owner: &str,
    name: &str,
) -> Result<Option<RepositoryMetadata>, sqlx::Error> {
    let result = sqlx::query_as!(
        RepositoryMetadata,
        r#"
        SELECT
            r.repository_id as id,
            r.owner,
            r.name,
            COUNT(c.commit_id) as "commit_count!",
            MAX(c.date) as "latest_commit_date?",
            (SELECT author FROM commit WHERE repository_id = r.repository_id ORDER BY date DESC LIMIT 1) as "latest_commit_author?",
            r.indexed_at,
            CONCAT('https://github.com/', r.owner, '/', r.name) as "github_url!"
        FROM
            repository r
        LEFT JOIN
            commit c ON r.repository_id = c.repository_id
        WHERE
            r.owner = $1 AND r.name = $2
        GROUP BY
            r.repository_id
        "#,
        owner,
        name
    )
    .fetch_optional(pool)
    .await?;

    Ok(result)
}

pub async fn repository_exists(
    octocrab: &Octocrab,
    repo_owner: &str,
    repo_name: &str,
) -> Result<bool, AppError> {
    match octocrab.repos(repo_owner, repo_name).get().await {
        Ok(_) => Ok(true),
        Err(octocrab::Error::GitHub { source, .. }) if source.message == "Not Found" => Ok(false),
        Err(e) => Err(AppError::from(e)),
    }
}

pub async fn create_repository(
    state: web::Data<AppState>,
    new_repo: web::Json<NewRepository>,
) -> impl Responder {
    let new_repo = new_repo.into_inner();
    match repository_exists(&state.octocrab, &new_repo.owner, &new_repo.name).await {
        Ok(true) => match write_repository(&state.db_pool, new_repo).await {
            Ok(repo) => {
                let job = Job {
                    repository_id: repo.id,
                    owner: repo.owner.clone(),
                    name: repo.name.clone(),
                };
                state.job_queue.push(job).await;

                HttpResponse::Created().json(repo)
            }
            Err(e) => {
                if let Some(db_err) = e.as_database_error() {
                    if db_err
                        .constraint()
                        .map_or(false, |c| c == "repository_name_owner_key")
                    {
                        return HttpResponse::Conflict()
                            .body("Repository already exists in the database");
                    }
                }
                error!("Failed to create repository in database: {:?}", e);
                HttpResponse::InternalServerError().finish()
            }
        },
        Ok(false) => {
            error!("Repository does not exist on GitHub");
            HttpResponse::BadRequest().body("Repository does not exist on GitHub")
        }
        Err(e) => {
            error!("Failed to check repository existence: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

async fn write_repository(
    pool: &PgPool,
    new_repo: NewRepository,
) -> Result<Repository, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        INSERT INTO repository (name, owner)
        VALUES ($1, $2)
        RETURNING repository_id, name, owner, indexed_at, created_at, updated_at
        "#,
        new_repo.name,
        new_repo.owner
    )
    .fetch_one(pool)
    .await?;

    Ok(Repository {
        id: row.repository_id,
        name: row.name,
        owner: row.owner,
        indexed_at: row.indexed_at,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

pub async fn sync_repository(
    state: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (owner, name) = path.into_inner();

    // Check if the repository exists in our database
    match get_repository_id(&state.db_pool, &owner, &name).await {
        Ok(Some(repository_id)) => {
            let job = Job {
                repository_id,
                owner: owner.clone(),
                name: name.clone(),
            };
            state.job_queue.push(job).await;
            info!("Queued sync job for repository: {}/{}", owner, name);
            HttpResponse::Accepted().json(json!({
                "message": "Repository sync job queued",
                "owner": owner,
                "name": name
            }))
        }
        Ok(None) => {
            error!("Repository {}/{} not found in database", owner, name);
            HttpResponse::NotFound().json(json!({
                "error": "Repository not found in database"
            }))
        }
        Err(e) => {
            error!("Database error while checking repository: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

async fn get_repository_id(
    pool: &PgPool,
    owner: &str,
    name: &str,
) -> Result<Option<i32>, sqlx::Error> {
    let result = sqlx::query!(
        "SELECT repository_id FROM repository WHERE owner = $1 AND name = $2",
        owner,
        name
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.map(|row| row.repository_id))
}
