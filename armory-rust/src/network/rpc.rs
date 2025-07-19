/// Bitcoin Core RPC client with failover support
/// 
/// Provides robust RPC communication with multiple Bitcoin Core nodes,
/// automatic failover, and comprehensive error handling.

use crate::error::{NetworkError, NetworkResult};
use bitcoin::{Transaction, Txid, Address, Amount, Network, Block, BlockHash};
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, warn, error, info};
use url::Url;

/// RPC endpoint configuration
#[derive(Debug, Clone)]
pub struct RpcEndpoint {
    pub url: Url,
    pub username: Option<String>,
    pub password: Option<String>,
    pub timeout: Duration,
}

impl RpcEndpoint {
    /// Create new RPC endpoint with authentication
    pub fn new(url: &str, username: Option<String>, password: Option<String>) -> NetworkResult<Self> {
        let url = Url::parse(url)
            .map_err(|e| NetworkError::InvalidResponse(format!("Invalid RPC URL: {}", e)))?;
        
        Ok(Self {
            url,
            username,
            password,
            timeout: Duration::from_secs(30),
        })
    }

    /// Create endpoint with custom timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

/// Bitcoin Core RPC client with multiple endpoint failover
#[derive(Debug)]
pub struct RpcClient {
    endpoints: Vec<RpcEndpoint>,
    client: Client,
    current_endpoint: usize,
    network: Network,
}

/// RPC request structure
#[derive(Debug, Serialize)]
struct RpcRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: Value,
}

/// RPC response structure
#[derive(Debug, Deserialize)]
struct RpcResponse<T> {
    result: Option<T>,
    error: Option<RpcError>,
    id: u64,
}

/// RPC error structure
#[derive(Debug, Deserialize)]
struct RpcError {
    code: i32,
    message: String,
}

