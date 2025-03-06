// src/payment.rs
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use chrono::Utc;
use uuid::Uuid;
use crate::AppState;

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
    pub amount: String,
    pub currency: String,
    pub method: String,
}

#[derive(Deserialize)]
pub struct PaymentVerification {
    pub session_id: String,
}

#[derive(Serialize)]
pub struct PaymentResponse {
    pub status: String,
    pub message: String,
    pub session_id: Option<String>,
}

pub async fn initiate_payment(
    payment_data: web::Json<PaymentRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let payment = payment_data.into_inner();
    
    // Validate order exists
    let order_exists = sqlx::query!("SELECT id FROM orders WHERE id = ?", payment.order_id)
        .fetch_optional(&state.db)
        .await;
    
    match order_exists {
        Ok(Some(_)) => {/* Order exists */},
        Ok(None) => {
            return HttpResponse::BadRequest().json(
                serde_json::json!({"error": "Order not found"})
            );
        }
        Err(e) => {
            log::error!("Database error: {}", e);
            return HttpResponse::InternalServerError().json(
                serde_json::json!({"error": "Error validating order"})
            );
        }
    }
    
    // Parse amount
    let amount = match payment.amount.parse::<f64>() {
        Ok(a) => a,
        Err(_) => {
            return HttpResponse::BadRequest().json(
                serde_json::json!({"error": "Invalid amount format"})
            );
        }
    };
    
    // Create transaction
    let transaction_id = Uuid::new_v4().to_string();
    let session_id = Uuid::new_v4().to_string();
    let now = Utc::now();
    
    // Create a local binding for session_id to extend its lifetime
    let session_id_clone = session_id.clone();
    
    let result = sqlx::query!(
        "INSERT INTO transactions (id, order_id, amount, currency, method, status, session_id, created_at) 
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        transaction_id,
        payment.order_id,
        amount,
        payment.currency,
        payment.method,
        "pending",
        session_id_clone,  // Use the cloned value
        now
    )
    .execute(&state.db)
    .await;
    
    match result {
        Ok(_) => {
            HttpResponse::Ok().json(PaymentResponse {
                status: "pending".to_string(),
                message: "Payment initiated successfully".to_string(),
                session_id: Some(session_id),
            })
        }
        Err(e) => {
            log::error!("Failed to initiate payment: {}", e);
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
    let session_id = &verification.session_id;
    
    // Find transaction by session_id
    let transaction = sqlx::query!(
        "SELECT id, order_id FROM transactions WHERE session_id = ?",
        session_id
    )
    .fetch_optional(&state.db)
    .await;
    
    match transaction {
        Ok(Some(transaction)) => {
            // Update transaction status to completed
            let update_result = sqlx::query!(
                "UPDATE transactions SET status = ? WHERE id = ?",
                "completed",
                transaction.id
            )
            .execute(&state.db)
            .await;
            
            // Also update order status
            let order_update = sqlx::query!(
                "UPDATE orders SET status = ? WHERE id = ?",
                "completed",
                transaction.order_id
            )
            .execute(&state.db)
            .await;
            
            match (update_result, order_update) {
                (Ok(_), Ok(_)) => {
                    HttpResponse::Ok().json(PaymentResponse {
                        status: "completed".to_string(),
                        message: "Payment verified successfully".to_string(),
                        session_id: Some(session_id.clone()),
                    })
                }
                _ => {
                    log::error!("Failed to update transaction status");
                    HttpResponse::InternalServerError().json(
                        serde_json::json!({"error": "Failed to verify payment"})
                    )
                }
            }
        }
        Ok(None) => {
            HttpResponse::NotFound().json(
                serde_json::json!({"error": "Payment session not found"})
            )
        }
        Err(e) => {
            log::error!("Database error: {}", e);
            HttpResponse::InternalServerError().json(
                serde_json::json!({"error": "Error verifying payment"})
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