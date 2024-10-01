use crate::job_queue::JobQueue;
use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use log::info;
use octocrab::Octocrab;
use sqlx::postgres::PgPool;
use std::io;
use std::sync::Arc;

mod commit;
mod db;
mod error;
mod job_processor;
mod job_queue;
mod repository;

use repository::{create_repository, sync_repository};

pub struct AppState {
    pub db_pool: PgPool,
    pub octocrab: Arc<Octocrab>,
    pub job_queue: Arc<JobQueue>,
}

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
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

    // Create the application state
    let app_state = web::Data::new(AppState {
        db_pool: pool,
        octocrab,
        job_queue,
    });

    info!("Starting server at http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .app_data(app_state.clone())
            .route("/", web::get().to(index))
            .route(
                "/repositories/{owner}/{name}",
                web::put().to(sync_repository),
            )
            .route("/repositories", web::post().to(create_repository))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
