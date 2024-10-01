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
