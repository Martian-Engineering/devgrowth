mod db;

use sqlx::postgres::PgPool;
use std::sync::Arc;

struct AppState {
    db_pool: PgPool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool: PgPool = db::create_pool().await?;

    // Create the application state
    let app_state = Arc::new(AppState { db_pool: pool });

    // Use the app_state in your application...
    run_app(app_state).await?;

    Ok(())
}

async fn run_app(state: Arc<AppState>) -> Result<(), Box<dyn std::error::Error>> {
    // Example of using the connection pool
    let result = sqlx::query("SELECT 1").fetch_one(&state.db_pool).await?;

    println!("Query result: {:?}", result);

    // Rest of your application logic...
    Ok(())
}
