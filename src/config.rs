use std::env;

pub struct Config {
    pub database_url: String,
    pub monero_rpc_url: String,
    pub monero_rpc_user: Option<String>,
    pub monero_rpc_password: Option<String>,
    pub monero_wallet_filename: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv::dotenv().ok();
        
        Self {
            database_url: env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:data/store.db".to_string()),
            monero_rpc_url: env::var("MONERO_RPC_URL").unwrap_or_else(|_| "http://localhost:18081/json_rpc".to_string()),
            monero_rpc_user: env::var("MONERO_RPC_USER").ok(),
            monero_rpc_password: env::var("MONERO_RPC_PASSWORD").ok(),
            monero_wallet_filename: env::var("MONERO_WALLET_FILENAME").unwrap_or_else(|_| "store_wallet".to_string()),
        }
    }
} 