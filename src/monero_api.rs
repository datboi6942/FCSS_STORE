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

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckoutRequest {
    pub product_id: String,
    pub price: f64,
    pub shipping_info: Option<ShippingInfo>,
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
                let update_result = app_state.monero_payments.update_payment_status(&payment_id, PaymentStatus::Confirmed);
                if update_result.is_some() {
                    log::info!("Successfully updated payment status in memory");
                    
                    // Get the updated payment
                    let updated_payment = app_state.monero_payments.get_payment(&payment_id);
                    
                    return HttpResponse::Ok().json(PaymentResponse {
                        success: true,
                        message: Some("Payment confirmed".to_string()),
                        payment: updated_payment,
                    });
                } else {
                    log::error!("Failed to update payment status in memory - payment not found");
                }
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
    req: HttpRequest,
    data: web::Json<CheckoutRequest>,
    app_state: web::Data<AppState>
) -> impl Responder {
    // Extract the shipping info or use default
    let shipping_info = data.shipping_info.clone().unwrap_or(ShippingInfo {
        name: "Customer".to_string(),
        address: "Address".to_string(),
        city: "City".to_string(),
        state: "State".to_string(),
        zip: "12345".to_string(),
        country: "Country".to_string(),
        email: "customer@example.com".to_string(),
    });
    
    // Create a single order item
    let order_items = vec![OrderItem {
        product_id: data.product_id.clone(),
        quantity: 1,  // Default to 1
        price: data.price,
    }];

    // Extract user ID from JWT token or use guest
    let user_id = match crate::auth::validate_token(req) {
        Ok(claims) => claims.sub,
        Err(_) => "guest".to_string()
    };

    // Create the order
    let order = match create_order(
        &app_state.db,
        shipping_info,
        order_items,
        data.price,
        Some(user_id),
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
    .bind(data.product_id.clone())
    .execute(&app_state.db)
    .await {
        Ok(_) => log::info!("Updated payment record with order ID: {}", order.id),
        Err(e) => log::warn!("Failed to update payment record with order ID: {}", e),
    }
    
    // Update order with payment ID
    match sqlx::query(
        "UPDATE orders SET payment_id = ? WHERE id = ?"
    )
    .bind(data.product_id.clone())
    .bind(order.id.clone())
    .execute(&app_state.db)
    .await {
        Ok(_) => log::info!("Updated order with payment ID: {}", data.product_id),
        Err(e) => {
            log::error!("Failed to update order with payment ID: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": format!("Failed to update order with payment ID: {}", e)
            }));
        }
    }
    
    // Update the in-memory payment as well
    let update_result = app_state.monero_payments.update_payment_order_id(&data.product_id, &order.id);
    if update_result.is_ok() {
        log::info!("Updated payment in memory with order ID: {}", order.id);
    } else {
        log::warn!("Failed to update payment in memory - payment not found");
    }
    
    // Return success response
    HttpResponse::Ok().json(json!({
        "success": true,
        "order_id": order.id,
        "payment": data.product_id,
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

// Add a new endpoint to check payment status
#[get("/check_payment/{order_id}")]
pub async fn check_payment_status(
    path: web::Path<String>,
    app_state: web::Data<AppState>
) -> impl Responder {
    let order_id = path.into_inner();
    
    // Find the payment for this order
    match sqlx::query!(
        "SELECT mp.payment_id, mp.status, mp.amount, mp.address FROM monero_payments mp
         JOIN orders o ON mp.payment_id = o.payment_id
         WHERE o.id = ?",
        order_id
    )
    .fetch_optional(&app_state.db)
    .await {
        Ok(Some(payment)) => {
            HttpResponse::Ok().json(json!({
                "success": true,
                "status": payment.status,
                "amount": payment.amount,
                "address": payment.address,
                "payment_id": payment.payment_id
            }))
        },
        Ok(None) => {
            // No payment found, check if order exists
            match sqlx::query!("SELECT id FROM orders WHERE id = ?", order_id)
                .fetch_optional(&app_state.db)
                .await {
                    Ok(Some(_)) => {
                        HttpResponse::Ok().json(json!({
                            "success": false,
                            "error": "No payment found for this order",
                            "status": "not_found"
                        }))
                    },
                    _ => {
                        HttpResponse::NotFound().json(json!({
                            "success": false,
                            "error": "Order not found",
                            "status": "invalid_order"
                        }))
                    }
                }
        },
        Err(e) => {
            log::error!("Database error checking payment: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": "Database error",
                "status": "error"
            }))
        }
    }
}

// Add this endpoint to manually sync all order statuses from payments
#[post("/admin/sync-all-payments")]
pub async fn sync_all_payment_statuses(
    app_state: web::Data<AppState>
) -> impl Responder {
    log::info!("🔄 Manual sync of all payment statuses to orders triggered");
    
    // Get all confirmed payments
    let confirmed_payments = sqlx::query!(
        "SELECT payment_id, status FROM monero_payments WHERE status = 'Confirmed' OR status = 'confirmed'"
    )
    .fetch_all(&app_state.db)
    .await;
    
    match confirmed_payments {
        Ok(payments) => {
            log::info!("Found {} confirmed payments to sync", payments.len());
            
            let mut success_count = 0;
            
            for payment in &payments {
                // Safely unwrap the payment_id Option or skip this record
                if let Some(payment_id) = &payment.payment_id {
                    match sync_payment_status_to_order(&app_state.db, payment_id, "Confirmed").await {
                        Ok(_) => {
                            log::info!("✅ Successfully synced payment {} to order", payment_id);
                            success_count += 1;
                        },
                        Err(e) => {
                            log::error!("❌ Failed to sync payment {} to order: {}", payment_id, e);
                        }
                    }
                } else {
                    log::warn!("Skipping payment with null payment_id");
                }
            }
            
            HttpResponse::Ok().json(json!({
                "success": true,
                "message": format!("Synced {}/{} confirmed payments to orders", success_count, payments.len())
            }))
        },
        Err(e) => {
            log::error!("Failed to query confirmed payments: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": format!("Database error: {}", e)
            }))
        }
    }
}

