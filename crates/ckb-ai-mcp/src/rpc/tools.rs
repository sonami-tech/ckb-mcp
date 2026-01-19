//! RPC tool definitions with new naming convention.
//!
//! Tools are renamed from the original CKB RPC names to follow the pattern:
//! `rpc_{action}_{target}`

use rmcp::model::Tool;
use serde_json::json;
use std::sync::LazyLock;

/// Static list of all RPC tools.
pub static RPC_TOOLS: LazyLock<Vec<Tool>> = LazyLock::new(RpcToolDefinitions::all);

/// Helper to create a tool with all required fields.
fn make_tool(name: &'static str, description: &'static str, input_schema: serde_json::Value) -> Tool {
	Tool {
		name: name.into(),
		description: Some(description.into()),
		input_schema: input_schema.as_object().unwrap().clone().into(),
		annotations: None,
		output_schema: None,
		title: None,
		icons: None,
		meta: None,
	}
}

/// RPC tool definitions.
pub struct RpcToolDefinitions;

impl RpcToolDefinitions {
	/// Get all RPC tool definitions.
	pub fn all() -> Vec<Tool> {
		vec![
			// Category: query (Chain query methods)
			Self::rpc_get_block(),
			Self::rpc_get_block_by_number(),
			Self::rpc_get_header(),
			Self::rpc_get_header_by_number(),
			Self::rpc_get_transaction(),
			Self::rpc_get_block_hash(),
			Self::rpc_get_tip_header(),
			Self::rpc_get_tip_block_number(),
			Self::rpc_get_current_epoch(),
			Self::rpc_get_epoch_by_number(),
			Self::rpc_get_live_cell(),
			Self::rpc_get_fork_block(),
			// Category: search (Indexer methods)
			Self::rpc_get_indexer_tip(),
			Self::rpc_search_cells(),
			Self::rpc_search_transactions(),
			Self::rpc_get_cells_capacity(),
			// Category: submit (Transaction submission)
			Self::rpc_submit_transaction(),
			Self::rpc_test_transaction(),
			// Category: status (Node/network status)
			Self::rpc_get_node_info(),
			Self::rpc_get_sync_state(),
			Self::rpc_get_peers(),
			Self::rpc_get_pool_info(),
			Self::rpc_get_pool_ready(),
			Self::rpc_get_pool_transactions(),
			Self::rpc_get_pool_tx_detail(),
			Self::rpc_get_blockchain_info(),
			Self::rpc_get_consensus(),
			Self::rpc_get_deployments(),
			// Category: calculate (Estimation and calculation)
			Self::rpc_estimate_cycles(),
			Self::rpc_estimate_fee_rate(),
			Self::rpc_calculate_dao_withdraw(),
			Self::rpc_get_block_economics(),
			Self::rpc_get_block_median_time(),
			Self::rpc_get_block_filter(),
			// Category: verify (Proof verification)
			Self::rpc_get_transaction_proof(),
			Self::rpc_verify_transaction_proof(),
		]
	}

	// Category: query

	fn rpc_get_block() -> Tool {
		make_tool(
			"rpc_get_block",
			"Get CKB block by hash. Returns header, transactions, proposals, and uncles.",
			json!({
				"type": "object",
				"properties": {
					"block_hash": {
						"type": "string",
						"description": "Block hash (0x-prefixed, 64 hex characters)"
					}
				},
				"required": ["block_hash"]
			}),
		)
	}

	fn rpc_get_block_by_number() -> Tool {
		make_tool(
			"rpc_get_block_by_number",
			"Get CKB block by number.",
			json!({
				"type": "object",
				"properties": {
					"block_number": {
						"type": "integer",
						"description": "Block number"
					}
				},
				"required": ["block_number"]
			}),
		)
	}

	fn rpc_get_header() -> Tool {
		make_tool(
			"rpc_get_header",
			"Get CKB block header by hash.",
			json!({
				"type": "object",
				"properties": {
					"block_hash": {
						"type": "string",
						"description": "Block hash"
					}
				},
				"required": ["block_hash"]
			}),
		)
	}

