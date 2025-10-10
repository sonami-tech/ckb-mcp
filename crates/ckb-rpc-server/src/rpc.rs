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
	async fn estimate_cycles(&self, tx: Value) -> Result<Value>;
	async fn send_transaction(&self, tx: Value, outputs_validator: Option<&str>) -> Result<Value>;
	async fn test_tx_pool_accept(&self, tx: Value, outputs_validator: Option<&str>) -> Result<Value>;
	async fn get_blockchain_info(&self) -> Result<Value>;
	async fn get_consensus(&self) -> Result<Value>;
	async fn tx_pool_info(&self) -> Result<Value>;
	async fn get_raw_tx_pool(&self, verbose: Option<bool>) -> Result<Value>;
	async fn get_pool_tx_detail_info(&self, tx_hash: &str) -> Result<Value>;
	async fn tx_pool_ready(&self) -> Result<Value>;
	async fn sync_state(&self) -> Result<Value>;
	async fn get_peers(&self) -> Result<Value>;
	async fn get_deployments_info(&self) -> Result<Value>;
	async fn calculate_dao_maximum_withdraw(&self, out_point: Value, kind: Value) -> Result<Value>;
	async fn estimate_fee_rate(&self, estimate_mode: Option<&str>, enable_fallback: Option<bool>) -> Result<Value>;
	async fn get_transaction_proof(&self, tx_hashes: Vec<String>, block_hash: Option<&str>) -> Result<Value>;
	async fn verify_transaction_proof(&self, tx_proof: Value) -> Result<Value>;
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

	/// Estimate transaction execution cycles.
	async fn estimate_cycles(&self, tx: Value) -> Result<Value> {
		let params = serde_json::json!([tx]);
		self.call("estimate_cycles", params).await
	}

	/// Send transaction to the network.
	async fn send_transaction(&self, tx: Value, outputs_validator: Option<&str>) -> Result<Value> {
		let params = serde_json::json!([tx, outputs_validator]);
		self.call("send_transaction", params).await
	}

	/// Test if transaction would be accepted by pool without broadcasting.
	async fn test_tx_pool_accept(&self, tx: Value, outputs_validator: Option<&str>) -> Result<Value> {
		let params = serde_json::json!([tx, outputs_validator]);
		self.call("test_tx_pool_accept", params).await
	}

	/// Get blockchain information.
	async fn get_blockchain_info(&self) -> Result<Value> {
		self.call("get_blockchain_info", serde_json::json!([])).await
	}

	/// Get consensus parameters.
	async fn get_consensus(&self) -> Result<Value> {
		self.call("get_consensus", serde_json::json!([])).await
	}

	/// Get transaction pool information.
	async fn tx_pool_info(&self) -> Result<Value> {
		self.call("tx_pool_info", serde_json::json!([])).await
	}

	/// Get all transaction ids in tx pool.
	async fn get_raw_tx_pool(&self, verbose: Option<bool>) -> Result<Value> {
		let params = serde_json::json!([verbose]);
		self.call("get_raw_tx_pool", params).await
	}

	/// Get details of a transaction in the pool for troubleshooting.
	async fn get_pool_tx_detail_info(&self, tx_hash: &str) -> Result<Value> {
		let params = serde_json::json!([tx_hash]);
		self.call("get_pool_tx_detail_info", params).await
	}

	/// Check if tx-pool service is started and ready for requests.
	async fn tx_pool_ready(&self) -> Result<Value> {
		self.call("tx_pool_ready", serde_json::json!([])).await
	}

	/// Get chain synchronization state.
	async fn sync_state(&self) -> Result<Value> {
		self.call("sync_state", serde_json::json!([])).await
	}

	/// Get connected peers information.
	async fn get_peers(&self) -> Result<Value> {
		self.call("get_peers", serde_json::json!([])).await
	}

	/// Get soft fork deployments information.
	async fn get_deployments_info(&self) -> Result<Value> {
		self.call("get_deployments_info", serde_json::json!([])).await
	}

	/// Calculate maximum DAO withdrawal amount.
	async fn calculate_dao_maximum_withdraw(&self, out_point: Value, kind: Value) -> Result<Value> {
		let params = serde_json::json!([out_point, kind]);
		self.call("calculate_dao_maximum_withdraw", params).await
	}

	/// Estimate transaction fee rate.
	async fn estimate_fee_rate(&self, estimate_mode: Option<&str>, enable_fallback: Option<bool>) -> Result<Value> {
		let params = serde_json::json!([estimate_mode, enable_fallback]);
		self.call("estimate_fee_rate", params).await
	}

	/// Generate Merkle proof that transactions are included in a block.
	async fn get_transaction_proof(&self, tx_hashes: Vec<String>, block_hash: Option<&str>) -> Result<Value> {
		let params = serde_json::json!([tx_hashes, block_hash]);
		self.call("get_transaction_proof", params).await
	}

	/// Verify Merkle proof that transactions are included in a block.
	async fn verify_transaction_proof(&self, tx_proof: Value) -> Result<Value> {
		let params = serde_json::json!([tx_proof]);
		self.call("verify_transaction_proof", params).await
	}
}
