//! CKB composite tool handlers.
//!
//! These handlers combine multiple RPC calls to provide high-level operations.

use rmcp::model::{CallToolResult, Content};
use serde_json::{Value, json};
use shared::ckb_client::CkbRpcClient;
use shared::error::{CkbMcpError, Result};
use shared::params::{extract_bool, extract_object, extract_str, extract_str_opt, extract_u64_opt};
use tracing::{debug, info};

/// CKB composite tool handlers.
pub struct CkbHandlers {
	client: CkbRpcClient,
}

impl CkbHandlers {
	/// Create a new CKB handlers instance.
	pub fn new(client: CkbRpcClient) -> Self {
		Self { client }
	}

	/// Check if a tool name is a CKB composite tool.
	pub fn is_ckb_tool(name: &str) -> bool {
		name.starts_with("ckb_")
	}

	/// Handle a CKB composite tool call.
	pub async fn handle(&self, name: &str, arguments: &Value) -> Result<CallToolResult> {
		debug!("CKB tool call: {} with arguments: {}", name, arguments);

		let result = match name {
			"ckb_query_address" => self.handle_query_address(arguments).await,
			"ckb_query_chain_status" => self.handle_query_chain_status().await,
			"ckb_query_transaction" => self.handle_query_transaction(arguments).await,
			"ckb_validate_transaction" => self.handle_validate_transaction(arguments).await,
			"ckb_query_script_cells" => self.handle_query_script_cells(arguments).await,
			_ => {
				return Err(CkbMcpError::InvalidParameter(format!(
					"Unknown CKB tool: {}",
					name
				)));
			}
		};

		match result {
			Ok(data) => Ok(CallToolResult::success(vec![Content::text(
				serde_json::to_string_pretty(&data)?,
			)])),
			Err(e) => {
				tracing::error!("CKB tool call failed: {}", e);
				Ok(CallToolResult::error(vec![Content::text(e.to_string())]))
			}
		}
	}

	/// Query complete address state: balance breakdown, recent cells, lock info.
	async fn handle_query_address(&self, args: &Value) -> Result<Value> {
		let address = extract_str_opt(args, "address");
		let include_cells = extract_bool(args, "include_cells", true);
		let cell_limit = extract_u64_opt(args, "cell_limit").unwrap_or(10);

		debug!(
			"Querying address: {:?}, include_cells: {}, limit: {}",
			address, include_cells, cell_limit
		);

		// If no address provided, we need to derive from default account.
		// For now, require address parameter.
		let address = address.ok_or_else(|| {
			CkbMcpError::InvalidParameter(
				"address parameter is required. Use dev_get_default_account_info for default account.".to_string()
			)
		})?;

		// Parse address to get lock script.
		let lock_script = self.address_to_lock_script(address)?;

		// Build search key for cells capacity.
		let search_key = json!({
			"script": lock_script,
			"script_type": "lock"
		});

		// Query total capacity.
		let capacity_result = self
			.client
			.call("get_cells_capacity", json!([search_key]))
			.await?;

		let mut result = json!({
			"address": address,
			"lock_script": lock_script,
			"capacity": capacity_result
		});

		// Optionally include recent cells.
		if include_cells {
			let cells_params = json!([search_key, "desc", format!("{:#x}", cell_limit)]);
			let cells_result = self.client.call("get_cells", cells_params).await?;
			result["cells"] = cells_result;
		}

		info!("Address query completed for: {}", address);
		Ok(result)
	}

	/// Query chain health snapshot: tip block, sync state, indexer status, mempool info.
	async fn handle_query_chain_status(&self) -> Result<Value> {
		debug!("Querying chain status");

		// Execute multiple RPC calls.
		let tip_header = self.client.get_tip_header().await?;
		let sync_state = self.client.call("sync_state", json!([])).await?;
		let indexer_tip = self.client.call("get_indexer_tip", json!([])).await?;
		let pool_info = self.client.call("tx_pool_info", json!([])).await?;

		let result = json!({
			"tip": tip_header,
			"sync": sync_state,
			"indexer": indexer_tip,
			"mempool": pool_info
		});

		info!("Chain status query completed");
		Ok(result)
	}

