use sqlx::sqlite::SqlitePoolOptions;
use std::fs;
use std::path::Path;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use the absolute path to the database
    let db_path = "/home/john/FCSS_Store/FCSS_STORE/data/secure_store.db";
    println!("Database path: {}", db_path);
    
    // Ensure the directory exists
    let dir_path = Path::new(db_path).parent().unwrap();
    println!("Ensuring directory exists: {}", dir_path.display());
    fs::create_dir_all(dir_path)?;
    
    // Make database URL from the absolute path
    let db_url = format!("sqlite:{}", db_path);
    println!("Database URL: {}", db_url);
    
    // Connect to the database
    println!("Connecting to database...");
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;
    
    // Create tables if they don't exist
    println!("Creating tables...");
    
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            role TEXT NOT NULL,
            created_at INTEGER NOT NULL
        )"
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS products (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            price REAL NOT NULL,
            available INTEGER NOT NULL,
            created_at INTEGER NOT NULL
        )"
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS orders (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            product_id TEXT NOT NULL,
            quantity INTEGER NOT NULL,
            total_price REAL NOT NULL,
            status TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            FOREIGN KEY(user_id) REFERENCES users(id),
            FOREIGN KEY(product_id) REFERENCES products(id)
        )"
    )
    .execute(&pool)
    .await?;

    println!("Database initialized successfully!");

    Ok(())
} 