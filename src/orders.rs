use actix_web::{web, HttpResponse, Responder, HttpRequest};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;
use crate::AppState;
use log::{info, error, warn};
use crate::auth;
use sqlx::Row;

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
pub struct CreateOrderData {
    pub user_id: String,
    pub product_id: String,
}

#[derive(Deserialize)]
pub struct GetOrdersQuery {
    pub user_id: String,
}

/// Endpoint for creating a new order.
/// The status is set to "pending" by default.
pub async fn create_order(
    order_data: web::Json<CreateOrderData>,
    state: web::Data<AppState>
) -> impl Responder {
    let order = order_data.into_inner();
    
    info!("Creating order for user_id: {}, product_id: {}", 
          order.user_id, order.product_id);
    
    // Validate user exists
    let user_exists = sqlx::query!("SELECT id FROM users WHERE id = ?", order.user_id)
        .fetch_optional(&state.db)
        .await;
    
    match user_exists {
        Ok(Some(_)) => info!("User {} exists", order.user_id),
        Ok(None) => {
            warn!("User {} not found", order.user_id);
            return HttpResponse::BadRequest().json(
                serde_json::json!({"error": "User not found"})
            );
        }
        Err(e) => {
            error!("Database error checking user: {}", e);
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
        Ok(Some(_)) => info!("Product {} exists", order.product_id),
        Ok(None) => {
            warn!("Product {} not found", order.product_id);
            return HttpResponse::BadRequest().json(
                serde_json::json!({"error": "Product not found"})
            );
        }
        Err(e) => {
            error!("Database error checking product: {}", e);
            return HttpResponse::InternalServerError().json(
                serde_json::json!({"error": "Error validating product"})
            );
        }
    }
    
    // Create order
    let order_id = Uuid::new_v4().to_string();
    let now = Utc::now();
    
    info!("Inserting order with ID: {}", order_id);
    
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
            info!("Order created successfully: {}", order_id);
            HttpResponse::Created().json(
                serde_json::json!({
                    "id": order_id,
                    "user_id": order.user_id,
                    "product_id": order.product_id,
                    "status": "pending",
                    "created_at": now
                })
            )
        }
        Err(e) => {
            error!("Failed to create order: {}", e);
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
    // Get the order ID and new status from the payload
    let order_id = &payload.order_id;
    let new_status = &payload.new_status;
    
    // Verify JWT token (reuse the auth middleware)
    match auth::validate_token(req) {
        Ok(claims) => {
            // Get user role from the token claims
            let user_id = &claims.sub;
            let user_role = match sqlx::query!(
                "SELECT role FROM users WHERE id = ?",
                user_id
            )
            .fetch_optional(&data.db)
            .await {
                Ok(Some(user)) => user.role,
                Ok(None) => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "User not found"})),
                Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("Database error: {}", e)})),
            };
            
            // Only admins can update order status
            if user_role != "admin" {
                return HttpResponse::Forbidden().json(serde_json::json!({"error": "Only admins can update order status"}));
            }
            
            // Update the order status
            match sqlx::query!(
                "UPDATE orders SET status = ? WHERE id = ?",
                new_status,
                order_id
            )
            .execute(&data.db)
            .await {
                Ok(_) => HttpResponse::Ok().json(serde_json::json!({"message": "Order status updated successfully"})),
                Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("Failed to update order status: {}", e)})),
            }
        }
        Err(e) => HttpResponse::Unauthorized().json(serde_json::json!({"error": format!("Invalid token: {}", e)})),
    }
}

