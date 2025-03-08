use actix_web::{web, HttpResponse, Responder, get, post, HttpRequest};
use serde::{Deserialize, Serialize};
use crate::AppState;
use crate::monero::{PaymentStatus, MoneroPaymentRequest};
use serde_json::json;
use rand;
use log;
use crate::orders::create_order;
use crate::types::ShippingInfo;
use crate::orders::OrderItem;
use uuid;
use chrono;
use sqlx::Row;

#[derive(Deserialize)]
pub struct CreatePaymentRequest {
    pub order_id: String,
    pub amount: f64,
}

#[derive(Serialize)]
pub struct PaymentResponse {
    pub success: bool,
    pub message: Option<String>,
    pub payment: Option<MoneroPaymentRequest>,
}

#[derive(Deserialize)]
pub struct TransactionProof {
    pub tx_hash: String,
    pub tx_key: String,
}

#[derive(Serialize)]
pub struct TransactionHistoryResponse {
    pub success: bool,
    pub transactions: Vec<MoneroPaymentRequest>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct CheckoutData {
    pub items: Vec<Item>,
    pub shipping_info: ShippingInfo,
    pub user_id: String,
    pub total: f64,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Item {
    pub id: String,
    pub price: f64,
    pub quantity: u32,
}

#[post("/api/monero/create_payment")]
pub async fn create_payment(
    app_state: web::Data<AppState>,
    payment_req: web::Json<CreatePaymentRequest>,
) -> impl Responder {
    let payment = app_state.monero_payments.create_payment_usd(
        payment_req.order_id.clone(),
        payment_req.amount,
    );

    HttpResponse::Ok().json(PaymentResponse {
        success: true,
        message: Some("Payment request created".to_string()),
        payment: Some(payment),
    })
}

#[get("/api/monero/check_payment/{payment_id}")]
pub async fn check_payment(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let payment_id = path.into_inner();
    
    if let Some(payment) = app_state.monero_payments.get_payment(&payment_id) {
        // In a real implementation, you would check with the Monero wallet
        // to see if the payment has been received
        
        HttpResponse::Ok().json(PaymentResponse {
            success: true,
            message: None,
            payment: Some(payment),
        })
    } else {
        HttpResponse::NotFound().json(PaymentResponse {
            success: false,
            message: Some("Payment not found".to_string()),
            payment: None,
        })
    }
}

// Mock endpoint to simulate payment confirmation (for testing)
#[post("/api/monero/mock_confirm/{payment_id}")]
pub async fn mock_confirm_payment(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let payment_id = path.into_inner();
    
    if let Some(updated_payment) = app_state.monero_payments.update_payment_status(&payment_id, PaymentStatus::Confirmed) {
        HttpResponse::Ok().json(PaymentResponse {
            success: true,
            message: Some("Payment confirmed".to_string()),
            payment: Some(updated_payment),
        })
    } else {
        HttpResponse::NotFound().json(PaymentResponse {
            success: false,
            message: Some("Payment not found".to_string()),
            payment: None,
        })
    }
}

#[post("/api/monero/finalize_order/{payment_id}")]
pub async fn finalize_order(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let payment_id = path.into_inner();
    
    if let Some(payment) = app_state.monero_payments.get_payment(&payment_id) {
        if payment.status == PaymentStatus::Confirmed || payment.status == PaymentStatus::Completed {
            // In a real implementation, you would:
            // 1. Create an order record in your database
            // 2. Update inventory
            // 3. Send confirmation email
            
            // Update payment status to Completed if it's not already
            if payment.status != PaymentStatus::Completed {
                app_state.monero_payments.update_payment_status(&payment_id, PaymentStatus::Completed);
            }
            
            // For demo purposes, just log the order
            println!("Order {} finalized with payment {}", payment.order_id, payment_id);
            
            HttpResponse::Ok().json(PaymentResponse {
                success: true,
                message: Some("Order finalized successfully".to_string()),
                payment: app_state.monero_payments.get_payment(&payment_id),
            })
        } else {
            HttpResponse::BadRequest().json(PaymentResponse {
                success: false,
                message: Some("Payment not confirmed yet".to_string()),
                payment: Some(payment),
            })
        }
    } else {
        HttpResponse::NotFound().json(PaymentResponse {
            success: false,
            message: Some("Payment not found".to_string()),
            payment: None,
        })
    }
}

// Update the payment checker to be more robust
pub fn start_payment_checker(app_state: web::Data<AppState>) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            // Check payments every 1 minute
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            
            // This would integrate with the Monero wallet RPC in a real implementation
            app_state.monero_payments.check_payments();
            
            // Expire old pending payments
            app_state.monero_payments.expire_old_payments();
            
            // Log number of pending payments for monitoring
            let pending_count = app_state.monero_payments.get_pending_payments().len();
            if pending_count > 0 {
                println!("Currently monitoring {} pending Monero payments", pending_count);
            }
        }
    })
}

