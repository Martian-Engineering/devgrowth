use crate::auth_utils::get_account_id;
use crate::error::AppError;
use crate::github::get_github_client;
use crate::AppState;
use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
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
        ON CONFLICT (owner_id, is_default) DO NOTHING
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
    starred_repositories: Vec<StarredRepo>,
    repo_collections: HashMap<i32, Vec<i32>>,
}

#[derive(Serialize, Deserialize)]
pub struct StarredRepo {
    id: u64,
    name: String,
    owner: String,
    html_url: String,
    description: Option<String>,
    stargazers_count: Option<u32>,
}

async fn get_repo_collections(
    pool: &PgPool,
    req: HttpRequest,
) -> Result<HashMap<i32, Vec<i32>>, AppError> {
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
    .fetch_all(pool)
    .await?;

    let result: HashMap<i32, Vec<i32>> = repo_collections
        .into_iter()
        .map(|row| (row.repository_id, row.collection_ids.unwrap_or_default()))
        .collect();

    return Ok(result);
}

pub async fn get_profile_data(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
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

    // Fetch the user's starred repositories
    let starred_repos = github_client
        .current()
        .list_repos_starred_by_authenticated_user()
        .per_page(100) // Adjust this number as needed
        .send()
        .await?;

    let starred_repositories: Vec<StarredRepo> = starred_repos
        .items
        .into_iter()
        .map(|repo| StarredRepo {
            id: repo.id.0,
            name: repo.name,
            owner: repo.owner.map(|owner| owner.login).unwrap_or_default(),
            html_url: repo.html_url.map(|url| url.to_string()).unwrap_or_default(),
            description: repo.description,
            stargazers_count: repo.stargazers_count,
        })
        .collect();

    let repo_collections: HashMap<i32, Vec<i32>> =
        get_repo_collections(&state.db_pool, req).await?;

    let profile_data = ProfileData {
        github_id: user.id.to_string(),
        name: "Josh Lehman".to_string(), // user.name.unwrap_or_else(|| user.login.clone()),
        email: user.email.unwrap_or_default(),
        starred_repositories,
        repo_collections,
    };

    Ok(HttpResponse::Ok().json(profile_data))
}
