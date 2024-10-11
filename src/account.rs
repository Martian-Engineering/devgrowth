use crate::error::AppError;
use crate::github::get_github_client;
use actix_web::{HttpRequest, HttpResponse};
use log::info;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

pub async fn upsert_account(
    pool: &PgPool,
    github_id: &str,
    email: Option<&str>,
) -> Result<i32, sqlx::Error> {
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
    .fetch_one(pool)
    .await?;

    Ok(result.account_id)
}

#[derive(Serialize)]
pub struct ProfileData {
    github_id: String,
    name: String,
    email: String,
    starred_repositories: Vec<StarredRepo>,
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

pub async fn get_profile_data(req: HttpRequest) -> Result<HttpResponse, AppError> {
    let github_client = get_github_client(&req)?;

    // Fetch the authenticated user's information
    let user = github_client.current().user().await?;

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

    let profile_data = ProfileData {
        github_id: user.id.to_string(),
        name: "Josh Lehman".to_string(), // user.name.unwrap_or_else(|| user.login.clone()),
        email: user.email.unwrap_or_default(),
        starred_repositories,
    };

    Ok(HttpResponse::Ok().json(profile_data))
}