impl RpcClient {
    /// Create new RPC client with multiple endpoints
    pub fn new(endpoints: Vec<RpcEndpoint>, network: Network) -> NetworkResult<Self> {
        if endpoints.is_empty() {
            return Err(NetworkError::Connection("No RPC endpoints provided".to_string()));
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .map_err(|e| NetworkError::Connection(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            endpoints,
            client,
            current_endpoint: 0,
            network,
        })
    }

    /// Create client with single endpoint
    pub fn single_endpoint(endpoint: RpcEndpoint, network: Network) -> NetworkResult<Self> {
        Self::new(vec![endpoint], network)
    }

    /// Get current active endpoint
    pub fn current_endpoint(&self) -> &RpcEndpoint {
        &self.endpoints[self.current_endpoint]
    }

    /// Get number of configured endpoints
    pub fn endpoint_count(&self) -> usize {
        self.endpoints.len()
    }

    /// Get current endpoint index (for testing)
    #[cfg(test)]
    pub fn current_endpoint_index(&self) -> usize {
        self.current_endpoint
    }

    /// Manually trigger failover (for testing)
    #[cfg(test)] 
    pub fn test_failover_to_next_endpoint(&mut self) {
        self.failover_to_next_endpoint();
    }

    /// Switch to next endpoint for failover
    fn failover_to_next_endpoint(&mut self) {
        let old_endpoint = self.current_endpoint;
        self.current_endpoint = (self.current_endpoint + 1) % self.endpoints.len();
        
        if self.current_endpoint != old_endpoint {
            info!("Failed over from endpoint {} to {}", old_endpoint, self.current_endpoint);
        }
    }

    /// Make RPC call with automatic failover
    async fn call_with_failover<T>(&mut self, method: &str, params: Value) -> NetworkResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut attempts = 0;
        let max_attempts = self.endpoints.len() * 2; // Try each endpoint twice

        while attempts < max_attempts {
            match self.make_rpc_call(method, params.clone()).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    warn!("RPC call failed on endpoint {}: {}", self.current_endpoint, e);
                    self.failover_to_next_endpoint();
                    attempts += 1;
                }
            }
        }

        Err(NetworkError::Rpc(format!(
            "All {} endpoints failed after {} attempts", 
            self.endpoints.len(), 
            attempts
        )))
    }

    /// Make individual RPC call to current endpoint
    async fn make_rpc_call<T>(&self, method: &str, params: Value) -> NetworkResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let endpoint = self.current_endpoint();
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: method.to_string(),
            params,
        };

        debug!("Making RPC call to {}: {}", endpoint.url, method);

        let mut req = self.client.post(endpoint.url.as_str())
            .json(&request)
            .header("Content-Type", "application/json");

        // Add authentication if provided
        if let (Some(username), Some(password)) = (&endpoint.username, &endpoint.password) {
            req = req.basic_auth(username, Some(password));
        }

        let response = timeout(endpoint.timeout, req.send())
            .await
            .map_err(|_| NetworkError::Timeout)?
            .map_err(|e| NetworkError::Connection(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(NetworkError::Rpc(format!(
                "HTTP error {}: {}", 
                response.status(), 
                response.text().await.unwrap_or_default()
            )));
        }

        let rpc_response: RpcResponse<T> = response
            .json()
            .await
            .map_err(|e| NetworkError::InvalidResponse(format!("Failed to parse JSON: {}", e)))?;

        if let Some(error) = rpc_response.error {
            return Err(NetworkError::Rpc(format!("RPC error {}: {}", error.code, error.message)));
        }

        rpc_response.result.ok_or_else(|| {
            NetworkError::InvalidResponse("RPC response missing result field".to_string())
        })
    }

    /// Get blockchain information
    pub async fn get_blockchain_info(&mut self) -> NetworkResult<BlockchainInfo> {
        self.call_with_failover("getblockchaininfo", json!([])).await
    }

    /// Get network information
    pub async fn get_network_info(&mut self) -> NetworkResult<NetworkInfo> {
        self.call_with_failover("getnetworkinfo", json!([])).await
    }

    /// Get best block hash
    pub async fn get_best_block_hash(&mut self) -> NetworkResult<BlockHash> {
        let hash_str: String = self.call_with_failover("getbestblockhash", json!([])).await?;
        hash_str.parse()
            .map_err(|e| NetworkError::InvalidResponse(format!("Invalid block hash: {}", e)))
    }

    /// Get block by hash
    pub async fn get_block(&mut self, block_hash: &BlockHash) -> NetworkResult<Block> {
        let block_hex: String = self.call_with_failover(
            "getblock", 
            json!([block_hash.to_string(), 0])
        ).await?;
        
        let block_bytes = hex::decode(&block_hex)
            .map_err(|e| NetworkError::InvalidResponse(format!("Invalid block hex: {}", e)))?;
        
        bitcoin::consensus::deserialize(&block_bytes)
            .map_err(|e| NetworkError::InvalidResponse(format!("Failed to deserialize block: {}", e)))
    }

    /// Get transaction by ID
    pub async fn get_transaction(&mut self, txid: &Txid) -> NetworkResult<Transaction> {
        let tx_hex: String = self.call_with_failover(
            "getrawtransaction", 
            json!([txid.to_string(), false])
        ).await?;
        
        let tx_bytes = hex::decode(&tx_hex)
            .map_err(|e| NetworkError::InvalidResponse(format!("Invalid transaction hex: {}", e)))?;
        
        bitcoin::consensus::deserialize(&tx_bytes)
            .map_err(|e| NetworkError::InvalidResponse(format!("Failed to deserialize transaction: {}", e)))
    }

    /// Broadcast transaction to network
    pub async fn send_raw_transaction(&mut self, transaction: &Transaction) -> NetworkResult<Txid> {
        let tx_hex = hex::encode(bitcoin::consensus::serialize(transaction));
        let txid_str: String = self.call_with_failover(
            "sendrawtransaction", 
            json!([tx_hex])
        ).await?;
        
        txid_str.parse()
            .map_err(|e| NetworkError::InvalidResponse(format!("Invalid transaction ID: {}", e)))
    }

    /// Test mempool acceptance of transaction
    pub async fn test_mempool_accept(&mut self, transaction: &Transaction) -> NetworkResult<MempoolAcceptResult> {
        let tx_hex = hex::encode(bitcoin::consensus::serialize(transaction));
        let results: Vec<MempoolAcceptResult> = self.call_with_failover(
            "testmempoolaccept", 
            json!([[tx_hex]])
        ).await?;
        
        results.into_iter().next()
            .ok_or_else(|| NetworkError::InvalidResponse("Empty mempool accept result".to_string()))
    }

    /// Get estimated fee rate for confirmation target
    pub async fn estimate_smart_fee(&mut self, conf_target: u32) -> NetworkResult<EstimateSmartFeeResult> {
        self.call_with_failover(
            "estimatesmartfee", 
            json!([conf_target])
        ).await
    }

    /// Get UTXO information for address
    pub async fn get_address_utxos(&mut self, address: &Address) -> NetworkResult<Vec<UtxoInfo>> {
        // Note: This requires Bitcoin Core with address indexing enabled
        self.call_with_failover(
            "getaddressutxos", 
            json!([{"addresses": [address.to_string()]}])
        ).await
    }

    /// Check if endpoint is reachable
    pub async fn ping(&mut self) -> NetworkResult<()> {
        let _: Option<Value> = self.call_with_failover("ping", json!([])).await?;
        Ok(())
    }

    /// Get connection count
    pub async fn get_connection_count(&mut self) -> NetworkResult<u32> {
        self.call_with_failover("getconnectioncount", json!([])).await
    }
}

