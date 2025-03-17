use actix_web::{web, HttpResponse, Responder, HttpRequest, get, post};
use serde::{Deserialize, Serialize};
use chrono::Utc;
use crate::AppState;
use log::{info, error};
use crate::auth;
use sqlx::Row;
use sqlx::SqlitePool;
use serde_json::json;
use rand::Rng;
use crate::types::ShippingInfo;
use sqlx::Column;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OrderStatus {
    Pending,
    AwaitingPayment,
    Paid,
    Shipped,
    Delivered,
    Completed,
    Cancelled
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub user_id: Option<String>,
    pub payment_id: String,
    pub status: OrderStatus,
    pub shipping_info: ShippingInfo,
    pub total_amount: f64,
    pub created_at: i64,
    pub updated_at: i64,
}

// Add OrderRecord struct for database queries
#[derive(sqlx::FromRow)]
struct OrderRecord {
    id: String,
    user_id: Option<String>,
    payment_id: String,
    status: String,
    shipping_name: String,
    shipping_address: String,
    shipping_city: String,
    shipping_state: String,
    shipping_zip: String,
    shipping_country: String,
    shipping_email: String,
    total_amount: f64,
    created_at: i64,
    updated_at: i64,
}

// Add conversion from OrderRecord to Order
impl From<OrderRecord> for Order {
    fn from(record: OrderRecord) -> Self {
        Order {
            id: record.id,
            user_id: record.user_id,
            payment_id: record.payment_id,
            status: match record.status.as_str() {
                "Pending" => OrderStatus::Pending,
                "AwaitingPayment" => OrderStatus::AwaitingPayment,
                "Paid" => OrderStatus::Paid,
                "Shipped" => OrderStatus::Shipped,
                "Delivered" => OrderStatus::Delivered,
                "Completed" => OrderStatus::Completed,
                "Cancelled" => OrderStatus::Cancelled,
                _ => OrderStatus::Pending, // Default case
            },
            shipping_info: ShippingInfo {
                name: record.shipping_name,
                address: record.shipping_address,
                city: record.shipping_city,
                state: record.shipping_state,
                zip: record.shipping_zip,
                country: record.shipping_country,
                email: record.shipping_email,
            },
            total_amount: record.total_amount,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

// Add this to your existing Order struct or create it if not present
#[derive(Debug, Serialize, Deserialize)]
pub struct NewOrder {
    pub order_number: String,
    pub user_id: Option<String>,
    pub shipping_info: ShippingInfo,
    pub items: Vec<OrderItem>,
    pub total_amount: f64,
    pub status: String,
    pub created_at: i64,
}

// Add this function to generate a unique order number
pub fn generate_order_number() -> String {
    let timestamp = Utc::now().timestamp();
    let random = rand::thread_rng().gen_range(1000..9999);
    format!("ORD-{}-{}", timestamp, random)
}

// Add OrderItem struct definition
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderItem {
    pub product_id: String,
    pub quantity: u32,
    pub price: f64,
}

// Update the create_order function signature to use the correct ShippingInfo type
pub async fn create_order(
    pool: &SqlitePool,
    shipping_info: ShippingInfo,
    items: Vec<OrderItem>,
    total_amount: f64,
    user_id: Option<String>,
) -> Result<Order, sqlx::Error> {
    log::info!("Creating order with shipping info: {:?}", shipping_info);
    
    let order_number = generate_order_number();
    let now = Utc::now().timestamp();
    let status = "Pending";

    // Start a transaction
    let mut tx = pool.begin().await?;

    // Insert the order
    sqlx::query!(
        "INSERT INTO orders (
            id, user_id, status, 
            shipping_name, shipping_address, shipping_city, 
            shipping_state, shipping_zip, shipping_country, 
            shipping_email, total_amount, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        order_number,
        user_id,
        status,
        shipping_info.name,
        shipping_info.address,
        shipping_info.city,
        shipping_info.state,
        shipping_info.zip,
        shipping_info.country,
        shipping_info.email,
        total_amount,
        now,
        now
    )
    .execute(&mut *tx)
    .await?;

    log::info!("Created order record, inserting {} items", items.len());

    // Insert order items
    for item in items.iter() {
        sqlx::query!(
            "INSERT INTO order_items (
                order_number, product_id, quantity, price
            ) VALUES (?, ?, ?, ?)",
            order_number,
            item.product_id,
            item.quantity,
            item.price
        )
        .execute(&mut *tx)
        .await?;
    }

    // Commit the transaction
    tx.commit().await?;

    log::info!("Order created successfully: {}", order_number);

    Ok(Order {
        id: order_number.clone(),
        user_id,
        payment_id: String::new(),
        status: OrderStatus::Pending,
        shipping_info,
        total_amount,
        created_at: now,
        updated_at: now,
    })
}

// Add this new endpoint for public order status lookup
#[get("/status/{id}")]
pub async fn get_order_status_endpoint(path: web::Path<String>, app_state: web::Data<AppState>) -> impl Responder {
    // Make it explicit that this endpoint is public - no auth check needed
    let order_id = path.into_inner();
    info!("Public order lookup for ID: {}", order_id);
    
    // Query to get order with limited information for public access
    let query = r#"
        SELECT id, status, total_amount, created_at, updated_at, 
               shipping_name, shipping_address, shipping_city, shipping_state,
               shipping_zip, shipping_country, payment_id
        FROM orders 
        WHERE id = ?
    "#;
    
    match sqlx::query(query)
        .bind(&order_id)
        .fetch_optional(&app_state.db)
        .await {
            Ok(Some(row)) => {
                info!("Order found: {}", order_id);
                // Return limited order information
                let order = json!({
                    "id": row.get::<String, _>("id"),
                    "status": row.get::<String, _>("status"),
                    "total_amount": row.get::<f64, _>("total_amount"),
                    "created_at": row.get::<i64, _>("created_at"),
                    "updated_at": row.get::<i64, _>("updated_at"),
                    "shipping_name": row.get::<String, _>("shipping_name"),
                    "shipping_address": row.get::<String, _>("shipping_address"),
                    "shipping_city": row.get::<String, _>("shipping_city"),
                    "shipping_state": row.get::<String, _>("shipping_state"),
                    "shipping_zip": row.get::<String, _>("shipping_zip"),
                    "shipping_country": row.get::<String, _>("shipping_country"),
                    "payment_id": row.get::<Option<String>, _>("payment_id")
                });
                
                HttpResponse::Ok().json(json!({
                    "success": true,
                    "order": order
                }))
            },
        Ok(None) => {
                info!("Order not found: {}", order_id);
                HttpResponse::NotFound().json(json!({
                    "success": false,
                    "error": "Order not found"
                }))
            },
            Err(e) => {
                error!("Database error looking up order status: {}", e);
                HttpResponse::InternalServerError().json(json!({
                    "success": false,
                    "error": "Failed to retrieve order information"
                }))
            }
        }
}

// Add this endpoint to get authenticated user's orders
#[get("/my-orders")]
pub async fn get_authenticated_user_orders(
    req: HttpRequest,
    app_state: web::Data<AppState>
) -> impl Responder {
    // Validate authentication
    let claims = match auth::validate_token(req) {
        Ok(claims) => claims,
        Err(e) => {
            return HttpResponse::Unauthorized().json(json!({
                "success": false,
                "error": e
            }));
        }
    };
    
    let user_id = claims.sub;
    
    // Special case for admin tokens
    if claims.role == "admin" {
        info!("Admin user accessing orders");
        // Return some dummy orders for admin users
        return HttpResponse::Ok().json(json!({
            "success": true,
            "count": 1,
            "orders": [{
                "id": "ORD-ADMIN-SAMPLE",
                "status": "Completed",
                "total_amount": 99.99,
                "created_at": chrono::Utc::now().timestamp(),
                "payment_id": "sample-payment-id",
                "monero_address": "44AFFq5kSiGBoZ4NMDwYtN18obc8AemS33DBLWs3H7otXft3XjrpDtQGv7SqSsaBYBb98uNbr2VBBEt7f2wfn3RVGQBEP3A" // Sample address
            }]
        }));
    }
    
    // Query orders with Monero addresses
    match sqlx::query(
        "SELECT o.id, o.status, o.total_amount, o.created_at, o.payment_id, 
         mp.address as monero_address, mp.status as payment_status
         FROM orders o
         LEFT JOIN monero_payments mp ON o.payment_id = mp.payment_id
         WHERE o.user_id = ?
         ORDER BY o.created_at DESC"
    )
    .bind(&user_id)
    .fetch_all(&app_state.db)
    .await {
        Ok(rows) => {
            let orders: Vec<serde_json::Value> = rows.iter().map(|row| {
                // Check if monero_address exists and print for debugging
                let address = row.get::<Option<String>, _>("monero_address");
                log::info!("Order {} has monero address: {:?}", 
                           row.get::<String, _>("id"), 
                           address);
                
                json!({
                    "id": row.get::<String, _>("id"),
                    "status": row.get::<String, _>("status"),
                    "total_amount": row.get::<f64, _>("total_amount"),
                    "created_at": row.get::<i64, _>("created_at"),
                    "payment_id": row.get::<String, _>("payment_id"),
                    "payment_status": row.get::<Option<String>, _>("payment_status"),
                    "monero_address": address
                })
            }).collect();
            
            HttpResponse::Ok().json(json!({
                "success": true,
                "count": orders.len(),
                "orders": orders
            }))
        },
        Err(e) => {
            error!("Database error: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": format!("Failed to fetch orders: {}", e)
            }))
        }
    }
}