#[post("/api/monero/submit_proof/{payment_id}")]
pub async fn submit_proof(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
    proof: web::Json<TransactionProof>,
) -> impl Responder {
    let payment_id = path.into_inner();
    
    match app_state.monero_payments.verify_payment_by_tx_hash(
        &payment_id,
        &proof.tx_hash,
        &proof.tx_key,
    ) {
        Ok(true) => {
            HttpResponse::Ok().json(PaymentResponse {
                success: true,
                message: Some("Payment verified and confirmed".to_string()),
                payment: app_state.monero_payments.get_payment(&payment_id),
            })
        },
        Ok(false) => {
            HttpResponse::BadRequest().json(PaymentResponse {
                success: false,
                message: Some("Payment verification failed".to_string()),
                payment: app_state.monero_payments.get_payment(&payment_id),
            })
        },
        Err(e) => {
            HttpResponse::BadRequest().json(PaymentResponse {
                success: false,
                message: Some(format!("Error verifying payment: {}", e)),
                payment: None,
            })
        }
    }
}

#[get("/api/monero/user_transactions")]
pub async fn get_user_transactions(
    app_state: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    // Extract user ID from JWT token
    // PLACEHOLDER: Implement proper JWT token validation
    let auth_header = req.headers().get("Authorization");
    let user_id = if let Some(_auth) = auth_header {
        // Mock implementation - in production get actual user ID from token
        "user123".to_string()
    } else {
        return HttpResponse::Unauthorized().json(json!({
            "success": false,
            "message": "Authentication required"
        }));
    };
    
    // PLACEHOLDER: In production, query the database for user's transactions
    // For now, return mock data from in-memory store
    let transactions = app_state.monero_payments.get_payments_by_user(&user_id);
    
    HttpResponse::Ok().json(TransactionHistoryResponse {
        success: true,
        transactions,
    })
}

// Add this endpoint to manually check a payment
#[post("/api/monero/check_now/{payment_id}")]
pub async fn force_check_payment(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let payment_id = path.into_inner();
    
    if let Some(payment) = app_state.monero_payments.get_payment(&payment_id) {
        // Only check pending payments
        if payment.status == PaymentStatus::Pending {
            println!("Manual check triggered for payment {}", payment_id);
            
            // In production, you would check the wallet RPC for this specific payment
            // For now, use a higher chance of confirmation for manually triggered checks
            if rand::random::<f64>() < 0.5 {  // 50% chance 
                app_state.monero_payments.update_payment_status(&payment_id, PaymentStatus::Confirmed);
                
                // Get the updated payment
                let updated_payment = app_state.monero_payments.get_payment(&payment_id);
                
                return HttpResponse::Ok().json(PaymentResponse {
                    success: true,
                    message: Some("Payment confirmed".to_string()),
                    payment: updated_payment,
                });
            }
        }
        
        HttpResponse::Ok().json(PaymentResponse {
            success: true,
            message: Some("Payment checked but not confirmed yet".to_string()),
            payment: Some(payment),
        })
    } else {
        HttpResponse::NotFound().json(PaymentResponse {
            success: false,
            message: Some("Payment not found".to_string()),
            payment: None,
        })
    }
}

