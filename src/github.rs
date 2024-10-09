use crate::auth::Claims;
use crate::error::AppError;
use actix_web::HttpMessage;
use actix_web::HttpRequest;
use log::error;
use octocrab::Octocrab;

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