// Near the top of your file, add this debugging route
#[get("/debug-orders")]
pub async fn debug_orders(app_state: web::Data<AppState>) -> impl Responder {
    // Print all orders in the database
    match sqlx::query("SELECT * FROM orders").fetch_all(&app_state.db).await {
        Ok(rows) => {
            let result: Vec<serde_json::Value> = rows.iter().map(|row| {
                json!({
                    "id": row.get::<String, _>("id"),
                    "status": row.get::<String, _>("status"),
                    "total_amount": row.get::<f64, _>("total_amount"),
                    "created_at": row.get::<i64, _>("created_at"),
                })
            }).collect();
            
            HttpResponse::Ok().json(json!({
                "success": true,
                "orders": result,
                "count": result.len()
            }))
        },
        Err(e) => {
            error!("Failed to fetch orders for debugging: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": format!("Database error: {}", e)
            }))
        }
    }
}

// Update the path for the debug token endpoint
#[get("/debug-token")]  // This may need to be changed to the correct path
pub async fn debug_token_endpoint(req: HttpRequest) -> impl Responder {
    // Extract and analyze the token
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                // Try to verify the token
                match crate::session::verify_jwt(token) {
                    Ok(claims) => {
                        return HttpResponse::Ok().json(json!({
                            "success": true,
                            "message": "Valid token",
                            "user_id": claims.sub,
                            "exp": claims.exp,
                            "raw_token": token
                        }));
                    },
                    Err(e) => {
                        return HttpResponse::BadRequest().json(json!({
                            "success": false,
                            "error": format!("Invalid token: {}", e),
                            "raw_token": token
                        }));
                    }
                }
            }
        }
    }
    
    HttpResponse::BadRequest().json(json!({
        "success": false,
        "error": "No token provided"
    }))
}

