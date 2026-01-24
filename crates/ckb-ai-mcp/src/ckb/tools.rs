//! CKB composite tool definitions.
//!
//! These tools combine multiple RPC calls into high-level operations,
//! reducing round trips and providing richer context.

use crate::util::{ToolHints, make_tool_with_output_schema};
use rmcp::model::Tool;
use serde_json::json;
use std::sync::LazyLock;

/// All CKB composite tools with ckb_* prefix.
pub static CKB_TOOLS: LazyLock<Vec<Tool>> = LazyLock::new(|| {
	vec![
		make_tool_with_output_schema(
			"ckb_query_address",
			"Query Address",
			"[Read-only] Get complete address state: balance breakdown, recent cells, lock info. Combines search_cells + get_cells_capacity.",
			json!({
				"type": "object",
				"properties": {
					"address": {
						"type": "string",
						"description": "CKB address to query. If omitted, uses the default address from private key."
					},
					"include_cells": {
						"type": "boolean",
						"description": "Include recent cells in response",
						"default": true
					},
					"cell_limit": {
						"type": "integer",
						"description": "Maximum number of cells to return",
						"default": 10
					}
				}
			}),
			ToolHints::query_live(),
			Some(json!({
				"type": "object",
				"properties": {
					"address": { "type": "string", "description": "The queried CKB address" },
					"lock_script": {
						"type": "object",
						"description": "Lock script derived from address",
						"properties": {
							"code_hash": { "type": "string" },
							"hash_type": { "type": "string" },
							"args": { "type": "string" }
						}
					},
					"capacity": {
						"type": "object",
						"description": "Capacity breakdown in shannons",
						"properties": {
							"capacity": { "type": "string", "description": "Total capacity in hex" },
							"block_hash": { "type": "string" },
							"block_number": { "type": "string" }
						}
					},
					"cells": {
						"type": "object",
						"description": "Recent cells (if include_cells=true)",
						"properties": {
							"objects": { "type": "array" },
							"last_cursor": { "type": "string" }
						}
					}
				},
				"required": ["address", "lock_script", "capacity"]
			})),
		),
		make_tool_with_output_schema(
			"ckb_query_chain_status",
			"Query Chain Status",
			"[Read-only] Get chain health snapshot: tip block, sync state, indexer status, mempool info. Single call replaces 4 RPC queries.",
			json!({
				"type": "object",
				"properties": {}
			}),
			ToolHints::query_live(),
			Some(json!({
				"type": "object",
				"properties": {
					"tip": {
						"type": "object",
						"description": "Current tip block header",
						"properties": {
							"number": { "type": "string", "description": "Block height in hex" },
							"hash": { "type": "string" },
							"timestamp": { "type": "string" }
						}
					},
					"sync": {
						"type": "object",
						"description": "Node sync state",
						"properties": {
							"best_known_block_number": { "type": "string" },
							"best_known_block_timestamp": { "type": "string" },
							"ibd": { "type": "boolean", "description": "Initial block download in progress" }
						}
					},
					"indexer": {
						"type": "object",
						"description": "Indexer sync status",
						"properties": {
							"block_hash": { "type": "string" },
							"block_number": { "type": "string" }
						}
					},
					"mempool": {
						"type": "object",
						"description": "Transaction pool info",
						"properties": {
							"pending": { "type": "string", "description": "Pending tx count in hex" },
							"proposed": { "type": "string" },
							"total_tx_size": { "type": "string" },
							"total_tx_cycles": { "type": "string" }
						}
					}
				},
				"required": ["tip", "sync", "indexer", "mempool"]
			})),
		),
		make_tool_with_output_schema(
			"ckb_query_transaction",
			"Query Transaction",
			"[Read-only] Get transaction with resolved input cells. Shows what was consumed, not just outpoints.",
			json!({
				"type": "object",
				"properties": {
					"tx_hash": {
						"type": "string",
						"description": "Transaction hash to query"
					}
				},
				"required": ["tx_hash"]
			}),
			ToolHints::query_live(),
			Some(json!({
				"type": "object",
				"properties": {
					"transaction": {
						"type": "object",
						"description": "Full transaction with status",
						"properties": {
							"transaction": { "type": "object" },
							"tx_status": {
								"type": "object",
								"properties": {
									"status": { "type": "string", "enum": ["pending", "proposed", "committed", "unknown", "rejected"] },
									"block_hash": { "type": ["string", "null"] }
								}
							}
						}
					},
					"resolved_inputs": {
						"type": "array",
						"description": "Input cells with resolved data",
						"items": {
							"type": "object",
							"properties": {
								"previous_output": { "type": "object" },
								"cell": { "type": ["object", "null"], "description": "Resolved cell data or null if consumed" },
								"note": { "type": "string", "description": "Status note if cell unavailable" }
							}
						}
					}
				},
				"required": ["transaction", "resolved_inputs"]
			})),
		),
		make_tool_with_output_schema(
			"ckb_validate_transaction",
			"Validate Transaction",
			"[Read-only] Complete pre-submission validation: dry-run, cycle estimation, fee rate check.",
			json!({
				"type": "object",
				"properties": {
					"tx": {
						"type": "object",
						"description": "Transaction object to validate"
					}
				},
				"required": ["tx"]
			}),
			ToolHints::query_live(),
			Some(json!({
				"type": "object",
				"properties": {
					"dry_run": {
						"type": "object",
						"description": "Dry-run test result",
						"properties": {
							"success": { "type": "boolean" },
							"result": { "type": "object", "description": "Test result if success" },
							"error": { "type": "string", "description": "Error message if failed" }
						},
						"required": ["success"]
					},
					"cycles": {
						"type": "object",
						"description": "Cycle estimation result",
						"properties": {
							"success": { "type": "boolean" },
							"result": { "type": "object", "description": "Cycles count if success" },
							"error": { "type": "string", "description": "Error message if failed" }
						},
						"required": ["success"]
					},
					"fee_rate": {
						"type": "object",
						"description": "Current network fee rate",
						"properties": {
							"success": { "type": "boolean" },
							"result": { "type": "object", "description": "Fee rate if success" },
							"error": { "type": "string", "description": "Error message if failed" }
						},
						"required": ["success"]
					}
				},
				"required": ["dry_run", "cycles", "fee_rate"]
			})),
		),
		make_tool_with_output_schema(
			"ckb_query_script_cells",
			"Query Script Cells",
			"[Read-only] Find cells by lock or type script with simplified parameters.",
			json!({
				"type": "object",
				"properties": {
					"script_type": {
						"type": "string",
						"enum": ["lock", "type"],
						"description": "Whether to search by lock script or type script"
					},
					"code_hash": {
						"type": "string",
						"description": "Script code hash (0x-prefixed hex)"
					},
					"hash_type": {
						"type": "string",
						"enum": ["type", "data", "data1", "data2"],
						"description": "Script hash type"
					},
					"args": {
						"type": "string",
						"description": "Script args (0x-prefixed hex). Optional prefix matching supported."
					},
					"limit": {
						"type": "integer",
						"description": "Maximum number of cells to return",
						"default": 20
					},
					"order": {
						"type": "string",
						"enum": ["asc", "desc"],
						"description": "Sort order by block number",
						"default": "desc"
					}
				},
				"required": ["script_type", "code_hash", "hash_type"]
			}),
			ToolHints::query_live(),
			Some(json!({
				"type": "object",
				"properties": {
					"search_key": {
						"type": "object",
						"description": "The search key used for query",
						"properties": {
							"script": { "type": "object" },
							"script_type": { "type": "string" },
							"script_search_mode": { "type": "string" }
						}
					},
					"cells": {
						"type": "object",
						"description": "Matching cells",
						"properties": {
							"objects": {
								"type": "array",
								"items": {
									"type": "object",
									"properties": {
										"out_point": { "type": "object" },
										"output": { "type": "object" },
										"output_data": { "type": "string" },
										"block_number": { "type": "string" },
										"tx_index": { "type": "string" }
									}
								}
							},
							"last_cursor": { "type": "string" }
						}
					},
					"total_capacity": {
						"type": "object",
						"description": "Total capacity of all matching cells",
						"properties": {
							"capacity": { "type": "string" },
							"block_hash": { "type": "string" },
							"block_number": { "type": "string" }
						}
					}
				},
				"required": ["search_key", "cells", "total_capacity"]
			})),
		),
	]
});
