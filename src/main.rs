use crate::job_queue::JobQueue;
use crate::middleware::{AuthMiddleware, SessionLogger};
// use crate::middleware::AuthMiddleware;
use actix_cors::Cors;
use actix_files as fs;
use actix_files::NamedFile;
use actix_session::Session;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::cookie::Key;
use actix_web::http::header;
use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer};
use error::AppError;
use github_oauth::{create_client, github_callback, login, logout, protected};
use log::info;
use oauth2::basic::BasicClient;
use octocrab::Octocrab;
use sqlx::postgres::PgPool;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

mod account;
mod collection;
mod commit;
mod db;
mod error;
mod github_oauth;
mod job_processor;
mod job_queue;
mod middleware;
mod repository;

use collection::{
    add_repository_to_collection, create_collection, delete_collection, get_collection,
    get_collections, remove_repository_from_collection, update_collection,
};
use repository::{
    create_repository, get_repository_ga, get_repository_metadata, list_repositories,
    sync_repository,
};

pub struct AppState {
    pub db_pool: PgPool,
    pub job_queue: Arc<JobQueue>,
    pub oauth_client: BasicClient,
}

impl AppState {
    pub fn get_github_client(&self, session: &Session) -> Result<Octocrab, AppError> {
        let token = session
            .get::<String>("github_token")
            .map_err(|_| AppError::Unauthorized("No GitHub token found".into()))?
            .ok_or_else(|| AppError::Unauthorized("No GitHub token found".into()))?;

        info!("GitHub token: {}", token);
        Octocrab::builder()
            .personal_token(token)
            .build()
            .map_err(|e| AppError::GitHub(e))
    }
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
    let job_queue = JobQueue::new();

    tokio::spawn(job_processor::process_jobs(job_queue.clone(), pool.clone()));

    let oauth_client = create_client().expect("Failed to create OAuth client");

    // Create the application state
    let app_state = web::Data::new(AppState {
        db_pool: pool,
        job_queue,
        oauth_client: oauth_client.clone(),
    });

    info!("Starting server at http://localhost:8080");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
            .allowed_header(header::CONTENT_TYPE)
            .max_age(3600);

        App::new()
            .wrap(SessionLogger)
            .wrap(cors)
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
            .service(
                web::scope("/api")
                    .wrap(AuthMiddleware)
                    .default_service(web::to(|| HttpResponse::NotFound()))
                    .service(
                        web::scope("/repositories")
                            .route("", web::get().to(list_repositories))
                            .route("", web::post().to(create_repository))
                            .service(
                                web::resource("/{owner}/{name}")
                                    .route(web::put().to(sync_repository))
                                    .route(web::get().to(get_repository_metadata)),
                            )
                            .route("/{owner}/{name}/ga", web::get().to(get_repository_ga)),
                    )
                    .service(
                        web::scope("/collections")
                            .route("", web::post().to(create_collection))
                            .route("", web::get().to(get_collections))
                            .service(
                                web::resource("/{collection_id}")
                                    .route(web::get().to(get_collection))
                                    .route(web::put().to(update_collection))
                                    .route(web::delete().to(delete_collection)),
                            )
                            .route(
                                "/{collection_id}/repositories",
                                web::post().to(add_repository_to_collection),
                            )
                            .route(
                                "/{collection_id}/repositories/{repository_id}",
                                web::delete().to(remove_repository_from_collection),
                            ), // .route(
                               //     "/{collection_id}/growth-accounting",
                               //     web::get().to(get_collection_growth_accounting),
                               // ),
                    ),
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
