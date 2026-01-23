//! CKB composite tool definitions.
//!
//! These tools combine multiple RPC calls into high-level operations,
//! reducing round trips and providing richer context.

use crate::util::{make_tool_annotated, ToolHints};
use rmcp::model::Tool;
use serde_json::json;
use std::sync::LazyLock;

/// All CKB composite tools with ckb_* prefix.
pub static CKB_TOOLS: LazyLock<Vec<Tool>> = LazyLock::new(|| {
	vec![
		make_tool_annotated(
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
		),
		make_tool_annotated(
			"ckb_query_chain_status",
			"Query Chain Status",
			"[Read-only] Get chain health snapshot: tip block, sync state, indexer status, mempool info. Single call replaces 4 RPC queries.",
			json!({
				"type": "object",
				"properties": {}
			}),
			ToolHints::query_live(),
		),
		make_tool_annotated(
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
		),
		make_tool_annotated(
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
		),
		make_tool_annotated(
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
		),
	]
});