#[post("/checkout")]
pub async fn checkout_handler(
    app_state: web::Data<AppState>,
    checkout_data: web::Json<CheckoutData>,
) -> impl Responder {
    // Log the method and request info
    log::info!("Checkout handler called with POST method");
    log::info!("Received checkout data: {:?}", checkout_data);
    
    // First ensure the schema is correct
    if let Err(e) = ensure_monero_payment_schema(&app_state.db).await {
        log::error!("Failed to ensure schema: {}", e);
        return HttpResponse::InternalServerError().json(json!({
            "success": false,
            "error": format!("Database schema error: {}", e)
        }));
    }
    
    // Create Monero payment request
    let payment = app_state.monero_payments.create_payment_usd(
        format!("temp-{}", uuid::Uuid::new_v4().to_string()),
        checkout_data.total
    );
    
    log::info!("Created payment with ID: {}", payment.payment_id);
    
    // Insert the payment record with the order_id initially set to empty string
    let now = chrono::Utc::now().timestamp();
    match sqlx::query(
        "INSERT INTO monero_payments (payment_id, amount, address, status, created_at, updated_at, order_id) 
         VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(payment.payment_id.clone())
    .bind(payment.amount)
    .bind(payment.address.clone())
    .bind("Pending")  // Convert the enum to string
    .bind(now)
    .bind(now)
    .bind("")  // Empty string for order_id initially
    .execute(&app_state.db)
    .await {
        Ok(_) => log::info!("Inserted payment record into database"),
        Err(e) => {
            log::error!("Failed to insert payment record: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": format!("Failed to insert payment record: {}", e)
            }));
        }
    }
    
    // Convert checkout items to OrderItems
    let order_items: Vec<OrderItem> = checkout_data.items
        .iter()
        .map(|item| OrderItem {
            product_id: item.id.clone(),
            quantity: item.quantity,
            price: item.price,
        })
        .collect();

    // Create the order
    let order = match create_order(
        &app_state.db,
        checkout_data.shipping_info.clone(),
        order_items,
        checkout_data.total,
        Some(checkout_data.user_id.clone()),
    ).await {
        Ok(order) => order,
        Err(e) => {
            log::error!("Failed to create order: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": "Failed to create order"
            }));
        }
    };
    
    log::info!("Created order with ID: {}", order.id);
    
    // Update the payment with the order ID using a regular query instead of the macro
    match sqlx::query(
        "UPDATE monero_payments SET order_id = ? WHERE payment_id = ?"
    )
    .bind(order.id.clone())
    .bind(payment.payment_id.clone())
    .execute(&app_state.db)
    .await {
        Ok(_) => log::info!("Updated payment record with order ID: {}", order.id),
        Err(e) => log::warn!("Failed to update payment record with order ID: {}", e),
    }
    
    // Update order with payment ID
    match sqlx::query(
        "UPDATE orders SET payment_id = ? WHERE id = ?"
    )
    .bind(payment.payment_id.clone())
    .bind(order.id.clone())
    .execute(&app_state.db)
    .await {
        Ok(_) => log::info!("Updated order with payment ID: {}", payment.payment_id),
        Err(e) => {
            log::error!("Failed to update order with payment ID: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": format!("Failed to update order with payment ID: {}", e)
            }));
        }
    }
    
    // Update the in-memory payment as well
    if let Err(e) = app_state.monero_payments.update_payment_order_id(&payment.payment_id, &order.id) {
        log::warn!("Failed to update payment in memory: {}", e);
    }
    
    // Return success response
    HttpResponse::Ok().json(json!({
        "success": true,
        "order_id": order.id,
        "payment": payment,
        "message": "Please send Monero to the provided address"
    }))
}

#[get("/checkout/test")]
pub async fn checkout_test() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Checkout endpoint is available"
    }))
}

#[get("/debug/{path:.*}")]
pub async fn debug_handler(req: HttpRequest) -> impl Responder {
    log::info!("Debug request: {:?}", req);
    HttpResponse::Ok().json(json!({
        "success": true,
        "message": "Debug endpoint reached",
        "method": req.method().as_str(),
        "path": req.path(),
        "headers": format!("{:?}", req.headers())
    }))
}

#[get("/debug-info")]
pub async fn debug_info(req: HttpRequest) -> impl Responder {
    log::info!("Debug info request: {:?}", req);
    HttpResponse::Ok().json(json!({
        "success": true,
        "message": "Debug info endpoint reached",
        "method": req.method().as_str(),
        "path": req.path(),
        "headers": format!("{:?}", req.headers())
    }))
}

#[get("/routes")]
pub async fn list_routes() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "success": true,
        "message": "Available routes",
        "routes": [
            "/monero/api/monero/check_payment/{payment_id}",
            "/monero/checkout",
            "/monero/checkout/test",
            "/monero/debug-info",
            // Add other routes
        ]
    }))
}

#[get("/order_payment/{order_id}")]
pub async fn get_order_payment(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let order_id = path.into_inner();
    log::info!("Looking up payment for order ID: {}", order_id);
    
    // First, check if the order exists
    let order_count = match sqlx::query!(
        "SELECT COUNT(*) as count FROM orders WHERE id = ?",
        order_id
    )
    .fetch_one(&app_state.db)
    .await {
        Ok(row) => row.count,
        Err(e) => {
            log::error!("Database error checking if order exists {}: {}", order_id, e);
            return HttpResponse::InternalServerError().json(PaymentResponse {
                success: false,
                message: Some(format!("Database error: {}", e)),
                payment: None,
            });
        }
    };
    
    if order_count == 0 {
        log::warn!("Order ID {} does not exist in database", order_id);
        return HttpResponse::NotFound().json(PaymentResponse {
            success: false,
            message: Some(format!("Order with ID {} not found", order_id)),
            payment: None,
        });
    }
    
    // Now get the payment ID
    match sqlx::query!(
        "SELECT payment_id FROM orders WHERE id = ?",
        order_id
    )
    .fetch_optional(&app_state.db)
    .await {
        Ok(Some(row)) => {
            // Use unwrap_or_default() to handle null payment_id
            let payment_id = row.payment_id.unwrap_or_default();
            
            // Check if payment_id is empty
            if payment_id.is_empty() {
                log::warn!("Order {} has no payment ID", order_id);
                return HttpResponse::NotFound().json(PaymentResponse {
                    success: false,
                    message: Some(format!("Order {} has no payment ID", order_id)),
                    payment: None,
                });
            }
            
            log::info!("Found payment ID {} for order {}", payment_id, order_id);
            
            // Now get the payment details
            if let Some(payment) = app_state.monero_payments.get_payment(&payment_id) {
                HttpResponse::Ok().json(PaymentResponse {
                    success: true,
                    message: Some(format!("Payment found for order {}", order_id)),
                    payment: Some(payment),
                })
            } else {
                log::warn!("Payment ID {} found but no payment details", payment_id);
                HttpResponse::NotFound().json(PaymentResponse {
                    success: false,
                    message: Some(format!("Payment details not found for payment ID {}", payment_id)),
                    payment: None,
                })
            }
        },
        Ok(None) => {
            log::warn!("No order found with ID: {}", order_id);
            HttpResponse::NotFound().json(PaymentResponse {
                success: false,
                message: Some(format!("No order found with ID: {}", order_id)),
                payment: None,
            })
        },
        Err(e) => {
            log::error!("Database error looking up order {}: {}", order_id, e);
            HttpResponse::InternalServerError().json(PaymentResponse {
                success: false,
                message: Some(format!("Database error: {}", e)),
                payment: None,
            })
        }
    }
}

