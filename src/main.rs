use crate::account::get_profile_data;
use crate::auth::logout;
use crate::job_queue::JobQueue;
use crate::middleware::{AuthMiddleware, SessionLogger};
use actix_cors::Cors;
use actix_web::http::header;
use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer};
use log::info;
use sqlx::postgres::PgPool;
use std::io;
use std::sync::Arc;

mod account;
mod auth;
mod auth_utils;
mod collection;
mod commit;
mod db;
mod error;
mod github;
mod growth_accounting;
mod job_processor;
mod job_queue;
mod middleware;
mod repository;

use collection::{
    add_repository_to_collection, create_collection, delete_collection, get_collection,
    get_collection_growth_accounting, get_collections, remove_repository_from_collection,
    update_collection,
};
use repository::{
    create_repository, get_repository_ga, get_repository_metadata, list_repositories,
    sync_repository,
};

pub struct AppState {
    pub db_pool: PgPool,
    pub job_queue: Arc<JobQueue>,
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let pool: PgPool = db::create_pool().await.expect("Failed to create pool");
    let job_queue = JobQueue::new();

    tokio::spawn(job_processor::process_jobs(job_queue.clone(), pool.clone()));

    // Create the application state
    let app_state = web::Data::new(AppState {
        db_pool: pool.clone(),
        job_queue,
    });

    info!("Starting server at http://localhost:8080");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![
                header::AUTHORIZATION,
                header::ACCEPT,
                header::CONTENT_TYPE,
            ])
            .max_age(3600)
            .supports_credentials();

        let auth_middleware = AuthMiddleware::new(web::Data::new(pool.clone()));

        App::new()
            // .wrap(SessionLogger)
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .wrap(cors)
            .app_data(app_state.clone())
            .service(
                web::scope("/api")
                    .wrap(auth_middleware.clone())
                    .default_service(web::to(|req: actix_web::HttpRequest| {
                        log::error!("Unmatched request: {} {}", req.method(), req.path());
                        HttpResponse::NotFound()
                    }))
                    .service(web::scope("/auth").route("/logout", web::post().to(logout)))
                    .service(
                        web::scope("/github")
                            .route("/starred", web::get().to(github::get_starred_repositories)),
                    )
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
                                // web::post().to(|req: actix_web::HttpRequest, payload: web::Payload| {
                                //     log::error!("Received request at /collections/{{collection_id}}/repositories");
                                //     log::error!("Request path: {:?}", req.path());
                                //     log::error!("Request method: {:?}", req.method());
                                //     log::error!("Request headers: {:?}", req.headers());
                                //     add_repository_to_collection(req, payload)
                                // }),
                            )
                            .route(
                                "/{collection_id}/repositories/{repository_id}",
                                web::delete().to(remove_repository_from_collection),
                            )
                            .route(
                                "/{collection_id}/ga",
                                web::get().to(get_collection_growth_accounting),
                            ),
                    )
                    .service(
                        web::scope("/account").route("/profile", web::get().to(get_profile_data)),
                    ),
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
