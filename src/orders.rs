use actix_web::{web, HttpResponse, Responder, HttpRequest};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{Utc, DateTime};
use crate::AppState;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Order {
    pub id: String,
    pub user_id: String,
    pub product_id: String,
    pub product: Option<String>, // For displaying product name in responses
    pub status: String,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub created_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Deserialize)]
pub struct OrderInput {
    pub user_id: String,
    pub product_id: String,
}

/// Endpoint for creating a new order.
/// The status is set to "pending" by default.
pub async fn create_order(
    order_data: web::Json<OrderInput>,
    state: web::Data<AppState>,
) -> impl Responder {
    let order = order_data.into_inner();
    
    // Validate user exists
    let user_exists = sqlx::query!("SELECT id FROM users WHERE id = ?", order.user_id)
        .fetch_optional(&state.db)
        .await;
    
    match user_exists {
        Ok(Some(_)) => {/* User exists */},
        Ok(None) => {
            return HttpResponse::BadRequest().json(
                serde_json::json!({"error": "User not found"})
            );
        }
        Err(e) => {
            log::error!("Database error: {}", e);
            return HttpResponse::InternalServerError().json(
                serde_json::json!({"error": "Error validating user"})
            );
        }
    }
    
    // Validate product exists
    let product_exists = sqlx::query!("SELECT id FROM products WHERE id = ?", order.product_id)
        .fetch_optional(&state.db)
        .await;
    
    match product_exists {
        Ok(Some(_)) => {/* Product exists */},
        Ok(None) => {
            return HttpResponse::BadRequest().json(
                serde_json::json!({"error": "Product not found"})
            );
        }
        Err(e) => {
            log::error!("Database error: {}", e);
            return HttpResponse::InternalServerError().json(
                serde_json::json!({"error": "Error validating product"})
            );
        }
    }
    
    // Create order
    let order_id = Uuid::new_v4().to_string();
    let now = Utc::now();
    
    let result = sqlx::query!(
        "INSERT INTO orders (id, user_id, product_id, status, created_at) VALUES (?, ?, ?, ?, ?)",
        order_id,
        order.user_id,
        order.product_id,
        "pending", // Default status
        now
    )
    .execute(&state.db)
    .await;
    
    match result {
        Ok(_) => {
            HttpResponse::Created().json(
                serde_json::json!({
                    "message": "Order created successfully",
                    "order_id": order_id
                })
            )
        }
        Err(e) => {
            log::error!("Failed to create order: {}", e);
            HttpResponse::InternalServerError().json(
                serde_json::json!({"error": "Failed to create order"})
            )
        }
    }
}

/// Structure for updating an order's status.
#[derive(Deserialize)]
pub struct OrderStatusUpdate {
    pub order_id: String,
    pub new_status: String,
}

/// Admin endpoint to update an order's status.
/// Checks for the dummy Authorization header ("Bearer dummy_jwt").
pub async fn update_order_status(data: web::Data<crate::AppState>, req: HttpRequest, payload: web::Json<OrderStatusUpdate>) -> impl Responder {
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str == "Bearer dummy_jwt" {
                // Simplified authentication (use proper JWT verification in production)
                
                let update = payload.into_inner();
                
                // Update order status in database
                match sqlx::query!(
                    "UPDATE orders SET status = ? WHERE id = ?",
                    update.new_status,
                    update.order_id
                )
                .execute(&data.db)
                .await {
                    Ok(_) => return HttpResponse::Ok().json(
                        serde_json::json!({"status": "updated", "order_id": update.order_id})
                    ),
                    Err(e) => {
                        log::error!("Failed to update order: {}", e);
                        return HttpResponse::InternalServerError().json(
                            serde_json::json!({"error": "Failed to update order"})
                        );
                    }
                };
            }
        }
    }
    
    HttpResponse::Unauthorized().body("Access denied")
}

/// Query structure to capture the user_id from the query string.
#[derive(Deserialize)]
pub struct GetOrdersQuery {
    pub user_id: String,
}

/// Endpoint to get all orders associated with a specific user.
pub async fn get_user_orders(data: web::Data<crate::AppState>, query: web::Query<GetOrdersQuery>) -> impl Responder {
    let orders = match sqlx::query!(
        "SELECT id, user_id, product_id, status, created_at FROM orders WHERE user_id = ?",
        query.user_id
    )
    .fetch_all(&data.db)
    .await {
        Ok(orders) => orders,
        Err(e) => {
            log::error!("Failed to fetch orders: {}", e);
            return HttpResponse::InternalServerError().json(
                serde_json::json!({"error": "Failed to fetch orders"})
            );
        }
    };
    
    // Convert the database rows to Order objects
    let orders: Vec<Order> = orders.iter().map(|row| Order {
        id: row.id.clone(),
        user_id: row.user_id.clone(),
        product_id: row.product_id.clone(),
        product: None,
        status: row.status.clone(),
        created_at: Some(DateTime::<Utc>::from_naive_utc_and_offset(
            row.created_at, 
            Utc
        )),
    }).collect();
    
    HttpResponse::Ok().json(orders)
}

/// Register all order-related routes.
pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/order")
            .route("", web::post().to(create_order))
            .route("/history", web::get().to(get_user_orders))
            .route("/update", web::post().to(update_order_status))
    );
}

pub async fn order_history(
    _req: HttpRequest,
    query: web::Query<GetOrdersQuery>,
    data: web::Data<AppState>,
) -> impl Responder {
    let user_id = &query.user_id;
    
    // Query orders from the database including product name
    let rows = sqlx::query!(
        r#"
        SELECT o.id, o.user_id, o.product_id, o.status, o.created_at, p.name as product_name 
        FROM orders o
        JOIN products p ON o.product_id = p.id
        WHERE o.user_id = ?
        ORDER BY o.created_at DESC
        "#,
        user_id
    )
    .fetch_all(&data.db)
    .await;
    
    match rows {
        Ok(rows) => {
            let orders: Vec<Order> = rows.iter().map(|row| Order {
                id: row.id.clone(),
                user_id: row.user_id.clone(),
                product_id: row.product_id.clone(),
                product: Some(row.product_name.clone()),
                status: row.status.clone(),
                created_at: Some(Utc::now()), // Use actual timestamp from DB in production
            }).collect();
            
            HttpResponse::Ok().json(orders)
        },
        Err(e) => {
            log::error!("Failed to fetch orders: {}", e);
            HttpResponse::InternalServerError().json(
                serde_json::json!({"error": "Failed to fetch orders"})
            )
        }
    }
}
