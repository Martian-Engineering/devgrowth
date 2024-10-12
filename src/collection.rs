use crate::auth_utils::get_account_id;
use crate::error::AppError;
use crate::repository::{upsert_repository, NewRepository};
use crate::AppState;
use actix_web::{web, HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
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
    let collection = sqlx::query_as!(
        Collection,
        r#"
        SELECT collection_id, owner_id, name, description, is_default, created_at, updated_at
        FROM collection
        WHERE collection_id = $1
        "#,
        collection_id.into_inner(),
    )
    .fetch_optional(&state.db_pool)
    .await?;

    match collection {
        Some(c) => Ok(HttpResponse::Ok().json(c)),
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

pub async fn add_repository_to_collection(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<(i32, i32)>,
) -> Result<HttpResponse, AppError> {
    let account_id = get_account_id(&req)?;
    let (collection_id, repository_id) = path.into_inner();

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

    // Create a NewRepository with just the ID
    let new_repo = NewRepository {
        id: Some(repository_id),
        name: String::new(), // These will be filled by upsert_repository if needed
        owner: String::new(),
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
        repository_id
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

// pub async fn get_collection_growth_accounting(
//     state: web::Data<AppState>,
//     account_id: i32,
//     collection_id: web::Path<i32>,
// ) -> Result<HttpResponse, AppError> {
//     // Implement the logic to calculate growth accounting for all repositories in the collection
//     // This will involve joining the collections, collection_repositories, and commit tables,
//     // and then performing the growth accounting calculations
//     todo!()
// }
