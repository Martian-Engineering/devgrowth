use actix_session::SessionGetError;
use actix_web::http::StatusCode;
use actix_web::ResponseError;
use sqlx::migrate::MigrateError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] MigrateError),

    #[error("GitHub API error: {0}")]
    GitHub(#[from] octocrab::Error),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Environment error: {0}")]
    Environment(String),

    #[error("Bad Request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Session error: {0}")]
    Session(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal server error: {0}")]
    InternalServerError(String),
}

impl From<SessionGetError> for AppError {
    fn from(error: SessionGetError) -> Self {
        AppError::Session(error.to_string())
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Migration(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::GitHub(_) => StatusCode::BAD_GATEWAY,
            AppError::Configuration(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Environment(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Session(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
        }
    }
}

impl From<backoff::Error<octocrab::Error>> for AppError {
    fn from(error: backoff::Error<octocrab::Error>) -> Self {
        match error {
            backoff::Error::Permanent(e) => AppError::GitHub(e),
            backoff::Error::Transient {
                err,
                retry_after: _,
            } => AppError::GitHub(err),
        }
    }
}

impl From<actix_web::error::PayloadError> for AppError {
    fn from(err: actix_web::error::PayloadError) -> Self {
        AppError::BadRequest(format!("Payload error: {}", err))
    }
}
