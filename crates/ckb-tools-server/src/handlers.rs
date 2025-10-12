use shared::{
	error::Result,
	mcp::{create_error_response, create_success_response, McpRequest, McpResponse, ToolDefinition},
};
use serde_json::{json, Value};
use tracing::{debug, error, info};

use crate::tools::ToolsProvider;

pub struct McpHandler {
	tools_provider: ToolsProvider,
}

impl McpHandler {
	pub fn new(tools_provider: ToolsProvider) -> Self {
		Self { tools_provider }
	}

	pub async fn handle_request(&self, request: McpRequest) -> Result<McpResponse> {
		debug!("Handling MCP request: {}", request.method);

		match request.method.as_str() {
			"initialize" => self.handle_initialize(request.id).await,
			"tools/list" => self.handle_tools_list(request.id).await,
			"tools/call" => self.handle_tools_call(request.params, request.id).await,
			_ => Ok(create_error_response(
				request.id,
				-32601,
				format!("Method not found: {}", request.method),
			)),
		}
	}

	async fn handle_initialize(&self, id: Option<Value>) -> Result<McpResponse> {
		let result = json!({
			"protocolVersion": "2024-11-05",
			"capabilities": {
				"tools": {
					"listChanged": false
				}
			},
			"serverInfo": {
				"name": "ckb-tools-server",
				"version": "0.1.0"
			}
		});

		Ok(create_success_response(id, result))
	}

	async fn handle_tools_list(&self, id: Option<Value>) -> Result<McpResponse> {
		let tools = vec![
			ToolDefinition {
				name: "DeployCellData".to_string(),
				description: "Deploy a cell with data provided directly to the MCP server. Maximum data size: 20KB (20,480 bytes). For larger files, use DeployCellDataChunked.".to_string(),
				input_schema: json!({
					"type": "object",
					"properties": {
						"data": {
							"type": "string",
							"description": "Hex-encoded data to deploy in the cell (without 0x prefix). Maximum 20KB (20,480 bytes) after decoding."
						}
					},
					"required": ["data"]
				}),
			},
			ToolDefinition {
				name: "GetAddressBalance".to_string(),
				description: "Get the CKB balance for an address. If no address is provided, returns balance of the default sender address.".to_string(),
				input_schema: json!({
					"type": "object",
					"properties": {
						"address": {
							"type": "string",
							"description": "Optional CKB address to check balance for. If omitted, checks the default address from private key."
						}
					}
				}),
			},
			ToolDefinition {
				name: "GetChainType".to_string(),
				description: "Get the chain type of the connected CKB node (mainnet, testnet, or devnet)".to_string(),
				input_schema: json!({
					"type": "object",
					"properties": {}
				}),
			},
			ToolDefinition {
				name: "GetGenesisHash".to_string(),
				description: "Get the genesis block hash of the connected CKB chain".to_string(),
				input_schema: json!({
					"type": "object",
					"properties": {}
				}),
			},
			ToolDefinition {
				name: "GenerateLockInfo".to_string(),
				description: "Generate all lock values from a private key, showing the complete transformation chain: Private Key → Public Key → Lock Arg → Lock Script → Lock Hash → Address. The private key must be provided and will be included in the response for educational purposes.".to_string(),
				input_schema: json!({
					"type": "object",
					"properties": {
						"private_key": {
							"type": "string",
							"description": "Private key in hex format (with or without 0x prefix). Required parameter."
						}
					},
					"required": ["private_key"]
				}),
			},
			ToolDefinition {
				name: "GetLockInfoFromAddress".to_string(),
				description: "Extract lock information from a CKB address. Returns lock script, lock hash, lock arg, and both testnet/mainnet addresses. Note: Private key and public key cannot be derived from an address.".to_string(),
				input_schema: json!({
					"type": "object",
					"properties": {
						"address": {
							"type": "string",
							"description": "CKB address (testnet or mainnet format)"
						}
					},
					"required": ["address"]
				}),
			},
			ToolDefinition {
				name: "RequestTestnetFunds".to_string(),
				description: "Request CKB testnet funds from the faucet. If no address is provided, funds are sent to the default address from the configured private key.".to_string(),
				input_schema: json!({
					"type": "object",
					"properties": {
						"address": {
							"type": "string",
							"description": "Optional CKB testnet address to receive funds. If omitted, uses the default address from private key."
						}
					}
				}),
			},
			ToolDefinition {
				name: "GetDefaultAccountInfo".to_string(),
				description: "Get information about the default account configured in the server (derived from the private key). Returns address, lock script details, and current balance. Private key is never exposed.".to_string(),
				input_schema: json!({
					"type": "object",
					"properties": {}
				}),
			},
		];

		let result = json!({ "tools": tools });
		Ok(create_success_response(id, result))
	}