	fn rpc_get_header_by_number() -> Tool {
		make_tool(
			"rpc_get_header_by_number",
			"Get CKB block header by number.",
			json!({
				"type": "object",
				"properties": {
					"block_number": {
						"type": "integer",
						"description": "Block number"
					}
				},
				"required": ["block_number"]
			}),
		)
	}

	fn rpc_get_transaction() -> Tool {
		make_tool(
			"rpc_get_transaction",
			"Get CKB transaction by hash.",
			json!({
				"type": "object",
				"properties": {
					"tx_hash": {
						"type": "string",
						"description": "Transaction hash"
					}
				},
				"required": ["tx_hash"]
			}),
		)
	}

	fn rpc_get_block_hash() -> Tool {
		make_tool(
			"rpc_get_block_hash",
			"Get block hash by number.",
			json!({
				"type": "object",
				"properties": {
					"block_number": {
						"type": "integer",
						"description": "Block number"
					}
				},
				"required": ["block_number"]
			}),
		)
	}

	fn rpc_get_tip_header() -> Tool {
		make_tool(
			"rpc_get_tip_header",
			"Get tip block header.",
			json!({
				"type": "object",
				"properties": {}
			}),
		)
	}

	fn rpc_get_tip_block_number() -> Tool {
		make_tool(
			"rpc_get_tip_block_number",
			"Get tip block number.",
			json!({
				"type": "object",
				"properties": {}
			}),
		)
	}

	fn rpc_get_current_epoch() -> Tool {
		make_tool(
			"rpc_get_current_epoch",
			"Get current epoch information.",
			json!({
				"type": "object",
				"properties": {}
			}),
		)
	}

	fn rpc_get_epoch_by_number() -> Tool {
		make_tool(
			"rpc_get_epoch_by_number",
			"Get epoch by number.",
			json!({
				"type": "object",
				"properties": {
					"epoch_number": {
						"type": "integer",
						"description": "Epoch number"
					}
				},
				"required": ["epoch_number"]
			}),
		)
	}

	fn rpc_get_live_cell() -> Tool {
		make_tool(
			"rpc_get_live_cell",
			"Get live cell by outpoint.",
			json!({
				"type": "object",
				"properties": {
					"tx_hash": {
						"type": "string",
						"description": "Transaction hash"
					},
					"index": {
						"type": "integer",
						"description": "Output index"
					},
					"with_data": {
						"type": "boolean",
						"description": "Include cell data",
						"default": false
					}
				},
				"required": ["tx_hash", "index"]
			}),
		)
	}

	fn rpc_get_fork_block() -> Tool {
		make_tool(
			"rpc_get_fork_block",
			"Get fork block information by hash.",
			json!({
				"type": "object",
				"properties": {
					"block_hash": {
						"type": "string",
						"description": "Fork block hash"
					},
					"verbosity": {
						"type": "integer",
						"description": "Result format: 0 (hex string) or 2 (JSON object, default)"
					}
				},
				"required": ["block_hash"]
			}),
		)
	}

	// Category: search (Indexer)

	fn rpc_get_indexer_tip() -> Tool {
		make_tool(
			"rpc_get_indexer_tip",
			"Get indexer sync tip.",
			json!({
				"type": "object",
				"properties": {}
			}),
		)
	}

	fn rpc_search_cells() -> Tool {
		make_tool(
			"rpc_search_cells",
			"Search for cells by criteria. Returns matching cells with pagination.",
			json!({
				"type": "object",
				"properties": {
					"search_key": {
						"type": "object",
						"description": "Search criteria (script, script_type, filter, etc.)"
					},
					"order": {
						"type": "string",
						"enum": ["asc", "desc"],
						"default": "asc",
						"description": "Sort order"
					},
					"limit": {
						"type": "integer",
						"description": "Maximum number of results",
						"default": 100
					},
					"after_cursor": {
						"type": "string",
						"description": "Pagination cursor (optional)"
					}
				},
				"required": ["search_key"]
			}),
		)
	}

