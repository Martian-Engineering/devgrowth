use crate::error::AppError;
use crate::github::{get_github_client, get_github_token};
use crate::growth_accounting::{
    ltv_cohorts_cumulative, mau_growth_accounting, mrr_growth_accounting,
    LTVCohortsCumulativeResult, MAUGrowthAccountingResult, MRRGrowthAccountingResult,
};
use crate::job_queue::Job;
use crate::AppState;
use actix_web::web::Query;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use log::{error, info};
use octocrab::Octocrab;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::postgres::PgPool;

#[derive(Serialize)]
pub struct PaginatedResponse<T> {
    data: Vec<T>,
    total: i64,
    page: i64,
    page_size: i64,
    total_pages: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Repository {
    pub repository_id: i32,
    pub name: String,
    pub owner: String,
    pub stargazers_count: i32,
    pub description: Option<String>,
    pub indexed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NewRepository {
    pub id: Option<i32>,
    pub name: String,
    pub owner: String,
}

#[derive(Deserialize)]
pub struct RepositoryListQuery {
    page: Option<i64>,
    page_size: Option<i64>,
}

#[derive(Serialize)]
pub struct RepositoryMetadata {
    repository_id: i32,
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
            r.repository_id,
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

pub async fn get_repository_gh(
    octocrab: &Octocrab,
    new_repo: &NewRepository,
) -> Result<Option<octocrab::models::Repository>, AppError> {
    match new_repo.id {
        Some(id) => {
            // Search by ID
            match octocrab.repos_by_id(id as u64).get().await {
                Ok(repo) => Ok(Some(repo)),
                Err(octocrab::Error::GitHub { source, .. }) if source.message == "Not Found" => {
                    Ok(None)
                }
                Err(e) => Err(AppError::from(e)),
            }
        }
        None => {
            // Search by owner and name
            match octocrab
                .repos(new_repo.owner.as_str(), new_repo.name.as_str())
                .get()
                .await
            {
                Ok(repo) => Ok(Some(repo)),
                Err(octocrab::Error::GitHub { source, .. }) if source.message == "Not Found" => {
                    Ok(None)
                }
                Err(e) => Err(AppError::from(e)),
            }
        }
    }
}

pub async fn list_repositories(
    state: web::Data<AppState>,
    query: Query<RepositoryListQuery>,
) -> impl Responder {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(10);
    let offset = (page - 1) * page_size;

    match fetch_repositories_with_count(&state.db_pool, page_size, offset).await {
        Ok((repositories, total)) => {
            let total_pages = (total as f64 / page_size as f64).ceil() as i64;
            HttpResponse::Ok().json(PaginatedResponse {
                data: repositories,
                total,
                page,
                page_size,
                total_pages,
            })
        }
        Err(e) => {
            error!("Failed to fetch repositories: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "An error occurred while fetching repositories"
            }))
        }
    }
}

async fn fetch_repositories_with_count(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<(Vec<Repository>, i64), sqlx::Error> {
    let total = sqlx::query_scalar!("SELECT COUNT(*) FROM repository")
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

    let repositories = sqlx::query_as!(
        Repository,
        r#"
        SELECT repository_id, name, owner, stargazers_count, description, indexed_at, created_at, updated_at
        FROM repository
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
        limit,
        offset
    )
    .fetch_all(pool)
    .await?;

    Ok((repositories, total))
}

async fn write_repository(
    pool: &PgPool,
    repository_id: i32,
    repository_name: &str,
    repository_owner: &str,
    stargazers_count: i32,
    description: Option<&str>,
    updated_at: DateTime<Utc>,
) -> Result<Repository, sqlx::Error> {
    let row = match sqlx::query!(
        r#"
        INSERT INTO repository (repository_id, name, owner,
        stargazers_count, description, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (repository_id) DO UPDATE
        SET name = EXCLUDED.name,
            owner = EXCLUDED.owner,
            stargazers_count = EXCLUDED.stargazers_count,
            description = EXCLUDED.description,
            updated_at = EXCLUDED.updated_at
        RETURNING repository_id, name, owner, indexed_at, created_at, updated_at, stargazers_count, description
        "#,
        repository_id,
        repository_name,
        repository_owner,
        stargazers_count,
        description,
        updated_at
    )
    .fetch_one(pool)
    .await
    {
        Ok(row) => {
            log::info!("Successfully inserted repository: {:?}", row);
            row
        }
        Err(e) => {
            log::error!("Failed to insert repository: {:?}", e);
            return Err(e);
        }
    };

    Ok(Repository {
        repository_id: row.repository_id,
        name: row.name,
        owner: row.owner,
        indexed_at: row.indexed_at,
        created_at: row.created_at,
        updated_at: row.updated_at,
        stargazers_count: row.stargazers_count,
        description: row.description,
    })
}

pub async fn upsert_repository(
    state: &AppState,
    req: &HttpRequest,
    new_repo: NewRepository,
) -> Result<Repository, AppError> {
    let pool = &state.db_pool;
    if let Some(id) = new_repo.id {
        if let Some(existing_repo) = get_repository_by_id(pool, id).await? {
            return Ok(existing_repo);
        }
    }

    if let Some(existing_repo) =
        get_repository_by_name_owner(pool, &new_repo.name, &new_repo.owner).await?
    {
        return Ok(existing_repo);
    }

    let github_client = match get_github_client(&req) {
        Ok(client) => client,
        Err(_) => {
            error!("Failed to get GitHub client from claims");
            return Err(AppError::Unauthorized("User not authenticated".into()));
        }
    };

    // If no id or repository not found, check GitHub
    let gh_repo = get_repository_gh(&github_client, &new_repo).await?;

    match gh_repo {
        Some(repo) => {
            // Repository exists on GitHub, create or update in the database
            let repository = write_repository(
                pool,
                repo.id.0 as i32,
                &repo.name,
                &repo.owner.as_ref().map(|o| o.login.as_str()).unwrap_or(""),
                repo.stargazers_count.unwrap_or(0) as i32,
                repo.description.as_deref(),
                repo.updated_at.unwrap_or_else(|| Utc::now()),
            )
            .await?;

            let github_token = match get_github_token(&req) {
                Ok(token) => token,
                Err(e) => {
                    error!("Failed to get github_token from session: {:?}", e);
                    return Err(AppError::Unauthorized("User not authenticated".into()));
                }
            };
            let job = Job {
                repository_id: repository.repository_id,
                owner: repository.owner.clone(),
                name: repository.name.clone(),
                github_token,
            };
            state.job_queue.push(job).await;

            Ok(repository)
        }
        None => Err(AppError::NotFound("Repository not found on GitHub".into())),
    }
}

async fn get_repository_by_id(pool: &PgPool, id: i32) -> Result<Option<Repository>, sqlx::Error> {
    sqlx::query_as!(
        Repository,
        r#"
        SELECT repository_id, name, owner, stargazers_count,
        description, indexed_at, created_at, updated_at
        FROM repository
        WHERE repository_id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await
}

async fn get_repository_by_name_owner(
    pool: &PgPool,
    name: &str,
    owner: &str,
) -> Result<Option<Repository>, sqlx::Error> {
    sqlx::query_as!(
        Repository,
        r#"
        SELECT repository_id, name, owner, stargazers_count,
        description, indexed_at, created_at, updated_at
        FROM repository
        WHERE name = $1 AND owner = $2
        "#,
        name,
        owner
    )
    .fetch_optional(pool)
    .await
}

async fn update_repository_from_github(
    pool: &PgPool,
    gh_repo: &octocrab::models::Repository,
) -> Result<Repository, sqlx::Error> {
    let row = sqlx::query_as!(
        Repository,
        r#"
        UPDATE repository
        SET stargazers_count = $1,
            description = $2,
            updated_at = $3
        WHERE repository_id = $4
        RETURNING repository_id, name, owner, indexed_at, created_at, updated_at, stargazers_count, description
        "#,
        gh_repo.stargazers_count.unwrap_or(0) as i32,
        gh_repo.description.as_deref(),
        gh_repo.updated_at.unwrap_or_else(|| Utc::now()),
        gh_repo.id.0 as i32
    )
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn create_repository(
    state: web::Data<AppState>,
    new_repo: web::Json<NewRepository>,
    req: HttpRequest,
) -> impl Responder {
    match upsert_repository(&state, &req, new_repo.into_inner()).await {
        Ok(repo) => HttpResponse::Created().json(repo),
        Err(AppError::NotFound(_)) => {
            HttpResponse::BadRequest().body("Repository does not exist on GitHub")
        }
        Err(e) => {
            error!("Failed to create repository: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn sync_repository(
    state: web::Data<AppState>,
    path: web::Path<(String, String)>,
    req: HttpRequest,
) -> impl Responder {
    let (owner, name) = path.into_inner();

    let github_token = match get_github_token(&req) {
        Ok(token) => token,
        Err(_) => {
            error!("Failed to get access token from claims");
            return HttpResponse::Unauthorized().finish();
        }
    };

    let github_client = match get_github_client(&req) {
        Ok(client) => client,
        Err(_) => {
            error!("Failed to get GitHub client from claims");
            return HttpResponse::Unauthorized().finish();
        }
    };

    match get_repository_by_name_owner(&state.db_pool, &name, &owner).await {
        Ok(Some(_)) => {
            // Fetch latest data from GitHub
            match github_client.repos(owner.clone(), name.clone()).get().await {
                Ok(gh_repo) => {
                    // Update repository with latest GitHub data
                    match update_repository_from_github(&state.db_pool, &gh_repo).await {
                        Ok(updated_repo) => {
                            let job = Job {
                                repository_id: updated_repo.repository_id,
                                owner: owner.clone(),
                                name: name.clone(),
                                github_token,
                            };
                            state.job_queue.push(job).await;
                            info!("Queued sync job for repository: {}/{}", owner, name);
                            HttpResponse::Accepted().json(json!({
                                "message": "Repository sync job queued",
                                "owner": owner,
                                "name": name
                            }))
                        }
                        Err(e) => {
                            error!("Failed to update repository data: {:?}", e);
                            HttpResponse::InternalServerError().finish()
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to fetch repository data from GitHub: {:?}", e);
                    HttpResponse::InternalServerError().finish()
                }
            }
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

pub async fn get_repository_ga(
    state: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (owner, name) = path.into_inner();

    // Check if the repository exists in our database
    match get_repository_id(&state.db_pool, &owner, &name).await {
        Ok(Some(repository_id)) => {
            match fetch_growth_accounting(&state.db_pool, repository_id).await {
                Ok(results) => HttpResponse::Ok().json(results),
                Err(e) => {
                    error!("Error fetching growth accounting data: {:?}", e);
                    HttpResponse::InternalServerError().json(json!({
                        "error": "An error occurred while fetching growth accounting data"
                    }))
                }
            }
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

#[derive(Debug, Deserialize, Serialize)]
pub struct GrowthAccountingResult {
    mau_growth_accounting: Vec<MAUGrowthAccountingResult>,
    mrr_growth_accounting: Vec<MRRGrowthAccountingResult>,
    ltv_cumulative_cohort: Vec<LTVCohortsCumulativeResult>,
}

async fn fetch_growth_accounting(
    pool: &PgPool,
    repository_id: i32,
) -> Result<GrowthAccountingResult, sqlx::Error> {
    let dau_query = format!(
        r#"
        SELECT
            author AS user_id,
            date_trunc('day',
                "date") AS dt,
            count(*) AS inc_amt
        FROM
            "commit"
        WHERE
            repository_id = {}
        GROUP BY
            1,
            2
        "#,
        repository_id
    );

    let mau_ga = mau_growth_accounting(pool, dau_query.clone()).await?;
    let mrr_ga = mrr_growth_accounting(pool, dau_query.clone()).await?;
    let ltv_cumulative = ltv_cohorts_cumulative(pool, dau_query.clone()).await?;

    Ok(GrowthAccountingResult {
        mau_growth_accounting: mau_ga,
        mrr_growth_accounting: mrr_ga,
        ltv_cumulative_cohort: ltv_cumulative,
    })
}
