use shared::{
	error::Result,
	mcp::{create_error_response, create_success_response, McpRequest, McpResponse, ToolDefinition},
};
use serde_json::{json, Value};
use tracing::{debug, error, info};

use crate::session::{SessionManager, SessionState};
use crate::tools::ToolsProvider;
use blake2::{digest::consts::U32, Blake2b, Digest};
use sha2::Sha256;

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
			ToolDefinition {
				name: "DeployCellDataChunked".to_string(),
				description: "Multi-phase chunked upload for deploying large data files (up to 350KB) to CKB blockchain. \
				Operations: initialize, append, status, finalize, deploy, cancel. \
				Workflow: 1) initialize with expected_size → get session_key, 2) append chunks repeatedly with session_key + chunk_data, \
				3) finalize to compute hashes, 4) deploy to blockchain. \
				Examples: initialize: {\"operation\":\"initialize\",\"expected_size\":102400}; \
				append: {\"operation\":\"append\",\"session_key\":\"uuid\",\"chunk_data\":\"base64...\"}; \
				status: {\"operation\":\"status\",\"session_key\":\"uuid\"}; \
				finalize: {\"operation\":\"finalize\",\"session_key\":\"uuid\"}; \
				deploy: {\"operation\":\"deploy\",\"session_key\":\"uuid\"}. \
				Chunk size: recommended 10KB, max 50KB. Total size: max 350KB.".to_string(),
				input_schema: json!({
					"type": "object",
					"properties": {
						"operation": {
							"type": "string",
							"enum": ["initialize", "append", "status", "finalize", "deploy", "cancel"],
							"description": "Operation to perform: initialize (create session with expected_size), append (add chunk_data), status (query state), finalize (validate and hash), deploy (send to chain), cancel (delete)."
						},
						"expected_size": {
							"type": "number",
							"description": "For initialize: Total expected size in bytes (max 358,400 = 350KB). Required for initialize operation only."
						},
						"session_key": {
							"type": "string",
							"description": "Session key from initialize. Required for append, status, finalize, deploy, cancel operations."
						},
						"chunk_data": {
							"type": "string",
							"description": "For append: Base64-encoded chunk data (max 50KB after decoding). Required for append operation only."
						}
					},
					"required": ["operation"]
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

		// Log tool calls with intelligent truncation for data-bearing tools
		// Note: DeployCellDataChunked logging happens in individual operation handlers after validation
		match tool_name {
			"DeployCellData" => {
				if let Some(data) = arguments.get("data").and_then(|v| v.as_str()) {
					let size_bytes = data.len() / 2; // Hex encoding: 2 chars per byte
					info!("Calling tool: DeployCellData with {} bytes of data", size_bytes);
				} else {
					info!("Calling tool: DeployCellData");
				}
			}
			"DeployCellDataChunked" => {
				// Don't log here - logging happens after validation in each operation handler
			}
			_ => {
				// For other tools with small arguments, log normally
				info!("Calling tool: {} with arguments: {}", tool_name, arguments);
			}
		}

		let result = match tool_name {
			"DeployCellData" => self.call_deploy_cell_data(arguments).await,
			"GetAddressBalance" => self.call_get_address_balance(arguments).await,
			"GetChainType" => self.call_get_chain_type().await,
			"GetGenesisHash" => self.call_get_genesis_hash().await,
			"GenerateLockInfo" => self.call_generate_lock_info(arguments),
			"GetLockInfoFromAddress" => self.call_get_lock_info_from_address(arguments),
			"RequestTestnetFunds" => self.call_request_testnet_funds(arguments).await,
			"GetDefaultAccountInfo" => self.call_get_default_account_info().await,
			"DeployCellDataChunked" => {
				// Dispatch based on operation parameter
				let operation = match arguments.get("operation").and_then(|v| v.as_str()) {
					Some(op) => op,
					None => {
						return Ok(create_error_response(
							id,
							-32602,
							"Missing required parameter 'operation'. Must be one of: initialize, append, status, finalize, deploy, cancel. \
							Example: {\"operation\": \"initialize\", \"expected_size\": 102400}".to_string(),
						))
					}
				};

				match operation {
					"initialize" => self.call_initialize_chunked(arguments).await,
					"append" => self.call_append_chunk(arguments).await,
					"status" => self.call_status_chunked(arguments).await,
					"finalize" => self.call_finalize_chunked(arguments).await,
					"deploy" => self.call_deploy_chunked(arguments).await,
					"cancel" => self.call_cancel_chunked(arguments).await,
					_ => {
						return Ok(create_error_response(
							id,
							-32602,
							format!(
								"Unknown operation '{}'. Valid operations: initialize, append, status, finalize, deploy, cancel. \
								See tool description for examples of each operation.",
								operation
							),
						))
					}
				}
			}
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

		info!("Deployed {} bytes to tx {}, output index {}",
			result.data_size, result.tx_hash, result.output_index);

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

		let result = self.tools_provider.request_testnet_funds(address.clone()).await?;

		if let Some(addr) = address {
			info!("Testnet funds requested for address: {}", addr);
		} else {
			info!("Testnet funds requested for default address");
		}

		Ok(result)
	}

	async fn call_get_default_account_info(&self) -> Result<String> {
		let result = self.tools_provider.get_default_account_info().await?;

		Ok(serde_json::to_string_pretty(&result)?)
	}

	// Chunked upload handlers

	async fn call_initialize_chunked(&self, args: &Value) -> Result<String> {
		let expected_size = args
			.get("expected_size")
			.and_then(|v| v.as_u64())
			.ok_or_else(|| {
				shared::error::CkbMcpError::InvalidParameter("Missing or invalid expected_size parameter".to_string())
			})? as usize;

		let metadata = SessionManager::create_session(expected_size)?;

		info!("Initialized chunked upload session {} expecting {} bytes",
			metadata.session_key, expected_size);

		Ok(json!({
			"session_key": metadata.session_key,
			"expected_size": metadata.expected_size,
			"status": "initialized"
		}).to_string())
	}

	async fn call_append_chunk(&self, args: &Value) -> Result<String> {
		let session_key = args
			.get("session_key")
			.and_then(|v| v.as_str())
			.ok_or_else(|| {
				shared::error::CkbMcpError::InvalidParameter(
					"Missing required parameter 'session_key' for append operation. \
					Example: {\"operation\": \"append\", \"session_key\": \"uuid-from-initialize\", \"chunk_data\": \"base64...\"}".to_string()
				)
			})?
			.to_string();

		let chunk_data_b64 = args
			.get("chunk_data")
			.and_then(|v| v.as_str())
			.ok_or_else(|| {
				shared::error::CkbMcpError::InvalidParameter(
					"Missing required parameter 'chunk_data' for append operation. Must be base64-encoded data (max 50KB after decoding). \
					Example: {\"operation\": \"append\", \"session_key\": \"...\", \"chunk_data\": \"SGVsbG8gd29ybGQ=\"}".to_string()
				)
			})?;

		// Decode base64 chunk data
		let chunk_data = base64::decode(chunk_data_b64).map_err(|e| {
			shared::error::CkbMcpError::InvalidParameter(format!("Invalid base64 chunk_data: {}", e))
		})?;

		info!("Appending {} bytes to session {}", chunk_data.len(), session_key);

		let metadata = SessionManager::append_data(&session_key, &chunk_data)?;

		Ok(json!({
			"session_key": metadata.session_key,
			"bytes_received": metadata.total_bytes,
			"bytes_remaining": metadata.expected_size - metadata.total_bytes,
			"status": "receiving"
		}).to_string())
	}

	async fn call_status_chunked(&self, args: &Value) -> Result<String> {
		let session_key = args
			.get("session_key")
			.and_then(|v| v.as_str())
			.ok_or_else(|| {
				shared::error::CkbMcpError::InvalidParameter(
					"Missing required parameter 'session_key' for status operation. \
					Example: {\"operation\": \"status\", \"session_key\": \"uuid-from-initialize\"}".to_string()
				)
			})?
			.to_string();

		info!("Querying status for session {}", session_key);

		let metadata = SessionManager::read_metadata_unlocked(&session_key)?;

		let mut response = json!({
			"session_key": metadata.session_key,
			"state": metadata.state,
			"expected_size": metadata.expected_size,
			"total_bytes": metadata.total_bytes
		});

		// Add error message if in error state
		if let Some(error_msg) = metadata.error_message {
			response["error_message"] = json!(error_msg);
			response["hint"] = json!("Session is in error state. Only cancel operation is allowed.");
		}

		// Add hashes if finalized
		if metadata.state == SessionState::Finalized || metadata.state == SessionState::Deployed {
			if let Some(sha256) = metadata.sha256_hash {
				response["sha256_hash"] = json!(sha256);
			}
			if let Some(blake2b) = metadata.blake2b_hash {
				response["blake2b_hash"] = json!(blake2b);
			}
			if let Some(ckb_hash) = metadata.ckb_hash {
				response["ckb_hash"] = json!(ckb_hash);
			}
		}

		Ok(serde_json::to_string_pretty(&response)?)
	}

	async fn call_finalize_chunked(&self, args: &Value) -> Result<String> {
		let session_key = args
			.get("session_key")
			.and_then(|v| v.as_str())
			.ok_or_else(|| {
				shared::error::CkbMcpError::InvalidParameter(
					"Missing required parameter 'session_key' for finalize operation. \
					Example: {\"operation\": \"finalize\", \"session_key\": \"uuid-from-initialize\"}".to_string()
				)
			})?
			.to_string();

		info!("Finalizing session {}", session_key);

		let (mut metadata, _lock) = SessionManager::read_metadata_locked(&session_key)?;

		// Check state
		if metadata.state != SessionState::Receiving {
			return Err(shared::error::CkbMcpError::InvalidParameter(format!(
				"Cannot finalize session in {:?} state. Current state must be 'receiving'. \
				Use status operation to check current state.",
				metadata.state
			)));
		}

		// Validate size match
		if metadata.total_bytes != metadata.expected_size {
			let error_msg = format!(
				"Size mismatch: received {} bytes, expected {} bytes",
				metadata.total_bytes,
				metadata.expected_size
			);
			SessionManager::set_error_state(&session_key, &error_msg)?;
			return Err(shared::error::CkbMcpError::InvalidParameter(error_msg));
		}

		// Read data
		let data = SessionManager::read_data(&session_key)?;

		// Verify data length matches metadata (detects corruption from crashes/partial writes)
		if data.len() != metadata.total_bytes {
			let error_msg = format!(
				"Data corruption detected: file contains {} bytes but metadata claims {} bytes",
				data.len(),
				metadata.total_bytes
			);
			SessionManager::set_error_state(&session_key, &error_msg)?;
			return Err(shared::error::CkbMcpError::Internal(error_msg));
		}

		// Compute SHA-256
		let mut sha256_hasher = Sha256::new();
		sha256_hasher.update(&data);
		let sha256_hash = format!("0x{}", hex::encode(sha256_hasher.finalize()));

		// Compute Blake2b-256 (no personalization) - using blake2 crate directly
		let mut blake2b_hasher = Blake2b::<U32>::new();
		blake2b_hasher.update(&data);
		let blake2b_hash = format!("0x{}", hex::encode(blake2b_hasher.finalize()));

		// Compute CKB hash (Blake2b-256 with "ckb-default-hash" personalization)
		let ckb_hash_bytes = ckb_hash::blake2b_256(&data);
		let ckb_hash = format!("0x{}", hex::encode(ckb_hash_bytes));

		// Update metadata
		metadata.state = SessionState::Finalized;
		metadata.sha256_hash = Some(sha256_hash.clone());
		metadata.blake2b_hash = Some(blake2b_hash.clone());
		metadata.ckb_hash = Some(ckb_hash.clone());

		SessionManager::update_metadata(&session_key, &metadata)?;

		info!("Finalized session {} with {} bytes", session_key, metadata.total_bytes);

		Ok(json!({
			"session_key": session_key,
			"state": "finalized",
			"total_bytes": metadata.total_bytes,
			"sha256_hash": sha256_hash,
			"blake2b_hash": blake2b_hash,
			"ckb_hash": ckb_hash
		}).to_string())
	}

	async fn call_deploy_chunked(&self, args: &Value) -> Result<String> {
		let session_key = args
			.get("session_key")
			.and_then(|v| v.as_str())
			.ok_or_else(|| {
				shared::error::CkbMcpError::InvalidParameter(
					"Missing required parameter 'session_key' for deploy operation. \
					Example: {\"operation\": \"deploy\", \"session_key\": \"uuid-from-initialize\"}".to_string()
				)
			})?
			.to_string();

		info!("Deploying session {} to blockchain", session_key);

		let (mut metadata, _lock) = SessionManager::read_metadata_locked(&session_key)?;

		// Check state
		if metadata.state != SessionState::Finalized {
			return Err(shared::error::CkbMcpError::InvalidParameter(format!(
				"Cannot deploy session in {:?} state. Session must be finalized first. \
				Use finalize operation before deploying.",
				metadata.state
			)));
		}

		// Read data
		let data = SessionManager::read_data(&session_key)?;

		// Attempt deploy
		match self.tools_provider.deploy_cell_data(data).await {
			Ok(result) => {
				// Success - update state to deployed
				metadata.state = SessionState::Deployed;
				SessionManager::update_metadata(&session_key, &metadata)?;

				info!("Deployed session {} successfully", session_key);

				// Release lock before deleting session (delete_session acquires its own lock)
				drop(_lock);

				// Clean up session files (best effort - don't fail deploy if cleanup fails)
				if let Err(e) = SessionManager::delete_session(&session_key) {
					error!("Failed to clean up session {} after successful deploy: {}", session_key, e);
					// Continue anyway - deploy succeeded, cleanup failure is not critical
				}

				Ok(json!({
					"transaction_hash": result.tx_hash,
					"output_index": result.output_index,
					"data_hash": metadata.ckb_hash,
					"status": "deployed"
				}).to_string())
			}
			Err(e) => {
				// Failure - stay in finalized state, return error
				error!("Deploy failed for session {}: {}", session_key, e);
				Err(shared::error::CkbMcpError::CkbRpc(format!(
					"Deploy failed: {}. Session remains in finalized state. \
					 Fix the issue and retry deploy, or call cancel to delete session.",
					e
				)))
			}
		}
	}

	async fn call_cancel_chunked(&self, args: &Value) -> Result<String> {
		let session_key = args
			.get("session_key")
			.and_then(|v| v.as_str())
			.ok_or_else(|| {
				shared::error::CkbMcpError::InvalidParameter(
					"Missing required parameter 'session_key' for cancel operation. \
					Example: {\"operation\": \"cancel\", \"session_key\": \"uuid-from-initialize\"}".to_string()
				)
			})?
			.to_string();

		info!("Cancelling session {}", session_key);

		SessionManager::delete_session(&session_key)?;

		Ok(json!({
			"session_key": session_key,
			"status": "cancelled"
		}).to_string())
	}
}
