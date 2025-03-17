// Remove unused functions and structs
// pub async fn setup(...) { ... }
// pub struct TableCounts { ... }
// pub async fn check_tables(...) { ... }
// pub async fn validate_schema(...) { ... }

use sqlx::SqlitePool;
use std::fs;
use std::path::Path;
use crate::get_db_path;

pub async fn initialize_database() -> Result<SqlitePool, sqlx::Error> {
    // Create data directory if it doesn't exist
    let db_path = get_db_path();
    let data_dir = Path::new(&db_path).parent().unwrap_or(Path::new("."));
    
    println!("Creating directory: {}", data_dir.display());
    fs::create_dir_all(data_dir).expect("Failed to create data directory");

    // Set directory permissions to 777 on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(data_dir, fs::Permissions::from_mode(0o777))
            .expect("Failed to set data directory permissions");
    }

    // Create an empty database file with proper permissions
    println!("Creating database at: {}", db_path);
    fs::write(&db_path, "").expect("Failed to create database file");
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&db_path, fs::Permissions::from_mode(0o666))
            .expect("Failed to set database file permissions");
    }

    let database_url = format!("sqlite:{}", db_path);
    println!("Connecting to database at: {}", database_url);

    // Create the database pool
    let pool = SqlitePool::connect(&database_url).await?;

    // Enable foreign keys
    sqlx::query("PRAGMA foreign_keys = ON;")
        .execute(&pool)
        .await?;

    // Create users table
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
    println!("✅ Created users table");
    
    // Create products table
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
    println!("✅ Created products table");
    
    // Create orders table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS orders (
            id TEXT PRIMARY KEY,
            user_id TEXT,
            payment_id TEXT,
            status TEXT NOT NULL,
            shipping_name TEXT NOT NULL,
            shipping_address TEXT NOT NULL,
            shipping_city TEXT NOT NULL,
            shipping_state TEXT NOT NULL,
            shipping_zip TEXT NOT NULL,
            shipping_country TEXT NOT NULL,
            shipping_email TEXT NOT NULL,
            total_amount REAL NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )"
    )
    .execute(&pool)
    .await?;
    println!("✅ Created orders table");
    
    // Create order_items table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS order_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            order_number TEXT NOT NULL,
            product_id TEXT NOT NULL,
            quantity INTEGER NOT NULL,
            price REAL NOT NULL,
            FOREIGN KEY (order_number) REFERENCES orders(id)
        )"
    )
    .execute(&pool)
    .await?;
    println!("✅ Created order_items table");
    
    // Create monero_payments table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS monero_payments (
            payment_id TEXT PRIMARY KEY,
            order_id TEXT,
            amount REAL NOT NULL,
            address TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )"
    )
    .execute(&pool)
    .await?;
    println!("✅ Created monero_payments table");
    
    // Create addresses table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS addresses (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            name TEXT NOT NULL,
            address TEXT NOT NULL,
            city TEXT NOT NULL,
            state TEXT NOT NULL,
            zip TEXT NOT NULL,
            country TEXT NOT NULL,
            is_default BOOLEAN NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id)
        )"
    )
    .execute(&pool)
    .await?;
    println!("✅ Created addresses table");
    
    // Create an admin user for testing
    let now = chrono::Utc::now().timestamp();
    sqlx::query(
        "INSERT OR IGNORE INTO users (id, username, password_hash, role, created_at) 
         VALUES (?, ?, ?, ?, ?)"
    )
    .bind("usr-admin")
    .bind("admin")
    .bind("$2a$12$K9oSWH9P1lnSCNw6imJAWOkXReJLQNZ3ug7H8lbYOqhR5ptXdFdQe") // hash for 'admin123'
    .bind("admin")
    .bind(now)
    .execute(&pool)
    .await?;
    println!("✅ Created admin user (username: admin, password: admin123)");
    
    println!("🎉 Database setup complete!");

    Ok(pool)
}
