//! RPC tool handlers that execute CKB RPC calls.

use rmcp::model::{CallToolResult, Content};
use serde_json::{Value, json};
use shared::ckb_client::CkbRpcClient;
use shared::error::{CkbMcpError, Result};
use shared::params::{
	extract_array, extract_bool, extract_object, extract_str, extract_str_opt, extract_u64,
	extract_u64_opt,
};
use tracing::{debug, error, info};

/// RPC handlers for executing CKB RPC calls.
pub struct RpcHandlers {
	client: CkbRpcClient,
}

impl RpcHandlers {
	/// Create a new RPC handlers instance.
	pub fn new(client: CkbRpcClient) -> Self {
		Self { client }
	}

	/// Handle a tool call by name.
	pub async fn handle(&self, name: &str, arguments: &Value) -> Result<CallToolResult> {
		// Log RPC calls with intelligent truncation for large payload tools.
		match name {
			"rpc_submit_transaction" | "rpc_test_transaction" | "rpc_estimate_cycles" => {
				info!("Calling tool: {} with transaction object", name);
			}
			_ => {
				debug!("Calling tool: {} with arguments: {}", name, arguments);
			}
		}

		let result = match name {
			// Category: query
			"rpc_get_block" => self.get_block(arguments).await,
			"rpc_get_block_by_number" => self.get_block_by_number(arguments).await,
			"rpc_get_header" => self.get_header(arguments).await,
			"rpc_get_header_by_number" => self.get_header_by_number(arguments).await,
			"rpc_get_transaction" => self.get_transaction(arguments).await,
			"rpc_get_block_hash" => self.get_block_hash(arguments).await,
			"rpc_get_tip_header" => self.get_tip_header().await,
			"rpc_get_tip_block_number" => self.get_tip_block_number().await,
			"rpc_get_current_epoch" => self.get_current_epoch().await,
			"rpc_get_epoch_by_number" => self.get_epoch_by_number(arguments).await,
			"rpc_get_live_cell" => self.get_live_cell(arguments).await,
			"rpc_get_fork_block" => self.get_fork_block(arguments).await,
			// Category: search
			"rpc_get_indexer_tip" => self.get_indexer_tip().await,
			"rpc_search_cells" => self.search_cells(arguments).await,
			"rpc_search_transactions" => self.search_transactions(arguments).await,
			"rpc_get_cells_capacity" => self.get_cells_capacity(arguments).await,
			// Category: submit
			"rpc_submit_transaction" => self.submit_transaction(arguments).await,
			"rpc_test_transaction" => self.test_transaction(arguments).await,
			// Category: status
			"rpc_get_node_info" => self.get_node_info().await,
			"rpc_get_sync_state" => self.get_sync_state().await,
			"rpc_get_peers" => self.get_peers().await,
			"rpc_get_pool_info" => self.get_pool_info().await,
			"rpc_get_pool_ready" => self.get_pool_ready().await,
			"rpc_get_pool_transactions" => self.get_pool_transactions(arguments).await,
			"rpc_get_pool_tx_detail" => self.get_pool_tx_detail(arguments).await,
			"rpc_get_blockchain_info" => self.get_blockchain_info().await,
			"rpc_get_consensus" => self.get_consensus().await,
			"rpc_get_deployments" => self.get_deployments().await,
			// Category: calculate
			"rpc_estimate_cycles" => self.estimate_cycles(arguments).await,
			"rpc_estimate_fee_rate" => self.estimate_fee_rate(arguments).await,
			"rpc_calculate_dao_withdraw" => self.calculate_dao_withdraw(arguments).await,
			"rpc_get_block_economics" => self.get_block_economics(arguments).await,
			"rpc_get_block_median_time" => self.get_block_median_time(arguments).await,
			"rpc_get_block_filter" => self.get_block_filter(arguments).await,
			// Category: verify
			"rpc_get_transaction_proof" => self.get_transaction_proof(arguments).await,
			"rpc_verify_transaction_proof" => self.verify_transaction_proof(arguments).await,
			_ => {
				return Err(CkbMcpError::InvalidParameter(format!(
					"Unknown RPC tool: {}",
					name
				)));
			}
		};

		match result {
			Ok(data) => Ok(CallToolResult::success(vec![Content::text(
				serde_json::to_string_pretty(&data)?,
			)])),
			Err(e) => {
				error!("RPC tool call failed: {}", e);
				Ok(CallToolResult::error(vec![Content::text(e.to_string())]))
			}
		}
	}

	/// Check if a tool name is an RPC tool.
	pub fn is_rpc_tool(name: &str) -> bool {
		name.starts_with("rpc_")
	}