	fn rpc_search_transactions() -> Tool {
		make_tool(
			"rpc_search_transactions",
			"Search for transactions by criteria.",
			json!({
				"type": "object",
				"properties": {
					"search_key": {
						"type": "object",
						"description": "Search criteria"
					},
					"order": {
						"type": "string",
						"enum": ["asc", "desc"],
						"default": "asc",
						"description": "Sort order"
					},
					"limit": {
						"type": "integer",
						"description": "Maximum number of results",
						"default": 100
					},
					"after_cursor": {
						"type": "string",
						"description": "Pagination cursor (optional)"
					},
					"group_by_transaction": {
						"type": "boolean",
						"description": "Group results by transaction hash",
						"default": false
					}
				},
				"required": ["search_key"]
			}),
		)
	}

	fn rpc_get_cells_capacity() -> Tool {
		make_tool(
			"rpc_get_cells_capacity",
			"Get total capacity of cells matching search criteria.",
			json!({
				"type": "object",
				"properties": {
					"search_key": {
						"type": "object",
						"description": "Search criteria"
					}
				},
				"required": ["search_key"]
			}),
		)
	}

	// Category: submit

	fn rpc_submit_transaction() -> Tool {
		make_tool(
			"rpc_submit_transaction",
			"Submit transaction to the network.",
			json!({
				"type": "object",
				"properties": {
					"tx": {
						"type": "object",
						"description": "Transaction object to send"
					},
					"outputs_validator": {
						"type": "string",
						"description": "Outputs validator mode",
						"enum": ["passthrough", "well_known_scripts_only"],
						"default": "passthrough"
					}
				},
				"required": ["tx"]
			}),
		)
	}

	fn rpc_test_transaction() -> Tool {
		make_tool(
			"rpc_test_transaction",
			"Test if transaction would be accepted without broadcasting.",
			json!({
				"type": "object",
				"properties": {
					"tx": {
						"type": "object",
						"description": "Transaction object to test"
					},
					"outputs_validator": {
						"type": "string",
						"description": "Outputs validator mode",
						"enum": ["passthrough", "well_known_scripts_only"],
						"default": "passthrough"
					}
				},
				"required": ["tx"]
			}),
		)
	}

	// Category: status

	fn rpc_get_node_info() -> Tool {
		make_tool(
			"rpc_get_node_info",
			"Get local node information.",
			json!({
				"type": "object",
				"properties": {}
			}),
		)
	}

	fn rpc_get_sync_state() -> Tool {
		make_tool(
			"rpc_get_sync_state",
			"Get chain synchronization state.",
			json!({
				"type": "object",
				"properties": {}
			}),
		)
	}

	fn rpc_get_peers() -> Tool {
		make_tool(
			"rpc_get_peers",
			"Get connected peers information.",
			json!({
				"type": "object",
				"properties": {}
			}),
		)
	}

	fn rpc_get_pool_info() -> Tool {
		make_tool(
			"rpc_get_pool_info",
			"Get transaction pool information.",
			json!({
				"type": "object",
				"properties": {}
			}),
		)
	}

	fn rpc_get_pool_ready() -> Tool {
		make_tool(
			"rpc_get_pool_ready",
			"Check if tx-pool service is ready.",
			json!({
				"type": "object",
				"properties": {}
			}),
		)
	}

	fn rpc_get_pool_transactions() -> Tool {
		make_tool(
			"rpc_get_pool_transactions",
			"Get all transaction IDs in tx pool.",
			json!({
				"type": "object",
				"properties": {
					"verbose": {
						"type": "boolean",
						"description": "True for detailed JSON, false for array of tx IDs",
						"default": false
					}
				}
			}),
		)
	}

	fn rpc_get_pool_tx_detail() -> Tool {
		make_tool(
			"rpc_get_pool_tx_detail",
			"Get detailed information about a transaction in the pool.",
			json!({
				"type": "object",
				"properties": {
					"tx_hash": {
						"type": "string",
						"description": "Hash of transaction to query"
					}
				},
				"required": ["tx_hash"]
			}),
		)
	}

	fn rpc_get_blockchain_info() -> Tool {
		make_tool(
			"rpc_get_blockchain_info",
			"Get blockchain information including chain type and difficulty.",
			json!({
				"type": "object",
				"properties": {}
			}),
		)
	}

	fn rpc_get_consensus() -> Tool {
		make_tool(
			"rpc_get_consensus",
			"Get consensus parameters.",
			json!({
				"type": "object",
				"properties": {}
			}),
		)
	}

