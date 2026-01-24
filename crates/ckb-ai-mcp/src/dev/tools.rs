//! Development tool definitions.

use crate::util::{ToolHints, make_tool_annotated};
use rmcp::model::Tool;
use serde_json::json;
use std::sync::LazyLock;

/// All development tools with dev_* prefix.
pub static DEV_TOOLS: LazyLock<Vec<Tool>> = LazyLock::new(|| {
	vec![
		make_tool_annotated(
			"dev_deploy_cell_data",
			"Deploy Cell Data",
			"[Modifies state] Deploy a cell with hex-encoded data (max 1KB). Creates on-chain data cell using configured private key.",
			json!({
				"type": "object",
				"properties": {
					"data": {
						"type": "string",
						"description": "Hex-encoded data to deploy in the cell (without 0x prefix). Maximum 1KB after decoding."
					}
				},
				"required": ["data"]
			}),
			ToolHints::submit(),
		),
		make_tool_annotated(
			"dev_get_address_balance",
			"Get Address Balance",
			"[Read-only] Get address balance breakdown (total/free/occupied). For richer address info: rpc_search_cells.",
			json!({
				"type": "object",
				"properties": {
					"address": {
						"type": "string",
						"description": "Optional CKB address to check balance for. If omitted, checks the default address from private key."
					}
				}
			}),
			ToolHints::query_live(),
		),
		make_tool_annotated(
			"dev_get_chain_type",
			"Get Chain Type",
			"[Read-only] Get connected CKB node chain type (mainnet/testnet/devnet). Value is fixed for a given node.",
			json!({
				"type": "object",
				"properties": {}
			}),
			// Idempotent: chain type is fixed for a given node.
			ToolHints::query_idempotent(),
		),
		make_tool_annotated(
			"dev_get_genesis_hash",
			"Get Genesis Hash",
			"[Read-only] Get the genesis block hash of the connected CKB chain. Value is immutable.",
			json!({
				"type": "object",
				"properties": {}
			}),
			ToolHints::query_idempotent(),
		),
		make_tool_annotated(
			"dev_generate_lock_info",
			"Generate Lock Info",
			"[Read-only] Pure computation: derive pubkey, lock arg, lock script, lock hash, and addresses from private key.",
			json!({
				"type": "object",
				"properties": {
					"private_key": {
						"type": "string",
						"description": "Private key in hex format (with or without 0x prefix). Required parameter."
					}
				},
				"required": ["private_key"]
			}),
			ToolHints::query_idempotent(),
		),
		make_tool_annotated(
			"dev_get_lock_info_from_address",
			"Get Lock Info from Address",
			"[Read-only] Pure computation: extract lock script, lock hash, and lock arg from a CKB address.",
			json!({
				"type": "object",
				"properties": {
					"address": {
						"type": "string",
						"description": "CKB address (testnet or mainnet format)"
					}
				},
				"required": ["address"]
			}),
			ToolHints::query_idempotent(),
		),
		make_tool_annotated(
			"dev_request_testnet_funds",
			"Request Testnet Funds",
			"[External call] Request CKB testnet funds from the faucet service. Only works on testnet.",
			json!({
				"type": "object",
				"properties": {
					"address": {
						"type": "string",
						"description": "Optional CKB testnet address to receive funds. If omitted, uses the default address from private key."
					}
				}
			}),
			// Not destructive (no data loss), not open_world (single fixed URL).
			ToolHints::write_non_destructive(),
		),
		make_tool_annotated(
			"dev_get_default_account_info",
			"Get Default Account Info",
			"[Read-only] Get server's default account: address, lock script, and capacity breakdown (total/free/occupied).",
			json!({
				"type": "object",
				"properties": {}
			}),
			ToolHints::query_live(),
		),
	]
});
