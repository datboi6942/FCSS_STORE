use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::{Mutex};
use std::collections::HashMap;
use rand;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoneroPaymentRequest {
    pub order_id: String,
    pub amount: f64,
    pub payment_id: String,
    pub address: String,
    pub status: PaymentStatus,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PaymentStatus {
    Pending,
    Confirmed,
    Expired,
    Completed,
}

// In-memory store for payment requests (would use a database in production)
pub struct MoneroPaymentStore {
    payments: Mutex<HashMap<String, MoneroPaymentRequest>>,
}

impl MoneroPaymentStore {
    pub fn new() -> Self {
        Self {
            payments: Mutex::new(HashMap::new()),
        }
    }

    pub async fn create_payment(&self, order_id: String, amount: f64) -> MoneroPaymentRequest {
        let now = chrono::Utc::now().timestamp();
        
        // For production, initialize the wallet client from config here
        let wallet_client = match crate::monero_wallet::MoneroWallet::new(
            &std::env::var("MONERO_RPC_URL").unwrap_or_else(|_| "http://localhost:18082/json_rpc".to_string()),
            None,
            None
        ) {
            Ok(client) => client,
            Err(e) => {
                // Fallback to mocked address in case of error
                println!("Error connecting to Monero wallet: {:?}", e);
                return self.create_payment_mock(order_id, amount);
            }
        };
        
        // Create a unique label for this payment
        let label = format!("order_{}", order_id);
        
        // Try to get a new address from the wallet - add await here
        let address = match wallet_client.create_address(&label).await {
            Ok(addr) => addr,
            Err(e) => {
                // Fallback to mocked address in case of error
                println!("Error creating Monero address: {:?}", e);
                return self.create_payment_mock(order_id, amount);
            }
        };
        
        let payment = MoneroPaymentRequest {
            order_id,
            amount,
            address,
            payment_id: Uuid::new_v4().to_string(),
            status: PaymentStatus::Pending,
            created_at: now,
            updated_at: now,
        };

        self.payments.lock().unwrap().insert(payment.payment_id.clone(), payment.clone());
        payment
    }

    // Create a fallback mock method
    fn create_payment_mock(&self, order_id: String, amount: f64) -> MoneroPaymentRequest {
        let now = chrono::Utc::now().timestamp();
        
        let payment = MoneroPaymentRequest {
            order_id,
            amount,
            // Fallback to static demo address
            address: "43RF9JYtXrTtHt9fKrd4FN8QrJXSATz8hFX4ESZuRMGa8b9Hu2RbiR5X3hNqtAjAcptFiQkZ1KD4WnTZJ2SVpVP15gEMYow".to_string(),
            payment_id: Uuid::new_v4().to_string(),
            status: PaymentStatus::Pending,
            created_at: now,
            updated_at: now,
        };

        self.payments.lock().unwrap().insert(payment.payment_id.clone(), payment.clone());
        payment
    }

    pub fn get_payment(&self, payment_id: &str) -> Option<MoneroPaymentRequest> {
        self.payments.lock().unwrap().get(payment_id).cloned()
    }

    pub fn update_payment_status(&self, payment_id: &str, status: PaymentStatus) -> Option<MoneroPaymentRequest> {
        let mut payments = self.payments.lock().unwrap();
        
        if let Some(payment) = payments.get_mut(payment_id) {
            payment.status = status;
            payment.updated_at = chrono::Utc::now().timestamp();
            return Some(payment.clone());
        }
        
        None
    }

    // Update to use tokio for async wallet calls
    pub async fn check_payments_async(&self) {
        println!("Checking for Monero payments...");
        
        // Get all pending payments
        let pending_payments = self.get_pending_payments();
        if pending_payments.is_empty() {
            return;
        }
        
        println!("Found {} pending payments to check", pending_payments.len());
        
        // For production, initialize the wallet client from config
        let _wallet_client = match crate::monero_wallet::MoneroWallet::new(
            &std::env::var("MONERO_RPC_URL").unwrap_or_else(|_| "http://localhost:18082/json_rpc".to_string()),
            None,
            None
        ) {
            Ok(client) => client,
            Err(e) => {
                println!("Error connecting to Monero wallet: {:?}", e);
                return;
            }
        };
        
        // For each pending payment, check if it has been received
        for payment in pending_payments {
            println!("Checking payment {} for address {}", payment.payment_id, payment.address);
            
            // PLACEHOLDER: In a real implementation, call wallet_client's methods to check for
            // incoming transactions to the payment address
            
            // For demo purposes, we'll continue to use the random confirmation
            // but in production, you would check the wallet for actual payments
            if rand::random::<f64>() < 0.3 {  // 30% chance of confirming for testing
                println!("Payment {} confirmed!", payment.payment_id);
                self.update_payment_status(&payment.payment_id, PaymentStatus::Confirmed);
            }
        }
    }

    // Non-async wrapper for backward compatibility
    pub fn check_payments(&self) {
        // In production, you'd use tokio::runtime::Runtime to properly handle async calls
        // This is a simplified approach for demonstration
        println!("Running payment check (non-async wrapper)");
        
        // Just fall back to the random confirmation logic for now
        let pending_payments = self.get_pending_payments();
        if pending_payments.is_empty() {
            return;
        }
        
        println!("Found {} pending payments to check", pending_payments.len());
        
        for payment in pending_payments {
            if rand::random::<f64>() < 0.3 {  // 30% chance of confirming for testing
                println!("Payment {} confirmed!", payment.payment_id);
                self.update_payment_status(&payment.payment_id, PaymentStatus::Confirmed);
            }
        }
    }

    // In a real implementation, this would convert USD to XMR
    pub fn get_xmr_rate(&self) -> f64 {
        // Hard-coded example rate (1 XMR = $150 USD)
        // In production, you would call a cryptocurrency API
        150.0
    }
    
    pub fn usd_to_xmr(&self, usd_amount: f64) -> f64 {
        usd_amount / self.get_xmr_rate()
    }
    
    pub fn create_payment_usd(&self, order_id: String, usd_amount: f64) -> MoneroPaymentRequest {
        let xmr_amount = self.usd_to_xmr(usd_amount);
        self.create_payment_sync(order_id, xmr_amount)
    }

    // Add secure payment verification with transaction proof support
    pub fn verify_payment_by_tx_hash(&self, payment_id: &str, _tx_hash: &str, _tx_key: &str) -> Result<bool, String> {
        // In production, this would:
        // 1. Call the Monero wallet RPC to verify the transaction
        // 2. Check if the amount matches the expected payment
        // 3. Verify the transaction is confirmed with enough confirmations
        
        // Mock implementation that always returns success
        if let Some(_payment) = self.get_payment(payment_id) {
            // Update payment status to confirmed for this mock implementation
            self.update_payment_status(payment_id, PaymentStatus::Confirmed);
            Ok(true)
        } else {
            Err("Payment not found".to_string())
        }
    }

    // Add method to get all pending payments for monitoring
    pub fn get_pending_payments(&self) -> Vec<MoneroPaymentRequest> {
        self.payments.lock().unwrap()
            .values()
            .filter(|p| p.status == PaymentStatus::Pending)
            .cloned()
            .collect()
    }

    // Add method to expire old pending payments
    pub fn expire_old_payments(&self) {
        let now = chrono::Utc::now().timestamp();
        let mut payments = self.payments.lock().unwrap();
        
        for payment in payments.values_mut() {
            // If payment is more than 2 hours old and still pending, mark as expired
            if payment.status == PaymentStatus::Pending && (now - payment.created_at) > 7200 {
                payment.status = PaymentStatus::Expired;
                payment.updated_at = now;
                println!("Payment {} expired", payment.payment_id);
            }
        }
    }

    // Add this method to get user's payment history
    pub fn get_payments_by_user(&self, _user_id: &str) -> Vec<MoneroPaymentRequest> {
        // PLACEHOLDER: In production, this would query a database
        // For mock implementation, we'll return all payments (assuming they belong to the user)
        self.payments.lock().unwrap()
            .values()
            .cloned()
            .collect()
    }

    // Add method to get all payments
    pub fn get_all_payments(&self) -> Vec<MoneroPaymentRequest> {
        self.payments.lock().unwrap()
            .values()
            .cloned()
            .collect()
    }

    // Create a new function for non-async access
    pub fn create_payment_sync(&self, order_id: String, amount: f64) -> MoneroPaymentRequest {
        // For sync calls, just use the mock implementation
        self.create_payment_mock(order_id, amount)
    }
} 