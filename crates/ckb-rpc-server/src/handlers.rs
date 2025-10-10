use shared::{
	error::Result,
	mcp::{create_error_response, create_success_response, McpRequest, McpResponse, ToolDefinition},
};
use serde_json::{json, Value};
use tracing::{debug, error, info};

use crate::rpc::{CkbRpcClient, CkbRpcClientExt};

pub struct McpHandler {
	rpc_client: CkbRpcClient,
}

impl McpHandler {
	pub fn new(rpc_client: CkbRpcClient) -> Self {
		Self { rpc_client }
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
				"name": "ckb-rpc-server",
				"version": "0.1.0"
			}
		});

		Ok(create_success_response(id, result))
	}

	async fn handle_tools_list(&self, id: Option<Value>) -> Result<McpResponse> {
		let tools = vec![
			// Chain Methods
			ToolDefinition {
				name: "get_block".to_string(),
				description: "Get CKB block by hash".to_string(),
				input_schema: json!({
					"type": "object",
					"properties": {
						"block_hash": {
							"type": "string",
							"description": "Block hash"
						}
					},
					"required": ["block_hash"]
				}),
			},
			ToolDefinition {
				name: "get_block_by_number".to_string(),
				description: "Get CKB block by number".to_string(),
				input_schema: json!({
					"type": "object",
					"properties": {
						"block_number": {
							"type": "integer",
							"description": "Block number"
						}
					},
					"required": ["block_number"]
				}),
			},
			ToolDefinition {
				name: "get_header".to_string(),
				description: "Get CKB block header by hash".to_string(),
				input_schema: json!({
					"type": "object",
					"properties": {
						"block_hash": {
							"type": "string",
							"description": "Block hash"
						}
					},
					"required": ["block_hash"]
				}),
			},
			ToolDefinition {
				name: "get_header_by_number".to_string(),
				description: "Get CKB block header by number".to_string(),
				input_schema: json!({
					"type": "object",
					"properties": {
						"block_number": {
							"type": "integer",
							"description": "Block number"
						}
					},
					"required": ["block_number"]
				}),
			},
			ToolDefinition {
				name: "get_transaction".to_string(),
				description: "Get CKB transaction by hash".to_string(),
				input_schema: json!({
					"type": "object",
					"properties": {
						"tx_hash": {
							"type": "string",
							"description": "Transaction hash"
						}
					},
					"required": ["tx_hash"]
				}),
			},
			ToolDefinition {
				name: "get_block_hash".to_string(),
				description: "Get block hash by number".to_string(),
				input_schema: json!({
					"type": "object",
					"properties": {
						"block_number": {
							"type": "integer",
							"description": "Block number"
						}
					},
					"required": ["block_number"]
				}),
			},
			ToolDefinition {
				name: "get_tip_header".to_string(),
				description: "Get tip block header".to_string(),
				input_schema: json!({
					"type": "object",
					"properties": {}
				}),
			},
			ToolDefinition {
				name: "get_live_cell".to_string(),
				description: "Get live cell by outpoint".to_string(),
				input_schema: json!({
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
			},
			ToolDefinition {
				name: "get_tip_block_number".to_string(),
				description: "Get tip block number".to_string(),
				input_schema: json!({
					"type": "object",
					"properties": {}
				}),
			},
			ToolDefinition {
				name: "get_current_epoch".to_string(),
				description: "Get current epoch information".to_string(),
				input_schema: json!({
					"type": "object",
					"properties": {}
				}),
			},
			ToolDefinition {
				name: "get_epoch_by_number".to_string(),
				description: "Get epoch by number".to_string(),
				input_schema: json!({
					"type": "object",
					"properties": {
						"epoch_number": {
							"type": "integer",
							"description": "Epoch number"
						}
					},
					"required": ["epoch_number"]
				}),
			},
			// Indexer Methods
			ToolDefinition {
				name: "get_indexer_tip".to_string(),
				description: "Get indexer tip".to_string(),
				input_schema: json!({
					"type": "object",
					"properties": {}
				}),
			},
			ToolDefinition {
				name: "get_cells".to_string(),
				description: "Search for cells by criteria".to_string(),
				input_schema: json!({
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
						}
					},
					"required": ["search_key"]
				}),
			},
			ToolDefinition {
				name: "get_transactions".to_string(),
				description: "Search for transactions by criteria".to_string(),
				input_schema: json!({
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
							"description": "Group results by transaction hash (default: false returns individual entries with io_type/io_index)",
							"default": false
						}
					},
					"required": ["search_key"]
				}),
			},
			ToolDefinition {
				name: "get_cells_capacity".to_string(),
				description: "Get total capacity of cells by search criteria".to_string(),
				input_schema: json!({
					"type": "object",
					"properties": {
						"search_key": {
							"type": "object",
							"description": "Search criteria"
						}
					},
					"required": ["search_key"]
				}),
			},
			// Network Methods
			ToolDefinition {
				name: "local_node_info".to_string(),
				description: "Get local node information".to_string(),
				input_schema: json!({
					"type": "object",
					"properties": {}
				}),
			},
			// Chain Methods - Advanced
			ToolDefinition {
				name: "estimate_cycles".to_string(),
				description: "Estimate transaction execution cycles".to_string(),
				input_schema: json!({
					"type": "object",
					"properties": {
						"tx": {
							"type": "object",
							"description": "Transaction object to estimate"
						}
					},
					"required": ["tx"]
				}),
			},
			// Pool Methods
			ToolDefinition {
				name: "send_transaction".to_string(),
				description: "Submit transaction to the network".to_string(),
				input_schema: json!({
					"type": "object",
					"properties": {
						"tx": {
							"type": "object",
							"description": "Transaction object to send"
						},
						"outputs_validator": {
							"type": "string",
							"description": "Outputs validator mode (optional, default: passthrough)",
							"enum": ["passthrough", "well_known_scripts_only"],
							"default": "passthrough"
						}
					},
					"required": ["tx"]
				}),
			},
			ToolDefinition {
				name: "test_tx_pool_accept".to_string(),
				description: "Test if transaction would be accepted without broadcasting".to_string(),
				input_schema: json!({
					"type": "object",
					"properties": {
						"tx": {
							"type": "object",
							"description": "Transaction object to test"
						},
						"outputs_validator": {
							"type": "string",
							"description": "Outputs validator mode (optional, default: passthrough)",
							"enum": ["passthrough", "well_known_scripts_only"],
							"default": "passthrough"
						}
					},
					"required": ["tx"]
				}),
			},
			// Stats Methods
			ToolDefinition {
				name: "get_blockchain_info".to_string(),
				description: "Get blockchain information including chain type, difficulty, and epoch".to_string(),
				input_schema: json!({
					"type": "object",
					"properties": {}
				}),
			},
			ToolDefinition {
				name: "get_consensus".to_string(),
				description: "Get consensus parameters including genesis hash, epoch duration, and hard fork features".to_string(),
				input_schema: json!({
					"type": "object",
					"properties": {}
				}),
			},
			ToolDefinition {
				name: "tx_pool_info".to_string(),
				description: "Get transaction pool information including pending count, size limits, and fee rates".to_string(),
				input_schema: json!({
					"type": "object",
					"properties": {}
				}),
			},
			ToolDefinition {
				name: "get_raw_tx_pool".to_string(),
				description: "Get all transaction ids in tx pool (verbose=false) or detailed info per transaction (verbose=true)".to_string(),
				input_schema: json!({
					"type": "object",
					"properties": {
						"verbose": {
							"type": "boolean",
							"description": "True for detailed json object with tx info, false for array of tx ids (default: false)"
						}
					}
				}),
			},
			ToolDefinition {
				name: "get_pool_tx_detail_info".to_string(),
				description: "Get detailed information about a specific transaction in the pool (for troubleshooting)".to_string(),
				input_schema: json!({
					"type": "object",
					"properties": {
						"tx_hash": {
							"type": "string",
							"description": "Hash of transaction to query"
						}
					},
					"required": ["tx_hash"]
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
			// Chain Methods
			"get_block" => self.call_get_block(arguments).await,
			"get_block_by_number" => self.call_get_block_by_number(arguments).await,
			"get_header" => self.call_get_header(arguments).await,
			"get_header_by_number" => self.call_get_header_by_number(arguments).await,
			"get_transaction" => self.call_get_transaction(arguments).await,
			"get_block_hash" => self.call_get_block_hash(arguments).await,
			"get_tip_header" => self.call_get_tip_header().await,
			"get_live_cell" => self.call_get_live_cell(arguments).await,
			"get_tip_block_number" => self.call_get_tip_block_number().await,
			"get_current_epoch" => self.call_get_current_epoch().await,
			"get_epoch_by_number" => self.call_get_epoch_by_number(arguments).await,
			// Indexer Methods
			"get_indexer_tip" => self.call_get_indexer_tip().await,
			"get_cells" => self.call_get_cells(arguments).await,
			"get_transactions" => self.call_get_transactions(arguments).await,
			"get_cells_capacity" => self.call_get_cells_capacity(arguments).await,
			// Network Methods
			"local_node_info" => self.call_local_node_info().await,
			// Chain Methods - Advanced
			"estimate_cycles" => self.call_estimate_cycles(arguments).await,
			// Pool Methods
			"send_transaction" => self.call_send_transaction(arguments).await,
			"test_tx_pool_accept" => self.call_test_tx_pool_accept(arguments).await,
			"get_raw_tx_pool" => self.call_get_raw_tx_pool(arguments).await,
			"get_pool_tx_detail_info" => self.call_get_pool_tx_detail_info(arguments).await,
			// Stats Methods
			"get_blockchain_info" => self.call_get_blockchain_info().await,
			"get_consensus" => self.call_get_consensus().await,
			"tx_pool_info" => self.call_tx_pool_info().await,
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
						"text": serde_json::to_string_pretty(&data)?
					}]
				}),
			)),
			Err(e) => {
				error!("Tool call failed: {}", e);
				Ok(create_error_response(id, -32603, e.to_string()))
			}
		}
	}

	// Chain Method Handlers
	async fn call_get_block(&self, args: &Value) -> Result<Value> {
		let block_hash = args
			.get("block_hash")
			.and_then(|v| v.as_str())
			.ok_or_else(|| {
				shared::error::CkbMcpError::InvalidParameter("Missing block_hash".to_string())
			})?;
		self.rpc_client.get_block(block_hash).await
	}

	async fn call_get_block_by_number(&self, args: &Value) -> Result<Value> {
		let block_number = args
			.get("block_number")
			.and_then(|v| v.as_u64())
			.ok_or_else(|| {
				shared::error::CkbMcpError::InvalidParameter("Missing block_number".to_string())
			})?;
		self.rpc_client.get_block_by_number(block_number).await
	}

	async fn call_get_header(&self, args: &Value) -> Result<Value> {
		let block_hash = args
			.get("block_hash")
			.and_then(|v| v.as_str())
			.ok_or_else(|| {
				shared::error::CkbMcpError::InvalidParameter("Missing block_hash".to_string())
			})?;
		self.rpc_client.get_header(block_hash).await
	}

	async fn call_get_header_by_number(&self, args: &Value) -> Result<Value> {
		let block_number = args
			.get("block_number")
			.and_then(|v| v.as_u64())
			.ok_or_else(|| {
				shared::error::CkbMcpError::InvalidParameter("Missing block_number".to_string())
			})?;
		self.rpc_client.get_header_by_number(block_number).await
	}

	async fn call_get_transaction(&self, args: &Value) -> Result<Value> {
		let tx_hash = args
			.get("tx_hash")
			.and_then(|v| v.as_str())
			.ok_or_else(|| {
				shared::error::CkbMcpError::InvalidParameter("Missing tx_hash".to_string())
			})?;
		self.rpc_client.get_transaction(tx_hash).await
	}

	async fn call_get_block_hash(&self, args: &Value) -> Result<Value> {
		let block_number = args
			.get("block_number")
			.and_then(|v| v.as_u64())
			.ok_or_else(|| {
				shared::error::CkbMcpError::InvalidParameter("Missing block_number".to_string())
			})?;
		self.rpc_client.get_block_hash(block_number).await
	}

	async fn call_get_tip_header(&self) -> Result<Value> {
		self.rpc_client.get_tip_header().await
	}

	async fn call_get_live_cell(&self, args: &Value) -> Result<Value> {
		let tx_hash = args
			.get("tx_hash")
			.and_then(|v| v.as_str())
			.ok_or_else(|| {
				shared::error::CkbMcpError::InvalidParameter("Missing tx_hash".to_string())
			})?;

		let index = args
			.get("index")
			.and_then(|v| v.as_u64())
			.ok_or_else(|| {
				shared::error::CkbMcpError::InvalidParameter("Missing or invalid index".to_string())
			})? as u32;

		let with_data = args.get("with_data").and_then(|v| v.as_bool()).unwrap_or(false);

		self.rpc_client.get_live_cell(tx_hash, index, with_data).await
	}

	async fn call_get_tip_block_number(&self) -> Result<Value> {
		self.rpc_client.get_tip_block_number().await
	}

	async fn call_get_current_epoch(&self) -> Result<Value> {
		self.rpc_client.get_current_epoch().await
	}

	async fn call_get_epoch_by_number(&self, args: &Value) -> Result<Value> {
		let epoch_number = args
			.get("epoch_number")
			.and_then(|v| v.as_u64())
			.ok_or_else(|| {
				shared::error::CkbMcpError::InvalidParameter("Missing epoch_number".to_string())
			})?;
		self.rpc_client.get_epoch_by_number(epoch_number).await
	}

	// Indexer Method Handlers
	async fn call_get_indexer_tip(&self) -> Result<Value> {
		self.rpc_client.get_indexer_tip().await
	}

	async fn call_get_cells(&self, args: &Value) -> Result<Value> {
		let search_key = args
			.get("search_key")
			.ok_or_else(|| {
				shared::error::CkbMcpError::InvalidParameter("Missing search_key".to_string())
			})?
			.clone();

		let order = args
			.get("order")
			.and_then(|v| v.as_str())
			.unwrap_or("asc");

		let limit = args.get("limit").and_then(|v| v.as_u64()).map(|l| l as u32);
		let after_cursor = args.get("after_cursor").and_then(|v| v.as_str());

		self.rpc_client.get_cells(search_key, order, limit, after_cursor).await
	}

	async fn call_get_transactions(&self, args: &Value) -> Result<Value> {
		let mut search_key = args
			.get("search_key")
			.ok_or_else(|| {
				shared::error::CkbMcpError::InvalidParameter("Missing search_key".to_string())
			})?
			.clone();

		let order = args
			.get("order")
			.and_then(|v| v.as_str())
			.unwrap_or("asc");

		let limit = args.get("limit").and_then(|v| v.as_u64()).map(|l| l as u32);
		let after_cursor = args.get("after_cursor").and_then(|v| v.as_str());
		let group_by_transaction = args.get("group_by_transaction").and_then(|v| v.as_bool()).unwrap_or(false);

		// Add group_by_transaction to search_key if specified
		if group_by_transaction {
			if let Some(obj) = search_key.as_object_mut() {
				obj.insert("group_by_transaction".to_string(), json!(true));
			}
		}

		self.rpc_client.get_transactions(search_key, order, limit, after_cursor).await
	}

	async fn call_get_cells_capacity(&self, args: &Value) -> Result<Value> {
		let search_key = args
			.get("search_key")
			.ok_or_else(|| {
				shared::error::CkbMcpError::InvalidParameter("Missing search_key".to_string())
			})?
			.clone();

		self.rpc_client.get_cells_capacity(search_key).await
	}

	// Network Method Handlers
	async fn call_local_node_info(&self) -> Result<Value> {
		self.rpc_client.local_node_info().await
	}

	// Chain Method Handlers - Advanced
	async fn call_estimate_cycles(&self, args: &Value) -> Result<Value> {
		let tx = args
			.get("tx")
			.ok_or_else(|| {
				shared::error::CkbMcpError::InvalidParameter("Missing tx".to_string())
			})?
			.clone();
		self.rpc_client.estimate_cycles(tx).await
	}

	// Pool Method Handlers
	async fn call_send_transaction(&self, args: &Value) -> Result<Value> {
		let tx = args
			.get("tx")
			.ok_or_else(|| {
				shared::error::CkbMcpError::InvalidParameter("Missing tx".to_string())
			})?
			.clone();

		let outputs_validator = args.get("outputs_validator").and_then(|v| v.as_str());

		self.rpc_client.send_transaction(tx, outputs_validator).await
	}

	async fn call_test_tx_pool_accept(&self, args: &Value) -> Result<Value> {
		let tx = args
			.get("tx")
			.ok_or_else(|| {
				shared::error::CkbMcpError::InvalidParameter("Missing tx".to_string())
			})?
			.clone();

		let outputs_validator = args.get("outputs_validator").and_then(|v| v.as_str());

		self.rpc_client.test_tx_pool_accept(tx, outputs_validator).await
	}

	// Stats Method Handlers
	async fn call_get_blockchain_info(&self) -> Result<Value> {
		self.rpc_client.get_blockchain_info().await
	}

	async fn call_get_consensus(&self) -> Result<Value> {
		self.rpc_client.get_consensus().await
	}

	async fn call_tx_pool_info(&self) -> Result<Value> {
		self.rpc_client.tx_pool_info().await
	}

	async fn call_get_raw_tx_pool(&self, args: &Value) -> Result<Value> {
		let verbose = args.get("verbose").and_then(|v| v.as_bool());
		self.rpc_client.get_raw_tx_pool(verbose).await
	}

	async fn call_get_pool_tx_detail_info(&self, args: &Value) -> Result<Value> {
		let tx_hash = args
			.get("tx_hash")
			.and_then(|v| v.as_str())
			.ok_or_else(|| {
				shared::error::CkbMcpError::InvalidParameter("Missing tx_hash".to_string())
			})?;

		self.rpc_client.get_pool_tx_detail_info(tx_hash).await
	}
}