// Add this function to your orders.rs file
pub fn init_orders_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/orders")
            .service(debug_orders)
            .service(get_authenticated_user_orders)
            .service(get_order_status)
            .service(dump_order_data)
            .service(force_update_order_status)
            .service(diagnose_and_fix_status_mismatch)
    );
    
    // Register the debug-token endpoint at the root level
    cfg.service(debug_token_endpoint);
}

// Add this endpoint for order status lookup
#[get("/status/{order_id}")]
pub async fn get_order_status(
    order_id: web::Path<String>,
    app_state: web::Data<AppState>
) -> impl Responder {
    info!("Looking up status for order: {}", order_id);
    
    // Create a longer-lived value
    let order_id_str = order_id.as_ref().to_string();
    
    match sqlx::query!(
        r#"
        SELECT id, status, total_amount, created_at, updated_at, payment_id,
               shipping_name, shipping_address, shipping_city, shipping_state,
               shipping_zip, shipping_country
        FROM orders 
        WHERE id = ?
        "#,
        order_id_str  // Use the longer-lived value here
    )
    .fetch_optional(&app_state.db)
    .await {
        Ok(Some(order)) => {
            HttpResponse::Ok().json(json!({
                "success": true,
                "order": {
                    "id": order.id,
                    "status": order.status,
                    "total_amount": order.total_amount,
                    "created_at": order.created_at,
                    "updated_at": order.updated_at,
                    "payment_id": order.payment_id,
                    "shipping": {
                        "name": order.shipping_name,
                        "address": order.shipping_address,
                        "city": order.shipping_city,
                        "state": order.shipping_state,
                        "zip": order.shipping_zip,
                        "country": order.shipping_country
                    }
                }
            }))
        },
        Ok(None) => {
            HttpResponse::NotFound().json(json!({
                "success": false,
                "error": "Order not found"
            }))
        },
        Err(e) => {
            error!("Database error looking up order: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": "Failed to retrieve order"
            }))
        }
    }
}

