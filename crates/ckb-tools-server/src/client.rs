use reqwest::Client;
use shared::error::{CkbMcpError, Result};
use serde_json::Value;
use tracing::{debug, error};

pub struct CkbClient {
	client: Client,
	url: String,
	next_id: std::sync::atomic::AtomicU64,
}

impl Clone for CkbClient {
	fn clone(&self) -> Self {
		Self {
			client: self.client.clone(),
			url: self.url.clone(),
			next_id: std::sync::atomic::AtomicU64::new(1),
		}
	}
}

impl CkbClient {
	pub fn new(url: String) -> Result<Self> {
		Ok(Self {
			client: Client::new(),
			url,
			next_id: std::sync::atomic::AtomicU64::new(1),
		})
	}

	pub async fn call_rpc(&self, method: &str, params: Value) -> Result<Value> {
		let id = self
			.next_id
			.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

		let request = serde_json::json!({
			"jsonrpc": "2.0",
			"method": method,
			"params": params,
			"id": id
		});

		debug!("CKB RPC request: {} - {}", method, serde_json::to_string(&params)?);

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

		let rpc_response: Value = response
			.json()
			.await
			.map_err(|e| CkbMcpError::Http(e.to_string()))?;

		if let Some(error) = rpc_response.get("error") {
			error!("CKB RPC error: {}", error);
			return Err(CkbMcpError::CkbRpc(format!(
				"RPC error: {}",
				error
			)));
		}

		Ok(rpc_response
			.get("result")
			.cloned()
			.unwrap_or(Value::Null))
	}

	pub async fn get_block_by_number(&self, block_number: u64) -> Result<Value> {
		self.call_rpc("get_block_by_number", serde_json::json!([format!("0x{:x}", block_number)])).await
	}
}