// Add this endpoint to fix the missing payment_id issue
#[post("/fix-orphaned-payments")]
pub async fn fix_orphaned_payments(
    app_state: web::Data<AppState>
) -> impl Responder {
    log::info!("🔧 Running orphaned payments fix");
    
    // 1. Get all monero payments with order_id that aren't linked back to an order
    let orphaned = sqlx::query!(
        "SELECT mp.payment_id, mp.status, mp.order_id 
         FROM monero_payments mp
         LEFT JOIN orders o ON o.payment_id = mp.payment_id
         WHERE mp.order_id IS NOT NULL 
         AND mp.order_id != '' 
         AND (o.payment_id IS NULL OR o.payment_id = '')"
    )
    .fetch_all(&app_state.db)
    .await;
    
    if let Err(e) = &orphaned {
        log::error!("Database error: {}", e);
        return HttpResponse::InternalServerError().json(json!({
            "success": false,
            "error": format!("Database error: {}", e)
        }));
    }
    
    let orphaned = orphaned.unwrap();
    log::info!("Found {} orphaned payments with order IDs", orphaned.len());
    
    let mut fixed_orders = Vec::new();
    
    // 2. For each payment, link it to its order
    for payment in orphaned {
        if let Some(payment_id) = &payment.payment_id {
            if let Some(order_id) = &payment.order_id {
                if !order_id.is_empty() {
                    log::info!("Connecting payment {} to order {}", payment_id, order_id);
                    
                    // Update the order with the payment ID
                    let update_result = sqlx::query!(
                        "UPDATE orders SET payment_id = ? WHERE id = ?",
                        payment_id,
                        order_id
                    )
                    .execute(&app_state.db)
                    .await;
                    
                    match update_result {
                        Ok(_) => {
                            log::info!("✅ Successfully linked payment {} to order {}", payment_id, order_id);
                            
                            // If the payment is confirmed, also update the order status
                            if payment.status == "Confirmed" || payment.status == "confirmed" {
                                log::info!("Payment is confirmed, updating order status too");
                                
                                let status_update = sqlx::query!(
                                    "UPDATE orders SET status = 'Confirmed' WHERE id = ?",
                                    order_id
                                )
                                .execute(&app_state.db)
                                .await;
                                
                                if let Ok(_) = status_update {
                                    log::info!("✅ Updated order status to Confirmed");
                                }
                            }
                            
                            fixed_orders.push(json!({
                                "order_id": order_id,
                                "payment_id": payment_id,
                                "payment_status": payment.status
                            }));
                        },
                        Err(e) => {
                            log::error!("❌ Failed to link payment to order: {}", e);
                        }
                    }
                }
            }
        }
    }
    
    // Return the fix results
    HttpResponse::Ok().json(json!({
        "success": true,
        "fixed_count": fixed_orders.len(),
        "fixed_orders": fixed_orders
    }))
}

