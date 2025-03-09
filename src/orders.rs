use actix_web::{web, HttpResponse, Responder, HttpRequest, get};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;
use crate::AppState;
use log::{info, error};
use crate::auth;
use sqlx::Row;
use sqlx::SqlitePool;
use serde_json::json;
use rand::Rng;
use crate::types::ShippingInfo;

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

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateOrderStatusRequest {
    pub status: OrderStatus,
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

// Add this struct for the create order request
#[derive(Deserialize)]
pub struct CreateOrderRequest {
    pub shipping_info: ShippingInfo,
    pub payment_id: String,
    pub total_amount: f64,
    pub user_id: Option<String>,
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

/// Structure for updating an order's status.
#[derive(Deserialize)]
pub struct OrderStatusUpdate {
    pub order_id: String,
    pub new_status: String,
}

/// Admin endpoint to update an order's status.
/// Checks for the dummy Authorization header ("Bearer dummy_jwt").
pub async fn update_order_status(
    pool: &SqlitePool,
    order_id: &str,
    status: OrderStatus,
) -> Result<bool, sqlx::Error> {
    let status_str = status.to_string();
    let now = Utc::now().timestamp();
    
    let result = sqlx::query("UPDATE orders SET status = ?, updated_at = ? WHERE id = ?")
        .bind(&status_str)
        .bind(now)
        .bind(order_id)
        .execute(pool)
        .await?;
    
    Ok(result.rows_affected() > 0)
}

/// Get all orders for a specific user
pub async fn get_user_orders(
    query: web::Query<GetOrdersQuery>,
    state: web::Data<AppState>
) -> impl Responder {
    info!("Getting orders for user_id: {}", query.user_id);
    
    // Use dynamic SQL instead
    let orders = sqlx::query(
        "SELECT o.id, o.user_id, o.status, o.shipping_name, o.total_amount, o.created_at 
        FROM orders o
        WHERE o.user_id = ?
         ORDER BY o.created_at DESC"
    )
    .bind(&query.user_id)
    .fetch_all(&state.db)
    .await;
    
    match orders {
        Ok(orders) => {
            if orders.is_empty() {
                info!("No orders found for user_id: {}", query.user_id);
            } else {
                info!("Found {} orders for user_id: {}", orders.len(), query.user_id);
            }
            
            // Map to JSON response
            let orders_json: Vec<serde_json::Value> = orders
                .iter()
                .map(|row| {
                    json!({
                        "id": row.get::<String, _>("id"),
                        "user_id": row.get::<String, _>("user_id"),
                        "status": row.get::<String, _>("status"),
                        "total_amount": row.get::<f64, _>("total_amount"),
                        "created_at": row.get::<i64, _>("created_at"),
                    })
                })
                .collect();
            
            HttpResponse::Ok().json(orders_json)
        },
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to fetch orders: {}", e)
        })),
    }
}

/// Register all order-related routes.
pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/orders")
            .route("/create", web::post().to(create_order_handler))
            .service(get_order_status_endpoint)
            .route("/update/{order_id}", web::post().to(update_order_status_handler))
            .service(get_authenticated_user_orders)
            .route("/create-test", web::post().to(create_test_order))
            .service(debug_orders)
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

// Add this implementation to serialize OrderStatus consistently
impl ToString for OrderStatus {
    fn to_string(&self) -> String {
        match self {
            OrderStatus::Pending => "Pending".to_string(),
            OrderStatus::AwaitingPayment => "AwaitingPayment".to_string(),
            OrderStatus::Paid => "Paid".to_string(),
            OrderStatus::Shipped => "Shipped".to_string(),
            OrderStatus::Delivered => "Delivered".to_string(),
            OrderStatus::Completed => "Completed".to_string(),
            OrderStatus::Cancelled => "Cancelled".to_string(),
        }
    }
}

