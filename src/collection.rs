use crate::auth_utils::get_account_id;
use crate::error::AppError;
use crate::growth_accounting::{
    ltv_cohorts_cumulative, mau_growth_accounting, mrr_growth_accounting,
    LTVCohortsCumulativeResult, MAUGrowthAccountingResult, MRRGrowthAccountingResult,
};
use crate::repository::{upsert_repository, NewRepository, Repository};
use crate::AppState;
use actix_web::web::BytesMut;
use actix_web::{web, HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
use futures::StreamExt;
use log::error;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;

#[derive(Serialize, Deserialize)]
pub struct Collection {
    pub collection_id: i32,
    pub owner_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub repository_count: Option<i64>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct CollectionWithRepositories {
    collection_id: i32,
    owner_id: i32,
    name: String,
    description: Option<String>,
    is_default: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    repository_count: Option<i64>,
    repositories: serde_json::Value,
}

#[derive(Deserialize)]
pub struct CreateCollection {
    name: String,
    description: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateCollection {
    name: Option<String>,
    description: Option<String>,
}

pub async fn create_collection(
    state: web::Data<AppState>,
    req: HttpRequest,
    collection: web::Json<CreateCollection>,
) -> Result<HttpResponse, AppError> {
    let account_id = get_account_id(&req)?;
    if collection.name.to_lowercase() == "default" {
        return Err(AppError::BadRequest(
            "Cannot create a new default collection".into(),
        ));
    }

    let result = match sqlx::query_as!(
        Collection,
        r#"
        INSERT INTO collection (owner_id, name, description)
        VALUES ($1, $2, $3)
        RETURNING collection_id, owner_id, name, description, is_default, created_at, updated_at, 0::bigint as repository_count
        "#,
        account_id,
        collection.name,
        collection.description
    )
    .fetch_one(&state.db_pool)
    .await
    {
        Ok(result) => result,
        Err(sqlx::Error::Database(e)) => {
            log::error!("Error creating collection: {:?}", e);
            return Err(AppError::BadRequest(
                "Collection with the same name already exists".into(),
            ));
        }
        Err(err) => return Err(err.into()),
    };

    Ok(HttpResponse::Created().json(result))
}

pub async fn get_collections(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let account_id = get_account_id(&req)?;

    let collections = sqlx::query_as!(
        CollectionWithRepositories,
        r#"
        SELECT
            c.collection_id,
            c.owner_id,
            c.name,
            c.description,
            c.is_default,
            c.created_at,
            c.updated_at,
            COUNT(cr.repository_id)::bigint AS repository_count,
            COALESCE(
                json_agg(
                    json_build_object(
                        'repository_id', r.repository_id,
                        'name', r.name,
                        'owner', r.owner,
                        'description', r.description,
                        'stargazers_count', r.stargazers_count,
                        'indexed_at', r.indexed_at,
                        'created_at', r.created_at,
                        'updated_at', r.updated_at
                    )
                ) FILTER (WHERE r.repository_id IS NOT NULL),
                '[]'::json
            ) AS repositories
        FROM collection c
        LEFT JOIN collection_repository cr ON c.collection_id = cr.collection_id
        LEFT JOIN repository r ON cr.repository_id = r.repository_id
        WHERE c.owner_id = $1
        GROUP BY c.collection_id
        ORDER BY c.created_at DESC
        "#,
        account_id
    )
    .fetch_all(&state.db_pool)
    .await?;

    Ok(HttpResponse::Ok().json(collections))
}

pub async fn get_collection(
    state: web::Data<AppState>,
    collection_id: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let collection_id = collection_id.into_inner();
    let collection = sqlx::query_as!(
        Collection,
        r#"
        SELECT
            c.collection_id,
            c.owner_id,
            c.name,
            c.description,
            c.is_default,
            c.created_at,
            c.updated_at,
            COUNT(cr.repository_id) AS repository_count
        FROM collection c
        LEFT JOIN collection_repository cr ON c.collection_id = cr.collection_id
        WHERE c.collection_id = $1
        GROUP BY c.collection_id
        ORDER BY c.created_at DESC
        "#,
        collection_id,
    )
    .fetch_optional(&state.db_pool)
    .await?;

    match collection {
        Some(collection) => {
            // If the collection exists, fetch its repositories
            let repositories = sqlx::query_as!(
                Repository,
                r#"
                SELECT r.repository_id, r.name, r.owner,
                r.description, r.stargazers_count,
                r.indexed_at, r.created_at, r.updated_at
                FROM repository r
                JOIN collection_repository cr ON r.repository_id = cr.repository_id
                WHERE cr.collection_id = $1
                "#,
                collection_id
            )
            .fetch_all(&state.db_pool)
            .await?;

            let mut collection_json = serde_json::to_value(collection).unwrap();
            if let serde_json::Value::Object(ref mut obj) = collection_json {
                obj.insert(
                    "repositories".to_string(),
                    serde_json::to_value(repositories).unwrap(),
                );
            }
            let response = collection_json;

            Ok(HttpResponse::Ok().json(response))
        }
        None => Ok(HttpResponse::NotFound().finish()),
    }
}

pub async fn update_collection(
    state: web::Data<AppState>,
    req: HttpRequest,
    collection_id: web::Path<i32>,
    collection: web::Json<UpdateCollection>,
) -> Result<HttpResponse, AppError> {
    let account_id = get_account_id(&req)?;

    let collection_id = collection_id.into_inner();

    // Check if the collection belongs to the user
    let existing_collection = sqlx::query!(
        r#"
        SELECT owner_id, is_default FROM collection
        WHERE collection_id = $1
        "#,
        collection_id
    )
    .fetch_optional(&state.db_pool)
    .await?;

    match existing_collection {
        Some(existing) if existing.owner_id == account_id => {
            if existing.is_default {
                return Err(AppError::BadRequest(
                    "Cannot modify the default collection".into(),
                ));
            }

            // Update the collection
            let updated_collection = sqlx::query_as!(
                Collection,
                r#"
                UPDATE collection
                SET name = COALESCE($1, name),
                    description = COALESCE($2, description),
                    updated_at = NOW()
                WHERE collection_id = $3
                RETURNING collection_id, owner_id, name, description, is_default, created_at, updated_at,
                          (SELECT COUNT(*)::bigint FROM collection_repository WHERE collection_id = $3) AS repository_count
                "#,
                collection.name,
                collection.description,
                collection_id
            )
            .fetch_one(&state.db_pool)
            .await?;

            let repositories = sqlx::query_as!(
                Repository,
                r#"
                SELECT r.repository_id, r.name, r.owner, r.description,
                r.stargazers_count, r.indexed_at, r.created_at, r.updated_at
                FROM repository r
                JOIN collection_repository cr ON r.repository_id = cr.repository_id
                WHERE cr.collection_id = $1
                "#,
                collection_id
            )
            .fetch_all(&state.db_pool)
            .await?;

            let mut collection_json = serde_json::to_value(updated_collection).unwrap();
            if let serde_json::Value::Object(ref mut obj) = collection_json {
                obj.insert(
                    "repositories".to_string(),
                    serde_json::to_value(repositories).unwrap(),
                );
            }
            let response = collection_json;

            Ok(HttpResponse::Ok().json(response))
        }
        Some(_) => Err(AppError::Unauthorized(
            "You do not own this collection".into(),
        )),
        None => Ok(HttpResponse::NotFound().finish()),
    }
}

pub async fn delete_collection(
    state: web::Data<AppState>,
    req: HttpRequest,
    collection_id: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let account_id = get_account_id(&req)?;

    let collection_id = collection_id.into_inner();

    // Check if the collection belongs to the user
    let collection = sqlx::query!(
        r#"
        SELECT owner_id, is_default FROM collection
        WHERE collection_id = $1
        "#,
        collection_id
    )
    .fetch_optional(&state.db_pool)
    .await?;

    match collection {
        Some(collection) if collection.owner_id == account_id => {
            // Delete the collection
            if collection.is_default {
                return Err(AppError::BadRequest(
                    "Cannot delete the default collection".into(),
                ));
            }
            sqlx::query!(
                r#"
                DELETE FROM collection
                WHERE collection_id = $1
                "#,
                collection_id
            )
            .execute(&state.db_pool)
            .await?;

            Ok(HttpResponse::NoContent().finish())
        }
        Some(_) => Err(AppError::Unauthorized(
            "You do not own this collection".into(),
        )),
        None => Ok(HttpResponse::NotFound().finish()),
    }
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum AddRepositoryToCollectionRequest {
    ById { repository_id: i32 },
    ByNameAndOwner { name: String, owner: String },
}

pub async fn add_repository_to_collection(
    state: web::Data<AppState>,
    req: HttpRequest,
    mut payload: actix_web::web::Payload,
    path: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let account_id = get_account_id(&req)?;
    let collection_id = path.into_inner();

    // Check if the collection belongs to the user
    let collection = sqlx::query!(
        r#"
            SELECT owner_id FROM collection
            WHERE collection_id = $1
            "#,
        collection_id
    )
    .fetch_optional(&state.db_pool)
    .await?;

    if let Some(collection) = collection {
        if collection.owner_id != account_id {
            return Err(AppError::Unauthorized(
                "You do not own this collection".into(),
            ));
        }
    } else {
        return Ok(HttpResponse::NotFound().finish());
    }

    let mut body = BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        body.extend_from_slice(&chunk);
    }

    let str_body = std::str::from_utf8(&body).map_err(|e| {
        log::error!("Error decoding body: {:?}", e);
        AppError::BadRequest("Invalid UTF-8 sequence".to_string())
    })?;

    // Try to parse the body as JSON
    let parsed_body: AddRepositoryToCollectionRequest =
        serde_json::from_str(str_body).map_err(|e| {
            log::error!("Error parsing JSON body: {:?}", e);
            AppError::BadRequest("Invalid JSON".to_string())
        })?;

    // Create a NewRepository based on the input
    let new_repo = match parsed_body {
        AddRepositoryToCollectionRequest::ById { repository_id } => NewRepository {
            id: Some(repository_id),
            name: String::new(),
            owner: String::new(),
        },
        AddRepositoryToCollectionRequest::ByNameAndOwner { name, owner } => NewRepository {
            id: None,
            name: name.clone(),
            owner: owner.clone(),
        },
    };

    let repository = upsert_repository(&state, &req, new_repo).await?;

    // Add the repository to the collection
    match sqlx::query!(
        r#"
            INSERT INTO collection_repository (collection_id, repository_id)
            VALUES ($1, $2)
            ON CONFLICT (collection_id, repository_id) DO NOTHING
            "#,
        collection_id,
        repository.repository_id
    )
    .execute(&state.db_pool)
    .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                Ok(HttpResponse::Created().json(json!({
                    "message": "Repository added to collection",
                    "repository": repository
                })))
            } else {
                Ok(HttpResponse::Ok()
                    .json(json!({ "message": "Repository already in collection" })))
            }
        }
        Err(e) => {
            log::error!("Failed to add repository to collection {:?}", e);
            Err(AppError::InternalServerError(format!(
                "Failed to add repository to collection: {}",
                e
            )))
        }
    }
}

