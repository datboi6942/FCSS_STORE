// src/admin.rs
use actix_web::{web, HttpResponse, Responder, HttpRequest};
use crate::session;
use crate::AppState;
use sqlx;
use sqlx::Row;
use serde_json;

// Existing admin dashboard endpoint.
pub async fn admin_dashboard(req: HttpRequest) -> impl Responder {
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            // Expect token format "Bearer <token>"
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                if session::verify_jwt(token) {
                    return HttpResponse::Ok().body("Admin Dashboard: Welcome, Admin!");
                }
            }
        }
    }
    HttpResponse::Unauthorized().body("Access denied")
}

// New endpoint to list all orders (admin-only).
pub async fn get_all_orders(data: web::Data<crate::AppState>, req: HttpRequest) -> impl Responder {
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                if session::verify_jwt(token) {
                    // Use query instead of query! to have better control
                    let result = sqlx::query(
                        "SELECT id, user_id, product_id, status, created_at FROM orders"
                    )
                    .fetch_all(&data.db)
                    .await;
                    
                    match result {
                        Ok(rows) => {
                            // Convert to a serializable format (Vec of hashmaps)
                            let orders: Vec<serde_json::Value> = rows.iter().map(|row| {
                                let id: &str = row.get("id");
                                let user_id: &str = row.get("user_id");
                                let product_id: &str = row.get("product_id");
                                let status: &str = row.get("status");
                                
                                serde_json::json!({
                                    "id": id,
                                    "user_id": user_id, 
                                    "product_id": product_id,
                                    "status": status,
                                    "created_at": row.get::<chrono::NaiveDateTime, _>("created_at").to_string()
                                })
                            }).collect();
                            
                            return HttpResponse::Ok().json(orders);
                        },
                        Err(e) => {
                            log::error!("Failed to fetch orders: {}", e);
                            return HttpResponse::InternalServerError().json(
                                serde_json::json!({"error": "Failed to fetch orders"})
                            );
                        }
                    }
                }
            }
        }
    }
    HttpResponse::Unauthorized().body("Access denied")
}

pub async fn admin_panel(_data: web::Data<AppState>) -> impl Responder {
    let html = r#"
    <!DOCTYPE html>
    <html>
      <head>
        <title>Admin Panel</title>
        <style>
          body { font-family: Arial, sans-serif; margin: 40px; }
          h1 { color: #333; }
        </style>
      </head>
      <body>
        <h1>Admin Panel</h1>
        <p>This is a placeholder for the admin panel.</p>
        <p>In a production application, this would require proper authentication.</p>
      </body>
    </html>
    "#;
    
    HttpResponse::Ok().content_type("text/html").body(html)
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin")
            .route("", web::get().to(admin_dashboard))
            .route("/orders", web::get().to(get_all_orders))
            .route("/panel", web::get().to(admin_panel))
    );
}
