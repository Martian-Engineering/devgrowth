use crate::account::upsert_account;
use crate::error::AppError;
use crate::AppState;
use actix_web::Result as ActixResult;
use actix_web::{get, web, HttpRequest, HttpResponse};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use log::{error, info};
use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use oauth2::{reqwest::async_http_client, AuthorizationCode, TokenResponse};
use octocrab::Octocrab;
use serde::{Deserialize, Serialize};
use std::env;
use url::Url;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String, // subject (user id)
    exp: i64,    // expiration time
    iat: i64,    // issued at
}

pub fn create_client() -> Result<BasicClient, AppError> {
    let client_id =
        env::var("GITHUB_CLIENT_ID").map_err(|e| AppError::Environment(e.to_string()))?;
    let client_secret =
        env::var("GITHUB_CLIENT_SECRET").map_err(|e| AppError::Environment(e.to_string()))?;
    let redirect_uri =
        env::var("GITHUB_REDIRECT_URI").map_err(|e| AppError::Environment(e.to_string()))?;

    let oauth_client = BasicClient::new(
        ClientId::new(client_id.to_string()),
        Some(ClientSecret::new(client_secret.to_string())),
        AuthUrl::new("https://github.com/login/oauth/authorize".to_string()).unwrap(),
        Some(TokenUrl::new("https://github.com/login/oauth/access_token".to_string()).unwrap()),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_uri.to_string()).unwrap());

    Ok(oauth_client)
}

#[get("/auth/github/callback")]
async fn github_callback(
    req: HttpRequest,
    oauth_client: web::Data<BasicClient>,
    state: web::Data<AppState>,
) -> ActixResult<HttpResponse> {
    // Extract the authorization code from the request
    let code = req.query_string();
    let parsed_url = Url::parse(&format!("http://localhost?{}", code)).unwrap();
    let code = parsed_url
        .query_pairs()
        .find(|(key, _)| key == "code")
        .map(|(_, value)| value.into_owned());

    let code =
        code.ok_or_else(|| actix_web::error::ErrorBadRequest("Missing authorization code"))?;

    // Exchange the code for an access token
    let token_result = oauth_client
        .exchange_code(AuthorizationCode::new(code.to_string()))
        .request_async(async_http_client)
        .await
        .map_err(|e| {
            error!("Failed to exchange code: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to exchange code")
        })?;

    let access_token = token_result.access_token().secret();

    // Create a new Octocrab instance with the user's access token
    let user_octocrab = Octocrab::builder()
        .personal_token(access_token.to_string())
        .build()
        .map_err(|e| {
            error!("Failed to create Octocrab instance: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to create GitHub client")
        })?;

    // Fetch the user's information
    let user = user_octocrab.current().user().await.map_err(|e| {
        info!("Failed to fetch account information: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to fetch user information")
    })?;

    let account_id = upsert_account(
        &state.db_pool,
        &user.login.to_string(),
        user.email.as_deref(),
    )
    .await
    .map_err(|e| {
        error!("Failed to upsert account in database: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to update account information")
    })?;

    // Generate JWT
    let claims = Claims {
        sub: account_id.to_string(),
        exp: (Utc::now() + Duration::hours(24)).timestamp(),
        iat: Utc::now().timestamp(),
    };

    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    )
    .map_err(|e| {
        error!("Failed to generate JWT: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to generate token")
    })?;

    // Return the JWT to the client
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "token": token,
        "user": {
            "id": account_id,
            "login": user.login,
            "avatar_url": user.avatar_url,
        }
    })))
}
