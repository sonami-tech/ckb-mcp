//! Development tool definitions.

use crate::util::{make_tool_annotated, ToolHints};
use rmcp::model::Tool;
use serde_json::json;
use std::sync::LazyLock;

/// All development tools with dev_* prefix.
pub static DEV_TOOLS: LazyLock<Vec<Tool>> = LazyLock::new(|| {
	vec![
		make_tool_annotated(
			"dev_deploy_cell_data",
			"Deploy Cell Data",
			"Deploy a CKB cell with hex-encoded data (max 1KB). Creates on-chain data cell.",
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
			"Get CKB balance for an address in shannons and CKB.",
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
			"Get connected CKB node chain type: mainnet, testnet, or devnet.",
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
			"Get the genesis block hash of the connected CKB chain.",
			json!({
				"type": "object",
				"properties": {}
			}),
			ToolHints::query_idempotent(),
		),
		make_tool_annotated(
			"dev_generate_lock_info",
			"Generate Lock Info",
			"Generate CKB lock values from private key: pubkey, lock arg, lock script, lock hash, address.",
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
			"Extract CKB lock script, lock hash, and lock arg from an address.",
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
			"Request CKB testnet funds from the faucet. External service call.",
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
			"Get default CKB account info: address, lock script, capacity breakdown.",
			json!({
				"type": "object",
				"properties": {}
			}),
			ToolHints::query_live(),
		),
	]
});
