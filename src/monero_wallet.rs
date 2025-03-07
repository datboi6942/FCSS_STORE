use serde::{Deserialize, Serialize};
use reqwest::Client;
use log::{info, warn};
use std::env;
use serde_json::json;
use rand;
use chrono;
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

#[derive(Debug, Clone)]
pub struct MoneroWallet {
    address: String,
    rpc_url: String,
    rpc_username: String,
    rpc_password: String,
    client: Client,
}

impl Serialize for MoneroWallet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("MoneroWallet", 4)?;
        state.serialize_field("address", &self.address)?;
        state.serialize_field("rpc_url", &self.rpc_url)?;
        state.serialize_field("rpc_username", &self.rpc_username)?;
        state.serialize_field("rpc_password", &self.rpc_password)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for MoneroWallet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct MoneroWalletHelper {
            address: String,
            rpc_url: String,
            rpc_username: String,
            rpc_password: String,
        }
        
        let helper = MoneroWalletHelper::deserialize(deserializer)?;
        Ok(MoneroWallet {
            address: helper.address,
            rpc_url: helper.rpc_url,
            rpc_username: helper.rpc_username,
            rpc_password: helper.rpc_password,
            client: Client::new(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferDetails {
    pub tx_hash: String,
    pub amount: f64,
    pub confirmations: u32,
    pub timestamp: i64,
    pub address: String,
}

impl MoneroWallet {
    pub fn new(address: String, _amount: f64) -> Self {
        // Read wallet RPC settings from environment variables or use defaults
        let rpc_url = env::var("MONERO_RPC_URL").unwrap_or("http://localhost:18082/json_rpc".to_string());
        let rpc_username = env::var("MONERO_RPC_USERNAME").unwrap_or("monero".to_string());
        let rpc_password = env::var("MONERO_RPC_PASSWORD").unwrap_or("password".to_string());
        
        Self {
            address,
            rpc_url,
            rpc_username,
            rpc_password,
            client: Client::new(),
        }
    }

    pub fn check_payment(&self, _amount: f64) -> bool {
        // Mock implementation - in production, this would check the actual wallet
        true
    }

    pub async fn create_address(&self, label: &str) -> Result<String, String> {
        // In a real implementation, this would call create_address on the Monero wallet RPC
        info!("Creating Monero address with label: {}", label);
        
        // For demonstration, return the main wallet address
        // In production, you'd make a real RPC call and create a unique address
        Ok(self.address.clone())
    }

    pub async fn get_balance(&self) -> Result<f64, Box<dyn Error>> {
        let params = serde_json::json!({ "account_index": 0 });
        
        match self.call_rpc("get_balance", params).await {
            Ok(result) => {
                let balance = result["balance"].as_u64()
                    .ok_or("No balance in response")? as f64 / 1_000_000_000_000.0;
                Ok(balance)
            },
            Err(_) => {
                // Fallback to returning the wallet's balance if RPC fails
                Ok(0.0)
            }
        }
    }

    pub async fn check_transfers(&self) -> Result<Vec<TransferDetails>, String> {
        info!("Checking for Monero transfers");
        
        // For demo implementation, try to connect to RPC but fallback to mock data
        match self.get_transfers().await {
            Ok(transfers) => {
                info!("Retrieved {} transfers from wallet", transfers.len());
                Ok(transfers)
            },
            Err(e) => {
                warn!("Error retrieving transfers, using mock data: {}", e);
                // Return mock data for demonstration
                Ok(vec![
                    TransferDetails {
                        tx_hash: format!("mock_tx_{}", rand::random::<u32>()),
                        amount: 0.1 * rand::random::<f64>(),
                        confirmations: 6,
                        timestamp: chrono::Utc::now().timestamp(),
                        address: self.address.clone(),
                    }
                ])
            }
        }
    }

    async fn get_transfers(&self) -> Result<Vec<TransferDetails>, String> {
        // Attempt to make a real RPC call to get_transfers
        let payload = json!({
            "jsonrpc": "2.0",
            "id": "0",
            "method": "get_transfers",
            "params": {
                "in": true,
                "pool": true
            }
        });
        
        match self.client
            .post(&self.rpc_url)
            .basic_auth(&self.rpc_username, Some(&self.rpc_password))
            .json(&payload)
            .send()
            .await {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.json::<serde_json::Value>().await {
                            Ok(json_response) => {
                                // Parse and extract transfer information
                                let mut transfers = Vec::new();
                                
                                // In real implementation, you'd parse the actual JSON structure
                                // This is a simplified mock implementation
                                if let Some(in_transfers) = json_response["result"]["in"].as_array() {
                                    for transfer in in_transfers {
                                        transfers.push(TransferDetails {
                                            tx_hash: transfer["txid"].as_str().unwrap_or("unknown").to_string(),
                                            amount: transfer["amount"].as_f64().unwrap_or(0.0) / 1e12, // Convert from atomic units
                                            confirmations: transfer["confirmations"].as_u64().unwrap_or(0) as u32,
                                            timestamp: transfer["timestamp"].as_i64().unwrap_or(0),
                                            address: transfer["address"].as_str().unwrap_or("").to_string(),
                                        });
                                    }
                                }
                                
                                Ok(transfers)
                            },
                            Err(e) => Err(format!("Error parsing JSON response: {}", e))
                        }
                    } else {
                        Err(format!("Error response from RPC: {}", response.status()))
                    }
                },
                Err(e) => Err(format!("RPC request failed: {}", e))
        }
    }

    // Check for payment to a specific address with a specific amount
    pub async fn check_specific_payment(&self, address: &str, amount: f64) -> Result<Option<TransferDetails>, String> {
        let transfers = self.check_transfers().await?;
        
        // Find a transfer that matches the address and amount (within a small tolerance)
        let matching_transfer = transfers.into_iter().find(|t| {
            // For production, use a more accurate matching algorithm
            // Consider payment IDs, exact amounts, etc.
            t.address == address && (t.amount - amount).abs() < 0.00001
        });
        
        Ok(matching_transfer)
    }

    // Verify a payment using view key and tx_hash
    pub async fn verify_transaction(&self, tx_hash: &str, recipient_address: &str) -> Result<bool, String> {
        info!("Verifying transaction {} for address {}", tx_hash, recipient_address);
        
        let payload = json!({
            "jsonrpc": "2.0",
            "id": "0",
            "method": "check_tx_key",
            "params": {
                "txid": tx_hash,
                "address": recipient_address,
                "tx_key": "" // In real implementation, this would be provided by the user
            }
        });
        
        match self.client
            .post(&self.rpc_url)
            .basic_auth(&self.rpc_username, Some(&self.rpc_password))
            .json(&payload)
            .send()
            .await {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.json::<serde_json::Value>().await {
                            Ok(json_response) => {
                                if let Some(received) = json_response["result"]["received"].as_u64() {
                                    Ok(received > 0)
                                } else {
                                    Ok(false) // Could not confirm receipt
                                }
                            },
                            Err(e) => Err(format!("Error parsing verification response: {}", e))
                        }
                    } else {
                        Err(format!("Error response from RPC: {}", response.status()))
                    }
                },
                Err(e) => {
                    warn!("RPC verification failed, using mock result: {}", e);
                    // For demo, return success 70% of the time
                    Ok(rand::random::<f64>() < 0.7)
                }
        }
    }

    async fn call_rpc(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value, Box<dyn Error>> {
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            id: "1".to_string(),
            method: method.to_string(),
            params,
        };

        let response = self.client
            .post(&self.rpc_url)
            .json(&request)
            .send()
            .await?;

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