// main.rs (top portion)
mod types;
mod auth;
mod orders;
mod admin;
mod chat;
mod auto_purge;
mod payment;
mod session;
mod products;
mod db; // New database module
mod setup_db;
mod middleware;
mod cart;
mod monero;
mod monero_api;
mod monero_admin;
mod monero_wallet;
mod db_reset;  // Add at the top with other mod declarations
use secure_store::get_db_path;

use actix_web::{web, App, HttpResponse, HttpServer, Responder, middleware::Logger};
use std::sync::{Mutex, Arc};
use actix_cors::Cors;
use sqlx::{SqlitePool, Row};
use dotenv::dotenv;
use actix_web::http::header;
use log;
use std::collections::HashMap;
use crate::cart::{CartStore, Cart};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use actix_web::post;
use crate::monero::MoneroPaymentStore;
use actix_files;
mod payment_websocket;
use payment_websocket::{WebsocketConnections, payment_ws};

// Remove the derive and implement Clone manually
pub struct AppState {
    pub db: SqlitePool,
    pub chat_history: Arc<Mutex<Vec<chat::ChatMessage>>>,
    pub carts: Arc<CartStore>,  // Wrap in Arc
    pub monero_payments: Arc<MoneroPaymentStore>,  // Wrap in Arc
    pub ws_connections: Arc<Mutex<WebsocketConnections>>,
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            chat_history: self.chat_history.clone(),
            carts: self.carts.clone(),
            monero_payments: self.monero_payments.clone(),
            ws_connections: self.ws_connections.clone(),
        }
    }
}

async fn index() -> actix_web::Result<actix_files::NamedFile> {
    Ok(actix_files::NamedFile::open("./frontend/dist/index.html")?)
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CartItem {
    pub id: String,
    pub name: String,
    pub price: f64,
    pub image: String,
    pub quantity: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Order {
    pub order_id: String,
    pub items: Vec<CartItem>,
    pub total: f64,
}

#[post("/api/checkout")]
pub async fn checkout(cart_items: web::Json<Vec<CartItem>>) -> impl Responder {
    // Read the cart items from the request body
    let items = cart_items.into_inner();
    // Calculate the total order amount
    let total: f64 = items.iter().map(|item| item.price * (item.quantity as f64)).sum();

    // Create an order with a generated UUID
    let order = Order {
        order_id: Uuid::new_v4().to_string(),
        items,
        total,
    };

    // Here you might persist the order in a database.
    println!("New order received: {:?}", order);

    // Return a JSON response indicating success and include the order details.
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "order": order
    }))
}

#[post("/api/direct-checkout")]
pub async fn direct_checkout(
    app_state: web::Data<AppState>,
    checkout_data: web::Json<serde_json::Value>,
) -> impl Responder {
    println!("Received direct checkout request: {:?}", checkout_data);
    
    // Generate a unique order ID
    let order_id = Uuid::new_v4().to_string();
    
    // Extract total amount from checkout data or use a default
    let total_amount = checkout_data.get("total")
        .and_then(|v| v.as_f64())
        .unwrap_or(10.0); // Default to 10.0 if total is not provided
    
    // Create Monero payment request
    let payment = app_state.monero_payments.create_payment_usd(order_id.clone(), total_amount);
    
    // Return the checkout response
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "order_id": order_id,
        "payment": payment,
        "message": "Please send Monero to the provided address"
    }))
}