/// Get all orders for a specific user
pub async fn get_user_orders(
    query: web::Query<GetOrdersQuery>,
    state: web::Data<AppState>
) -> impl Responder {
    info!("Getting orders for user_id: {}", query.user_id);
    
    match sqlx::query!(
        r#"
        SELECT o.id, o.user_id, u.username, o.product_id, 
               p.name as product_name, p.price, o.status, o.created_at
        FROM orders o
        JOIN users u ON o.user_id = u.id
        JOIN products p ON o.product_id = p.id
        WHERE o.user_id = ?
        ORDER BY o.created_at DESC
        "#,
        query.user_id
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(orders) => {
            if orders.is_empty() {
                info!("No orders found for user_id: {}", query.user_id);
            } else {
                info!("Found {} orders for user_id: {}", orders.len(), query.user_id);
            }
            
            let orders_json: Vec<serde_json::Value> = orders
                .iter()
                .map(|o| {
                    serde_json::json!({
                        "id": o.id,
                        "user_id": o.user_id,
                        "username": o.username,
                        "product_id": o.product_id,
                        "product": o.product_name,
                        "price": o.price,
                        "status": o.status,
                        "created_at": o.created_at
                    })
                })
                .collect();
            
            HttpResponse::Ok().json(orders_json)
        }
        Err(e) => {
            error!("Database error: {}", e);
            HttpResponse::InternalServerError().json(
                serde_json::json!({"error": "Failed to fetch orders"})
            )
        }
    }
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

pub async fn order_history(data: web::Data<crate::AppState>, req: HttpRequest) -> impl Responder {
    // Verify JWT token
    match auth::validate_token(req) {
        Ok(claims) => {
            let user_id = &claims.sub;
            
            // Get user role
            let user_role = match sqlx::query!(
                "SELECT role FROM users WHERE id = ?",
                user_id
            )
            .fetch_optional(&data.db)
            .await {
                Ok(Some(user)) => user.role,
                Ok(None) => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "User not found"})),
                Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("Database error: {}", e)})),
            };
            
            // Define the query format
            let query_string = if user_role == "admin" {
                format!(
                    r#"
                    SELECT o.id, o.user_id, u.username, o.product_id, 
                           p.name as product_name, p.price, o.status, o.created_at
                    FROM orders o
                    JOIN users u ON o.user_id = u.id
                    JOIN products p ON o.product_id = p.id
                    ORDER BY o.created_at DESC
                    "#
                )
            } else {
                format!(
                    r#"
                    SELECT o.id, o.user_id, u.username, o.product_id, 
                           p.name as product_name, p.price, o.status, o.created_at
                    FROM orders o
                    JOIN users u ON o.user_id = u.id
                    JOIN products p ON o.product_id = p.id
                    WHERE o.user_id = ?
                    ORDER BY o.created_at DESC
                    "#
                )
            };
            
            // Execute the appropriate query
            let orders = if user_role == "admin" {
                sqlx::query(&query_string)
                    .fetch_all(&data.db)
                    .await
            } else {
                sqlx::query(&query_string)
                    .bind(user_id)
                    .fetch_all(&data.db)
                    .await
            };
            
            match orders {
                Ok(rows) => {
                    // Convert rows to JSON manually
                    let mut orders_json = Vec::new();
                    for row in rows {
                        // Extract values from row
                        let id: &str = row.get("id");
                        let user_id: &str = row.get("user_id");
                        let username: &str = row.get("username");
                        let product_id: &str = row.get("product_id");
                        let product_name: &str = row.get("product_name");
                        let price: f64 = row.get("price");
                        let status: &str = row.get("status");
                        let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
                        
                        orders_json.push(serde_json::json!({
                            "id": id,
                            "user_id": user_id,
                            "username": username,
                            "product_id": product_id,
                            "product_name": product_name,
                            "price": price,
                            "status": status,
                            "created_at": created_at,
                        }));
                    }
                    
                    HttpResponse::Ok().json(orders_json)
                },
                Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("Failed to fetch orders: {}", e)})),
            }
        }
        Err(e) => HttpResponse::Unauthorized().json(serde_json::json!({"error": format!("Invalid token: {}", e)})),
    }
}

// Add this new function to create a test order for admin panel verification
pub async fn create_test_order(state: web::Data<AppState>) -> impl Responder {
    // Generate unique values to prove this is a new DB entry
    let order_id = format!("ord-test-{}", Uuid::new_v4().simple());
    let now = chrono::Utc::now();
    
    // Create a variable to hold the formatted string
    let status = format!("test-{}", now.timestamp());
    
    // Insert new test order
    let result = sqlx::query!(
        "INSERT INTO orders (id, user_id, product_id, status, created_at) VALUES (?, ?, ?, ?, ?)",
        order_id,
        "usr-user1",
        "prod-1",
        status,  // Use the variable instead of the format! call directly
        now
    )
    .execute(&state.db)
    .await;
    
    match result {
        Ok(_) => {
            info!("Test order created successfully: {}", order_id);
            HttpResponse::Created().json(
                serde_json::json!({
                    "id": order_id,
                    "user_id": "usr-user1",
                    "product_id": "prod-1",
                    "status": status,
                    "created_at": now
                })
            )
        }
        Err(e) => {
            error!("Failed to create test order: {}", e);
            HttpResponse::InternalServerError().json(
                serde_json::json!({"error": "Failed to create test order"})
            )
        }
    }
}
