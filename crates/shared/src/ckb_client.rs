use crate::{
	error::{CkbMcpError, Result},
	types::{JsonRpcRequest, JsonRpcResponse},
};
use rand::Rng;
use reqwest::Client;
use serde_json::Value;
use tracing::{debug, error};

/// Shared CKB RPC client for communicating with CKB nodes.
///
/// This client is used by the ckb-ai-mcp server to make
/// JSON-RPC calls to CKB nodes. It handles request ID generation, timeout
/// configuration, and error handling consistently across all servers.
#[derive(Clone)]
pub struct CkbRpcClient {
	client: Client,
	url: String,
}

impl CkbRpcClient {
	/// Create a new CKB RPC client with the given URL.
	///
	/// Configures HTTP client with reasonable timeouts:
	/// - Request timeout: 30 seconds
	/// - Connection timeout: 5 seconds
	pub fn new(url: impl Into<String>) -> Result<Self> {
		let client = Client::builder()
			.timeout(std::time::Duration::from_secs(30))
			.connect_timeout(std::time::Duration::from_secs(5))
			.build()
			.map_err(|e| CkbMcpError::Internal(format!("Failed to build HTTP client: {}", e)))?;

		Ok(Self {
			client,
			url: url.into(),
		})
	}

	/// Make a JSON-RPC call to the CKB node.
	///
	/// This is the core method that handles:
	/// - Request ID generation
	/// - HTTP POST to the CKB node
	/// - Error handling for both HTTP and RPC errors
	/// - Response parsing
	pub async fn call(&self, method: &str, params: Value) -> Result<Value> {
		// Use random u32 for request IDs to avoid collisions when cloning clients
		let id = rand::thread_rng().gen::<u32>() as u64;

		let request = JsonRpcRequest {
			jsonrpc: "2.0".to_string(),
			method: method.to_string(),
			params,
			id,
		};

		debug!("CKB RPC request: {} - {:?}", method, request.params);

		let response = self
			.client
			.post(&self.url)
			.header("Content-Type", "application/json")
			.json(&request)
			.send()
			.await
			.map_err(|e| CkbMcpError::Http(e.to_string()))?;

		if !response.status().is_success() {
			return Err(CkbMcpError::Http(format!(
				"HTTP error: {}",
				response.status()
			)));
		}

		let rpc_response: JsonRpcResponse = response
			.json()
			.await
			.map_err(|e| CkbMcpError::Http(e.to_string()))?;

		if let Some(error) = rpc_response.error {
			error!("CKB RPC error: {} - {}", error.code, error.message);
			return Err(CkbMcpError::CkbRpc(format!(
				"[{}] {}",
				error.code, error.message
			)));
		}

		Ok(rpc_response.result.unwrap_or(Value::Null))
	}

	// Common RPC methods used by multiple servers

	/// Get transaction by hash.
	pub async fn get_transaction(&self, tx_hash: &str) -> Result<Value> {
		let params = serde_json::json!([tx_hash]);
		self.call("get_transaction", params).await
	}

	/// Get block by number.
	pub async fn get_block_by_number(&self, block_number: u64) -> Result<Value> {
		let params = serde_json::json!([format!("{:#x}", block_number)]);
		self.call("get_block_by_number", params).await
	}

	/// Get block by hash.
	pub async fn get_block(&self, block_hash: &str) -> Result<Value> {
		let params = serde_json::json!([block_hash]);
		self.call("get_block", params).await
	}

	/// Get tip block number.
	pub async fn get_tip_block_number(&self) -> Result<Value> {
		self.call("get_tip_block_number", serde_json::json!([]))
			.await
	}

	/// Get tip header.
	pub async fn get_tip_header(&self) -> Result<Value> {
		self.call("get_tip_header", serde_json::json!([])).await
	}
}