pub async fn remove_repository_from_collection(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<(i32, i32)>,
) -> Result<HttpResponse, AppError> {
    let account_id = get_account_id(&req)?;
    let (collection_id, repository_id) = path.into_inner();
    let collection = sqlx::query!(
        r#"
        SELECT owner_id FROM collection
        WHERE collection_id = $1
        "#,
        collection_id
    )
    .fetch_optional(&state.db_pool)
    .await?;

    match collection {
        Some(collection) if collection.owner_id == account_id => {
            let result = sqlx::query!(
                r#"
                DELETE FROM collection_repository
                WHERE collection_id = $1 AND repository_id = $2
                "#,
                collection_id,
                repository_id
            )
            .execute(&state.db_pool)
            .await?;

            if result.rows_affected() > 0 {
                Ok(HttpResponse::NoContent().finish())
            } else {
                Ok(HttpResponse::NotFound().finish())
            }
        }
        Some(_) => Err(AppError::Unauthorized(
            "You do not own this collection".into(),
        )),
        None => Ok(HttpResponse::NotFound().finish()),
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GrowthAccountingResult {
    mau_growth_accounting: Vec<MAUGrowthAccountingResult>,
    mrr_growth_accounting: Vec<MRRGrowthAccountingResult>,
    ltv_cumulative_cohort: Vec<LTVCohortsCumulativeResult>,
}

pub async fn get_collection_growth_accounting(
    state: web::Data<AppState>,
    req: HttpRequest,
    collection_id: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    // Implement the logic to calculate growth accounting for all repositories in the collection
    // This will involve joining the collections, collection_repositories, and commit tables,
    // and then performing the growth accounting calculations
    let account_id = get_account_id(&req)?;
    let collection_id = collection_id.into_inner();
    let collection = sqlx::query!(
        r#"
        SELECT owner_id FROM collection
        WHERE collection_id = $1
        "#,
        collection_id
    )
    .fetch_optional(&state.db_pool)
    .await?;

    match collection {
        Some(collection) if collection.owner_id == account_id => {
            match fetch_growth_accounting(&state.db_pool, collection_id).await {
                Ok(results) => Ok(HttpResponse::Ok().json(results)),
                Err(e) => {
                    error!("Error fetching growth accounting data: {:?}", e);
                    Err(AppError::InternalServerError(
                        "An error occurred while fetching growth accounting data".to_string(),
                    ))
                }
            }
        }
        Some(_) => Err(AppError::Unauthorized(
            "You do not own this collection".into(),
        )),
        None => Ok(HttpResponse::NotFound().finish()),
    }
}

async fn fetch_growth_accounting(
    pool: &PgPool,
    collection_id: i32,
) -> Result<GrowthAccountingResult, sqlx::Error> {
    let dau_query = format!(
        r#"
        SELECT
            author AS user_id,
            date_trunc('day', "date") AS dt,
            count(*) AS inc_amt
        FROM
            "commit" c
            LEFT JOIN collection_repository cr ON cr.repository_id = c.repository_id
        WHERE
            cr.collection_id = {}
        GROUP BY
            1,
            2
        "#,
        collection_id
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
