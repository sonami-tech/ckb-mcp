use reqwest::Client;
use shared::{
	error::{CkbMcpError, Result},
	types::{JsonRpcRequest, JsonRpcResponse},
};
use serde_json::Value;
use tracing::{debug, error};

pub struct CkbRpcClient {
	client: Client,
	url: String,
	next_id: std::sync::atomic::AtomicU64,
}

impl Clone for CkbRpcClient {
	fn clone(&self) -> Self {
		Self {
			client: self.client.clone(),
			url: self.url.clone(),
			next_id: std::sync::atomic::AtomicU64::new(1),
		}
	}
}

impl CkbRpcClient {
	pub fn new(url: &str) -> Result<Self> {
		Ok(Self {
			client: Client::new(),
			url: url.to_string(),
			next_id: std::sync::atomic::AtomicU64::new(1),
		})
	}

	pub async fn call(&self, method: &str, params: Value) -> Result<Value> {
		let id = self
			.next_id
			.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

		let request = JsonRpcRequest {
			jsonrpc: "2.0".to_string(),
			method: method.to_string(),
			params,
			id,
		};

		debug!("CKB RPC request: {} - {}", method, serde_json::to_string(&request.params)?);

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

	// Chain Methods
	/// Get block by hash
	pub async fn get_block(&self, block_hash: &str) -> Result<Value> {
		let params = serde_json::json!([block_hash]);
		self.call("get_block", params).await
	}

	/// Get block by number
	pub async fn get_block_by_number(&self, block_number: u64) -> Result<Value> {
		let params = serde_json::json!([format!("0x{:x}", block_number)]);
		self.call("get_block_by_number", params).await
	}

	/// Get header by hash
	pub async fn get_header(&self, block_hash: &str) -> Result<Value> {
		let params = serde_json::json!([block_hash]);
		self.call("get_header", params).await
	}

	/// Get header by number
	pub async fn get_header_by_number(&self, block_number: u64) -> Result<Value> {
		let params = serde_json::json!([format!("0x{:x}", block_number)]);
		self.call("get_header_by_number", params).await
	}

	/// Get transaction by hash
	pub async fn get_transaction(&self, tx_hash: &str) -> Result<Value> {
		let params = serde_json::json!([tx_hash]);
		self.call("get_transaction", params).await
	}

	/// Get block hash by number
	pub async fn get_block_hash(&self, block_number: u64) -> Result<Value> {
		let params = serde_json::json!([format!("0x{:x}", block_number)]);
		self.call("get_block_hash", params).await
	}

	/// Get tip header
	pub async fn get_tip_header(&self) -> Result<Value> {
		self.call("get_tip_header", serde_json::json!([])).await
	}

	/// Get live cell by out point
	pub async fn get_live_cell(&self, tx_hash: &str, index: u32, with_data: bool) -> Result<Value> {
		let out_point = serde_json::json!({
			"tx_hash": tx_hash,
			"index": format!("0x{:x}", index)
		});
		let params = serde_json::json!([out_point, with_data]);
		self.call("get_live_cell", params).await
	}

	/// Get tip block number
	pub async fn get_tip_block_number(&self) -> Result<Value> {
		self.call("get_tip_block_number", serde_json::json!([])).await
	}

	/// Get current epoch
	pub async fn get_current_epoch(&self) -> Result<Value> {
		self.call("get_current_epoch", serde_json::json!([])).await
	}

	/// Get epoch by number
	pub async fn get_epoch_by_number(&self, epoch_number: u64) -> Result<Value> {
		let params = serde_json::json!([format!("0x{:x}", epoch_number)]);
		self.call("get_epoch_by_number", params).await
	}

	// Indexer Methods
	/// Get indexer tip
	pub async fn get_indexer_tip(&self) -> Result<Value> {
		self.call("get_indexer_tip", serde_json::json!([])).await
	}

	/// Get cells by search criteria
	pub async fn get_cells(&self, search_key: Value, order: &str, limit: Option<u32>, after_cursor: Option<&str>) -> Result<Value> {
		let params = serde_json::json!([
			search_key,
			order,
			limit.map(|l| format!("0x{:x}", l)),
			after_cursor
		]);
		self.call("get_cells", params).await
	}

	/// Get transactions by search criteria
	pub async fn get_transactions(&self, search_key: Value, order: &str, limit: Option<u32>, after_cursor: Option<&str>) -> Result<Value> {
		let params = serde_json::json!([
			search_key,
			order,
			limit.map(|l| format!("0x{:x}", l)),
			after_cursor
		]);
		self.call("get_transactions", params).await
	}

	/// Get cells capacity by search criteria
	pub async fn get_cells_capacity(&self, search_key: Value) -> Result<Value> {
		let params = serde_json::json!([search_key]);
		self.call("get_cells_capacity", params).await
	}

	// Network Methods
	/// Get local node info
	pub async fn local_node_info(&self) -> Result<Value> {
		self.call("local_node_info", serde_json::json!([])).await
	}
}