use crate::auth_utils::get_account_id;
use crate::error::AppError;
use crate::github::get_github_client;
use crate::AppState;
use actix_web::{web, HttpRequest, HttpResponse};
use serde::Serialize;
use sqlx::{PgPool, Postgres, Transaction};
use std::collections::HashMap;

async fn create_default_collection(
    tx: &mut Transaction<'_, Postgres>,
    owner_id: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO collection (owner_id, name, description, is_default)
        VALUES ($1, 'Default', 'Default collection', true)
        ON CONFLICT (owner_id) WHERE is_default = true
        DO NOTHING
        "#,
        owner_id
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub async fn upsert_account(
    pool: &PgPool,
    github_id: &str,
    email: Option<&str>,
) -> Result<i32, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let result = sqlx::query!(
        r#"
        INSERT INTO account (github_id, email, last_login)
        VALUES ($1, $2, NOW())
        ON CONFLICT (github_id)
        DO UPDATE SET
            last_login = NOW(),
            email = COALESCE(EXCLUDED.email, account.email)
        RETURNING account_id
        "#,
        github_id,
        email
    )
    .fetch_one(&mut *tx)
    .await?;

    // Create default collection if it doesn't exist
    create_default_collection(&mut tx, result.account_id).await?;

    tx.commit().await?;

    Ok(result.account_id)
}

#[derive(Serialize)]
pub struct ProfileData {
    github_id: String,
    name: String,
    email: String,
}

pub async fn get_repo_collections(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let account_id = get_account_id(&req)?;

    let repo_collections = sqlx::query!(
        r#"
        SELECT cr.repository_id, ARRAY_AGG(cr.collection_id) as collection_ids
        FROM collection_repository cr
        JOIN collection c ON cr.collection_id = c.collection_id
        WHERE c.owner_id = $1
        GROUP BY cr.repository_id
        "#,
        account_id
    )
    .fetch_all(&state.db_pool)
    .await?;

    let result: HashMap<i32, Vec<i32>> = repo_collections
        .into_iter()
        .map(|row| (row.repository_id, row.collection_ids.unwrap_or_default()))
        .collect();

    Ok(HttpResponse::Ok().json(result))
}

pub async fn get_profile_data(req: HttpRequest) -> Result<HttpResponse, AppError> {
    let github_client = get_github_client(&req)?;

    // Fetch the authenticated user's information
    let user = match github_client.current().user().await {
        Ok(user) => user,
        Err(e) => {
            log::error!("Failed to fetch user data: {:?}", e);
            return Err(AppError::Unauthorized(
                "Failed to fetch user data".to_string(),
            ));
        }
    };

    let profile_data = ProfileData {
        github_id: user.id.to_string(),
        name: user.login.to_string(),
        email: user.email.unwrap_or_default(),
    };

    Ok(HttpResponse::Ok().json(profile_data))
}