// Add a comprehensive diagnostic endpoint for all payments
#[get("/debug/dump-all-payments")]
pub async fn dump_all_payments(
    app_state: web::Data<AppState>
) -> impl Responder {
    log::info!("🔍 Dumping all payment records for debugging");
    
    // Get all payments from the database
    let db_payments = sqlx::query(
        "SELECT payment_id, amount, address, status, created_at, updated_at, order_id FROM monero_payments"
    )
    .fetch_all(&app_state.db)
    .await;
    
    // Get all in-memory payments
    let memory_payments: Vec<_> = app_state.monero_payments
        .get_all_payments()
        .into_iter()
        .collect();
    
    match db_payments {
        Ok(rows) => {
            // Convert the rows to a serializable format
            let serializable_payments: Vec<serde_json::Value> = rows.iter().map(|row| {
                json!({
                    "payment_id": row.get::<String, _>("payment_id"),
                    "amount": row.get::<f64, _>("amount"),
                    "address": row.get::<String, _>("address"),
                    "status": row.get::<String, _>("status"),
                    "created_at": row.get::<i64, _>("created_at"),
                    "updated_at": row.get::<i64, _>("updated_at"),
                    "order_id": row.try_get::<String, _>("order_id").unwrap_or_default()
                })
            }).collect();
            
            HttpResponse::Ok().json(json!({
                "success": true,
                "db_payments_count": serializable_payments.len(),
                "memory_payments_count": memory_payments.len(),
                "db_payments": serializable_payments,
                "memory_payments": memory_payments
            }))
        },
        Err(e) => {
            log::error!("Failed to query payments: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": format!("Database error: {}", e),
                "memory_payments_count": memory_payments.len(),
                "memory_payments": memory_payments
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
        .service(check_payment_status)
        .service(sync_all_payment_statuses)
        .service(fix_orphaned_payments)
        .service(dump_all_payments)
        .service(force_create_payment_links)
        .service(force_update_order_status)
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

// Rename our utility function to avoid the name conflict
// And remove it from being registered as a service
pub async fn create_payment_for_order(
    app_state: &web::Data<AppState>,
    order_id: &str,
    amount: f64
) -> Result<MoneroPaymentRequest, String> {
    // Implement the payment creation logic here
    log::info!("Creating Monero payment for order {}, amount: {}", order_id, amount);
    
    let payment = app_state.monero_payments.create_payment_sync(
        order_id.to_string(),
        amount
    );
    
    // Log the new payment
    log::info!("Created payment with ID: {}", payment.payment_id);
    
    // Create an association between the order and payment in the database
    match sqlx::query(
        "UPDATE monero_payments SET order_id = ? WHERE payment_id = ?"
    )
    .bind(order_id)
    .bind(&payment.payment_id)
    .execute(&app_state.db)
    .await {
        Ok(_) => {},
        Err(e) => {
            log::error!("Failed to update payment with order_id: {}", e);
            // Continue anyway, the payment still works
        }
    }
    
    Ok(payment)
}

#[allow(unused_imports)]
use uuid;
#[allow(unused_imports)]
use chrono;

// Add this function to synchronize payment status with order status
async fn sync_payment_status_to_order(pool: &sqlx::SqlitePool, payment_id: &str, status: &str) -> Result<(), sqlx::Error> {
    log::info!("🔄 Synchronizing payment status '{}' for payment ID {} to order", status, payment_id);
    
    // First, log all orders and payments for debugging
    log::info!("Debugging all orders and payments in the system:");
    let all_orders = sqlx::query("SELECT id, payment_id, status FROM orders").fetch_all(pool).await?;
    for row in &all_orders {
        let order_id: String = row.get("id");
        let payment_id_opt: Option<String> = row.get("payment_id");
        let status: String = row.get("status");
        log::info!("Order: {} | Payment ID: {:?} | Status: {}", order_id, payment_id_opt, status);
    }
    
    // Find the order ID associated with this payment - more detailed query
    log::info!("Searching for order with payment_id = {}", payment_id);
    let query = format!("SELECT id, status FROM orders WHERE payment_id = '{}'", payment_id);
    log::info!("Query: {}", query);
    
    let orders = sqlx::query(&query).fetch_all(pool).await?;
    if orders.is_empty() {
        log::warn!("❌ No orders found with payment_id = {}", payment_id);
        
        // Try alternative lookup via monero_payments table
        let result = sqlx::query!(
            "SELECT order_id FROM monero_payments WHERE payment_id = ?",
            payment_id
        )
        .fetch_optional(pool)
        .await?;
        
        if let Some(record) = result {
            if let Some(order_id) = record.order_id {
                if !order_id.is_empty() {
                    log::info!("✅ Found order_id {} via monero_payments table for payment {}", order_id, payment_id);
                    // Update the order status directly
                    log::info!("Updating order {} status to {}", order_id, status);
                    sqlx::query!(
                        "UPDATE orders SET status = ? WHERE id = ?",
                        status,
                        order_id
                    )
                    .execute(pool)
                    .await?;
                    
                    // Also make sure the payment_id is set on the order
                    log::info!("Ensuring payment_id is set on order {}", order_id);
                    sqlx::query!(
                        "UPDATE orders SET payment_id = ? WHERE id = ?",
                        payment_id,
                        order_id
                    )
                    .execute(pool)
                    .await?;
                    
                    return Ok(());
                }
            }
        }
        
        log::error!("🔍 Still couldn't find any order for payment {}", payment_id);
        return Ok(());
    }
    
    // Process found orders
    for row in orders {
        let order_id: String = row.get("id");
        let current_status: String = row.get("status");
        
        log::info!("✅ Found order {} with current status {}, updating to {}", order_id, current_status, status);
        
        // Update the order status
        sqlx::query!(
            "UPDATE orders SET status = ? WHERE id = ?",
            status,
            order_id
        )
        .execute(pool)
        .await?;
    }
    
    Ok(())
}

// Update this function to be more thorough when updating payment status
pub async fn check_payment_status_and_update(app_state: &web::Data<AppState>, payment_id: &str) -> Result<bool, String> {
    // First, check if the payment exists and get its current status
    let payment = match app_state.monero_payments.get_payment(payment_id) {
        Some(p) => p,
        None => return Err(format!("Payment with ID {} not found", payment_id))
    };
    
    // In a real implementation, you would check with the Monero RPC here
    // For this example, we'll just use a random check with a 50% chance of confirmation
    let is_confirmed = rand::random::<f64>() < 0.5;
    
    log::info!("Checking payment {} with current status {:?}, simulation result: confirmed={}", 
               payment_id, payment.status, is_confirmed);
    
    // Only proceed with confirmation if the payment is currently pending
    if is_confirmed && payment.status == PaymentStatus::Pending {
        log::info!("🔔 Payment {} is confirmed, updating status", payment_id);
        
        // First update the database directly to ensure it's updated
        match sqlx::query!(
            "UPDATE monero_payments SET status = ? WHERE payment_id = ?",
            "Confirmed",
            payment_id
        )
        .execute(&app_state.db)
        .await {
            Ok(_) => log::info!("✅ Updated payment status in database for {}", payment_id),
            Err(e) => log::error!("❌ Failed to update payment in database: {}", e)
        }
        
        // Now also update the order status
        match sync_payment_status_to_order(&app_state.db, payment_id, "Confirmed").await {
            Ok(_) => log::info!("✅ Successfully synced payment status to order"),
            Err(e) => {
                log::error!("❌ Failed to update order status: {}", e);
                return Err(format!("Failed to update order status: {}", e));
            }
        }
        
        // Also update in-memory store
        if let Some(updated) = app_state.monero_payments.update_payment_status(&payment_id, PaymentStatus::Confirmed) {
            log::info!("✅ Successfully updated payment status in memory: {:?}", updated.status);
        } else {
            log::error!("❌ Failed to update payment status in memory - payment not found");
        }
        
        // Verify that the order status was actually updated in the database
        let order_check = sqlx::query("SELECT id, status FROM orders WHERE payment_id = ?")
            .bind(payment_id)
            .fetch_optional(&app_state.db)
            .await;
            
        match order_check {
            Ok(Some(row)) => {
                let order_id: String = row.get("id");
                let status: String = row.get("status");
                log::info!("🔍 Verification - Order {} has status {} after update", order_id, status);
            }
            Ok(None) => log::warn!("⚠️ Verification - No order found with payment_id {}", payment_id),
            Err(e) => log::error!("❌ Verification query error: {}", e)
        }
        
        Ok(true)
    } else {
        // Payment is not confirmed or not in pending state
        log::info!("⏳ Payment {} remains in status {:?}", payment_id, payment.status);
        Ok(false)
    }
}

#[post("/admin/force-create-payment-links")]
pub async fn force_create_payment_links(
    app_state: web::Data<AppState>
) -> impl Responder {
    log::info!("🔧 Force creating payment links for all orders without payment_id");
    
    // 1. Find all orders without payment_id
    let orphaned_orders = sqlx::query(
        "SELECT id, user_id, total_amount FROM orders WHERE payment_id IS NULL OR payment_id = ''"
    )
    .fetch_all(&app_state.db)
    .await;
    
    if let Err(e) = &orphaned_orders {
        log::error!("Database error querying orphaned orders: {}", e);
        return HttpResponse::InternalServerError().json(json!({
            "success": false,
            "error": format!("Database error: {}", e)
        }));
    }
    
    let orphaned_orders = orphaned_orders.unwrap();
    if orphaned_orders.is_empty() {
        return HttpResponse::Ok().json(json!({
            "success": true,
            "message": "No orders without payment links found",
            "fixed_count": 0
        }));
    }
    
    log::info!("Found {} orders without payment links", orphaned_orders.len());
    
    let mut fixed_orders = Vec::new();
    
    // 2. For each order, create a new payment and link it
    for order in orphaned_orders {
        let order_id: String = order.get("id");
        let total_amount: f64 = order.get("total_amount");
        
        log::info!("Creating payment for order {} with amount {}", order_id, total_amount);
        
        // First create the payment in the database to avoid foreign key constraints
        let payment_id = format!("pay-{}", uuid::Uuid::new_v4().to_string());
        let address = "44AFFq5kSiGBoZ4NMDwYtN18obc8AemS33DBLWs3H7otXft3XjrpDtQGv7SqSsaBYBb98uNbr2VBBEt7f2wfn3RVGQBEP3A";
        let now = chrono::Utc::now().timestamp();
        
        // Insert the payment record first
        match sqlx::query(
            "INSERT INTO monero_payments (payment_id, order_id, amount, address, status, created_at, updated_at) 
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&payment_id)
        .bind(&order_id)
        .bind(total_amount)
        .bind(address)
        .bind("Pending")
        .bind(now)
        .bind(now)
        .execute(&app_state.db)
        .await {
            Ok(_) => {
                log::info!("✅ Created payment record in database: {}", payment_id);
                
                // Now update the order with the payment ID
                match sqlx::query(
                    "UPDATE orders SET payment_id = ? WHERE id = ?"
                )
                .bind(&payment_id)
                .bind(&order_id)
                .execute(&app_state.db)
                .await {
                    Ok(_) => {
                        log::info!("✅ Successfully linked payment {} to order {}", payment_id, order_id);
                        fixed_orders.push(json!({
                            "order_id": order_id,
                            "payment_id": payment_id,
                            "amount": total_amount
                        }));
                    },
                    Err(e) => {
                        log::error!("Failed to update order with payment ID: {}", e);
                    }
                }
            },
            Err(e) => {
                log::error!("Failed to create payment record: {}", e);
            }
        }
    }
    
    // Return the results
    HttpResponse::Ok().json(json!({
        "success": true,
        "fixed_count": fixed_orders.len(),
        "fixed_orders": fixed_orders
    }))
}

// Add a new endpoint to directly force update order status
#[post("/force-update-order-status/{order_id}")]
pub async fn force_update_order_status(
    app_state: web::Data<AppState>,
    path: web::Path<String>
) -> impl Responder {
    let order_id = path.into_inner();
    log::info!("🔄 Force updating order status for order: {}", order_id);
    
    // Get current order details
    let order = sqlx::query(
        "SELECT id, payment_id, status FROM orders WHERE id = ?"
    )
    .bind(&order_id)
    .fetch_optional(&app_state.db)
    .await;
    
    match order {
        Ok(Some(order_row)) => {
            let payment_id: String = order_row.get("payment_id");
            let current_status: String = order_row.get("status");
            
            log::info!("Order {} has payment_id {} and status {}", order_id, payment_id, current_status);
            
            // Update the order status directly to Confirmed
            match sqlx::query(
                "UPDATE orders SET status = 'Confirmed' WHERE id = ?"
            )
            .bind(&order_id)
            .execute(&app_state.db)
            .await {
                Ok(_) => {
                    log::info!("✅ Successfully updated order status to Confirmed");
                    
                    // If there's a payment ID, also make sure it's updated
                    if !payment_id.is_empty() {
                        match sqlx::query(
                            "UPDATE monero_payments SET status = 'Confirmed' WHERE payment_id = ?"
                        )
                        .bind(&payment_id)
                        .execute(&app_state.db)
                        .await {
                            Ok(_) => log::info!("✅ Also updated payment status"),
                            Err(e) => log::error!("Failed to update payment status: {}", e)
                        }
                    }
                    
                    HttpResponse::Ok().json(json!({
                        "success": true,
                        "message": "Order status updated to Confirmed"
                    }))
                },
                Err(e) => {
                    log::error!("Failed to update order status: {}", e);
                    HttpResponse::InternalServerError().json(json!({
                        "success": false,
                        "error": format!("Failed to update order status: {}", e)
                    }))
                }
            }
        },
        Ok(None) => {
            log::error!("Order {} not found", order_id);
            HttpResponse::NotFound().json(json!({
                "success": false,
                "error": "Order not found"
            }))
        },
        Err(e) => {
            log::error!("Database error: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": format!("Database error: {}", e)
            }))
        }
    }
}