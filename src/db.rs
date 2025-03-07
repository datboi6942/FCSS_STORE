// src/db.rs
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use log::info;

pub async fn create_pool() -> SqlitePool {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:src/secure_store.db".to_string());
    
    // Retry database connection multiple times
    let max_retries = 5;
    let retry_delay = std::time::Duration::from_secs(1);
    
    for attempt in 1..=max_retries {
        match SqlitePoolOptions::new()
            .max_connections(10)  // Increase connection pool size
            .min_connections(2)   // Keep min connections ready
            .connect(&db_url)
            .await
        {
            Ok(pool) => {
                log::info!("Database connection established (attempt {}/{})", attempt, max_retries);
                return pool;
            }
            Err(e) => {
                log::warn!("Failed to connect to database (attempt {}/{}): {}", attempt, max_retries, e);
                if attempt < max_retries {
                    tokio::time::sleep(retry_delay).await;
                } else {
                    panic!("Failed to connect to database after {} attempts: {}", max_retries, e);
                }
            }
        }
    }
    
    unreachable!();
}

// We won't use this function anymore as setup_db handles everything
pub async fn init_db() -> Result<(), sqlx::Error> {
    info!("Initialization delegated to setup_db");
    Ok(())
}