	async fn handle_tools_call(&self, params: Option<Value>, id: Option<Value>) -> Result<McpResponse> {
		let params = params.ok_or_else(|| {
			shared::error::CkbMcpError::InvalidParameter("Missing parameters".to_string())
		})?;

		let tool_name = params
			.get("name")
			.and_then(|v| v.as_str())
			.ok_or_else(|| {
				shared::error::CkbMcpError::InvalidParameter("Missing tool name".to_string())
			})?;

		let default_args = json!({});
		let arguments = params.get("arguments").unwrap_or(&default_args);

		info!("Calling tool: {} with arguments: {}", tool_name, arguments);

		let result = match tool_name {
			"DeployCellData" => self.call_deploy_cell_data(arguments).await,
			"GetAddressBalance" => self.call_get_address_balance(arguments).await,
			"GetChainType" => self.call_get_chain_type().await,
			"GetGenesisHash" => self.call_get_genesis_hash().await,
			"GenerateLockInfo" => self.call_generate_lock_info(arguments),
			"GetLockInfoFromAddress" => self.call_get_lock_info_from_address(arguments),
			"RequestTestnetFunds" => self.call_request_testnet_funds(arguments).await,
			"GetDefaultAccountInfo" => self.call_get_default_account_info().await,
			_ => {
				return Ok(create_error_response(
					id,
					-32602,
					format!("Unknown tool: {}", tool_name),
				))
			}
		};

		match result {
			Ok(data) => Ok(create_success_response(
				id,
				json!({
					"content": [{
						"type": "text",
						"text": data
					}]
				}),
			)),
			Err(e) => {
				error!("Tool call failed: {}", e);
				Ok(create_error_response(id, -32603, e.to_string()))
			}
		}
	}

	async fn call_deploy_cell_data(&self, args: &Value) -> Result<String> {
		let data_hex = args
			.get("data")
			.and_then(|v| v.as_str())
			.ok_or_else(|| {
				shared::error::CkbMcpError::InvalidParameter("Missing data parameter".to_string())
			})?;

		// Decode hex string to bytes
		let data = hex::decode(data_hex.trim_start_matches("0x")).map_err(|e| {
			shared::error::CkbMcpError::InvalidParameter(format!("Invalid hex data: {}", e))
		})?;

		// Enforce 20KB limit
		const MAX_DATA_SIZE: usize = 20 * 1024; // 20KB
		if data.len() > MAX_DATA_SIZE {
			return Err(shared::error::CkbMcpError::InvalidParameter(format!(
				"Data size {} bytes exceeds maximum limit of {} bytes (20KB). Use DeployCellDataChunked for larger files.",
				data.len(),
				MAX_DATA_SIZE
			)));
		}

		let result = self.tools_provider.deploy_cell_data(data).await?;

		Ok(serde_json::to_string_pretty(&result)?)
	}

	async fn call_get_chain_type(&self) -> Result<String> {
		let chain_type = self.tools_provider.get_chain_type().await?;
		Ok(chain_type)
	}

	async fn call_get_genesis_hash(&self) -> Result<String> {
		let genesis_hash = self.tools_provider.get_genesis_hash().await?;
		Ok(genesis_hash)
	}

	async fn call_get_address_balance(&self, args: &Value) -> Result<String> {
		let address = args
			.get("address")
			.and_then(|v| v.as_str())
			.map(|s| s.to_string());

		let result = self.tools_provider.get_address_balance(address).await?;

		Ok(serde_json::to_string_pretty(&result)?)
	}

	fn call_generate_lock_info(&self, args: &Value) -> Result<String> {
		let private_key = args
			.get("private_key")
			.and_then(|v| v.as_str())
			.ok_or_else(|| {
				shared::error::CkbMcpError::InvalidParameter(
					"private_key parameter is required. To get info about the default account, use GetDefaultAccountInfo instead.".to_string()
				)
			})?
			.to_string();

		let result = self.tools_provider.generate_lock_info(Some(private_key))?;

		Ok(serde_json::to_string_pretty(&result)?)
	}

	fn call_get_lock_info_from_address(&self, args: &Value) -> Result<String> {
		let address = args
			.get("address")
			.and_then(|v| v.as_str())
			.ok_or_else(|| {
				shared::error::CkbMcpError::InvalidParameter("Missing address parameter".to_string())
			})?
			.to_string();

		let result = self.tools_provider.get_lock_info_from_address(address)?;

		Ok(serde_json::to_string_pretty(&result)?)
	}

	async fn call_request_testnet_funds(&self, args: &Value) -> Result<String> {
		let address = args
			.get("address")
			.and_then(|v| v.as_str())
			.map(|s| s.to_string());

		let result = self.tools_provider.request_testnet_funds(address).await?;

		Ok(result)
	}

	async fn call_get_default_account_info(&self) -> Result<String> {
		let result = self.tools_provider.get_default_account_info().await?;

		Ok(serde_json::to_string_pretty(&result)?)
	}
}
