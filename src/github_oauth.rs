use crate::error::AppError;
use crate::user::upsert_user;
use crate::AppState;
use actix_session::Session;
use actix_web::http::header::ContentType;
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use log::{error, info};
use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use oauth2::{reqwest::async_http_client, AuthorizationCode, TokenResponse};
use octocrab::Octocrab;
use std::env;
use url::Url;

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

#[get("/login")]
async fn login(oauth_client: web::Data<BasicClient>) -> impl Responder {
    let (auth_url, _csrf_state) = oauth_client
        .authorize_url(oauth2::CsrfToken::new_random)
        .add_extra_param("prompt", "consent")
        .url();

    HttpResponse::Found()
        .append_header(("Location", auth_url.to_string()))
        .finish()
}

#[get("/logout")]
pub async fn logout(session: Session) -> impl Responder {
    // Clear the session
    session.clear();

    HttpResponse::Found()
        .append_header(("Location", "/login"))
        .finish()
}

#[get("/auth/github/callback")]
async fn github_callback(
    req: HttpRequest,
    oauth_client: web::Data<BasicClient>,
    session: Session,
    state: web::Data<AppState>,
) -> impl Responder {
    // Extract the authorization code from the request
    let code = req.query_string();
    let parsed_url = Url::parse(&format!("http://localhost?{}", code)).unwrap();
    let code = parsed_url
        .query_pairs()
        .find(|(key, _)| key == "code")
        .map(|(_, value)| value.into_owned());

    let code = match code {
        Some(code) => code,
        None => return HttpResponse::BadRequest().body("Missing authorization code"),
    };

    // Exchange the code for an access token
    let token_result = oauth_client
        .exchange_code(AuthorizationCode::new(code.to_string()))
        .request_async(async_http_client)
        .await;

    match token_result {
        Ok(token) => {
            let access_token = token.access_token().secret();
            // Store the access token in the session
            if let Err(e) = session.insert("github_token", access_token) {
                return HttpResponse::InternalServerError()
                    .body(format!("Failed to store token: {}", e));
            }

            // Create a new Octocrab instance with the user's access token
            let user_octocrab = Octocrab::builder()
                .personal_token(access_token.to_string())
                .build()
                .expect("Failed to create user Octocrab instance");

            // Fetch the user's information
            match user_octocrab.current().user().await {
                Ok(user) => {
                    // Add or update user in the database
                    if let Err(e) = upsert_user(
                        &state.db_pool,
                        &user.login.to_string(),
                        user.email.as_deref(),
                    )
                    .await
                    {
                        error!("Failed to upsert user in database: {}", e);
                        return HttpResponse::InternalServerError()
                            .body("Failed to update user information");
                    }

                    // Optionally store user information in the session
                    if let Err(e) = session.insert("github_user", &user) {
                        info!("Failed to store user info in session: {}", e);
                    }
                }
                Err(e) => {
                    info!("Failed to fetch user information: {}", e);
                }
            }

            // Redirect to a protected route or homepage
            HttpResponse::Found()
                .append_header(("Location", "/protected"))
                .finish()
        }
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Failed to exchange code: {}", e))
        }
    }
}

#[get("/protected")]
async fn protected(session: Session) -> impl Responder {
    match session.get::<String>("github_token") {
        Ok(Some(token)) => {
            // Create a new Octocrab instance with the user's access token
            let user_octocrab = Octocrab::builder()
                .personal_token(token.to_string())
                .build()
                .expect("Failed to create user Octocrab instance");

            // Fetch the user's information
            match user_octocrab.current().user().await {
                Ok(user) => {
                    // Log the user information
                    info!("Authenticated user: {:?}", user);

                    HttpResponse::Ok()
                        .content_type(ContentType::html())
                        .body(format!(
                            r#"
                            <!DOCTYPE html>
                            <html lang="en">
                            <head>
                                <meta charset="UTF-8">
                                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                                <title>Protected Content</title>
                            </head>
                            <body>
                                <h1>Protected Content</h1>
                                <h2>User Information</h2>
                                <p>Username: {}</p>
                                <p>Email: {}</p>
                                <img src="{}" alt="Avatar" style="width:100px;height:100px;">
                                <br>
                                <a href='/logout'>Logout</a>
                            </body>
                            </html>
                            "#,
                            user.login,
                            user.email.unwrap_or_else(|| "N/A".to_string()),
                            user.avatar_url
                        ))
                }
                Err(e) => {
                    info!("Failed to fetch user information: {}", e);
                    HttpResponse::InternalServerError().body("Failed to fetch user information")
                }
            }
        }
        Ok(None) => HttpResponse::Found()
            .append_header(("Location", "/login"))
            .finish(),
        Err(_) => HttpResponse::InternalServerError().body("Failed to get session data"),
    }
}
