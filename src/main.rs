use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use log::{error, info};
use sqlx::postgres::PgPool;
use std::io;
use std::sync::Arc;

mod db;
mod github;
mod repository;

struct AppState {
    db_pool: PgPool,
    octocrab: octocrab::Octocrab,
}

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

async fn create_repository(
    state: web::Data<Arc<AppState>>,
    new_repo: web::Json<repository::NewRepository>,
) -> impl Responder {
    let new_repo = new_repo.into_inner();
    match github::repository_exists(&state.octocrab, &new_repo.owner, &new_repo.name).await {
        Ok(true) => match repository::create_repository(&state.db_pool, new_repo).await {
            Ok(repo) => HttpResponse::Created().json(repo),
            Err(e) => {
                if let Some(db_err) = e.as_database_error() {
                    if db_err
                        .constraint()
                        .map_or(false, |c| c == "repository_name_owner_key")
                    {
                        return HttpResponse::Conflict()
                            .body("Repository already exists in the database");
                    }
                }
                error!("Failed to create repository in database: {:?}", e);
                HttpResponse::InternalServerError().finish()
            }
        },
        Ok(false) => {
            error!("Repository does not exist on GitHub");
            HttpResponse::BadRequest().body("Repository does not exist on GitHub")
        }
        Err(e) => {
            error!("Failed to check repository existence: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let pool: PgPool = db::create_pool().await.expect("Failed to create pool");

    // Create the Octocrab instance
    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN not set");
    let octocrab = octocrab::Octocrab::builder()
        .personal_token(token)
        .build()
        .expect("Failed to create Octocrab instance");

    // Create the application state
    let app_state = Arc::new(AppState {
        db_pool: pool,
        octocrab: octocrab,
    });

    info!("Starting server at http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .app_data(web::Data::new(app_state.clone()))
            .route("/", web::get().to(index))
            .route("/repositories", web::post().to(create_repository))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
