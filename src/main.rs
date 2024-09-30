use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use log::info;
use sqlx::postgres::PgPool;
use std::io;
use std::sync::Arc;

mod db;

struct AppState {
    db_pool: PgPool,
}

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let pool: PgPool = db::create_pool().await.expect("Failed to create pool");

    // Create the application state
    let app_state = Arc::new(AppState { db_pool: pool });

    info!("Starting server at http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .app_data(web::Data::new(app_state.clone()))
            .route("/", web::get().to(index))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
