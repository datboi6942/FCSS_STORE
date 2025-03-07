// src/db.rs
use sqlx::SqlitePool;
use log::{info, error};
use std::fs;
use std::path::Path;

async fn try_connect(url: &str, retries: u32) -> Result<SqlitePool, sqlx::Error> {
    let mut attempt = 0;
    let mut last_error = None;

    while attempt < retries {
        match SqlitePool::connect(url).await {
            Ok(pool) => {
                info!("Successfully connected to database on attempt {}", attempt + 1);
                return Ok(pool);
            }
            Err(e) => {
                error!("Failed to connect on attempt {}: {}", attempt + 1, e);
                last_error = Some(e);
                attempt += 1;
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    }

    Err(last_error.unwrap())
}

pub async fn create_pool() -> SqlitePool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:./data/secure_store.db".to_string());

    match try_connect(&database_url, 3).await {
        Ok(pool) => {
            // Enable foreign keys
            sqlx::query("PRAGMA foreign_keys = ON;")
                .execute(&pool)
                .await
                .expect("Failed to enable foreign keys");
            pool
        }
        Err(e) => panic!("Failed to create database pool after retries: {}", e),
    }
}

// We won't use this function anymore as setup_db handles everything
pub async fn init_db() -> Result<(), sqlx::Error> {
    info!("Initialization delegated to setup_db");
    Ok(())
}

pub async fn initialize_database() -> Result<SqlitePool, sqlx::Error> {
    // Create data directory if it doesn't exist
    let data_dir = Path::new("./data");
    fs::create_dir_all(data_dir).expect("Failed to create data directory");

    // Set directory permissions to 777 on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(data_dir, fs::Permissions::from_mode(0o777))
            .expect("Failed to set data directory permissions");
    }

    // Delete the old database file if it exists
    let db_path = data_dir.join("secure_store.db");
    if db_path.exists() {
        fs::remove_file(&db_path).expect("Failed to remove old database file");
        info!("Deleted old database file");
    }

    // Create an empty database file with proper permissions
    fs::write(&db_path, "").expect("Failed to create database file");
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&db_path, fs::Permissions::from_mode(0o666))
            .expect("Failed to set database file permissions");
    }

    let database_url = format!("sqlite:{}", db_path.display());
    info!("Connecting to database at: {}", database_url);

    // Create the database pool
    let pool = SqlitePool::connect(&database_url).await?;

    // Enable foreign keys
    sqlx::query("PRAGMA foreign_keys = ON;")
        .execute(&pool)
        .await?;

    Ok(pool)
}