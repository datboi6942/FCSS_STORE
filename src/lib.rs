#![allow(dead_code)]

use sqlx::SqlitePool;
use crate::monero::MoneroPaymentStore as MoneroPaymentManager;

pub struct AppState {
    pub db: SqlitePool,
    pub monero_payments: MoneroPaymentManager,
}

// Update the get_db_path function to point to the data directory
pub fn get_db_path() -> String {
    "data/secure_store.db".to_string()
}

pub mod admin;
pub mod auth;
pub mod middleware;
pub mod monero;
pub mod monero_api;
pub mod orders;
pub mod products;
pub mod types;
pub mod session;
pub mod monero_wallet;

// Re-export types for easier access
pub use types::*; 