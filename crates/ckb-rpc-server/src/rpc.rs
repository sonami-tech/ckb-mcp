// Re-export the shared CKB RPC client
pub use shared::ckb_client::CkbRpcClient;

use shared::error::Result;
use serde_json::Value;

/// Extension methods for CkbRpcClient specific to the RPC server.
///
/// These methods are only needed by ckb-rpc-server and not shared across servers.
pub trait CkbRpcClientExt {
	async fn get_header(&self, block_hash: &str) -> Result<Value>;
	async fn get_header_by_number(&self, block_number: u64) -> Result<Value>;
	async fn get_block_hash(&self, block_number: u64) -> Result<Value>;
	async fn get_live_cell(&self, tx_hash: &str, index: u32, with_data: bool) -> Result<Value>;
	async fn get_current_epoch(&self) -> Result<Value>;
	async fn get_epoch_by_number(&self, epoch_number: u64) -> Result<Value>;
	async fn get_indexer_tip(&self) -> Result<Value>;
	async fn get_cells(&self, search_key: Value, order: &str, limit: Option<u32>, after_cursor: Option<&str>) -> Result<Value>;
	async fn get_transactions(&self, search_key: Value, order: &str, limit: Option<u32>, after_cursor: Option<&str>) -> Result<Value>;
	async fn get_cells_capacity(&self, search_key: Value) -> Result<Value>;
	async fn local_node_info(&self) -> Result<Value>;
}

impl CkbRpcClientExt for CkbRpcClient {
	/// Get header by hash.
	async fn get_header(&self, block_hash: &str) -> Result<Value> {
		let params = serde_json::json!([block_hash]);
		self.call("get_header", params).await
	}

	/// Get header by number.
	async fn get_header_by_number(&self, block_number: u64) -> Result<Value> {
		let params = serde_json::json!([format!("{:#x}", block_number)]);
		self.call("get_header_by_number", params).await
	}

	/// Get block hash by number.
	async fn get_block_hash(&self, block_number: u64) -> Result<Value> {
		let params = serde_json::json!([format!("{:#x}", block_number)]);
		self.call("get_block_hash", params).await
	}

	/// Get live cell by out point.
	async fn get_live_cell(&self, tx_hash: &str, index: u32, with_data: bool) -> Result<Value> {
		let out_point = serde_json::json!({
			"tx_hash": tx_hash,
			"index": format!("{:#x}", index)
		});
		let params = serde_json::json!([out_point, with_data]);
		self.call("get_live_cell", params).await
	}

	/// Get current epoch.
	async fn get_current_epoch(&self) -> Result<Value> {
		self.call("get_current_epoch", serde_json::json!([])).await
	}

	/// Get epoch by number.
	async fn get_epoch_by_number(&self, epoch_number: u64) -> Result<Value> {
		let params = serde_json::json!([format!("{:#x}", epoch_number)]);
		self.call("get_epoch_by_number", params).await
	}

	/// Get indexer tip.
	async fn get_indexer_tip(&self) -> Result<Value> {
		self.call("get_indexer_tip", serde_json::json!([])).await
	}

	/// Get cells by search criteria.
	async fn get_cells(&self, search_key: Value, order: &str, limit: Option<u32>, after_cursor: Option<&str>) -> Result<Value> {
		let params = serde_json::json!([
			search_key,
			order,
			limit.map(|l| format!("{:#x}", l)),
			after_cursor
		]);
		self.call("get_cells", params).await
	}

	/// Get transactions by search criteria.
	async fn get_transactions(&self, search_key: Value, order: &str, limit: Option<u32>, after_cursor: Option<&str>) -> Result<Value> {
		let params = serde_json::json!([
			search_key,
			order,
			limit.map(|l| format!("{:#x}", l)),
			after_cursor
		]);
		self.call("get_transactions", params).await
	}

	/// Get cells capacity by search criteria.
	async fn get_cells_capacity(&self, search_key: Value) -> Result<Value> {
		let params = serde_json::json!([search_key]);
		self.call("get_cells_capacity", params).await
	}

	/// Get local node info.
	async fn local_node_info(&self) -> Result<Value> {
		self.call("local_node_info", serde_json::json!([])).await
	}
}
