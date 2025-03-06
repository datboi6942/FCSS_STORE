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

use actix_web::{web, App, HttpResponse, HttpServer, Responder, middleware::Logger};
use std::sync::{Mutex, Arc};
use actix_cors::Cors;
use sqlx::SqlitePool;
use dotenv::dotenv;

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
    dotenv().ok(); // Load .env file
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    // Setup database
    if let Err(e) = setup_db::setup().await {
        log::error!("Failed to set up database: {}", e);
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Database setup failed"));
    }
    
    // Create database pool
    let pool = db::create_pool().await;
    
    // Create application state with SQLite pool
    let app_data = web::Data::new(AppState {
        db: pool,
        chat_history: Arc::new(Mutex::new(Vec::new())),
    });

    HttpServer::new(move || {
        // Configure CORS
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();
            
        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .app_data(app_data.clone())
            // Routes remain the same
            .route("/", web::get().to(index))
            .service(web::scope("/auth")
                .route("/register", web::post().to(auth::register))
                .route("/login", web::post().to(auth::login)))
            .service(web::scope("/order")
                .route("", web::post().to(orders::create_order))
                .route("/history", web::get().to(orders::order_history)))
            .service(web::scope("/payment")
                .route("/initiate", web::post().to(payment::initiate_payment))
                .route("/verify", web::post().to(payment::verify_payment)))
            .service(web::scope("/products")
                .route("", web::get().to(products::list_products))
                .route("", web::post().to(products::add_product)))
            .route("/admin", web::get().to(admin::admin_panel))
            .service(chat::init_routes())
    })
    .bind("127.0.0.1:8443")?
    .run()
    .await
}
