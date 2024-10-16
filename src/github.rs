use crate::auth::Claims;
use crate::error::AppError;
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use log::error;
use octocrab::Octocrab;
use serde::{Deserialize, Serialize};

pub fn get_github_token(req: &HttpRequest) -> Result<String, AppError> {
    let claims = req.extensions().get::<Claims>().cloned();
    match claims {
        Some(claims) => Ok(claims.access_token),
        None => {
            error!("Failed to get access token from claims");
            Err(AppError::Unauthorized(
                "Failed to get access token from claims".to_string(),
            ))
        }
    }
}

pub fn get_github_client(req: &HttpRequest) -> Result<Octocrab, AppError> {
    let github_token = get_github_token(req)?;

    Octocrab::builder()
        .personal_token(github_token)
        .build()
        .map_err(|e| AppError::GitHub(e))
}

#[derive(Serialize, Deserialize)]
pub struct GithubRepo {
    id: u64,
    name: String,
    owner: String,
    html_url: String,
    description: Option<String>,
    stargazers_count: Option<u32>,
}

pub async fn get_starred_repositories(
    // state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let github_client = get_github_client(&req)?;
    // Fetch the authenticated user's information
    match github_client.current().user().await {
        Ok(user) => user,
        Err(e) => {
            log::error!("Failed to fetch user data: {:?}", e);
            return Err(AppError::Unauthorized("User not authenticated".to_string()));
        }
    };

    // Fetch the user's starred repositories
    let starred_repos = github_client
        .current()
        .list_repos_starred_by_authenticated_user()
        .per_page(100) // Adjust this number as needed
        .send()
        .await?;

    let starred_repositories: Vec<GithubRepo> = starred_repos
        .items
        .into_iter()
        .map(|repo| GithubRepo {
            id: repo.id.0,
            name: repo.name,
            owner: repo.owner.map(|owner| owner.login).unwrap_or_default(),
            html_url: repo.html_url.map(|url| url.to_string()).unwrap_or_default(),
            description: repo.description,
            stargazers_count: repo.stargazers_count,
        })
        .collect();

    Ok(HttpResponse::Ok().json(starred_repositories))
}
