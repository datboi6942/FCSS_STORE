use sqlx;
use chrono::Utc;
use log::{info, error};
use bcrypt::{hash, DEFAULT_COST};
use sqlx::SqlitePool;

pub async fn setup(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    info!("Starting database setup");
    
    // First, enable foreign key constraints
    info!("Enabling foreign key constraints");
    match sqlx::query("PRAGMA foreign_keys = ON;")
        .execute(pool)
        .await 
    {
        Ok(_) => info!("Foreign key constraints enabled"),
        Err(e) => {
            error!("Failed to enable foreign key constraints: {}", e);
            return Err(e);
        }
    }
    
    // Check if we already have data in the database
    let user_count = sqlx::query!("SELECT COUNT(*) as count FROM users")
        .fetch_one(pool)
        .await?
        .count;
    
    // If we already have users, skip data insertion
    if user_count > 0 {
        info!("Database already contains data, skipping sample data creation");
        return Ok(());
    }
    
    // Create tables in the correct order
    info!("Creating users table");
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY NOT NULL,
            username TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            role TEXT NOT NULL,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        "#
    )
    .execute(pool)
    .await?;
    
    info!("Creating products table");
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS products (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            price REAL NOT NULL,
            available BOOLEAN NOT NULL DEFAULT TRUE,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        "#
    )
    .execute(pool)
    .await?;
    
    info!("Creating orders table");
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
        )
        "#
    )
    .execute(pool)
    .await?;
    
    info!("Creating transactions table");
    sqlx::query(
        r#"
        DROP TABLE IF EXISTS transactions;
        CREATE TABLE transactions (
            id TEXT PRIMARY KEY NOT NULL,
            order_id TEXT NOT NULL,
            amount REAL NOT NULL,
            status TEXT NOT NULL,
            payment_method TEXT NOT NULL,
            session_id TEXT NOT NULL,
            currency TEXT NOT NULL,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (order_id) REFERENCES orders(id)
        )
        "#
    )
    .execute(pool)
    .await?;

    // Always add sample data for in-memory database since it starts empty
    info!("Adding sample data to in-memory database");
    
    // Create admin user with bcrypt password hash
    let admin_password_hash = match hash("admin123", DEFAULT_COST) {
        Ok(hash) => hash,
        Err(e) => {
            error!("Failed to hash admin password: {}", e);
            return Err(sqlx::Error::Protocol("Failed to hash password".into()));
        }
    };
    
    let user_password_hash = match hash("user123", DEFAULT_COST) {
        Ok(hash) => hash,
        Err(e) => {
            error!("Failed to hash user password: {}", e);
            return Err(sqlx::Error::Protocol("Failed to hash password".into()));
        }
    };
    
    let now = Utc::now();
    
    // Add admin user
    match sqlx::query!(
        "INSERT INTO users (id, username, password_hash, role, created_at) 
         VALUES (?, ?, ?, ?, ?)",
        "usr-admin1",
        "admin",
        admin_password_hash,
        "admin",
        now
    )
    .execute(pool)
    .await
    {
        Ok(_) => info!("Added admin user"),
        Err(e) => {
            error!("Failed to add admin user: {}", e);
            return Err(e);
        }
    }
    
    // Add regular user
    match sqlx::query!(
        "INSERT INTO users (id, username, password_hash, role, created_at) 
         VALUES (?, ?, ?, ?, ?)",
        "usr-user1",
        "user",
        user_password_hash,
        "user",
        now
    )
    .execute(pool)
    .await
    {
        Ok(_) => info!("Added regular user"),
        Err(e) => {
            error!("Failed to add regular user: {}", e);
            return Err(e);
        }
    }
    
    // Add sample products
    match sqlx::query!(
        "INSERT INTO products (id, name, description, price, available, created_at) 
         VALUES (?, ?, ?, ?, ?, ?)",
        "prod-1",
        "Premium Widget",
        "Our best-selling premium widget with all features",
        99.99,
        true,
        now
    )
    .execute(pool)
    .await
    {
        Ok(_) => info!("Added product 1"),
        Err(e) => {
            error!("Failed to add product 1: {}", e);
            return Err(e);
        }
    }
    
    match sqlx::query!(
        "INSERT INTO products (id, name, description, price, available, created_at) 
         VALUES (?, ?, ?, ?, ?, ?)",
        "prod-2",
        "Basic Widget",
        "Affordable widget with essential features",
        49.99,
        true,
        now
    )
    .execute(pool)
    .await
    {
        Ok(_) => info!("Added product 2"),
        Err(e) => {
            error!("Failed to add product 2: {}", e);
            return Err(e);
        }
    }
    
    // Add sample orders
    match sqlx::query!(
        "INSERT INTO orders (id, user_id, product_id, status, created_at) 
         VALUES (?, ?, ?, ?, ?)",
        "ord-1",
        "usr-user1",
        "prod-1",
        "completed",
        now
    )
    .execute(pool)
    .await
    {
        Ok(_) => info!("Added order 1"),
        Err(e) => {
            error!("Failed to add order 1: {}", e);
            return Err(e);
        }
    }
    
    match sqlx::query!(
        "INSERT INTO orders (id, user_id, product_id, status, created_at) 
         VALUES (?, ?, ?, ?, ?)",
        "ord-2",
        "usr-user1",
        "prod-2",
        "pending",
        now
    )
    .execute(pool)
    .await
    {
        Ok(_) => info!("Added order 2"),
        Err(e) => {
            error!("Failed to add order 2: {}", e);
            return Err(e);
        }
    }
    
    // Add sample transactions
    match sqlx::query!(
        r#"
        INSERT INTO transactions 
        (id, order_id, amount, status, payment_method, session_id, currency, created_at) 
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        "txn-sample1",
        "ord-1",
        29.99,
        "completed",
        "credit_card",
        "sess-sample1",
        "USD",
        now
    )
    .execute(pool)
    .await
    {
        Ok(_) => info!("Added sample transaction"),
        Err(e) => {
            error!("Failed to add sample transaction: {}", e);
            return Err(e);
        }
    }
    
    info!("Database setup complete");
    Ok(())
}

#[derive(Debug)]
pub struct TableCounts {
    pub users: i32,    // Changed from i64 to i32
    pub products: i32, // Changed from i64 to i32
    pub orders: i32,   // Changed from i64 to i32
}

pub async fn check_tables(pool: &sqlx::SqlitePool) -> TableCounts {
    let mut counts = TableCounts {
        users: 0,
        products: 0,
        orders: 0,
    };

    if let Ok(result) = sqlx::query!("SELECT COUNT(*) as count FROM users")
        .fetch_one(pool)
        .await
    {
        counts.users = result.count;
    }

    if let Ok(result) = sqlx::query!("SELECT COUNT(*) as count FROM products")
        .fetch_one(pool)
        .await
    {
        counts.products = result.count;
    }

    if let Ok(result) = sqlx::query!("SELECT COUNT(*) as count FROM orders")
        .fetch_one(pool)
        .await
    {
        counts.orders = result.count;
    }

    counts
}
