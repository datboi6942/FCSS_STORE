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
use middleware::JwtAuthentication;
use actix_web::http::header;

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
    // Load environment variables
    dotenv().ok();
    
    // Configure logging
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    
    log::info!("Starting up Secure Store server");
    
    // Database setup
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:secure_store.db".to_string());
    let db_pool = SqlitePool::connect(&db_url)
        .await
        .expect("Failed to connect to SQLite");
    
    // Uncomment if you want to setup database tables
    // setup_db::setup_database(&db_pool).await.expect("Failed to setup database");
    
    // Create app state
    let chat_history = Arc::new(Mutex::new(Vec::<chat::ChatMessage>::new()));
    let app_data = web::Data::new(AppState {
        db: db_pool.clone(),
        chat_history,
    });
    
    // Start auto-purge task in background
    let _ = auto_purge::spawn_purge_task(db_pool.clone());
    
    log::info!("Starting HTTP server at http://127.0.0.1:8443");
    
    HttpServer::new(move || {
        // Create CORS configuration inside the closure so each thread has its own
        let cors = Cors::default()
            .allowed_origin("http://localhost:5173")  // Svelte dev server
            .allowed_origin("http://localhost:3000")  // Alternative common dev port
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![
                header::AUTHORIZATION,
                header::CONTENT_TYPE,
                header::ACCEPT
            ])
            .supports_credentials()
            .max_age(3600);  // Cache preflight requests for 1 hour
            
        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .app_data(app_data.clone())
            // Public routes
            .route("/", web::get().to(index))
            .service(auth::init_routes())
            // Protected routes with middleware
            .service(
                web::scope("/order")
                    .wrap(JwtAuthentication)
                    .route("", web::post().to(orders::create_order))
                    .route("/history", web::get().to(orders::order_history))
            )
            .service(
                web::scope("/payment")
                    .wrap(JwtAuthentication)
                    .route("/initiate", web::post().to(payment::initiate_payment))
                    .route("/verify", web::post().to(payment::verify_payment))
            )
            .service(
                web::scope("/products")
                    .route("", web::get().to(products::list_products))
                    .service(
                        web::resource("")
                            .wrap(JwtAuthentication)
                            .route(web::post().to(products::add_product))
                    )
            )
            .service(
                web::scope("/admin")
                    .wrap(JwtAuthentication)
                    .route("", web::get().to(admin::admin_dashboard))
            )
            .service(chat::init_routes())
    })
    .bind(("127.0.0.1", 8443))?
    .run()
    .await
}
