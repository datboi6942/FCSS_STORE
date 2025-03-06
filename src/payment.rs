// src/payment.rs
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use chrono::Utc;
use uuid::Uuid;
use crate::AppState;
use log::{info, error};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub id: String,
    pub order_id: String,
    pub amount: f64,
    pub currency: String,
    pub method: String,
    pub status: String,
    pub session_id: Option<String>,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct PaymentRequest {
    pub order_id: String,
    pub amount: f64,
    pub currency: String,
    pub payment_method: String,
}

#[derive(Deserialize)]
pub struct PaymentVerification {
    pub session_id: String,
}

#[derive(Serialize)]
pub struct PaymentResponse {
    pub transaction_id: String,
    pub session_id: String,
    pub status: String,
}

pub async fn initiate_payment(
    payment_req: web::Json<PaymentRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let payment = payment_req.into_inner();
    
    info!("Initiating payment for order_id: {}, amount: {}", 
          payment.order_id, payment.amount);
    
    // Validate order exists
    let order_exists = sqlx::query!(
        "SELECT id FROM orders WHERE id = ?",
        payment.order_id
    )
    .fetch_optional(&state.db)
    .await;
    
    match order_exists {
        Ok(Some(_)) => info!("Order {} exists", payment.order_id),
        Ok(None) => {
            info!("Order {} not found", payment.order_id);
            return HttpResponse::BadRequest().json(
                serde_json::json!({"error": "Order not found"})
            );
        }
        Err(e) => {
            error!("Database error checking order: {}", e);
            return HttpResponse::InternalServerError().json(
                serde_json::json!({"error": "Error validating order"})
            );
        }
    }
    
    // Create transaction record
    let transaction_id = format!("txn-{}", Uuid::new_v4().simple());
    let session_id = format!("sess-{}", Uuid::new_v4().simple());
    let now = Utc::now();
    
    // Insert transaction using EXACT column names and order matching our schema
    let result = sqlx::query!(
        r#"
        INSERT INTO transactions 
        (id, order_id, amount, status, payment_method, session_id, currency, created_at) 
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        transaction_id,
        payment.order_id,
        payment.amount,
        "pending",
        payment.payment_method,
        session_id,
        payment.currency,
        now
    )
    .execute(&state.db)
    .await;
    
    match result {
        Ok(_) => {
            info!("Payment initiated: transaction_id={}, session_id={}", 
                 transaction_id, session_id);
            
            HttpResponse::Ok().json(PaymentResponse {
                transaction_id,
                session_id,
                status: "pending".to_string(),
            })
        }
        Err(e) => {
            error!("Failed to create transaction: {}", e);
            HttpResponse::InternalServerError().json(
                serde_json::json!({"error": "Failed to initiate payment"})
            )
        }
    }
}

pub async fn verify_payment(
    verification: web::Json<PaymentVerification>,
    state: web::Data<AppState>,
) -> impl Responder {
    let session_id = verification.session_id.clone();
    
    info!("Verifying payment with session_id: {}", session_id);
    
    // Find transaction by session_id - update the SELECT to match our schema
    let transaction = sqlx::query!(
        r#"
        SELECT id, order_id, payment_method
        FROM transactions 
        WHERE session_id = ?
        "#,
        session_id
    )
    .fetch_optional(&state.db)
    .await;
    
    match transaction {
        Ok(Some(txn)) => {
            // Update transaction status to 'completed'
            let update_result = sqlx::query!(
                "UPDATE transactions SET status = ? WHERE id = ?",
                "completed",
                txn.id
            )
            .execute(&state.db)
            .await;
            
            match update_result {
                Ok(_) => {
                    // Also update the order status
                    let order_update = sqlx::query!(
                        "UPDATE orders SET status = ? WHERE id = ?",
                        "paid",
                        txn.order_id
                    )
                    .execute(&state.db)
                    .await;
                    
                    match order_update {
                        Ok(_) => {
                            info!("Payment verified and completed for order: {}", txn.order_id);
                            HttpResponse::Ok().json(
                                serde_json::json!({
                                    "status": "success",
                                    "message": "Payment verified",
                                    "order_id": txn.order_id
                                })
                            )
                        }
                        Err(e) => {
                            error!("Failed to update order status: {}", e);
                            HttpResponse::InternalServerError().json(
                                serde_json::json!({"error": "Payment verified but order update failed"})
                            )
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to update transaction status: {}", e);
                    HttpResponse::InternalServerError().json(
                        serde_json::json!({"error": "Failed to complete payment"})
                    )
                }
            }
        }
        Ok(None) => {
            info!("No transaction found with session_id: {}", session_id);
            HttpResponse::NotFound().json(
                serde_json::json!({"error": "Payment session not found"})
            )
        }
        Err(e) => {
            error!("Database error: {}", e);
            HttpResponse::InternalServerError().json(
                serde_json::json!({"error": "Failed to verify payment"})
            )
        }
    }
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/payment")
            .route("/initiate", web::post().to(initiate_payment))
            .route("/verify", web::post().to(verify_payment))
    );
}