// Add this endpoint to see the raw order data
#[get("/debug/dump-order/{order_id}")]
pub async fn dump_order_data(
    app_state: web::Data<AppState>,
    path: web::Path<String>
) -> impl Responder {
    let order_id = path.into_inner();
    
    // Query the order with all related payment data
    let order_data = sqlx::query(
        "SELECT o.*, mp.payment_id, mp.status as payment_status, mp.address 
         FROM orders o 
         LEFT JOIN monero_payments mp ON o.payment_id = mp.payment_id 
         WHERE o.id = ?"
    )
    .bind(&order_id)
    .fetch_optional(&app_state.db)
    .await;
    
    match order_data {
        Ok(Some(row)) => {
            // Convert to a map for easy viewing
            let mut data = serde_json::Map::new();
            
            // Extract all columns
            for i in 0..row.columns().len() {
                let col = row.columns().get(i).unwrap();
                let name = col.name();
                
                if let Ok(val) = row.try_get::<String, _>(i) {
                    data.insert(name.to_string(), serde_json::Value::String(val));
                } else if let Ok(val) = row.try_get::<i64, _>(i) {
                    data.insert(name.to_string(), serde_json::Value::Number(val.into()));
                } else if let Ok(val) = row.try_get::<f64, _>(i) {
                    // Convert to string to avoid precision issues
                    data.insert(name.to_string(), serde_json::Value::String(val.to_string()));
                } else if let Ok(val) = row.try_get::<Option<String>, _>(i) {
                    match val {
                        Some(v) => data.insert(name.to_string(), serde_json::Value::String(v)),
                        None => data.insert(name.to_string(), serde_json::Value::Null),
                    };
                }
            }
            
            HttpResponse::Ok().json(serde_json::Value::Object(data))
        },
        Ok(None) => {
            HttpResponse::NotFound().json(json!({
                "error": "Order not found"
            }))
        },
        Err(e) => {
            log::error!("Database error: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": format!("Database error: {}", e)
            }))
        }
    }
}

// Add a direct endpoint to force update order status
#[post("/admin/force-update-order/{order_id}/{status}")]
pub async fn force_update_order_status(
    app_state: web::Data<AppState>,
    path: web::Path<(String, String)>
) -> impl Responder {
    let (order_id, status) = path.into_inner();
    
    log::info!("🔨 Manually forcing order {} status to {}", order_id, status);
    
    // First update the order status
    match sqlx::query!(
        "UPDATE orders SET status = ? WHERE id = ?",
        status,
        order_id
    )
    .execute(&app_state.db)
    .await {
        Ok(_) => {
            // Then find the payment_id for this order
            let payment_query = sqlx::query!(
                "SELECT payment_id FROM orders WHERE id = ?",
                order_id
            )
            .fetch_optional(&app_state.db)
            .await;
            
            // Now update the payment status if we found a payment_id
            match payment_query {
                Ok(Some(record)) if record.payment_id.is_some() => {
                    let payment_id = record.payment_id.unwrap();
                    
                    // Update the payment status directly - SQLite doesn't support JOIN in UPDATE
                    let payment_update = sqlx::query!(
                        "UPDATE monero_payments SET status = ? WHERE payment_id = ?",
                        status,
                        payment_id
                    )
                    .execute(&app_state.db)
                    .await;
                    
                    if let Err(e) = payment_update {
                        log::warn!("Couldn't update payment status: {}", e);
            } else {
                        log::info!("Successfully updated payment status for payment_id: {}", payment_id);
                    }
                },
                Ok(_) => log::warn!("No payment_id found for order {}", order_id),
                Err(e) => log::error!("Error looking up payment_id: {}", e)
            }
            
            HttpResponse::Ok().json(json!({
                "success": true,
                "message": format!("Order {} status updated to {}", order_id, status)
            }))
        },
        Err(e) => {
            log::error!("Failed to update order status: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to update status: {}", e)
            }))
        }
    }
}

