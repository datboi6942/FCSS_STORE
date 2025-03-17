// src/db.rs
use sqlx::SqlitePool;
use log::{info, error};
use std::fs;
use std::path::Path;

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