	fn rpc_get_deployments() -> Tool {
		make_tool(
			"rpc_get_deployments",
			"Get soft fork deployment information.",
			json!({
				"type": "object",
				"properties": {}
			}),
		)
	}

	// Category: calculate

	fn rpc_estimate_cycles() -> Tool {
		make_tool(
			"rpc_estimate_cycles",
			"Estimate transaction execution cycles.",
			json!({
				"type": "object",
				"properties": {
					"tx": {
						"type": "object",
						"description": "Transaction object to estimate"
					}
				},
				"required": ["tx"]
			}),
		)
	}

	fn rpc_estimate_fee_rate() -> Tool {
		make_tool(
			"rpc_estimate_fee_rate",
			"Estimate transaction fee rate in shannons per kilobyte.",
			json!({
				"type": "object",
				"properties": {
					"estimate_mode": {
						"type": "string",
						"description": "Fee estimate mode (optional)"
					},
					"enable_fallback": {
						"type": "boolean",
						"description": "Enable fallback algorithm",
						"default": true
					}
				}
			}),
		)
	}

	fn rpc_calculate_dao_withdraw() -> Tool {
		make_tool(
			"rpc_calculate_dao_withdraw",
			"Calculate maximum DAO withdrawal amount.",
			json!({
				"type": "object",
				"properties": {
					"out_point": {
						"type": "object",
						"description": "Reference to the DAO deposit cell",
						"properties": {
							"tx_hash": {"type": "string"},
							"index": {"type": "string"}
						},
						"required": ["tx_hash", "index"]
					},
					"kind": {
						"description": "Block hash (string) or out_point of phase 1 tx"
					}
				},
				"required": ["out_point", "kind"]
			}),
		)
	}

	fn rpc_get_block_economics() -> Tool {
		make_tool(
			"rpc_get_block_economics",
			"Get block issuance, miner rewards, and transaction fees.",
			json!({
				"type": "object",
				"properties": {
					"block_hash": {
						"type": "string",
						"description": "Block hash to analyze"
					}
				},
				"required": ["block_hash"]
			}),
		)
	}

	fn rpc_get_block_median_time() -> Tool {
		make_tool(
			"rpc_get_block_median_time",
			"Get median timestamp of 37 consecutive blocks.",
			json!({
				"type": "object",
				"properties": {
					"block_hash": {
						"type": "string",
						"description": "Block hash indicating highest block in sequence"
					}
				},
				"required": ["block_hash"]
			}),
		)
	}

	fn rpc_get_block_filter() -> Tool {
		make_tool(
			"rpc_get_block_filter",
			"Get BIP-157 block filter for light client SPV.",
			json!({
				"type": "object",
				"properties": {
					"block_hash": {
						"type": "string",
						"description": "Block hash to get filter for"
					}
				},
				"required": ["block_hash"]
			}),
		)
	}

	// Category: verify

	fn rpc_get_transaction_proof() -> Tool {
		make_tool(
			"rpc_get_transaction_proof",
			"Generate Merkle proof for transaction inclusion.",
			json!({
				"type": "object",
				"properties": {
					"tx_hashes": {
						"type": "array",
						"items": { "type": "string" },
						"description": "Transaction hashes (must be in same block)"
					},
					"block_hash": {
						"type": "string",
						"description": "Block hash (optional, searches if omitted)"
					}
				},
				"required": ["tx_hashes"]
			}),
		)
	}

	fn rpc_verify_transaction_proof() -> Tool {
		make_tool(
			"rpc_verify_transaction_proof",
			"Verify Merkle proof and return committed transaction hashes.",
			json!({
				"type": "object",
				"properties": {
					"tx_proof": {
						"type": "object",
						"description": "Transaction proof object",
						"properties": {
							"block_hash": { "type": "string" },
							"witnesses_root": { "type": "string" },
							"proof": {
								"type": "object",
								"properties": {
									"indices": { "type": "array", "items": { "type": "string" } },
									"lemmas": { "type": "array", "items": { "type": "string" } }
								}
							}
						},
						"required": ["block_hash", "witnesses_root", "proof"]
					}
				},
				"required": ["tx_proof"]
			}),
		)
	}
}
