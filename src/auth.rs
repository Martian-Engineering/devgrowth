use actix_session::Session;
use actix_web::{cookie::Cookie, HttpResponse, Responder};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, info};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub name: String,
    pub email: String,
    pub exp: usize,
    pub iat: usize,
    pub id: i64,
    pub access_token: String,
    pub db_id: Option<i32>,
}

unsafe impl Send for Claims {}

pub fn validate_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let decoding_key = DecodingKey::from_secret(std::env::var("JWT_SECRET").unwrap().as_bytes());
    let validation = Validation::new(Algorithm::HS256);

    match decode::<Claims>(token, &decoding_key, &validation) {
        Ok(token_data) => {
            // Check if the token has expired
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as usize;
            if token_data.claims.exp < now {
                Err(jsonwebtoken::errors::Error::from(
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature,
                ))
            } else {
                Ok(token_data.claims)
            }
        }
        Err(e) => {
            error!("Error decoding token: {:?}", e);
            Err(e)
        }
    }
}

pub async fn logout(session: Session) -> impl Responder {
    info!("Logging out user!");
    session.purge();
    HttpResponse::Ok()
        .cookie(
            Cookie::build("auth_token", "")
                .path("/")
                .max_age(actix_web::cookie::time::Duration::seconds(0))
                .http_only(true)
                .finish(),
        )
        .finish()
}
