use log::info;
use std::fs;
use std::path::Path;
use sqlx::SqlitePool;

pub async fn reset_database() -> Result<SqlitePool, sqlx::Error> {
    info!("Performing complete database reset");
    
    // Create data directory if it doesn't exist
    let data_dir = Path::new("./data");
    fs::create_dir_all(data_dir).expect("Failed to create data directory");
    
    // Set directory permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(data_dir, fs::Permissions::from_mode(0o777));
    }
    
    // Delete the database file to ensure clean slate
    let db_path = data_dir.join("secure_store.db");
    if db_path.exists() {
        fs::remove_file(&db_path).expect("Failed to delete database file");
        info!("Deleted existing database file");
    }
    
    // Give a moment for the file system to catch up
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // Create a new empty database file
    fs::write(&db_path, "").expect("Failed to create empty database file");
    
    // Set file permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&db_path, fs::Permissions::from_mode(0o666));
    }
    
    // Connect to the database
    let database_url = format!("sqlite:{}", db_path.display());
    info!("Connecting to fresh database at: {}", database_url);
    
    let pool = SqlitePool::connect(&database_url).await?;
    
    // Enable foreign keys
    sqlx::query("PRAGMA foreign_keys = ON;")
        .execute(&pool)
        .await?;
    
    info!("Database reset completed successfully");
    Ok(pool)
} 