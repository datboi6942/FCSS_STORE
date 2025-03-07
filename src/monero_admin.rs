use actix_web::{web, HttpResponse, Responder, get, post};
use serde::Serialize;
use crate::AppState;
use crate::monero::{PaymentStatus, MoneroPaymentRequest};

#[derive(Serialize)]
pub struct AdminPaymentResponse {
    pub success: bool,
    pub message: Option<String>,
    pub transactions: Option<Vec<MoneroPaymentRequest>>,
}

#[get("/admin/transactions")]
pub async fn list_transactions(
    app_state: web::Data<AppState>,
) -> impl Responder {
    // PLACEHOLDER: In production, check admin permissions here
    
    // Get all transactions from storage
    let transactions = app_state.monero_payments.get_all_payments();
    
    HttpResponse::Ok().json(AdminPaymentResponse {
        success: true,
        message: None,
        transactions: Some(transactions),
    })
}

#[post("/admin/confirm/{payment_id}")]
pub async fn admin_confirm_payment(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    // PLACEHOLDER: In production, check admin permissions here
    
    let payment_id = path.into_inner();
    
    if let Some(updated_payment) = app_state.monero_payments.update_payment_status(&payment_id, PaymentStatus::Confirmed) {
        // Log this admin action for audit purposes
        println!("ADMIN ACTION: Manual payment confirmation for payment {}", payment_id);
        
        HttpResponse::Ok().json(AdminPaymentResponse {
            success: true,
            message: Some("Payment manually confirmed by admin".to_string()),
            transactions: Some(vec![updated_payment]),
        })
    } else {
        HttpResponse::NotFound().json(AdminPaymentResponse {
            success: false,
            message: Some("Payment not found".to_string()),
            transactions: None,
        })
    }
}

#[post("/admin/refresh_wallet")]
pub async fn refresh_wallet(
    app_state: web::Data<AppState>,
) -> impl Responder {
    // PLACEHOLDER: In production, this would connect to your Monero wallet
    // and refresh/scan for new transactions
    
    println!("ADMIN ACTION: Manual wallet refresh triggered");
    
    // For the mock implementation, just run the payment checker
    app_state.monero_payments.check_payments();
    
    HttpResponse::Ok().json(AdminPaymentResponse {
        success: true,
        message: Some("Wallet refresh triggered".to_string()),
        transactions: None,
    })
}

pub fn init_routes() -> actix_web::Scope {
    web::scope("/monero")
        .service(list_transactions)
        .service(admin_confirm_payment)
        .service(refresh_wallet)
} 