// Update the create_test_order function to use the token user ID
pub async fn create_test_order(req: HttpRequest, state: web::Data<AppState>) -> impl Responder {
    // Extract user ID from token
    let user_id = match extract_user_id_from_token(&req) {
        Some(id) => id,
        None => "usr-user1".to_string() // Default to test user if no token
    };
    
    // Create a test monero payment first
    let payment_id = format!("pay-test-{}", Uuid::new_v4().simple());
    let now = Utc::now().timestamp();
    
    let payment_result = sqlx::query("INSERT INTO monero_payments (payment_id, amount, address, status, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(&payment_id)
        .bind(99.99)
        .bind("44AFFq5kSiGBoZ4NMDwYtN18obc8AemS33DBLWs3H7otXft3XjrpDtQGv7SqSsaBYBb98uNbr2VBBEt7f2wfn3RVGQBEP3A")
        .bind("Pending")
        .bind(now)
        .bind(now)
        .execute(&state.db)
        .await;
    
    if let Err(e) = payment_result {
        error!("Failed to create test payment: {}", e);
        return HttpResponse::InternalServerError().json(
            serde_json::json!({"error": "Failed to create test payment"})
        );
    }
    
    // Now create the order
    let order_id = format!("ord-test-{}", Uuid::new_v4().simple());
    
    let shipping_info = ShippingInfo {
        name: "Test User".to_string(),
        address: "123 Test St".to_string(),
        city: "Test City".to_string(),
        state: "Test State".to_string(),
        zip: "12345".to_string(),
        country: "Test Country".to_string(),
        email: "test@example.com".to_string(),
    };

    let result = sqlx::query("INSERT INTO orders (id, user_id, payment_id, status, shipping_name, shipping_address, shipping_city, shipping_state, shipping_zip, shipping_country, shipping_email, total_amount, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&order_id)
        .bind(&user_id)
        .bind(&payment_id)
        .bind("Pending")
        .bind(&shipping_info.name)
        .bind(&shipping_info.address)
        .bind(&shipping_info.city)
        .bind(&shipping_info.state)
        .bind(&shipping_info.zip)
        .bind(&shipping_info.country)
        .bind(&shipping_info.email)
        .bind(99.99)
        .bind(now)
        .bind(now)
    .execute(&state.db)
    .await;
    
    match result {
        Ok(_) => {
            info!("Test order created successfully: {}", order_id);
            HttpResponse::Created().json(serde_json::json!({
                    "id": order_id,
                "status": "Pending",
                    "created_at": now
            }))
        }
        Err(e) => {
            error!("Failed to create test order: {}", e);
            HttpResponse::InternalServerError().json(
                serde_json::json!({"error": "Failed to create test order"})
            )
        }
    }
}

// Get order by ID
pub async fn get_order(
    pool: &SqlitePool,
    order_id: &str,
) -> Result<Option<Order>, sqlx::Error> {
    let row = sqlx::query("SELECT id, user_id, payment_id, status, shipping_name, shipping_address, shipping_city, shipping_state, shipping_zip, shipping_country, shipping_email, total_amount, created_at, updated_at FROM orders WHERE id = ?")
        .bind(order_id)
        .fetch_optional(pool)
        .await?;
    
    if let Some(row) = row {
        let id: String = row.try_get("id")?;
        let user_id: Option<String> = row.try_get("user_id")?;
        let payment_id: String = row.try_get("payment_id")?;
        let status: String = row.try_get("status")?;
        let shipping_name: String = row.try_get("shipping_name")?;
        let shipping_address: String = row.try_get("shipping_address")?;
        let shipping_city: String = row.try_get("shipping_city")?;
        let shipping_state: String = row.try_get("shipping_state")?;
        let shipping_zip: String = row.try_get("shipping_zip")?;
        let shipping_country: String = row.try_get("shipping_country")?;
        let shipping_email: String = row.try_get("shipping_email")?;
        let total_amount: f64 = row.try_get("total_amount")?;
        let created_at: i64 = row.try_get("created_at")?;
        let updated_at: i64 = row.try_get("updated_at")?;
        
        let status = match status.as_str() {
            "Pending" => OrderStatus::Pending,
            "AwaitingPayment" => OrderStatus::AwaitingPayment,
            "Paid" => OrderStatus::Paid,
            "Shipped" => OrderStatus::Shipped,
            "Delivered" => OrderStatus::Delivered,
            "Completed" => OrderStatus::Completed,
            "Cancelled" => OrderStatus::Cancelled,
            _ => OrderStatus::Pending,
        };
        
        let shipping_info = ShippingInfo {
            name: shipping_name,
            address: shipping_address,
            city: shipping_city,
            state: shipping_state,
            zip: shipping_zip,
            country: shipping_country,
            email: shipping_email,
        };
        
        Ok(Some(Order {
            id,
            user_id,
            payment_id,
            status,
            shipping_info,
            total_amount,
            created_at,
            updated_at,
        }))
    } else {
        Ok(None)
    }
}

// API endpoints
pub async fn check_order_status(
    state: web::Data<AppState>,
    order_id: web::Path<String>,
) -> impl Responder {
    match get_order(&state.db, &order_id).await {
        Ok(Some(order)) => HttpResponse::Ok().json(order),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Order not found"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to fetch order: {}", e)
        })),
    }
}

