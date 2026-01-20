//! Development tool definitions.

use crate::util::make_tool;
use rmcp::model::Tool;
use serde_json::json;
use std::sync::LazyLock;

/// All development tools with dev_* prefix.
pub static DEV_TOOLS: LazyLock<Vec<Tool>> = LazyLock::new(|| {
	vec![
		make_tool(
			"dev_deploy_cell_data",
			"Deploy a cell with hex-encoded data (max 1KB). For larger files, POST multipart \
			form to /deploy/file endpoint on this server. Example: curl -F 'file=@/path/to/file' \
			<base_url>/deploy/file",
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
		),
		make_tool(
			"dev_get_address_balance",
			"Get the CKB balance for an address. If no address is provided, returns balance \
			of the default sender address.",
			json!({
				"type": "object",
				"properties": {
					"address": {
						"type": "string",
						"description": "Optional CKB address to check balance for. If omitted, checks the default address from private key."
					}
				}
			}),
		),
		make_tool(
			"dev_get_chain_type",
			"Get the chain type of the connected CKB node (mainnet, testnet, or devnet).",
			json!({
				"type": "object",
				"properties": {}
			}),
		),
		make_tool(
			"dev_get_genesis_hash",
			"Get the genesis block hash of the connected CKB chain.",
			json!({
				"type": "object",
				"properties": {}
			}),
		),
		make_tool(
			"dev_generate_lock_info",
			"Generate all lock values from a private key, showing the complete transformation \
			chain: Private Key → Public Key → Lock Arg → Lock Script → Lock Hash → Address. \
			The private key must be provided and will be included in the response for \
			educational purposes.",
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
		),
		make_tool(
			"dev_get_lock_info_from_address",
			"Extract lock information from a CKB address. Returns lock script, lock hash, \
			lock arg, and both testnet/mainnet addresses. Note: Private key and public key \
			cannot be derived from an address.",
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
		),
		make_tool(
			"dev_request_testnet_funds",
			"Request CKB testnet funds from the faucet. If no address is provided, funds \
			are sent to the default address from the configured private key.",
			json!({
				"type": "object",
				"properties": {
					"address": {
						"type": "string",
						"description": "Optional CKB testnet address to receive funds. If omitted, uses the default address from private key."
					}
				}
			}),
		),
		make_tool(
			"dev_get_default_account_info",
			"Get information about the default account configured in the server (derived \
			from the private key). Returns address, lock script details, and capacity \
			breakdown: capacity_shannons/capacity_ckb (total capacity), \
			free_capacity_shannons/free_capacity_ckb (immediately spendable), \
			occupied_capacity_shannons/occupied_capacity_ckb (locked in cells with data/tokens). \
			Private key is never exposed.",
			json!({
				"type": "object",
				"properties": {}
			}),
		),
	]
});
