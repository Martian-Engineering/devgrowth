use chrono::{DateTime, Utc};
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
