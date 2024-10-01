use crate::github::repository_exists;
use crate::job_queue::Job;
use crate::AppState;
use actix_web::{web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use log::error;
use serde::{Deserialize, Serialize};
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