	// Category: query handlers

	async fn get_block(&self, args: &Value) -> Result<Value> {
		let block_hash = extract_str(args, "block_hash")?;
		self.client.get_block(block_hash).await
	}

	async fn get_block_by_number(&self, args: &Value) -> Result<Value> {
		let block_number = extract_u64(args, "block_number")?;
		self.client.get_block_by_number(block_number).await
	}

	async fn get_header(&self, args: &Value) -> Result<Value> {
		let block_hash = extract_str(args, "block_hash")?;
		let params = json!([block_hash]);
		self.client.call("get_header", params).await
	}

	async fn get_header_by_number(&self, args: &Value) -> Result<Value> {
		let block_number = extract_u64(args, "block_number")?;
		let params = json!([format!("{:#x}", block_number)]);
		self.client.call("get_header_by_number", params).await
	}

	async fn get_transaction(&self, args: &Value) -> Result<Value> {
		let tx_hash = extract_str(args, "tx_hash")?;
		self.client.get_transaction(tx_hash).await
	}

	async fn get_block_hash(&self, args: &Value) -> Result<Value> {
		let block_number = extract_u64(args, "block_number")?;
		let params = json!([format!("{:#x}", block_number)]);
		self.client.call("get_block_hash", params).await
	}

	async fn get_tip_header(&self) -> Result<Value> {
		self.client.get_tip_header().await
	}

	async fn get_tip_block_number(&self) -> Result<Value> {
		self.client.get_tip_block_number().await
	}

	async fn get_current_epoch(&self) -> Result<Value> {
		self.client.call("get_current_epoch", json!([])).await
	}

	async fn get_epoch_by_number(&self, args: &Value) -> Result<Value> {
		let epoch_number = extract_u64(args, "epoch_number")?;
		let params = json!([format!("{:#x}", epoch_number)]);
		self.client.call("get_epoch_by_number", params).await
	}

	async fn get_live_cell(&self, args: &Value) -> Result<Value> {
		let tx_hash = extract_str(args, "tx_hash")?;
		let index = extract_u64(args, "index")? as u32;
		let with_data = extract_bool(args, "with_data", false);
		let out_point = json!({
			"tx_hash": tx_hash,
			"index": format!("{:#x}", index)
		});
		let params = json!([out_point, with_data]);
		self.client.call("get_live_cell", params).await
	}

	async fn get_fork_block(&self, args: &Value) -> Result<Value> {
		let block_hash = extract_str(args, "block_hash")?;
		let verbosity = extract_u64_opt(args, "verbosity").map(|v| format!("{:#x}", v));
		let params = json!([block_hash, verbosity]);
		self.client.call("get_fork_block", params).await
	}

	// Category: search handlers

	async fn get_indexer_tip(&self) -> Result<Value> {
		self.client.call("get_indexer_tip", json!([])).await
	}

	async fn search_cells(&self, args: &Value) -> Result<Value> {
		let search_key = extract_object(args, "search_key")?.clone();
		let order = extract_str_opt(args, "order").unwrap_or("asc");
		let limit = extract_u64_opt(args, "limit").map(|l| format!("{:#x}", l));
		let after_cursor = extract_str_opt(args, "after_cursor");
		let params = json!([search_key, order, limit, after_cursor]);
		self.client.call("get_cells", params).await
	}

	async fn search_transactions(&self, args: &Value) -> Result<Value> {
		let mut search_key = extract_object(args, "search_key")?.clone();
		let order = extract_str_opt(args, "order").unwrap_or("asc");
		let limit = extract_u64_opt(args, "limit").map(|l| format!("{:#x}", l));
		let after_cursor = extract_str_opt(args, "after_cursor");
		let group_by_transaction = extract_bool(args, "group_by_transaction", false);

		// Add group_by_transaction to search_key if specified.
		if group_by_transaction && let Some(obj) = search_key.as_object_mut() {
			obj.insert("group_by_transaction".to_string(), json!(true));
		}

		let params = json!([search_key, order, limit, after_cursor]);
		self.client.call("get_transactions", params).await
	}

	async fn get_cells_capacity(&self, args: &Value) -> Result<Value> {
		let search_key = extract_object(args, "search_key")?.clone();
		let params = json!([search_key]);
		self.client.call("get_cells_capacity", params).await
	}

	// Category: submit handlers

	async fn submit_transaction(&self, args: &Value) -> Result<Value> {
		let tx = extract_object(args, "tx")?.clone();
		let outputs_validator = extract_str_opt(args, "outputs_validator");
		let params = json!([tx, outputs_validator]);
		let result = self.client.call("send_transaction", params).await?;

		if let Some(tx_hash) = result.as_str() {
			info!("Transaction sent: {}", tx_hash);
		} else {
			info!("Transaction sent successfully");
		}

		Ok(result)
	}

