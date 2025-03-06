use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::env;
use log::info;

pub async fn create_pool() -> SqlitePool {
    // Load environment variables from .env file
    dotenv::dotenv().ok();
    
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:securestore.db".to_string());
    
    info!("Connecting to database at {}", database_url);
    
    SqlitePoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to create database pool")
}

pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    info!("Running database migrations");
    match sqlx::migrate!("./migrations").run(pool).await {
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!("Migration error: {}", e);
            Err(e.into())
        }
    }
}