	/// Query transaction with resolved input cells.
	async fn handle_query_transaction(&self, args: &Value) -> Result<Value> {
		let tx_hash = extract_str(args, "tx_hash")?;

		debug!("Querying transaction: {}", tx_hash);

		// Get the transaction.
		let tx_result = self.client.get_transaction(tx_hash).await?;

		// Extract inputs and resolve them.
		let mut resolved_inputs = Vec::new();

		if let Some(tx) = tx_result.get("transaction")
			&& let Some(inputs) = tx.get("inputs").and_then(|i| i.as_array())
		{
			for input in inputs {
				if let Some(previous_output) = input.get("previous_output") {
					let prev_tx_hash = previous_output.get("tx_hash").and_then(|h| h.as_str());
					let prev_index = previous_output.get("index").and_then(|i| i.as_str());

					if let (Some(prev_tx_hash), Some(prev_index)) = (prev_tx_hash, prev_index) {
						// Try to get the live cell (may fail if already consumed).
						let out_point = json!({
							"tx_hash": prev_tx_hash,
							"index": prev_index
						});
						let cell_params = json!([out_point, true]);

						// Attempt to resolve the cell.
						match self.client.call("get_live_cell", cell_params).await {
							Ok(cell_result) => {
								resolved_inputs.push(json!({
									"previous_output": previous_output,
									"cell": cell_result
								}));
							}
							Err(_) => {
								// Cell is consumed or unavailable.
								resolved_inputs.push(json!({
									"previous_output": previous_output,
									"cell": null,
									"note": "Cell consumed or unavailable"
								}));
							}
						}
					}
				}
			}
		}

		let result = json!({
			"transaction": tx_result,
			"resolved_inputs": resolved_inputs
		});

		info!("Transaction query completed for: {}", tx_hash);
		Ok(result)
	}

	/// Validate transaction before submission.
	async fn handle_validate_transaction(&self, args: &Value) -> Result<Value> {
		let tx = extract_object(args, "tx")?.clone();

		debug!("Validating transaction");

		// Test transaction acceptance (dry run).
		let test_result = self
			.client
			.call("test_tx_pool_accept", json!([tx, null]))
			.await;

		// Estimate cycles.
		let cycles_result = self.client.call("estimate_cycles", json!([tx])).await;

		// Get current fee rate.
		let fee_rate_result = self
			.client
			.call("estimate_fee_rate", json!([null, true]))
			.await;

		let result = json!({
			"dry_run": match test_result {
				Ok(v) => json!({"success": true, "result": v}),
				Err(e) => json!({"success": false, "error": e.to_string()})
			},
			"cycles": match cycles_result {
				Ok(v) => json!({"success": true, "result": v}),
				Err(e) => json!({"success": false, "error": e.to_string()})
			},
			"fee_rate": match fee_rate_result {
				Ok(v) => json!({"success": true, "result": v}),
				Err(e) => json!({"success": false, "error": e.to_string()})
			}
		});

		info!("Transaction validation completed");
		Ok(result)
	}

	/// Query cells by lock or type script.
	async fn handle_query_script_cells(&self, args: &Value) -> Result<Value> {
		let script_type = extract_str(args, "script_type")?;
		let code_hash = extract_str(args, "code_hash")?;
		let hash_type = extract_str(args, "hash_type")?;
		let args_value = extract_str_opt(args, "args");
		let limit = extract_u64_opt(args, "limit").unwrap_or(20);
		let order = extract_str_opt(args, "order").unwrap_or("desc");

		debug!(
			"Querying cells by {} script: code_hash={}, hash_type={}, args={:?}",
			script_type, code_hash, hash_type, args_value
		);

		// Build script object.
		let script = json!({
			"code_hash": code_hash,
			"hash_type": hash_type,
			"args": args_value.unwrap_or("0x")
		});

		// Build search key.
		let search_key = json!({
			"script": script,
			"script_type": script_type,
			"script_search_mode": if args_value.is_some() { "prefix" } else { "exact" }
		});

		// Query cells.
		let cells_params = json!([search_key, order, format!("{:#x}", limit)]);
		let cells_result = self.client.call("get_cells", cells_params).await?;

		// Also get capacity summary.
		let capacity_result = self
			.client
			.call("get_cells_capacity", json!([search_key]))
			.await?;

		let result = json!({
			"search_key": search_key,
			"cells": cells_result,
			"total_capacity": capacity_result
		});

		info!(
			"Script cells query completed: {} cells",
			cells_result
				.get("objects")
				.and_then(|o| o.as_array())
				.map(|a| a.len())
				.unwrap_or(0)
		);
		Ok(result)
	}

	/// Convert CKB address to lock script JSON.
	fn address_to_lock_script(&self, address: &str) -> Result<Value> {
		use ckb_sdk::Address;
		use ckb_types::packed::Script;
		use std::str::FromStr;

		let addr = Address::from_str(address)
			.map_err(|e| CkbMcpError::InvalidParameter(format!("Invalid address: {}", e)))?;

		let lock_script: Script = Script::from(&addr);

		Ok(json!({
			"code_hash": format!("{:#x}", lock_script.code_hash()),
			"hash_type": crate::util::hash_type_to_string(lock_script.hash_type()),
			"args": format!("0x{}", hex::encode(lock_script.args().raw_data()))
		}))
	}
}