// Add a special diagnostic endpoint for debugging payment/order status issues
#[post("/fix-order-status-mismatch")]
pub async fn diagnose_and_fix_status_mismatch(
    app_state: web::Data<AppState>
) -> impl Responder {
    log::info!("🔍 RUNNING FULL DIAGNOSTIC OF PAYMENT STATUS MISMATCH");
    
    // Step 1: Find all confirmed payments
    let confirmed_payments = sqlx::query!(
        "SELECT payment_id, status, order_id FROM monero_payments WHERE status = 'Confirmed' OR status = 'confirmed'"
    )
    .fetch_all(&app_state.db)
    .await;
    
    if let Err(e) = &confirmed_payments {
        log::error!("Database error querying payments: {}", e);
        return HttpResponse::InternalServerError().json(json!({
            "success": false,
            "error": format!("Database error: {}", e)
        }));
    }
    
    let confirmed_payments = confirmed_payments.unwrap();
    log::info!("Found {} confirmed payments", confirmed_payments.len());
    
    let mut fixed_orders = 0;
    let mut mismatched_orders = 0;
    let mut missing_orders = 0;
    let mut diagnostic_info = Vec::new();
    
    // Step 2: Check each confirmed payment and ensure the linked order is also confirmed
    for payment in confirmed_payments {
        let payment_id = payment.payment_id.clone().unwrap_or_default();
        let payment_order_id = payment.order_id.clone().unwrap_or_default();
        
        if payment_id.is_empty() {
            log::warn!("Skipping payment with empty payment_id");
            continue;
        }
        
        // Get the associated order(s) using standard query to avoid type issues
        let query = if !payment_order_id.is_empty() {
            // Try to find by order_id in payment record
            "SELECT id, status FROM orders WHERE id = ?"
        } else {
            // Otherwise look up by payment_id
            "SELECT id, status FROM orders WHERE payment_id = ?"
        };
        
        let param = if !payment_order_id.is_empty() {
            payment_order_id.clone()
        } else {
            payment_id.clone()
        };
        
        let orders = sqlx::query(query)
            .bind(param)
            .fetch_all(&app_state.db)
            .await;
        
        if let Err(e) = &orders {
            log::error!("Error looking up order for payment {}: {}", payment_id, e);
            continue;
        }
        
        let orders = orders.unwrap();
        
        if orders.is_empty() {
            log::warn!("❌ No orders found for confirmed payment {}", payment_id);
            missing_orders += 1;
            diagnostic_info.push(json!({
                "type": "missing_order",
                "payment_id": payment_id,
                "payment_status": payment.status,
                "payment_order_id": payment_order_id
            }));
            continue;
        }
        
        // Check each associated order
        for row in orders {
            let order_id: String = row.get("id");
            let order_status: String = row.get("status");
            
            if order_status != "Confirmed" && order_status != "Completed" {
                log::warn!("⚠️ Found status mismatch! Payment {} is confirmed but order {} has status {}", 
                           payment_id, order_id, order_status);
                
                // This is a mismatch - let's fix it
                match sqlx::query!(
                    "UPDATE orders SET status = ? WHERE id = ?",
                    "Confirmed",
                    order_id
                )
                .execute(&app_state.db)
                .await {
                    Ok(_) => {
                        log::info!("✅ Successfully fixed order {} status to Confirmed", order_id);
                        fixed_orders += 1;
                        diagnostic_info.push(json!({
                            "type": "fixed",
                            "order_id": order_id,
                            "payment_id": payment_id,
                            "old_status": order_status,
                            "new_status": "Confirmed"
                        }));
                    },
        Err(e) => {
                        log::error!("Failed to update order status: {}", e);
                        diagnostic_info.push(json!({
                            "type": "error",
                            "order_id": order_id,
                            "payment_id": payment_id,
                            "error": format!("Database error: {}", e)
                        }));
                    }
                }
                
                mismatched_orders += 1;
            } else {
                log::info!("✓ Order {} status matches payment status ({})", order_id, order_status);
                diagnostic_info.push(json!({
                    "type": "ok",
                    "order_id": order_id,
                    "payment_id": payment_id,
                    "status": order_status
                }));
            }
        }
    }
    
    // Return comprehensive report
    HttpResponse::Ok().json(json!({
        "success": true,
        "diagnostics": {
            "mismatched_orders": mismatched_orders,
            "fixed_orders": fixed_orders,
            "missing_orders": missing_orders,
            "details": diagnostic_info
        }
    }))
}
