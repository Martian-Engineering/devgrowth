use crate::error::AppError;
use crate::github::{get_github_client, get_github_token};
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

#[derive(Debug, Deserialize, Serialize)]
pub struct Repository {
    pub repository_id: i32,
    pub name: String,
    pub owner: String,
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
    limit: Option<i64>,
    offset: Option<i64>,
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

#[derive(sqlx::FromRow, Serialize)]
struct GrowthAccountingResult {
    date: String,
    mau: i64,
    retained: i64,
    new: i64,
    resurrected: i64,
    churned: i64,
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
    let limit = query.limit.unwrap_or(50); // Default limit to 50
    let offset = query.offset.unwrap_or(0); // Default offset to 0

    match fetch_repositories(&state.db_pool, limit, offset).await {
        Ok(repositories) => HttpResponse::Ok().json(repositories),
        Err(e) => {
            error!("Failed to fetch repositories: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "An error occurred while fetching repositories"
            }))
        }
    }
}

async fn fetch_repositories(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<Vec<Repository>, sqlx::Error> {
    sqlx::query_as!(
        Repository,
        r#"
        SELECT repository_id, name, owner, indexed_at, created_at, updated_at
        FROM repository
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
        limit,
        offset
    )
    .fetch_all(pool)
    .await
}

async fn write_repository(
    pool: &PgPool,
    repository_id: i32,
    repository_name: &str,
    repository_owner: &str,
) -> Result<Repository, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        INSERT INTO repository (repository_id, name, owner)
        VALUES ($1, $2, $3)
        RETURNING repository_id, name, owner, indexed_at, created_at, updated_at
        "#,
        repository_id,
        repository_name,
        repository_owner,
    )
    .fetch_one(pool)
    .await?;

    Ok(Repository {
        repository_id: row.repository_id,
        name: row.name,
        owner: row.owner,
        indexed_at: row.indexed_at,
        created_at: row.created_at,
        updated_at: row.updated_at,
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
        SELECT repository_id, name, owner, indexed_at, created_at, updated_at
        FROM repository
        WHERE repository_id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await
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

    match get_repository_id(&state.db_pool, &owner, &name).await {
        Ok(Some(repository_id)) => {
            let job = Job {
                repository_id,
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

async fn fetch_growth_accounting(
    pool: &PgPool,
    repository_id: i32,
) -> Result<Vec<GrowthAccountingResult>, sqlx::Error> {
    let query = r#"
        WITH dau AS (
            SELECT
                author AS user_id,
                date_trunc('day',
                    "date") AS dt,
                count(*) AS inc_amt
            FROM
                "commit"
            WHERE
                repository_id = $1
            GROUP BY
                1,
                2
        ),
        -- First, set up WAU and MAU tables for future use
        wau AS (
            SELECT
                date_trunc('week',
                    dt) AS week,
                user_id,
                sum(inc_amt) AS inc_amt
            FROM
                dau
            GROUP BY
                1,
                2
        ),
        mau AS (
            SELECT
                date_trunc('month',
                    dt) AS month,
                user_id,
                sum(inc_amt) AS inc_amt
            FROM
                dau
            GROUP BY
                1,
                2
        ),
        -- This determines the cohort date of each user. In this case we are
        -- deriving it from DAU data but you can feel free to replace it with
        -- registration date if that's more appropriate.
        first_dt AS (
            SELECT
                user_id,
                min(dt) AS first_dt,
                date_trunc('week',
                    min(dt)) AS first_week,
                date_trunc('month',
                    min(dt)) AS first_month
            FROM
                dau
            GROUP BY
                1
        ),
        mau_decorated AS (
            SELECT
                d.month,
                d.user_id,
                d.inc_amt,
                f.first_month
            FROM
                mau d,
                first_dt f
            WHERE
                d.user_id = f.user_id
                AND inc_amt > 0
        ),
        -- This is MAU growth accounting. Note that this does not require any
        -- information about inc_amt. As discussed in the articles, these
        -- quantities satisfy some identities:
        -- MAU(t) = retained(t) + new(t) + resurrected(t)
        -- MAU(t - 1 month) = retained(t) + churned(t)
        mau_growth_accounting AS (
            SELECT
                coalesce(tm.month,
                    lm.month + interval '1 month') AS month,
                count(DISTINCT tm.user_id) AS mau,
                count(DISTINCT CASE WHEN lm.user_id IS NOT NULL THEN
                        tm.user_id
                    ELSE
                        NULL
                    END) AS retained,
                count(DISTINCT CASE WHEN tm.first_month = tm.month THEN
                        tm.user_id
                    ELSE
                        NULL
                    END) AS new,
                count(DISTINCT CASE WHEN tm.first_month != tm.month
                        AND lm.user_id IS NULL THEN
                        tm.user_id
                    ELSE
                        NULL
                    END) AS resurrected,
                - 1 * count(DISTINCT CASE WHEN tm.user_id IS NULL THEN
                        lm.user_id
                    ELSE
                        NULL
                    END) AS churned
            FROM
                mau_decorated tm
            FULL OUTER JOIN mau_decorated lm ON (tm.user_id = lm.user_id
                AND tm.month = lm.month + interval '1 month')
        GROUP BY
            1
        ORDER BY
            1
        ),
        -- This generates the familiar monthly cohort retention dataset.
        mau_retention_by_cohort AS (
            SELECT
                first_month,
                12 * extract(year FROM age(month,
                        first_month)) + extract(month FROM age(month,
                        first_month)) AS months_since_first,
                count(1) AS active_users,
                sum(inc_amt) AS inc_amt
            FROM
                mau_decorated
            GROUP BY
                1,
                2
            ORDER BY
                1,
                2
        ),
        -- This is the MRR growth accounting (or growth accounting of whatever
        -- value you put in inc_amt). These also satisfy some identities:
        -- MRR(t) = retained(t) + new(t) + resurrected(t) + expansion(t)
        -- MAU(t - 1 month) = retained(t) + churned(t) + contraction(t)
        mrr_growth_accounting AS (
            SELECT
                coalesce(tm.month,
                    lm.month + interval '1 month') AS month,
                sum(tm.inc_amt) AS rev,
                sum(
                    CASE WHEN tm.user_id IS NOT NULL
                        AND lm.user_id IS NOT NULL
                        AND tm.inc_amt >= lm.inc_amt THEN
                        lm.inc_amt
                    WHEN tm.user_id IS NOT NULL
                        AND lm.user_id IS NOT NULL
                        AND tm.inc_amt < lm.inc_amt THEN
                        tm.inc_amt
                    ELSE
                        0
                    END) AS retained,
                sum(
                    CASE WHEN tm.first_month = tm.month THEN
                        tm.inc_amt
                    ELSE
                        0
                    END) AS new,
                sum(
                    CASE WHEN tm.month != tm.first_month
                        AND tm.user_id IS NOT NULL
                        AND lm.user_id IS NOT NULL
                        AND tm.inc_amt > lm.inc_amt
                        AND lm.inc_amt > 0 THEN
                        tm.inc_amt - lm.inc_amt
                    ELSE
                        0
                    END) AS expansion,
                sum(
                    CASE WHEN tm.user_id IS NOT NULL
                        and(lm.user_id IS NULL
                            OR lm.inc_amt = 0)
                        AND tm.inc_amt > 0
                        AND tm.first_month != tm.month THEN
                        tm.inc_amt
                    ELSE
                        0
                    END) AS resurrected,
                - 1 * sum(
                    CASE WHEN tm.month != tm.first_month
                        AND tm.user_id IS NOT NULL
                        AND lm.user_id IS NOT NULL
                        AND tm.inc_amt < lm.inc_amt
                        AND tm.inc_amt > 0 THEN
                        lm.inc_amt - tm.inc_amt
                    ELSE
                        0
                    END) AS contraction,
                - 1 * sum(
                    CASE WHEN lm.inc_amt > 0
                        and(tm.user_id IS NULL
                            OR tm.inc_amt = 0) THEN
                        lm.inc_amt
                    ELSE
                        0
                    END) AS churned
            FROM
                mau_decorated tm
            FULL OUTER JOIN mau_decorated lm ON (tm.user_id = lm.user_id
                AND tm.month = lm.month + interval '1 month')
        GROUP BY
            1
        ORDER BY
            1
        ),
        -- These next tables are to compute LTV via the cohorts_cumulative table.
        -- The LTV here is being computed for weekly cohorts on weekly intervals.
        -- The queries can be modified to compute it for cohorts of any size
        -- on any time window frequency.
        wau_decorated AS (
            SELECT
                week,
                w.user_id,
                w.inc_amt,
                f.first_week
            FROM
                wau w,
                first_dt f
        WHERE
            w.user_id = f.user_id
        ),
        cohorts AS (
            SELECT
                first_week,
                week AS active_week,
                ceil(extract(DAYS FROM (week - first_week)) / 7.0) AS weeks_since_first,
                count(DISTINCT user_id) AS users,
                sum(inc_amt) AS inc_amt
            FROM
                wau_decorated
            GROUP BY
                1,
                2,
                3
            ORDER BY
                1,
                2
        ),
        cohort_sizes AS (
            SELECT
                first_week,
                users,
                inc_amt
            FROM
                cohorts
            WHERE
                weeks_since_first = 0
        ),
        cohorts_cumulative AS (
            -- A semi-cartesian join accomplishes the cumulative behavior.
            SELECT
                c1.first_week,
                c1.active_week,
                c1.weeks_since_first,
                c1.users,
                cs.users AS cohort_num_users,
                1.0 * c1.users / cs.users AS retained_pctg,
                c1.inc_amt,
                sum(c2.inc_amt) AS cum_amt,
                1.0 * sum(c2.inc_amt) / cs.users AS cum_amt_per_user
            FROM
                cohorts c1,
                cohorts c2,
                cohort_sizes cs
            WHERE
                c1.first_week = c2.first_week
                AND c2.weeks_since_first <= c1.weeks_since_first
                AND cs.first_week = c1.first_week
            GROUP BY
                1,
                2,
                3,
                4,
                5,
                6,
                7
            ORDER BY
                1,
                2
        ),
        -- monthly cumulative cohorts
        cohorts_m AS (
            SELECT
                first_month,
                month AS active_month,
                extract(month FROM month) - extract(month FROM first_month) + 12 * (extract(year FROM month) - extract(year FROM first_month)) AS months_since_first,
                count(DISTINCT user_id) AS users,
                sum(inc_amt) AS inc_amt
            FROM
                mau_decorated
            GROUP BY
                1,
                2,
                3
            ORDER BY
                1,
                2
        ),
        cohort_sizes_m AS (
            SELECT
                first_month,
                users,
                inc_amt
            FROM
                cohorts_m
            WHERE
                months_since_first = 0
        ),
        cohorts_cumulative_m AS (
            -- A semi-cartesian join accomplishes the cumulative behavior.
            SELECT
                c1.first_month,
                c1.active_month,
                c1.months_since_first,
                c1.users,
                cs.users AS cohort_num_users,
                1.0 * c1.users / cs.users AS retained_pctg,
                c1.inc_amt,
                sum(c2.inc_amt) AS cum_amt,
                1.0 * sum(c2.inc_amt) / cs.users AS cum_amt_per_user
            FROM
                cohorts_m c1,
                cohorts_m c2,
                cohort_sizes_m cs
            WHERE
                c1.first_month = c2.first_month
                AND c2.months_since_first <= c1.months_since_first
                AND cs.first_month = c1.first_month
            GROUP BY
                1,
                2,
                3,
                4,
                5,
                6,
                7
            ORDER BY
                1,
                2
        )
        SELECT
            to_char("month", 'MM-DD-YYYY') AS date,
            mau,
            retained,
            new,
            resurrected,
            churned
        FROM
            mau_growth_accounting
        ORDER BY
            month ASC;
    "#;

    let results = sqlx::query_as::<_, GrowthAccountingResult>(query)
        .bind(repository_id)
        .fetch_all(pool)
        .await?;

    Ok(results)
}
