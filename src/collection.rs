use crate::auth_utils::get_account_id;
use crate::error::AppError;
use crate::growth_accounting::mau_growth_accounting;
use crate::repository::{upsert_repository, NewRepository, Repository};
use crate::AppState;
use actix_web::web::BytesMut;
use actix_web::{web, HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
pub struct Collection {
    collection_id: i32,
    owner_id: i32,
    name: String,
    description: Option<String>,
    is_default: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
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

    let result = sqlx::query_as!(
        Collection,
        r#"
        INSERT INTO collection (owner_id, name, description)
        VALUES ($1, $2, $3)
        RETURNING collection_id, owner_id, name, description, is_default, created_at, updated_at
        "#,
        account_id,
        collection.name,
        collection.description
    )
    .fetch_one(&state.db_pool)
    .await?;

    Ok(HttpResponse::Created().json(result))
}

pub async fn get_collections(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let account_id = get_account_id(&req)?;

    let collections = sqlx::query_as!(
        Collection,
        r#"
        SELECT collection_id, owner_id, name, description, is_default, created_at, updated_at
        FROM collection
        WHERE owner_id = $1
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
        SELECT collection_id, owner_id, name, description, is_default, created_at, updated_at
        FROM collection
        WHERE collection_id = $1
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
                SELECT r.repository_id, r.name, r.owner, r.indexed_at, r.created_at, r.updated_at
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
                RETURNING collection_id, owner_id, name, description, is_default, created_at, updated_at
                "#,
                collection.name,
                collection.description,
                collection_id
            )
            .fetch_one(&state.db_pool)
            .await?;

            Ok(HttpResponse::Ok().json(updated_collection))
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
            SELECT owner_id, is_default FROM collection
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
        SELECT owner_id, is_default FROM collection
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
        SELECT owner_id, is_default FROM collection
        WHERE collection_id = $1
        "#,
        collection_id
    )
    .fetch_optional(&state.db_pool)
    .await?;

    match collection {
        Some(collection) if collection.owner_id == account_id => {
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

            let results = mau_growth_accounting(&state.db_pool, dau_query).await?;

            Ok(HttpResponse::Ok().json(results))
        }
        Some(_) => Err(AppError::Unauthorized(
            "You do not own this collection".into(),
        )),
        None => Ok(HttpResponse::NotFound().finish()),
    }
}
