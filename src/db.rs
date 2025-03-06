// src/db.rs
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use log::info;
use std::{fs, path::Path};

pub async fn create_pool() -> SqlitePool {
    info!("Creating file-based SQLite database pool");
    
    // Ensure the database directory exists
    let data_dir = "./data";
    fs::create_dir_all(data_dir).unwrap_or_else(|e| {
        info!("Directory exists or created: {}", e);
    });
    
    // Build the database path
    let db_path = Path::new(data_dir).join("secure_store.db");
    let db_path_str = db_path.to_str().unwrap();
    
    // Explicitly create an empty file if it doesn't exist
    if !db_path.exists() {
        fs::File::create(&db_path).expect("Failed to create database file");
        info!("Created new database file at {}", db_path_str);
    }
    
    // Set permissive permissions on Unix systems
    #[cfg(unix)]
    {
        use std::process::Command;
        let _ = Command::new("chmod")
            .args(&["666", db_path_str])
            .output();
    }
    
    // Connect with SQLite's create if not exists mode
    let database_url = format!("sqlite:{}", db_path_str);
    info!("Connecting to database at {}", database_url);
    
    SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create database pool")
}

// We won't use this function anymore as setup_db handles everything
pub async fn init_db() -> Result<(), sqlx::Error> {
    info!("Initialization delegated to setup_db");
    Ok(())
}