#[get("/admin/orders")]
pub async fn get_all_orders(app_state: web::Data<AppState>) -> impl Responder {
    match sqlx::query!(
        r#"
        SELECT o.*, mp.address as monero_address, mp.status as payment_status
        FROM orders o
        LEFT JOIN monero_payments mp ON o.payment_id = mp.payment_id
        ORDER BY o.created_at DESC
        "#
    )
    .fetch_all(&app_state.db)
    .await {
        Ok(orders) => {
            let orders = orders.into_iter().map(|row| {
                json!({
                    "id": row.id,
                    "payment_id": row.payment_id,
                    "monero_address": row.monero_address,
                    "total_amount": row.total_amount,
                    "status": row.status,
                    "created_at": row.created_at,
                    "payment_status": row.payment_status
                })
            }).collect::<Vec<_>>();

            HttpResponse::Ok().json(json!({
                "success": true,
                "orders": orders
            }))
        },
        Err(e) => {
            log::error!("Failed to fetch orders: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": "Failed to fetch orders"
            }))
        }
    }
}

#[get("/admin/validate/{order_id}")]
pub async fn validate_order(
    app_state: web::Data<AppState>,
    path: web::Path<String>
) -> impl Responder {
    let order_id = path.into_inner();
    
    match sqlx::query!(
        r#"
        SELECT o.*, mp.address as monero_address, mp.status as payment_status
        FROM orders o
        LEFT JOIN monero_payments mp ON o.payment_id = mp.payment_id
        WHERE o.id = ?
        "#,
        order_id
    )
    .fetch_optional(&app_state.db)
    .await {
        Ok(Some(order)) => {
            HttpResponse::Ok().json(json!({
                "success": true,
                "valid": true,
                "order": {
                    "id": order.id,
                    "payment_id": order.payment_id,
                    "monero_address": order.monero_address,
                    "total_amount": order.total_amount,
                    "status": order.status,
                    "created_at": order.created_at,
                    "payment_status": order.payment_status
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
            log::error!("Failed to validate order: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": "Failed to validate order"
            }))
        }
    }
}

pub fn init_routes() -> actix_web::Scope {
    web::scope("/monero")
        .service(create_payment)
        .service(check_payment)
        .service(mock_confirm_payment)
        .service(finalize_order)
        .service(submit_proof)
        .service(get_user_transactions)
        .service(force_check_payment)
        .service(checkout_handler)
        .service(checkout_test)
        .service(debug_handler)
        .service(debug_info)
        .service(list_routes)
        .service(get_order_payment)
        .service(get_all_orders)
        .service(validate_order)
}

// First, let's fix the function that adds the column
pub async fn ensure_monero_payment_schema(pool: &sqlx::SqlitePool) -> Result<(), sqlx::Error> {
    // Use a query without the macro to avoid type issues with PRAGMA
    let result = sqlx::query("PRAGMA table_info(monero_payments)")
        .fetch_all(pool)
        .await?;
    
    // Check if order_id column already exists - look for it by name in the results
    let has_order_id = result.iter().any(|row| {
        row.try_get::<String, _>("name")
            .map(|name| name == "order_id")
            .unwrap_or(false)
    });
    
    if !has_order_id {
        log::info!("Adding order_id column to monero_payments table");
        // Execute the ALTER TABLE statement
        sqlx::query("ALTER TABLE monero_payments ADD COLUMN order_id TEXT")
            .execute(pool)
            .await?;
    }
    
    Ok(())
}