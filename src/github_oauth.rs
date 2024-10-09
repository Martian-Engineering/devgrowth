use crate::error::AppError;
use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use serde::{Deserialize, Serialize};
use std::env;

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
