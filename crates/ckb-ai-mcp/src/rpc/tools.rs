//! RPC tool definitions with new naming convention.
//!
//! Tools are renamed from the original CKB RPC names to follow the pattern:
//! `rpc_{action}_{target}`

use crate::util::{make_tool_annotated, ToolHints};
use rmcp::model::Tool;
use serde_json::json;
use std::sync::LazyLock;

/// Static list of all RPC tools.
pub static RPC_TOOLS: LazyLock<Vec<Tool>> = LazyLock::new(RpcToolDefinitions::all);

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
		make_tool_annotated(
			"rpc_get_block",
			"Get Block",
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
			ToolHints::query_idempotent(),
		)
	}

	fn rpc_get_block_by_number() -> Tool {
		make_tool_annotated(
			"rpc_get_block_by_number",
			"Get Block by Number",
			"Get CKB block by number. Returns header, transactions, proposals, and uncles.",
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
			ToolHints::query_idempotent(),
		)
	}

	fn rpc_get_header() -> Tool {
		make_tool_annotated(
			"rpc_get_header",
			"Get Header",
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
			ToolHints::query_idempotent(),
		)
	}

	fn rpc_get_header_by_number() -> Tool {
		make_tool_annotated(
			"rpc_get_header_by_number",
			"Get Header by Number",
			"Get CKB block header by block number.",
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
			ToolHints::query_idempotent(),
		)
	}

	fn rpc_get_transaction() -> Tool {
		make_tool_annotated(
			"rpc_get_transaction",
			"Get Transaction",
			"Get CKB transaction by hash. Returns transaction data and status.",
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
			// Not idempotent: status changes (pending → committed).
			ToolHints::query_live(),
		)
	}

	fn rpc_get_block_hash() -> Tool {
		make_tool_annotated(
			"rpc_get_block_hash",
			"Get Block Hash",
			"Get CKB block hash by block number.",
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
			ToolHints::query_idempotent(),
		)
	}

	fn rpc_get_tip_header() -> Tool {
		make_tool_annotated(
			"rpc_get_tip_header",
			"Get Tip Header",
			"Get the latest block header from the CKB chain tip.",
			json!({
				"type": "object",
				"properties": {}
			}),
			ToolHints::query_live(),
		)
	}

	fn rpc_get_tip_block_number() -> Tool {
		make_tool_annotated(
			"rpc_get_tip_block_number",
			"Get Tip Block Number",
			"Get the current CKB chain tip block number (height).",
			json!({
				"type": "object",
				"properties": {}
			}),
			ToolHints::query_live(),
		)
	}

	fn rpc_get_current_epoch() -> Tool {
		make_tool_annotated(
			"rpc_get_current_epoch",
			"Get Current Epoch",
			"Get current CKB epoch information including number, start, and length.",
			json!({
				"type": "object",
				"properties": {}
			}),
			ToolHints::query_live(),
		)
	}

	fn rpc_get_epoch_by_number() -> Tool {
		make_tool_annotated(
			"rpc_get_epoch_by_number",
			"Get Epoch by Number",
			"Get CKB epoch information by epoch number.",
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
			ToolHints::query_idempotent(),
		)
	}

	fn rpc_get_live_cell() -> Tool {
		make_tool_annotated(
			"rpc_get_live_cell",
			"Get Live Cell",
			"Get live cell by outpoint (tx_hash + index). Returns cell data and status.",
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
			// Not idempotent: cell may be consumed.
			ToolHints::query_live(),
		)
	}

	fn rpc_get_fork_block() -> Tool {
		make_tool_annotated(
			"rpc_get_fork_block",
			"Get Fork Block",
			"Get fork block information by hash. Used for chain reorganization analysis.",
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
			ToolHints::query_idempotent(),
		)
	}

	// Category: search (Indexer)

	fn rpc_get_indexer_tip() -> Tool {
		make_tool_annotated(
			"rpc_get_indexer_tip",
			"Get Indexer Tip",
			"Get CKB indexer sync tip. Shows indexer synchronization status.",
			json!({
				"type": "object",
				"properties": {}
			}),
			ToolHints::query_live(),
		)
	}

	fn rpc_search_cells() -> Tool {
		make_tool_annotated(
			"rpc_search_cells",
			"Search Cells",
			"Search for CKB cells by lock/type script criteria. Returns matching cells with pagination.",
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
			ToolHints::query_live(),
		)
	}

	fn rpc_search_transactions() -> Tool {
		make_tool_annotated(
			"rpc_search_transactions",
			"Search Transactions",
			"Search for CKB transactions by script criteria. Returns matching transactions.",
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
			ToolHints::query_live(),
		)
	}

	fn rpc_get_cells_capacity() -> Tool {
		make_tool_annotated(
			"rpc_get_cells_capacity",
			"Get Cells Capacity",
			"Get total CKB capacity of cells matching search criteria.",
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
			ToolHints::query_live(),
		)
	}

	// Category: submit

	fn rpc_submit_transaction() -> Tool {
		make_tool_annotated(
			"rpc_submit_transaction",
			"Submit Transaction",
			"Submit a CKB transaction to the network for inclusion in a block.",
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
			ToolHints::submit(),
		)
	}

	fn rpc_test_transaction() -> Tool {
		make_tool_annotated(
			"rpc_test_transaction",
			"Test Transaction",
			"Test if a CKB transaction would be accepted without broadcasting. Dry-run validation.",
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
			ToolHints::query_idempotent(),
		)
	}

	// Category: status

	fn rpc_get_node_info() -> Tool {
		make_tool_annotated(
			"rpc_get_node_info",
			"Get Node Info",
			"Get local CKB node information including version and protocols.",
			json!({
				"type": "object",
				"properties": {}
			}),
			ToolHints::query_live(),
		)
	}

	fn rpc_get_sync_state() -> Tool {
		make_tool_annotated(
			"rpc_get_sync_state",
			"Get Sync State",
			"Get CKB chain synchronization state. Shows sync progress.",
			json!({
				"type": "object",
				"properties": {}
			}),
			ToolHints::query_live(),
		)
	}

	fn rpc_get_peers() -> Tool {
		make_tool_annotated(
			"rpc_get_peers",
			"Get Peers",
			"Get connected CKB network peers information.",
			json!({
				"type": "object",
				"properties": {}
			}),
			ToolHints::query_live(),
		)
	}

	fn rpc_get_pool_info() -> Tool {
		make_tool_annotated(
			"rpc_get_pool_info",
			"Get Pool Info",
			"Get CKB transaction pool information including size and fees.",
			json!({
				"type": "object",
				"properties": {}
			}),
			ToolHints::query_live(),
		)
	}

	fn rpc_get_pool_ready() -> Tool {
		make_tool_annotated(
			"rpc_get_pool_ready",
			"Get Pool Ready",
			"Check if CKB tx-pool service is ready to accept transactions.",
			json!({
				"type": "object",
				"properties": {}
			}),
			ToolHints::query_live(),
		)
	}

	fn rpc_get_pool_transactions() -> Tool {
		make_tool_annotated(
			"rpc_get_pool_transactions",
			"Get Pool Transactions",
			"Get all transaction IDs currently in the CKB tx pool.",
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
			ToolHints::query_live(),
		)
	}

	fn rpc_get_pool_tx_detail() -> Tool {
		make_tool_annotated(
			"rpc_get_pool_tx_detail",
			"Get Pool TX Detail",
			"Get detailed information about a specific transaction in the CKB pool.",
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
			ToolHints::query_live(),
		)
	}

	fn rpc_get_blockchain_info() -> Tool {
		make_tool_annotated(
			"rpc_get_blockchain_info",
			"Get Blockchain Info",
			"Get CKB blockchain information including chain type, difficulty, and median time.",
			json!({
				"type": "object",
				"properties": {}
			}),
			ToolHints::query_live(),
		)
	}

	fn rpc_get_consensus() -> Tool {
		make_tool_annotated(
			"rpc_get_consensus",
			"Get Consensus",
			"Get CKB consensus parameters including block intervals and rewards.",
			json!({
				"type": "object",
				"properties": {}
			}),
			// Not idempotent: consensus may change after hard forks.
			ToolHints::query_live(),
		)
	}

	fn rpc_get_deployments() -> Tool {
		make_tool_annotated(
			"rpc_get_deployments",
			"Get Deployments",
			"Get CKB soft fork deployment information and activation status.",
			json!({
				"type": "object",
				"properties": {}
			}),
			ToolHints::query_live(),
		)
	}

	// Category: calculate

	fn rpc_estimate_cycles() -> Tool {
		make_tool_annotated(
			"rpc_estimate_cycles",
			"Estimate Cycles",
			"Estimate CKB transaction execution cycles for fee calculation.",
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
			ToolHints::query_idempotent(),
		)
	}

	fn rpc_estimate_fee_rate() -> Tool {
		make_tool_annotated(
			"rpc_estimate_fee_rate",
			"Estimate Fee Rate",
			"Estimate CKB transaction fee rate in shannons per kilobyte.",
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
			ToolHints::query_live(),
		)
	}

	fn rpc_calculate_dao_withdraw() -> Tool {
		make_tool_annotated(
			"rpc_calculate_dao_withdraw",
			"Calculate DAO Withdraw",
			"Calculate maximum CKB Nervos DAO withdrawal amount including interest.",
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
			ToolHints::query_idempotent(),
		)
	}

	fn rpc_get_block_economics() -> Tool {
		make_tool_annotated(
			"rpc_get_block_economics",
			"Get Block Economics",
			"Get CKB block issuance, miner rewards, and transaction fees breakdown.",
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
			ToolHints::query_idempotent(),
		)
	}

	fn rpc_get_block_median_time() -> Tool {
		make_tool_annotated(
			"rpc_get_block_median_time",
			"Get Block Median Time",
			"Get median timestamp of 37 consecutive CKB blocks.",
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
			ToolHints::query_idempotent(),
		)
	}

	fn rpc_get_block_filter() -> Tool {
		make_tool_annotated(
			"rpc_get_block_filter",
			"Get Block Filter",
			"Get BIP-157 block filter for CKB light client SPV verification.",
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
			ToolHints::query_idempotent(),
		)
	}

	// Category: verify

	fn rpc_get_transaction_proof() -> Tool {
		make_tool_annotated(
			"rpc_get_transaction_proof",
			"Get Transaction Proof",
			"Generate Merkle proof for CKB transaction inclusion in a block.",
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
			ToolHints::query_idempotent(),
		)
	}

	fn rpc_verify_transaction_proof() -> Tool {
		make_tool_annotated(
			"rpc_verify_transaction_proof",
			"Verify Transaction Proof",
			"Verify Merkle proof and return committed CKB transaction hashes.",
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
			ToolHints::query_idempotent(),
		)
	}
}