// Modify the existing setup_database_directly function
async fn setup_database_directly(pool: &SqlitePool) -> Result<(), std::io::Error> {
    log::info!("Setting up database directly");
    
    // Enable foreign keys
    match sqlx::query("PRAGMA foreign_keys = ON;")
        .execute(pool)
        .await {
        Ok(_) => log::info!("Enabled foreign keys"),
        Err(e) => {
            log::error!("Failed to enable foreign keys: {}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()));
        }
    }
    
    // Drop tables in reverse dependency order
    let tables = [
        "order_items",     // Drop child tables first
        "orders",
        "monero_payments",
        "products",
        "users",
        "transactions"
    ];

    for table in tables.iter() {
        log::info!("Dropping table if exists: {}", table);
        match sqlx::query(&format!("DROP TABLE IF EXISTS {}", table))
            .execute(pool)
            .await {
            Ok(_) => log::info!("Dropped table: {}", table),
            Err(e) => log::warn!("Error dropping table {}: {}", table, e),
        }
    }
    
    // Create all tables needed
    let create_tables = [
        // 1. Create monero_payments table
        r#"
        CREATE TABLE monero_payments (
            payment_id TEXT PRIMARY KEY,
            amount REAL NOT NULL,
            address TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )
        "#,
        
        // 2. Create orders table
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
        "#,
        
        // 3. Create order_items table
        r#"
        CREATE TABLE order_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            order_number TEXT NOT NULL,
            product_id TEXT NOT NULL,
            quantity INTEGER NOT NULL,
            price REAL NOT NULL,
            FOREIGN KEY (order_number) REFERENCES orders(id)
        )
        "#,
        
        // 4. Create users table
        r#"
        CREATE TABLE users (
            id TEXT PRIMARY KEY NOT NULL,
            username TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            role TEXT NOT NULL,
            created_at INTEGER NOT NULL
        )
        "#,
        
        // 5. Create products table
        r#"
        CREATE TABLE products (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            price REAL NOT NULL,
            available BOOLEAN NOT NULL DEFAULT TRUE,
            created_at INTEGER NOT NULL
        )
        "#,
        
        // 6. Create transactions table
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
    ];
    
    // Create tables in dependency order
    for (i, sql) in create_tables.iter().enumerate() {
        log::info!("Creating table #{}", i+1);
        match sqlx::query(sql).execute(pool).await {
            Ok(_) => log::info!("Successfully created table #{}", i+1),
            Err(e) => {
                log::error!("Failed to create table #{}: {}\nSQL: {}", i+1, e, sql);
                return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to create table: {}", e)));
            }
        }
    }
    
    // Verify tables were created
    match sqlx::query("SELECT name FROM sqlite_master WHERE type='table';")
        .fetch_all(pool)
        .await {
        Ok(tables) => {
            let table_names: Vec<String> = tables
                .iter()
                .map(|row| row.get::<String, _>("name"))
                .collect();
            log::info!("Created tables: {:?}", table_names);
        },
        Err(e) => log::error!("Error verifying tables: {}", e),
    }
    
    // Add sample data using dynamic SQL (bypassing compile-time checks)
    let now = chrono::Utc::now().timestamp();
    
    // Add sample user
    match sqlx::query("INSERT INTO users (id, username, password_hash, role, created_at) VALUES (?, ?, ?, ?, ?)")
        .bind("usr-user1")
        .bind("testuser")
        .bind("password123") // In a real app, this would be hashed
        .bind("user")
        .bind(now)
        .execute(pool)
        .await
    {
        Ok(_) => log::info!("Added sample user"),
        Err(e) => log::warn!("Failed to add sample user: {}", e)
    }
    
    // Add sample product
    match sqlx::query("INSERT INTO products (id, name, description, price, available, created_at) VALUES (?, ?, ?, ?, ?, ?)")
        .bind("prod-1")
        .bind("Test Product")
        .bind("A sample product for testing")
        .bind(99.99)
        .bind(true)
        .bind(now)
        .execute(pool)
        .await
    {
        Ok(_) => log::info!("Added sample product"),
        Err(e) => log::warn!("Failed to add sample product: {}", e)
    }
    
    // Add sample monero payment
    let payment_id = format!("pay-{}", uuid::Uuid::new_v4().simple());
    
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
        Ok(_) => log::info!("Added sample monero payment"),
        Err(e) => log::warn!("Failed to add sample monero payment: {}", e)
    }
    
    // Add sample order
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
        Ok(_) => log::info!("Added sample order"),
        Err(e) => log::warn!("Failed to add sample order: {}", e)
    }
    
    // Add sample order item
    match sqlx::query(
        "INSERT INTO order_items (order_number, product_id, quantity, price) 
         VALUES (?, ?, ?, ?)"
    )
        .bind("ord-1")  // matches the sample order id
        .bind("prod-1") // matches the sample product id
        .bind(1)        // quantity
        .bind(99.99)    // price
        .execute(pool)
        .await
    {
        Ok(_) => log::info!("Added sample order item"),
        Err(e) => log::warn!("Failed to add sample order item: {}", e)
    }
    
    log::info!("Database setup completed successfully");
    Ok(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    // Update the database connection string to use the correct path
    let db_url = format!("sqlite:{}", get_db_path());
    let pool = SqlitePool::connect(&db_url)
        .await
        .expect("Failed to create pool");

    // Reset and initialize database using the same pool
    log::info!("Resetting database...");
    if let Err(e) = setup_database_directly(&pool).await {
        log::error!("Direct database setup failed: {}", e);
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to set up database directly: {}", e)
        ));
    }
    log::info!("Direct database setup completed successfully");

    // Initialize chat history
    let chat_history = Arc::new(Mutex::new(Vec::new()));
    
    // Initialize cart store
    let carts = Mutex::new(HashMap::<String, Cart>::new());
    
    // Create WebSocket connections
    let ws_connections = Arc::new(Mutex::new(WebsocketConnections::new()));
    
    // Create app state with our pool
    let app_state = web::Data::new(AppState {
        db: pool,
        chat_history,
        carts: Arc::new(Mutex::new(HashMap::<String, Cart>::new())),  // Specify type parameters
        monero_payments: Arc::new(MoneroPaymentStore::new()),  // Wrap in Arc
        ws_connections,
    });
    
    // Update the app_state setting
    app_state.monero_payments.as_ref().set_app_state(&app_state);
    
    // Initialize the database schema
    match monero_api::ensure_monero_payment_schema(&app_state.db).await {
        Ok(_) => log::info!("Database schema initialized successfully"),
        Err(e) => log::warn!("Error initializing database schema: {}", e),
    }
    
    // Add auto-purge system
    let app_state_clone = app_state.clone();
    tokio::spawn(async move {
        auto_purge::start_auto_purge(app_state_clone).await;
    });

    // Start Monero payment checker
    let app_state_clone = app_state.clone();
    tokio::spawn(async move {
        monero_api::start_payment_checker(app_state_clone);
    });

    // Add the waiting delay before starting the server
    log::info!("Server starting, waiting for all components to initialize...");
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    log::info!("Server ready at http://127.0.0.1:5000");

    // Start HTTP server
    log::info!("Starting HTTP server at http://127.0.0.1:5000");
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_origin("http://127.0.0.1:3000")
            .allowed_origin("http://localhost:5000")
            .allowed_origin("http://127.0.0.1:5000")
            .allowed_origin("http://localhost:5173")
            .allowed_origin("http://127.0.0.1:5173")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec![header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT])
            .expose_headers(vec![header::CONTENT_TYPE])
            .supports_credentials()
            .max_age(3600);
        
        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .app_data(app_state.clone())
            .configure(orders::init_routes)
            .service(direct_checkout)
            .service(cart::checkout)
            .route("/", web::get().to(index))
            .route("/health", web::get().to(health_check))
            // Auth routes
            .service(auth::init_routes())
            // Admin routes
            .service(admin::init_routes())
            // Product routes
            .service(
                web::scope("/products")
                    .route("", web::get().to(products::list_products))
                    .route("", web::post().to(products::add_product))
                    .route("/purchase", web::post().to(products::purchase_product))
            )
            // Payment routes
            .service(
                web::scope("/payment")
                    .route("/initiate", web::post().to(payment::initiate_payment))
                    .route("/verify", web::post().to(payment::verify_payment))
                    .route("/crypto/confirm", web::post().to(payment::confirm_crypto_payment))
            )
            // Chat routes
            .service(
                web::scope("/chat")
                    .route("", web::get().to(chat::chat_handler))
                    .route("/message", web::post().to(chat::post_message))
            )
            // Cart routes
            .service(cart::init_routes())
            // Monero routes
            .service(monero_api::init_routes())
            // Monero admin routes
            .service(
                web::scope("/api")
                    .service(monero_admin::init_routes())
            )
            // WebSocket route
            .service(
                web::resource("/ws/payment/{order_id}")
                    .route(web::get().to(payment_ws))
            )
            // Serve static files from the frontend dist directory
            .service(actix_files::Files::new("/", "./frontend/public").index_file("index.html"))
            // Handle all other routes by returning index.html (for client-side routing)
            .default_service(web::get().to(index))
    })
    .bind("0.0.0.0:5000")?
    .run()
    .await
}