/// Blockchain information structure
#[derive(Debug, Deserialize)]
pub struct BlockchainInfo {
    pub chain: String,
    pub blocks: u64,
    pub headers: u64,
    pub bestblockhash: String,
    pub difficulty: f64,
    pub verificationprogress: f64,
}

/// Network information structure  
#[derive(Debug, Deserialize)]
pub struct NetworkInfo {
    pub version: u32,
    pub subversion: String,
    pub protocolversion: u32,
    pub connections: u32,
    pub networks: Vec<NetworkDetails>,
}

/// Network details structure
#[derive(Debug, Deserialize)]
pub struct NetworkDetails {
    pub name: String,
    pub limited: bool,
    pub reachable: bool,
    pub proxy: String,
    pub proxy_randomize_credentials: bool,
}

/// Mempool accept result
#[derive(Debug, Deserialize)]
pub struct MempoolAcceptResult {
    pub txid: String,
    pub allowed: bool,
    pub reject_reason: Option<String>,
}

/// Fee estimation result
#[derive(Debug, Deserialize)]
pub struct EstimateSmartFeeResult {
    pub feerate: Option<f64>, // BTC per kB
    pub errors: Option<Vec<String>>,
    pub blocks: u32,
}

/// UTXO information
#[derive(Debug, Deserialize)]
pub struct UtxoInfo {
    pub address: String,
    pub txid: String,
    pub outputIndex: u32,
    pub script: String,
    pub satoshis: u64,
    pub height: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_rpc_client_creation() {
        let endpoint = RpcEndpoint::new(
            "http://localhost:8332", 
            Some("user".to_string()), 
            Some("pass".to_string())
        ).unwrap();
        
        let client = RpcClient::single_endpoint(endpoint, Network::Regtest);
        assert!(client.is_ok());
    }

    #[tokio::test] 
    async fn test_rpc_endpoint_parsing() {
        let endpoint = RpcEndpoint::new(
            "http://localhost:8332",
            None,
            None
        ).unwrap();
        
        assert_eq!(endpoint.url.as_str(), "http://localhost:8332/");
        assert_eq!(endpoint.timeout, Duration::from_secs(30));
    }

    #[tokio::test]
    async fn test_rpc_failover_logic() {
        let endpoint1 = RpcEndpoint::new("http://localhost:8332", None, None).unwrap();
        let endpoint2 = RpcEndpoint::new("http://localhost:8333", None, None).unwrap();
        
        let mut client = RpcClient::new(vec![endpoint1, endpoint2], Network::Regtest).unwrap();
        
        assert_eq!(client.current_endpoint, 0);
        client.failover_to_next_endpoint();
        assert_eq!(client.current_endpoint, 1);
        client.failover_to_next_endpoint();
        assert_eq!(client.current_endpoint, 0); // Wraps around
    }
}