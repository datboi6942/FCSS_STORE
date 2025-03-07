use sqlx;
use chrono::Utc;
use log::{info, error};
use bcrypt::{hash, DEFAULT_COST};
use sqlx::SqlitePool;
use uuid::Uuid;

pub async fn setup(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    info!("Starting database setup");
    
    // First, enable foreign key constraints
    info!("Enabling foreign key constraints");
    sqlx::query("PRAGMA foreign_keys = ON;")
        .execute(pool)
        .await?;
    
    // Drop existing tables
    info!("Dropping existing tables");
    for table in &["transactions", "orders", "monero_payments", "products", "users"] {
        match sqlx::query(&format!("DROP TABLE IF EXISTS {}", table))
            .execute(pool)
            .await
        {
            Ok(_) => info!("Dropped table: {}", table),
            Err(e) => {
                error!("Failed to drop table {}: {}", table, e);
                // Continue anyway since the table might not exist
            }
        }
    }
    
    // Create monero_payments table FIRST
    info!("Creating monero_payments table");
    match sqlx::query(
        r#"
        CREATE TABLE monero_payments (
            payment_id TEXT PRIMARY KEY,
            amount REAL NOT NULL,
            address TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )
        "#
    )
    .execute(pool)
    .await {
        Ok(_) => info!("Created monero_payments table"),
        Err(e) => {
            error!("Failed to create monero_payments table: {}", e);
            return Err(e);
        }
    }
    
    // Create orders table SECOND (depends on monero_payments)
    info!("Creating orders table");
    match sqlx::query(
        r#"
        CREATE TABLE orders (
            id TEXT PRIMARY KEY,
            user_id TEXT,
            payment_id TEXT UNIQUE,
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
            updated_at INTEGER NOT NULL,
            FOREIGN KEY(payment_id) REFERENCES monero_payments(payment_id)
        )
        "#
    )
    .execute(pool)
    .await {
        Ok(_) => info!("Created orders table"),
        Err(e) => {
            error!("Failed to create orders table: {}", e);
            return Err(e);
        }
    }
    
    // Create other tables
    info!("Creating users table");
    sqlx::query(
        r#"
        CREATE TABLE users (
            id TEXT PRIMARY KEY NOT NULL,
            username TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            role TEXT NOT NULL,
            created_at INTEGER NOT NULL
        )
        "#
    )
    .execute(pool)
    .await?;
    
    info!("Creating products table");
    sqlx::query(
        r#"
        CREATE TABLE products (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            price REAL NOT NULL,
            available BOOLEAN NOT NULL DEFAULT TRUE,
            created_at INTEGER NOT NULL
        )
        "#
    )
    .execute(pool)
    .await?;
    
    info!("Creating transactions table");
    sqlx::query(
        r#"
        CREATE TABLE transactions (
            id TEXT PRIMARY KEY NOT NULL,
            order_id TEXT NOT NULL,
            amount REAL NOT NULL,
            status TEXT NOT NULL,
            payment_method TEXT NOT NULL,
            session_id TEXT NOT NULL,
            currency TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            FOREIGN KEY (order_id) REFERENCES orders(id)
        )
        "#
    )
    .execute(pool)
    .await?;

    // Add sample data
    let now = Utc::now().timestamp();

    // Add sample admin user
    info!("Adding admin user");
    let admin_password = hash("admin123", DEFAULT_COST).unwrap();
    sqlx::query!(
        r#"
        INSERT INTO users (id, username, password_hash, role, created_at)
        VALUES (?, ?, ?, ?, ?)
        "#,
        "usr-admin1",
        "admin",
        admin_password,
        "admin",
        now
    )
    .execute(pool)
    .await?;

    // Add sample regular user
    info!("Adding regular user");
    let user_password = hash("user123", DEFAULT_COST).unwrap();
    sqlx::query!(
        r#"
        INSERT INTO users (id, username, password_hash, role, created_at)
        VALUES (?, ?, ?, ?, ?)
        "#,
        "usr-user1",
        "testuser",
        user_password,
        "user",
        now
    )
    .execute(pool)
    .await?;

    // Add sample products
    info!("Adding sample products");
    sqlx::query!(
        r#"
        INSERT INTO products (id, name, description, price, available, created_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
        "prod-1",
        "Premium Widget",
        "A high-quality widget for all your needs",
        199.99,
        true,
        now
    )
    .execute(pool)
    .await?;

    sqlx::query!(
        r#"
        INSERT INTO products (id, name, description, price, available, created_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
        "prod-2",
        "Basic Widget",
        "An affordable widget for everyday use",
        49.99,
        true,
        now
    )
    .execute(pool)
    .await?;

    // Add sample monero payment
    info!("Adding sample monero payment");
    let payment_id = format!("pay-{}", Uuid::new_v4().simple());
    match sqlx::query("INSERT INTO monero_payments (payment_id, amount, address, status, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(&payment_id)
        .bind(99.99)
        .bind("44AFFq5kSiGBoZ4NMDwYtN18obc8AemS33DBLWs3H7otXft3XjrpDtQGv7SqSsaBYBb98uNbr2VBBEt7f2wfn3RVGQBEP3A")
        .bind("Pending")
        .bind(now)
        .bind(now)
        .execute(pool)
        .await
    {
        Ok(_) => info!("Added sample monero payment"),
        Err(e) => {
            error!("Failed to add sample monero payment: {}", e);
            // Continue anyway, don't return error
        }
    }

    // Add sample order
    info!("Adding sample order");
    match sqlx::query("INSERT INTO orders (id, user_id, payment_id, status, shipping_name, shipping_address, shipping_city, shipping_state, shipping_zip, shipping_country, shipping_email, total_amount, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind("ord-1")
        .bind("usr-user1")
        .bind(&payment_id)
        .bind("Pending")
        .bind("Test User")
        .bind("123 Test St")
        .bind("Test City")
        .bind("Test State")
        .bind("12345")
        .bind("Test Country")
        .bind("test@example.com")
        .bind(99.99)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await
    {
        Ok(_) => info!("Added sample order"),
        Err(e) => {
            error!("Failed to add sample order: {}", e);
            // Continue anyway, don't return error
        }
    }

    // Add sample transaction
    info!("Adding sample transaction");
    let txn_id = format!("txn-{}", Uuid::new_v4().simple());
    let session_id = format!("sess-{}", Uuid::new_v4().simple());

    sqlx::query!(
        r#"
        INSERT INTO transactions (
            id, order_id, amount, status, payment_method, session_id, currency, created_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        txn_id,
        "ord-1",
        99.99,
        "Pending",
        "monero",
        session_id,
        "XMR",
        now
    )
    .execute(pool)
    .await?;

    info!("Database setup complete");
    Ok(())
}

#[derive(Debug)]
pub struct TableCounts {
    pub users: i32,
    pub products: i32,
    pub orders: i32,
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

// Add this function to validate the database schema
pub async fn validate_schema(pool: &SqlitePool) -> Result<(), String> {
    // Check monero_payments table
    match sqlx::query("SELECT payment_id, amount, address, status, created_at, updated_at FROM monero_payments LIMIT 1")
        .execute(pool)
        .await
    {
        Ok(_) => info!("monero_payments table validated"),
        Err(e) => return Err(format!("monero_payments table validation failed: {}", e)),
    }

    // Check orders table
    match sqlx::query("SELECT id, user_id, payment_id, status, shipping_name, shipping_address, shipping_city, shipping_state, shipping_zip, shipping_country, shipping_email, total_amount, created_at, updated_at FROM orders LIMIT 1")
        .execute(pool)
        .await
    {
        Ok(_) => info!("orders table validated"),
        Err(e) => return Err(format!("orders table validation failed: {}", e)),
    }

    Ok(())
}
