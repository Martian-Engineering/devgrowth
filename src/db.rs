use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;

#[derive(Debug, Deserialize)]
struct DatabaseConfig {
    host: String,
    port: u16,
    name: String,
}

#[derive(Debug, Deserialize)]
struct Settings {
    database: DatabaseConfig,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name("config"))
            .add_source(Environment::with_prefix("app"))
            .build()?;

        s.try_deserialize()
    }
}

pub async fn create_pool() -> Result<PgPool, sqlx::Error> {
    // Load .env file
    dotenv::dotenv().ok();

    // Load configuration
    let settings = Settings::new().expect("Failed to load settings");

    // Get database credentials from environment variables
    let username = env::var("DB_USER").expect("DB_USER must be set");
    let password = env::var("DB_PASS").expect("DB_PASS must be set");

    // Construct the database URL
    let database_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        username, password, settings.database.host, settings.database.port, settings.database.name
    );

    // Create the connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