	async fn test_transaction(&self, args: &Value) -> Result<Value> {
		let tx = extract_object(args, "tx")?.clone();
		let outputs_validator = extract_str_opt(args, "outputs_validator");
		let params = json!([tx, outputs_validator]);
		self.client.call("test_tx_pool_accept", params).await
	}

	// Category: status handlers

	async fn get_node_info(&self) -> Result<Value> {
		self.client.call("local_node_info", json!([])).await
	}

	async fn get_sync_state(&self) -> Result<Value> {
		self.client.call("sync_state", json!([])).await
	}

	async fn get_peers(&self) -> Result<Value> {
		self.client.call("get_peers", json!([])).await
	}

	async fn get_pool_info(&self) -> Result<Value> {
		self.client.call("tx_pool_info", json!([])).await
	}

	async fn get_pool_ready(&self) -> Result<Value> {
		self.client.call("tx_pool_ready", json!([])).await
	}

	async fn get_pool_transactions(&self, args: &Value) -> Result<Value> {
		let verbose = args.get("verbose").and_then(|v| v.as_bool());
		let params = json!([verbose]);
		self.client.call("get_raw_tx_pool", params).await
	}

	async fn get_pool_tx_detail(&self, args: &Value) -> Result<Value> {
		let tx_hash = extract_str(args, "tx_hash")?;
		let params = json!([tx_hash]);
		self.client.call("get_pool_tx_detail_info", params).await
	}

	async fn get_blockchain_info(&self) -> Result<Value> {
		self.client.call("get_blockchain_info", json!([])).await
	}

	async fn get_consensus(&self) -> Result<Value> {
		self.client.call("get_consensus", json!([])).await
	}

	async fn get_deployments(&self) -> Result<Value> {
		self.client.call("get_deployments_info", json!([])).await
	}

	// Category: calculate handlers

	async fn estimate_cycles(&self, args: &Value) -> Result<Value> {
		let tx = extract_object(args, "tx")?.clone();
		let params = json!([tx]);
		self.client.call("estimate_cycles", params).await
	}

	async fn estimate_fee_rate(&self, args: &Value) -> Result<Value> {
		let estimate_mode = extract_str_opt(args, "estimate_mode");
		let enable_fallback = args.get("enable_fallback").and_then(|v| v.as_bool());
		let params = json!([estimate_mode, enable_fallback]);
		self.client.call("estimate_fee_rate", params).await
	}

	async fn calculate_dao_withdraw(&self, args: &Value) -> Result<Value> {
		let out_point = extract_object(args, "out_point")?.clone();
		let kind = args
			.get("kind")
			.ok_or_else(|| {
				CkbMcpError::InvalidParameter("Missing required field: kind".to_string())
			})?
			.clone();
		let params = json!([out_point, kind]);
		self.client
			.call("calculate_dao_maximum_withdraw", params)
			.await
	}

	async fn get_block_economics(&self, args: &Value) -> Result<Value> {
		let block_hash = extract_str(args, "block_hash")?;
		let params = json!([block_hash]);
		self.client.call("get_block_economic_state", params).await
	}

	async fn get_block_median_time(&self, args: &Value) -> Result<Value> {
		let block_hash = extract_str(args, "block_hash")?;
		let params = json!([block_hash]);
		self.client.call("get_block_median_time", params).await
	}

	async fn get_block_filter(&self, args: &Value) -> Result<Value> {
		let block_hash = extract_str(args, "block_hash")?;
		let params = json!([block_hash]);
		self.client.call("get_block_filter", params).await
	}

	// Category: verify handlers

	async fn get_transaction_proof(&self, args: &Value) -> Result<Value> {
		let tx_hashes = extract_array(args, "tx_hashes")?
			.iter()
			.filter_map(|v| v.as_str().map(|s| s.to_string()))
			.collect::<Vec<String>>();

		if tx_hashes.is_empty() {
			return Err(CkbMcpError::InvalidParameter(
				"tx_hashes array cannot be empty".to_string(),
			));
		}

		let block_hash = extract_str_opt(args, "block_hash");
		let params = json!([tx_hashes, block_hash]);
		self.client.call("get_transaction_proof", params).await
	}

	async fn verify_transaction_proof(&self, args: &Value) -> Result<Value> {
		let tx_proof = extract_object(args, "tx_proof")?.clone();
		let params = json!([tx_proof]);
		self.client.call("verify_transaction_proof", params).await
	}
}
