use crate::auth::Claims;
use crate::error::AppError;
use actix_web::HttpMessage;
use actix_web::HttpRequest;

pub fn get_account_id(req: &HttpRequest) -> Result<i32, AppError> {
    req.extensions()
        .get::<Claims>()
        .and_then(|claims| claims.db_id)
        .ok_or_else(|| AppError::Unauthorized("User not authenticated".into()))
}
