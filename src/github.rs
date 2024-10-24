use crate::error::AppError;
use crate::{auth::Claims, types::PaginatedResponse};
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use http::Uri;
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

#[derive(Deserialize)]
pub struct StarredReposQuery {
    page: Option<i64>,
    page_size: Option<i64>,
}

pub async fn get_starred_repositories(
    // state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<StarredReposQuery>, // Add query parameters
) -> Result<HttpResponse, AppError> {
    let github_client = get_github_client(&req)?;
    let page = query.page.unwrap_or(1) as u8;
    let page_size = query.page_size.unwrap_or(10) as u8;

    // Fetch the authenticated user's information
    match github_client.current().user().await {
        Ok(user) => user,
        Err(e) => {
            log::error!("Failed to fetch user data: {:?}", e);
            return Err(AppError::Unauthorized("User not authenticated".to_string()));
        }
    };

    let repos = github_client
        .current()
        .list_repos_starred_by_authenticated_user()
        .per_page(page_size)
        .page(page)
        .send()
        .await?;

    let total_pages = calculate_total_pages(page, repos.last.as_ref(), repos.prev.as_ref());
    let total = total_pages * (page_size as i64);

    let starred_repositories: Vec<GithubRepo> = repos
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

    Ok(HttpResponse::Ok().json(PaginatedResponse {
        data: starred_repositories,
        total,
        page: page as i64,
        page_size: page_size as i64,
        total_pages,
    }))
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
    let total_pages = calculate_total_pages(page, repos.last.as_ref(), repos.prev.as_ref());
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

#[derive(Deserialize)]
pub struct SearchQuery {
    q: String,
    page: Option<i64>,
    page_size: Option<i64>,
}

pub async fn search_repositories(
    req: HttpRequest,
    query: web::Query<SearchQuery>,
) -> Result<HttpResponse, AppError> {
    let github_client = get_github_client(&req)?;
    let page = query.page.unwrap_or(1) as u8;
    let page_size = query.page_size.unwrap_or(10) as u8;

    let repos = github_client
        .search()
        .repositories(&query.q)
        .sort("updated")
        .order("desc")
        .per_page(page_size)
        .page(page)
        .send()
        .await?;

    let total_pages = calculate_total_pages(page, repos.last.as_ref(), repos.prev.as_ref());
    let total = total_pages * (page_size as i64);

    let search_repositories: Vec<GithubRepo> = repos
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

    Ok(HttpResponse::Ok().json(PaginatedResponse {
        data: search_repositories,
        total,
        page: page as i64,
        page_size: page_size as i64,
        total_pages,
    }))
}

fn calculate_total_pages(page: u8, last_url: Option<&Uri>, prev_url: Option<&Uri>) -> i64 {
    if let Some(last_url) = last_url {
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
    } else if prev_url.is_some() {
        page as i64
    } else {
        1
    }
}
