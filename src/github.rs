use crate::error::AppError;
use crate::{auth::Claims, types::PaginatedResponse};
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use log::error;
use octocrab::params::repos::Type;
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

#[derive(Deserialize)]
pub struct OrgReposQuery {
    page: Option<i64>,
    page_size: Option<i64>,
}

pub async fn get_organization_repositories(
    req: HttpRequest,
    org: web::Path<String>,
    query: web::Query<OrgReposQuery>,
) -> Result<HttpResponse, AppError> {
    let github_client = get_github_client(&req)?;
    let page = query.page.unwrap_or(1) as u8;
    let page_size = query.page_size.unwrap_or(10) as u8;

    let repos = github_client
        .orgs(&*org)
        .list_repos()
        .repo_type(Type::All)
        .per_page(page_size)
        .page(page)
        .send()
        .await?;

    // Get the total pages from the last page URL
    let total_pages = if let Some(last_url) = &repos.last {
        // Parse the URL and get the page parameter
        let last_url_str = last_url.to_string();
        if let Some(page_param) = last_url_str
            .split('&')
            .find(|&s| s.starts_with("page="))
            .and_then(|s| s.split('=').nth(1))
            .and_then(|s| s.parse::<i64>().ok())
        {
            page_param
        } else {
            page as i64
        }
    } else if let Some(prev_url) = &repos.prev {
        // On last page, so current page number is the total number of pages
        page as i64
    } else {
        // single page of results
        1
    };

    let total = total_pages * (page_size as i64);

    let org_repositories: Vec<GithubRepo> = repos
        .items
        .into_iter()
        .map(|repo| {
            let github_repo = GithubRepo {
                id: repo.id.0,
                name: repo.name,
                owner: repo.owner.map(|owner| owner.login).unwrap_or_default(),
                html_url: repo.html_url.map(|url| url.to_string()).unwrap_or_default(),
                description: repo.description,
                stargazers_count: repo.stargazers_count,
            };
            github_repo
        })
        .collect();

    Ok(HttpResponse::Ok().json(PaginatedResponse {
        data: org_repositories,
        total,
        page: page as i64,
        page_size: page_size as i64,
        total_pages,
    }))
}

pub async fn search_repositories(
    req: HttpRequest,
    query: web::Query<SearchQuery>,
) -> Result<HttpResponse, AppError> {
    let github_client = get_github_client(&req)?;

    let search_results = github_client
        .search()
        .repositories(&query.q)
        .sort("updated")
        .order("desc")
        .per_page(100)
        .send()
        .await?;

    let search_repositories: Vec<GithubRepo> = search_results
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

    Ok(HttpResponse::Ok().json(search_repositories))
}

#[derive(Deserialize)]
pub struct SearchQuery {
    q: String,
}
