//! RPC tool definitions with new naming convention.
//!
//! Tools are renamed from the original CKB RPC names to follow the pattern:
//! `rpc_{action}_{target}`

use crate::util::{ToolHints, make_tool_annotated};
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
			"[Read-only] Retrieve complete block with all transactions. For header only: rpc_get_header.",
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
			"[Read-only] Retrieve complete block by number. For header only: rpc_get_header_by_number.",
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
			"[Read-only] Retrieve block header without transactions. For full block: rpc_get_block.",
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
			"[Read-only] Retrieve block header by number. For full block: rpc_get_block_by_number.",
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
			"[Read-only] Retrieve transaction data and status by hash. For pending txs: rpc_get_pool_tx_detail.",
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
			"[Read-only] Convert block number to hash. For full block data: rpc_get_block_by_number.",
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
			"[Read-only] Get latest block header from chain tip. For block number only: rpc_get_tip_block_number.",
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
			"[Read-only] Get current chain height. For full header: rpc_get_tip_header.",
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
			"[Read-only] Get current epoch number, start block, and length. For historical: rpc_get_epoch_by_number.",
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
			"[Read-only] Get historical epoch details. For current epoch: rpc_get_current_epoch.",
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
			"[Read-only] Get single cell by outpoint. For bulk queries: rpc_search_cells.",
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
			"[Read-only] Get orphaned block from chain reorganization. For canonical blocks: rpc_get_block.",
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
			"[Read-only] Get indexer sync status. Check before using rpc_search_cells or rpc_search_transactions.",
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
			"[Read-only] Search cells by script criteria with full filter control. For total capacity: rpc_get_cells_capacity.",
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
			"[Read-only] Search transaction history by script. For single tx: rpc_get_transaction.",
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
			"[Read-only] Get total capacity of matching cells. For cell details: rpc_search_cells.",
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
			"[Modifies state] Submit transaction to network for inclusion. Validate first with rpc_test_transaction.",
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
			"[Read-only] Dry-run transaction validation without broadcasting. Use before rpc_submit_transaction.",
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
			"[Read-only] Get node version, protocols, and addresses. For sync status: rpc_get_sync_state.",
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
			"[Read-only] Get chain sync progress and IBD status. For node info: rpc_get_node_info.",
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
			"[Read-only] Get connected peer addresses and protocols. For node status: rpc_get_node_info.",
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
			"[Read-only] Get mempool size, limits, and fee statistics. For pending txs: rpc_get_pool_transactions.",
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
			"[Read-only] Check if mempool is ready. Call before rpc_submit_transaction.",
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
			"[Read-only] List all pending transaction hashes. For tx details: rpc_get_pool_tx_detail.",
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
			"[Read-only] Get pending transaction details including fee and ancestors. For committed: rpc_get_transaction.",
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
			"[Read-only] Get chain type (mainnet/testnet), difficulty, and alerts. For consensus: rpc_get_consensus.",
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
			"[Read-only] Get consensus rules, block intervals, and reward parameters. For forks: rpc_get_deployments.",
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
			"[Read-only] Get soft fork activation status and thresholds. For consensus params: rpc_get_consensus.",
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
			"[Read-only] Estimate script execution cycles for fee calculation. For fee rate: rpc_estimate_fee_rate.",
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
			"[Read-only] Estimate fee rate in shannons/KB from recent blocks. For cycles: rpc_estimate_cycles.",
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
			"[Read-only] Calculate DAO withdrawal amount with accumulated interest. For block economics: rpc_get_block_economics.",
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
			"[Read-only] Get block issuance, miner rewards, and fee breakdown. For DAO: rpc_calculate_dao_withdraw.",
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
			"[Read-only] Get median time of 37 blocks for time-lock validation. For block header: rpc_get_header.",
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
			"[Read-only] Get BIP-157 block filter for light client SPV. For tx proof: rpc_get_transaction_proof.",
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
			"[Read-only] Generate Merkle proof for tx inclusion. Verify with rpc_verify_transaction_proof.",
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
			"[Read-only] Verify Merkle proof from rpc_get_transaction_proof. Returns verified tx hashes.",
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
