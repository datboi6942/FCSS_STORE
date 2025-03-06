// main.rs (top portion)
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

use actix_web::{web, App, HttpResponse, HttpServer, Responder, middleware::Logger};
use std::sync::{Mutex, Arc};
use actix_cors::Cors;
use sqlx::SqlitePool;
use dotenv::dotenv;
use actix_web::http::header;
use log;

pub struct AppState {
    pub db: SqlitePool,
    pub chat_history: Arc<Mutex<Vec<chat::ChatMessage>>>,
}

async fn index() -> impl Responder {
    let html_content = r#"
    <!DOCTYPE html>
    <html>
      <head>
        <meta charset="UTF-8">
        <title>Secure Store Test Interface</title>
        <style>
          body { font-family: Arial, sans-serif; margin: 40px; background: #f9f9f9; }
          h1 { color: #333; }
          ul { line-height: 1.6; }
          li { margin-bottom: 8px; }
        </style>
      </head>
      <body>
        <h1>Welcome to the Secure Store Test Interface</h1>
        <p>Available endpoints for testing:</p>
        <ul>
          <li><strong>POST</strong> <code>/auth/register</code> — Register a user</li>
          <li><strong>POST</strong> <code>/auth/login</code> — Login a user</li>
          <li><strong>POST</strong> <code>/order</code> — Create a new order</li>
          <li><strong>GET</strong> <code>/order/history?user_id=YOUR_USER_ID</code> — View order history for a user</li>
          <li><strong>GET</strong> <code>/admin</code> — Admin dashboard</li>
          <li><strong>GET</strong> <code>/products</code> — List available products</li>
          <li><strong>POST</strong> <code>/products</code> — Add a new product</li>
          <li><strong>POST</strong> <code>/payment/initiate</code> — Initiate a payment session</li>
          <li><strong>POST</strong> <code>/payment/verify</code> — Verify a payment</li>
          <li><strong>GET</strong> <code>/chat</code> — Connect to the chat (requires sign‑in)</li>
        </ul>
        <p>For POST endpoints, use tools like Postman or cURL with appropriate JSON payloads.</p>
      </body>
    </html>
    "#;
    
    HttpResponse::Ok().content_type("text/html").body(html_content)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();
    
    // Create database directory
    std::fs::create_dir_all("./data").unwrap_or_else(|e| {
        log::info!("Directory exists or error: {}", e);
    });
    
    // Give full permissions to the data directory
    #[cfg(unix)]
    std::process::Command::new("chmod")
        .args(&["777", "./data"])
        .output()
        .expect("Failed to chmod data directory");
    
    // Get database pool first
    let pool = db::create_pool().await;
    
    // Initialize database schema with the pool we just created
    log::info!("Setting up database schema...");
    match setup_db::setup(&pool).await {
        Ok(_) => log::info!("Database setup completed successfully"),
        Err(e) => {
            log::error!("Database setup failed: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other, 
                format!("Failed to setup database: {}", e)
            ));
        }
    }
    
    // Initialize chat history
    let chat_history = Arc::new(Mutex::new(Vec::new()));
    
    // Create app state
    let app_state = web::Data::new(AppState {
        db: pool,
        chat_history,
    });
    
    // Add auto-purge system
    let app_state_clone = app_state.clone();
    tokio::spawn(async move {
        auto_purge::start_auto_purge(app_state_clone).await;
    });

    // Start HTTP server
    log::info!("Starting HTTP server at http://127.0.0.1:8443");
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_origin("http://127.0.0.1:3000")
            .allowed_origin("http://localhost:5000")
            .allowed_origin("http://127.0.0.1:5000")
            .allowed_origin("http://localhost:5173") 
            .allowed_origin("http://127.0.0.1:5173") 
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![header::AUTHORIZATION, header::CONTENT_TYPE])
            .max_age(3600);
        
        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .app_data(app_state.clone())
            .route("/", web::get().to(index))
            .route("/health", web::get().to(|| async { HttpResponse::Ok().body("OK") }))
            // Auth routes
            .service(auth::init_routes())
            // Order routes
            .service(
                web::scope("/order")
                    .route("", web::post().to(orders::create_order))
                    .route("/status", web::post().to(orders::update_order_status))
                    .route("/history", web::get().to(orders::get_user_orders))
                    .route("/all", web::get().to(orders::order_history))
            )
            // Admin routes
            .service(admin::init_routes())
            // Product routes
            .service(
                web::scope("/products")
                    .route("", web::get().to(products::list_products))
                    .route("", web::post().to(products::add_product))
            )
            // Payment routes
            .service(
                web::scope("/payment")
                    .route("/initiate", web::post().to(payment::initiate_payment))
                    .route("/verify", web::post().to(payment::verify_payment))
            )
            // Chat routes
            .service(
                web::scope("/chat")
                    .route("", web::get().to(chat::chat_handler))
                    .route("/message", web::post().to(chat::post_message))
            )
    })
    .bind("0.0.0.0:8443")?
    .run()
    .await
}
