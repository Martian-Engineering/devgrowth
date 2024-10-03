use crate::job_queue::JobQueue;
use actix_files as fs;
use actix_files::NamedFile;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::cookie::Key;
use actix_web::{middleware::Logger, web, App, HttpServer};
use github_oauth::{create_client, github_callback, login, logout, protected};
use log::info;
use oauth2::basic::BasicClient;
use octocrab::Octocrab;
use sqlx::postgres::PgPool;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

mod commit;
mod db;
mod error;
mod github_oauth;
mod job_processor;
mod job_queue;
mod repository;
mod user;

use repository::{
    create_repository, get_repository_ga, get_repository_metadata, list_repositories,
    sync_repository,
};

pub struct AppState {
    pub db_pool: PgPool,
    pub octocrab: Arc<Octocrab>,
    pub job_queue: Arc<JobQueue>,
    pub oauth_client: BasicClient,
}

async fn index() -> Result<NamedFile, actix_web::Error> {
    let path: PathBuf = "src/frontend/index.html".parse().unwrap();
    Ok(NamedFile::open(path)?)
}

async fn repository_page() -> Result<NamedFile, actix_web::Error> {
    let path: PathBuf = "src/frontend/repository.html".parse().unwrap();
    Ok(NamedFile::open(path)?)
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let pool: PgPool = db::create_pool().await.expect("Failed to create pool");

    // Create the Octocrab instance
    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN not set");
    let octocrab = Arc::new(
        octocrab::Octocrab::builder()
            .personal_token(token)
            .build()
            .expect("Failed to create Octocrab instance"),
    );

    let job_queue = JobQueue::new();

    tokio::spawn(job_processor::process_jobs(
        job_queue.clone(),
        octocrab.clone(),
        pool.clone(),
    ));

    let oauth_client = create_client().expect("Failed to create OAuth client");

    // Create the application state
    let app_state = web::Data::new(AppState {
        db_pool: pool,
        octocrab,
        job_queue,
        oauth_client: oauth_client.clone(),
    });

    info!("Starting server at http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .app_data(web::Data::new(oauth_client.clone()))
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                Key::from(&[0; 64]), // TODO: use a real secret key in production
            ))
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .service(fs::Files::new("/static", "src/frontend").show_files_listing())
            .service(login)
            .service(github_callback)
            .service(protected)
            .service(logout)
            .route("/", web::get().to(index))
            .route("/repository/{owner}/{name}", web::get().to(repository_page))
            .route("/api/repositories", web::get().to(list_repositories))
            .route("/api/repositories", web::post().to(create_repository))
            .route(
                "/api/repositories/{owner}/{name}",
                web::put().to(sync_repository),
            )
            .route(
                "/api/repositories/{owner}/{name}",
                web::get().to(get_repository_metadata),
            )
            .route(
                "/api/repositories/{owner}/{name}/ga",
                web::get().to(get_repository_ga),
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