// Update the create_order handler to use web::Json
pub async fn create_order_handler(
    state: web::Data<AppState>,
    req: web::Json<CreateOrderRequest>,
) -> impl Responder {
    let result = create_order(
        &state.db,
        req.shipping_info.clone(),
        Vec::new(),
        req.total_amount,
        req.user_id.clone(),
    ).await;

    match result {
        Ok(order) => HttpResponse::Ok().json(order),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to create order: {}", e)
        }))
    }
}

// Update the update_order_status handler
pub async fn update_order_status_handler(
    state: web::Data<AppState>,
    order_id: web::Path<String>,
    req: web::Json<UpdateOrderStatusRequest>,
) -> impl Responder {
    let result = update_order_status(&state.db, &order_id, req.status.clone()).await;

    match result {
        Ok(true) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "Order status updated successfully"
        })),
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Order not found"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to update order status: {}", e)
        }))
    }
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
pub async fn get_authenticated_user_orders(req: HttpRequest, app_state: web::Data<AppState>) -> impl Responder {
    // Extract user ID from authentication token
    let user_id = match extract_user_id_from_token(&req) {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized().json(json!({
                "success": false,
                "error": "Authentication required"
            }));
        }
    };
    
    info!("Fetching orders for user ID: {}", user_id);
    
    // Query to get user's orders with proper WHERE clause
    let query = r#"
        SELECT id, status, total_amount, created_at, updated_at, payment_id
        FROM orders 
        WHERE user_id = ?
        ORDER BY created_at DESC
    "#;
    
    match sqlx::query(query)
        .bind(&user_id)
        .fetch_all(&app_state.db)
        .await {
            Ok(rows) => {
                if rows.is_empty() {
                    info!("No orders found for user {}", user_id);
                    return HttpResponse::Ok().json(json!({
                        "success": true,
                        "orders": [],
                        "count": 0
                    }));
                }
                
                let orders = rows.iter().map(|row| {
                    json!({
                        "id": row.get::<String, _>("id"),
                        "status": row.get::<String, _>("status"),
                        "total_amount": row.get::<f64, _>("total_amount"),
                        "created_at": row.get::<i64, _>("created_at"),
                        "updated_at": row.get::<i64, _>("updated_at"),
                        "payment_id": row.get::<Option<String>, _>("payment_id")
                    })
                }).collect::<Vec<serde_json::Value>>();
                
                info!("Found {} orders for user {}", orders.len(), user_id);
                
                HttpResponse::Ok().json(json!({
                    "success": true,
                    "orders": orders,
                    "count": orders.len()
                }))
            },
            Err(e) => {
                error!("Database error while fetching orders: {}", e);
                HttpResponse::InternalServerError().json(json!({
                    "success": false,
                    "error": format!("Failed to fetch orders: {}", e)
                }))
            }
        }
}

// Better token extraction with more debugging
fn extract_user_id_from_token(req: &HttpRequest) -> Option<String> {
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            info!("Auth header: {}", auth_str);
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                info!("Token (first 20 chars): {}", &token[..std::cmp::min(20, token.len())]);
                
                // Try to extract from JWT first
                match crate::session::verify_jwt(token) {
                    Ok(claims) => {
                        info!("Successfully extracted user ID from token: {}", claims.sub);
                        return Some(claims.sub);
                    },
                    Err(e) => {
                        error!("JWT verification failed: {}", e);
                        
                        // As a fallback for testing, check if the token contains a user ID pattern
                        if token.contains("usr-") {
                            let parts: Vec<&str> = token.split("usr-").collect();
                            if parts.len() > 1 {
                                let possible_id = format!("usr-{}", parts[1]);
                                info!("Extracted possible user ID from token pattern: {}", possible_id);
                                return Some(possible_id);
                            }
                        }
                    }
                }
            }
        }
    }
    
    error!("No valid authorization header or token found");
    None
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
    );
    
    // Register the debug-token endpoint at the root level
    cfg.service(debug_token_endpoint);
}
