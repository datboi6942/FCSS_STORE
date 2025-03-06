use sqlx::{Connection, SqliteConnection};
use std::fs;
use std::process::Command;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating database...");
    
    // Create database file if it doesn't exist
    if !std::path::Path::new("securestore.db").exists() {
        fs::File::create("securestore.db")?;
    }

    // Make sure it's writable
    #[cfg(unix)]
    {
        let _ = Command::new("chmod")
            .args(&["666", "securestore.db"])
            .output();
    }

    // Connect and create tables
    let mut conn = SqliteConnection::connect("sqlite:securestore.db").await?;

    println!("Creating users table...");
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY NOT NULL,
            username TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            role TEXT NOT NULL,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )
    .execute(&mut conn)
    .await?;

    println!("Creating products table...");
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS products (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            price REAL NOT NULL,
            available BOOLEAN NOT NULL DEFAULT TRUE,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )
    .execute(&mut conn)
    .await?;

    println!("Creating orders table...");
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS orders (
            id TEXT PRIMARY KEY NOT NULL,
            user_id TEXT NOT NULL,
            product_id TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users(id),
            FOREIGN KEY (product_id) REFERENCES products(id)
        );
        "#,
    )
    .execute(&mut conn)
    .await?;

    println!("Creating transactions table...");
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS transactions (
            id TEXT PRIMARY KEY NOT NULL,
            order_id TEXT NOT NULL,
            amount REAL NOT NULL,
            status TEXT NOT NULL,
            payment_method TEXT NOT NULL,
            session_id TEXT, 
            currency TEXT,
            method TEXT,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (order_id) REFERENCES orders(id)
        );
        "#,
    )
    .execute(&mut conn)
    .await?;

    // Add sample data
    println!("Adding sample data...");
    sqlx::query(
        r#"
        INSERT OR IGNORE INTO users (id, username, password_hash, role, created_at) 
        VALUES ('usr-1', 'admin', '$2a$12$5VxU9q4jF9pt2I/q1XOBCu2jC1MlJJeZ7GylP4OyVjaZmztIE5iHa', 'admin', CURRENT_TIMESTAMP);
        "#,
    )
    .execute(&mut conn)
    .await?;

    println!("Database setup completed successfully!");
    Ok(())
} 