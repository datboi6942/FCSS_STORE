use actix_web::{web, HttpResponse, Responder, get, post, HttpRequest};
use serde::{Deserialize, Serialize};
use crate::AppState;
use crate::monero::{PaymentStatus, MoneroPaymentRequest};
use serde_json::json;
use rand;

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

pub fn init_routes() -> actix_web::Scope {
    web::scope("/monero")
        .service(create_payment)
        .service(check_payment)
        .service(mock_confirm_payment)
        .service(finalize_order)
        .service(submit_proof)
        .service(get_user_transactions)
        .service(force_check_payment)
} 