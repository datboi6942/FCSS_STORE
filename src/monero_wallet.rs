use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::error::Error;

#[derive(Debug, Serialize)]
struct RpcRequest {
    jsonrpc: String,
    id: String,
    method: String,
    params: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct RpcResponse {
    id: String,
    jsonrpc: String,
    result: Option<serde_json::Value>,
    error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
struct RpcError {
    code: i32,
    message: String,
}

pub struct MoneroWallet {
    rpc_url: String,
    client: Client,
    username: Option<String>,
    password: Option<String>,
}

impl MoneroWallet {
    pub fn new(rpc_url: &str, username: Option<&str>, password: Option<&str>) -> Result<Self, String> {
        Ok(Self {
            rpc_url: rpc_url.to_string(),
            client: Client::new(),
            username: username.map(|s| s.to_string()),
            password: password.map(|s| s.to_string()),
        })
    }

    pub async fn create_address(&self, label: &str) -> Result<String, Box<dyn Error>> {
        let params = serde_json::json!({
            "account_index": 0,
            "label": label
        });

        let result = self.call_rpc("create_address", params).await?;
        
        let address = result["address"].as_str()
            .ok_or("No address in response")?
            .to_string();
            
        println!("Created new Monero address for label {}: {}", label, address);
        Ok(address)
    }

    pub async fn get_balance(&self) -> Result<f64, Box<dyn Error>> {
        let params = serde_json::json!({ "account_index": 0 });
        
        let result = self.call_rpc("get_balance", params).await?;
        
        // Convert from atomic units to XMR (1 XMR = 1e12 atomic units)
        let balance = result["balance"].as_u64()
            .ok_or("No balance in response")? as f64 / 1_000_000_000_000.0;
            
        Ok(balance)
    }

    pub async fn check_transaction(&self, txid: &str, address: &str, amount: f64) -> Result<bool, Box<dyn Error>> {
        let params = serde_json::json!({
            "txid": txid,
            "address": address,
        });
        
        let result = self.call_rpc("check_tx_proof", params).await?;
        
        let received = result["received"].as_u64()
            .ok_or("No received amount in response")? as f64 / 1_000_000_000_000.0;
            
        // Check if the received amount matches expected amount
        let confirmed = result["confirmations"].as_u64().unwrap_or(0) >= 10;
        
        Ok(received >= amount && confirmed)
    }

    async fn call_rpc(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value, Box<dyn Error>> {
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            id: "1".to_string(),
            method: method.to_string(),
            params,
        };

        let mut req_builder = self.client.post(&self.rpc_url).json(&request);
        
        // Add basic auth if credentials provided
        if let (Some(username), Some(password)) = (&self.username, &self.password) {
            req_builder = req_builder.basic_auth(username, Some(password));
        }
        
        let response = req_builder.send().await?;
        
        if !response.status().is_success() {
            return Err(format!("RPC request failed with status: {}", response.status()).into());
        }
        
        let rpc_response: RpcResponse = response.json().await?;
        
        if let Some(error) = rpc_response.error {
            return Err(format!("RPC error: {} (code: {})", error.message, error.code).into());
        }
        
        rpc_response.result.ok_or_else(|| "No result in response".into())
    }
} 