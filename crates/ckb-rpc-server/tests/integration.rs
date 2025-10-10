use serde_json::{json, Value};

#[path = "../../shared/tests/common/mod.rs"]
mod common;

use common::{SharedTestData, TestContext};

const RPC_SERVER_PORT: u16 = 8001;

/// Phase 1: Verify MCP server is running
#[tokio::test]
async fn test_00_server_running() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	ctx.verify_server_running()
		.await
		.expect("ckb-rpc-server must be running on port 8001. Start with: cargo run --bin ckb-rpc-server");
}

/// Phase 2: Verify CKB RPC is available (direct connection, not through MCP)
#[tokio::test]
async fn test_01_ckb_rpc_available() {
	use reqwest::Client;

	let ckb_rpc_url = TestContext::get_ckb_rpc_url()
		.expect("CKB_RPC_URL must be set");

	let client = Client::new();
	let response = client
		.post(&ckb_rpc_url)
		.json(&json!({
			"jsonrpc": "2.0",
			"id": 1,
			"method": "get_tip_block_number",
			"params": []
		}))
		.send()
		.await
		.expect("CKB RPC should be accessible");

	let body: serde_json::Value = response.json().await.expect("Should parse JSON response");

	assert!(body.get("error").is_none(), "CKB RPC should not return error");
	assert!(body.get("result").is_some(), "CKB RPC should return result");
}

/// Phase 3: Collect shared test data from CKB RPC (not through MCP)
#[tokio::test]
async fn test_02_collect_shared_data() {
	SharedTestData::initialize()
		.await
		.expect("Should successfully collect shared test data from CKB RPC");

	let data = SharedTestData::get().expect("Shared data should be initialized");

	// Verify data was collected correctly
	assert!(!data.chain_type.is_empty(), "Chain type should not be empty");
	assert!(data.genesis_hash.starts_with("0x"), "Genesis hash should be hex format");
	assert!(data.genesis_block.get("header").is_some(), "Genesis block should have header");
}

#[tokio::test]
async fn test_03_mcp_initialize() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call(
			"initialize",
			json!({
				"protocolVersion": "2024-11-05",
				"capabilities": {},
				"clientInfo": {
					"name": "test-client",
					"version": "1.0.0"
				}
			}),
		)
		.await
		.expect("initialize should succeed");

	assert_eq!(result["protocolVersion"], "2024-11-05");
	assert!(result["serverInfo"]["name"]
		.as_str()
		.unwrap()
		.contains("ckb-rpc"));
	assert!(result["capabilities"]["tools"].is_object());
}

// Chain Methods - Success Cases
#[tokio::test]
async fn test_get_tip_block_number() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_tip_block_number", "arguments": {}}))
		.await
		.expect("get_tip_block_number should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
	assert!(content.contains("0x"), "Should return hex number");
}

#[tokio::test]
async fn test_get_tip_header() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_tip_header", "arguments": {}}))
		.await
		.expect("get_tip_header should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
	assert!(content.contains("hash"), "Should contain hash field");
}

#[tokio::test]
async fn test_get_current_epoch() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_current_epoch", "arguments": {}}))
		.await
		.expect("get_current_epoch should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
	assert!(content.contains("number"), "Should contain epoch number");
}

#[tokio::test]
async fn test_get_block_by_number_genesis() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_block_by_number", "arguments": {"block_number": 0}}))
		.await
		.expect("get_block_by_number genesis should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
	assert!(content.contains("header"), "Should contain header");
}

#[tokio::test]
async fn test_get_block_by_number_recent() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	// Get a recent block (block 1 is always safe)
	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_block_by_number", "arguments": {"block_number": 1}}))
		.await
		.expect("get_block_by_number recent should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
}

#[tokio::test]
async fn test_get_header_by_number() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_header_by_number", "arguments": {"block_number": 0}}))
		.await
		.expect("get_header_by_number should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
	assert!(content.contains("hash"));
}

#[tokio::test]
async fn test_get_block_hash() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_block_hash", "arguments": {"block_number": 0}}))
		.await
		.expect("get_block_hash should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
	assert!(content.starts_with("\"0x"), "Should return hex hash");
}

#[tokio::test]
async fn test_get_block_with_hash() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	// First get a valid block hash
	let hash_result = ctx
		.mcp_call("tools/call", json!({"name": "get_block_hash", "arguments": {"block_number": 0}}))
		.await
		.expect("get_block_hash should succeed");

	let hash_content = hash_result["content"][0]["text"].as_str().unwrap();
	let hash: String = serde_json::from_str(hash_content).unwrap();

	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_block", "arguments": {"block_hash": hash}}))
		.await
		.expect("get_block should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
	assert!(content.contains("header"));
}

#[tokio::test]
async fn test_get_header_with_hash() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let hash_result = ctx
		.mcp_call("tools/call", json!({"name": "get_block_hash", "arguments": {"block_number": 0}}))
		.await
		.expect("get_block_hash should succeed");

	let hash_content = hash_result["content"][0]["text"].as_str().unwrap();
	let hash: String = serde_json::from_str(hash_content).unwrap();

	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_header", "arguments": {"block_hash": hash}}))
		.await
		.expect("get_header should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
}

#[tokio::test]
async fn test_get_epoch_by_number() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_epoch_by_number", "arguments": {"epoch_number": 0}}))
		.await
		.expect("get_epoch_by_number should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
}

// Chain Methods - Error Cases
#[tokio::test]
async fn test_get_block_invalid_hash_format() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_block", "arguments": {"block_hash": "invalid"}}))
		.await;

	assert!(result.is_err(), "Should fail for invalid hash format");
}

#[tokio::test]
async fn test_get_block_nonexistent_hash() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_block", "arguments": {"block_hash": "0x0000000000000000000000000000000000000000000000000000000000000000"}}))
		.await
		.expect("get_block should succeed even for nonexistent hash");

	// Should return null for nonexistent block
	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(content.contains("null"), "Nonexistent block should return null");
}

#[tokio::test]
async fn test_get_block_by_number_beyond_tip() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_block_by_number", "arguments": {"block_number": 99999999}}))
		.await
		.expect("get_block_by_number should succeed even for beyond tip");

	// Should return null for beyond tip
	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(content.contains("null"), "Block beyond tip should return null");
}

#[tokio::test]
async fn test_get_header_invalid_hash() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_header", "arguments": {"block_hash": "not_a_hash"}}))
		.await;

	assert!(result.is_err(), "Should fail for invalid hash");
}

#[tokio::test]
async fn test_get_epoch_by_number_future() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_epoch_by_number", "arguments": {"epoch_number": 999999}}))
		.await
		.expect("get_epoch_by_number should succeed even for future epoch");

	// Should return null for future epoch
	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(content.contains("null"), "Future epoch should return null");
}

#[tokio::test]
async fn test_get_header_by_number_beyond_tip() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_header_by_number", "arguments": {"block_number": 99999999}}))
		.await
		.expect("get_header_by_number should succeed even for beyond tip");

	// Should return null for beyond tip
	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(content.contains("null"), "Header beyond tip should return null");
}

// Live Cell Methods
#[tokio::test]
async fn test_get_live_cell_missing_index() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_live_cell", "arguments": {"tx_hash": "0x0000000000000000000000000000000000000000000000000000000000000000"}}))
		.await;

	assert!(result.is_err(), "Should fail when index is missing");
}

// Indexer Methods - Success
#[tokio::test]
async fn test_get_indexer_tip() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_indexer_tip", "arguments": {}}))
		.await
		.expect("get_indexer_tip should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
	assert!(content.contains("block_hash") || content.contains("block_number"));
}

#[tokio::test]
async fn test_get_cells_basic_search() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_cells",
			"arguments": {
				"search_key": {
					"script": {
						"code_hash": "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
						"hash_type": "type",
						"args": "0x"
					},
					"script_type": "lock"
				},
				"order": "asc",
				"limit": 10
			}
		}))
		.await
		.expect("get_cells should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
}

#[tokio::test]
async fn test_get_cells_capacity() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	// Use shared genesis block data (collected via direct CKB RPC in Phase 3)
	let shared_data = SharedTestData::get_or_init_async().await;
	let lock_script = &shared_data.genesis_block["transactions"][0]["outputs"][0]["lock"];

	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_cells_capacity",
			"arguments": {
				"search_key": {
					"script": {
						"code_hash": lock_script["code_hash"],
						"hash_type": lock_script["hash_type"],
						"args": lock_script["args"]
					},
					"script_type": "lock"
				}
			}
		}))
		.await
		.expect("get_cells_capacity should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
	assert!(content.contains("capacity"), "Response should contain capacity field");
}

// Indexer Methods - Error Cases
#[tokio::test]
async fn test_get_cells_missing_search_key() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_cells", "arguments": {}}))
		.await;

	assert!(result.is_err(), "Should fail when search_key is missing");
}

#[tokio::test]
async fn test_get_cells_capacity_missing_search_key() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_cells_capacity", "arguments": {}}))
		.await;

	assert!(result.is_err(), "Should fail when search_key is missing");
}

// Network Methods
#[tokio::test]
async fn test_local_node_info() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "local_node_info", "arguments": {}}))
		.await
		.expect("local_node_info should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();

	// Parse JSON to validate structure
	let node_info: serde_json::Value = serde_json::from_str(content)
		.expect("Response should be valid JSON");

	// Verify ALL required fields exist (not just one)
	assert!(node_info.get("version").is_some(), "Should contain version field");
	assert!(node_info.get("node_id").is_some(), "Should contain node_id field");
	assert!(node_info.get("addresses").is_some(), "Should contain addresses field");

	// Validate field types and formats
	let version = node_info["version"].as_str()
		.expect("version should be a string");
	assert!(!version.is_empty(), "version should not be empty");

	let node_id = node_info["node_id"].as_str()
		.expect("node_id should be a string");
	assert!(!node_id.is_empty(), "node_id should not be empty");

	let _addresses = node_info["addresses"].as_array()
		.expect("addresses should be an array");
	// Note: addresses could be empty if node has no peers, so we just check it's an array

	// Validate connections is a hex number string if present
	if let Some(connections) = node_info.get("connections") {
		let conn_str = connections.as_str().expect("connections should be a string");
		assert!(conn_str.starts_with("0x"), "connections should be hex format");
	}
}

// General Error Cases
#[tokio::test]
async fn test_unknown_tool_name() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "nonexistent_tool", "arguments": {}}))
		.await;

	assert!(result.is_err(), "Should fail for unknown tool");
}

#[tokio::test]
async fn test_missing_tool_name() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"arguments": {}}))
		.await;

	assert!(result.is_err(), "Should fail when tool name is missing");
}

#[tokio::test]
async fn test_missing_params() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({}))
		.await;

	assert!(result.is_err(), "Should fail when params are missing");
}

#[tokio::test]
async fn test_invalid_json_rpc_request() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"invalid": "structure"}))
		.await;

	assert!(result.is_err(), "Should fail for malformed JSON-RPC request");
}

// Additional Chain Method Tests
#[tokio::test]
async fn test_get_transaction_valid() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	// Use shared genesis block data (collected via direct CKB RPC in Phase 3)
	let shared_data = SharedTestData::get_or_init_async().await;
	let tx_hash = shared_data.genesis_block["transactions"][0]["hash"]
		.as_str()
		.expect("Genesis block should have at least one transaction with a hash");

	// Now query for this transaction
	let tx_result = ctx
		.mcp_call("tools/call", json!({"name": "get_transaction", "arguments": {"tx_hash": tx_hash}}))
		.await
		.expect("get_transaction should succeed for genesis transaction");

	let tx_content = tx_result["content"][0]["text"].as_str().unwrap();

	// Validate the response contains transaction details
	assert!(tx_content.contains("transaction"), "Response should contain transaction details");
	assert!(tx_content.contains(tx_hash), "Response should contain the queried transaction hash");
}

#[tokio::test]
async fn test_get_transaction_nonexistent() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_transaction", "arguments": {"tx_hash": "0x0000000000000000000000000000000000000000000000000000000000000000"}}))
		.await
		.expect("get_transaction should succeed even for nonexistent transaction");

	// Should return null for nonexistent transaction
	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(content.contains("null"), "Nonexistent transaction should return null");
}

#[tokio::test]
async fn test_get_block_hash_negative_number() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	// JSON numeric -1 will be deserialized. CKB RPC should handle gracefully.
	// Either returns null (treating as non-existent block) or returns an error.
	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_block_hash", "arguments": {"block_number": -1}}))
		.await;

	match result {
		Ok(res) => {
			// If it succeeds, should return null for invalid block number
			let content = res["content"][0]["text"].as_str().unwrap();
			assert!(content.contains("null"), "Negative block number should return null");
		}
		Err(_) => {
			// If it fails, that's also acceptable behavior for invalid input
			// Either outcome is valid
		}
	}
}

// Live Cell Tests
#[tokio::test]
async fn test_get_live_cell_valid() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	// Use shared genesis block data (collected via direct CKB RPC in Phase 3)
	let shared_data = SharedTestData::get_or_init_async().await;
	let tx_hash = shared_data.genesis_block["transactions"][0]["hash"]
		.as_str()
		.expect("Genesis block should have at least one transaction with a hash");

	// Query for the first output of this transaction
	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_live_cell",
			"arguments": {
				"tx_hash": tx_hash,
				"index": 0,
				"with_data": false
			}
		}))
		.await
		.expect("get_live_cell should succeed for genesis transaction output");

	let content = result["content"][0]["text"].as_str().unwrap();

	// Validate the response contains cell status information
	assert!(content.contains("status"), "Response should contain cell status");
}

#[tokio::test]
async fn test_get_live_cell_with_data() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	// Use shared genesis block data (collected via direct CKB RPC in Phase 3)
	let shared_data = SharedTestData::get_or_init_async().await;
	let tx_hash = shared_data.genesis_block["transactions"][0]["hash"]
		.as_str()
		.expect("Genesis block should have at least one transaction with a hash");

	// Query for the first output of this transaction WITH data
	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_live_cell",
			"arguments": {
				"tx_hash": tx_hash,
				"index": 0,
				"with_data": true
			}
		}))
		.await
		.expect("get_live_cell with_data should succeed for genesis transaction output");

	let content = result["content"][0]["text"].as_str().unwrap();

	// Validate the response contains cell status and data field
	assert!(content.contains("status"), "Response should contain cell status");
	// Note: Not all cells have data, but the field should be present in the response structure
}

#[tokio::test]
async fn test_get_live_cell_invalid_outpoint() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_live_cell",
			"arguments": {
				"tx_hash": "invalid_hash",
				"index": 0
			}
		}))
		.await;

	assert!(result.is_err(), "Should fail for invalid outpoint");
}

#[tokio::test]
async fn test_get_live_cell_spent() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	// Use shared genesis block data (collected via direct CKB RPC in Phase 3)
	let shared_data = SharedTestData::get_or_init_async().await;
	let tx_hash = shared_data.genesis_block["transactions"][0]["hash"]
		.as_str()
		.expect("Genesis block should have at least one transaction with a hash");

	// Query for the first output of this transaction
	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_live_cell",
			"arguments": {
				"tx_hash": tx_hash,
				"index": 0
			}
		}))
		.await
		.expect("get_live_cell should succeed for genesis transaction output");

	let content = result["content"][0]["text"].as_str().unwrap();

	// Validate the response contains status information (live, dead, or unknown)
	assert!(content.contains("status"), "Response should contain cell status information");
}

// Indexer Pagination & Filtering Tests - get_cells
#[tokio::test]
async fn test_get_transactions_basic_search() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_transactions",
			"arguments": {
				"search_key": {
					"script": {
						"code_hash": "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
						"hash_type": "type",
						"args": "0x"
					},
					"script_type": "lock"
				},
				"order": "asc",
				"limit": 10
			}
		}))
		.await
		.expect("get_transactions should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
}

#[tokio::test]
async fn test_get_cells_with_limit() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_cells",
			"arguments": {
				"search_key": {
					"script": {
						"code_hash": "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
						"hash_type": "type",
						"args": "0x"
					},
					"script_type": "lock"
				},
				"limit": 5
			}
		}))
		.await
		.expect("get_cells with limit should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
}

#[tokio::test]
async fn test_get_cells_pagination_order_desc() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	// First page with desc order
	let first_result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_cells",
			"arguments": {
				"search_key": {
					"script": {
						"code_hash": "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
						"hash_type": "type",
						"args": "0x"
					},
					"script_type": "lock"
				},
				"order": "desc",
				"limit": 2
			}
		}))
		.await
		.expect("get_cells with desc order should succeed");

	let first_content = first_result["content"][0]["text"].as_str().unwrap();
	let first_page: serde_json::Value = serde_json::from_str(first_content)
		.expect("First response should be valid JSON");

	// Extract cursor
	let cursor_value = first_page.get("last_cursor")
		.expect("Response should have last_cursor field");

	if cursor_value.is_null() {
		println!("Skipping desc pagination test - no cursor returned (insufficient data)");
		return;
	}

	let cursor = cursor_value.as_str()
		.expect("last_cursor should be a string when not null");

	// Second page with desc order and cursor
	let second_result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_cells",
			"arguments": {
				"search_key": {
					"script": {
						"code_hash": "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
						"hash_type": "type",
						"args": "0x"
					},
					"script_type": "lock"
				},
				"order": "desc",
				"limit": 2,
				"after_cursor": cursor
			}
		}))
		.await
		.expect("Second get_cells request with desc order should succeed");

	let second_content = second_result["content"][0]["text"].as_str().unwrap();
	let second_page: serde_json::Value = serde_json::from_str(second_content)
		.expect("Second response should be valid JSON");

	// Validate pagination
	let first_objects = first_page["objects"].as_array()
		.expect("First page should have objects array");
	let second_objects = second_page["objects"].as_array()
		.expect("Second page should have objects array");

	assert!(!second_objects.is_empty(), "Second page should contain results");

	// Ensure no overlap between pages
	for first_cell in first_objects.iter() {
		let first_outpoint = &first_cell["out_point"];
		for second_cell in second_objects.iter() {
			let second_outpoint = &second_cell["out_point"];
			assert_ne!(
				first_outpoint, second_outpoint,
				"Pages should not have overlapping cells"
			);
		}
	}
}

#[tokio::test]
async fn test_get_cells_pagination_multiple_pages() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let mut all_outpoints = Vec::new();
	let mut cursor: Option<String> = None;
	let max_pages = 5;

	for page_num in 0..max_pages {
		let mut args = json!({
			"search_key": {
				"script": {
					"code_hash": "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
					"hash_type": "type",
					"args": "0x"
				},
				"script_type": "lock"
			},
			"order": "asc",
			"limit": 2
		});

		if let Some(ref c) = cursor {
			args["after_cursor"] = json!(c);
		}

		let result = ctx
			.mcp_call("tools/call", json!({"name": "get_cells", "arguments": args}))
			.await
			.expect(&format!("Page {} request should succeed", page_num + 1));

		let content = result["content"][0]["text"].as_str().unwrap();
		let page: serde_json::Value = serde_json::from_str(content)
			.expect("Response should be valid JSON");

		let objects = page["objects"].as_array()
			.expect("Response should have objects array");

		if objects.is_empty() {
			println!("Reached end of results at page {}", page_num + 1);
			break;
		}

		// Collect outpoints from this page
		for obj in objects {
			let outpoint = obj["out_point"].clone();

			// Ensure this outpoint hasn't appeared before
			assert!(
				!all_outpoints.contains(&outpoint),
				"Duplicate outpoint found across pages: {:?}",
				outpoint
			);

			all_outpoints.push(outpoint);
		}

		// Get cursor for next page
		let cursor_value = page.get("last_cursor")
			.expect("Response should have last_cursor field");

		if cursor_value.is_null() {
			println!("No more pages after page {}", page_num + 1);
			break;
		}

		cursor = Some(cursor_value.as_str()
			.expect("last_cursor should be a string")
			.to_string());
	}

	// Should have traversed at least one page
	assert!(!all_outpoints.is_empty(), "Should have collected at least some results");
}

#[tokio::test]
async fn test_get_cells_pagination_malformed_cursor() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_cells",
			"arguments": {
				"search_key": {
					"script": {
						"code_hash": "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
						"hash_type": "type",
						"args": "0x"
					},
					"script_type": "lock"
				},
				"order": "asc",
				"limit": 2,
				"after_cursor": "invalid_cursor_format"
			}
		}))
		.await;

	// Verify CKB RPC rejects malformed cursor with error
	assert!(result.is_err(), "Malformed cursor should be rejected with error");
}

#[tokio::test]
async fn test_get_cells_empty_results() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_cells",
			"arguments": {
				"search_key": {
					"script": {
						"code_hash": "0x0000000000000000000000000000000000000000000000000000000000000000",
						"hash_type": "type",
						"args": "0xdeadbeef"
					},
					"script_type": "lock"
				},
				"limit": 10
			}
		}))
		.await
		.expect("get_cells should succeed even with no results");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
}

// Indexer Pagination & Filtering Tests - get_transactions
#[tokio::test]
async fn test_get_transactions_with_limit() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_transactions",
			"arguments": {
				"search_key": {
					"script": {
						"code_hash": "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
						"hash_type": "type",
						"args": "0x"
					},
					"script_type": "lock"
				},
				"limit": 5
			}
		}))
		.await
		.expect("get_transactions with limit should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
}

#[tokio::test]
async fn test_get_transactions_pagination_cursor() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	// First page
	let first_result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_transactions",
			"arguments": {
				"search_key": {
					"script": {
						"code_hash": "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
						"hash_type": "type",
						"args": "0x"
					},
					"script_type": "lock"
				},
				"order": "asc",
				"limit": 2
			}
		}))
		.await
		.expect("First get_transactions request should succeed");

	let first_content = first_result["content"][0]["text"].as_str().unwrap();
	let first_page: serde_json::Value = serde_json::from_str(first_content)
		.expect("First response should be valid JSON");

	// Extract cursor
	let cursor_value = first_page.get("last_cursor")
		.expect("Response should have last_cursor field");

	if cursor_value.is_null() {
		println!("Skipping transactions pagination test - no cursor returned (insufficient data)");
		return;
	}

	let cursor = cursor_value.as_str()
		.expect("last_cursor should be a string when not null");

	// Second page with cursor
	let second_result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_transactions",
			"arguments": {
				"search_key": {
					"script": {
						"code_hash": "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
						"hash_type": "type",
						"args": "0x"
					},
					"script_type": "lock"
				},
				"order": "asc",
				"limit": 2,
				"after_cursor": cursor
			}
		}))
		.await
		.expect("Second get_transactions request with cursor should succeed");

	let second_content = second_result["content"][0]["text"].as_str().unwrap();
	let second_page: serde_json::Value = serde_json::from_str(second_content)
		.expect("Second response should be valid JSON");

	// Validate pagination
	let first_objects = first_page["objects"].as_array()
		.expect("First page should have objects array");
	let second_objects = second_page["objects"].as_array()
		.expect("Second page should have objects array");

	assert!(!second_objects.is_empty(), "Second page should contain results");

	// Ensure no overlap between pages
	for first_tx in first_objects.iter() {
		let first_tx_hash = &first_tx["tx_hash"];
		for second_tx in second_objects.iter() {
			let second_tx_hash = &second_tx["tx_hash"];
			assert_ne!(
				first_tx_hash, second_tx_hash,
				"Pages should not have overlapping transactions"
			);
		}
	}
}

#[tokio::test]
async fn test_get_transactions_pagination_order_desc() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	// First page with desc order
	let first_result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_transactions",
			"arguments": {
				"search_key": {
					"script": {
						"code_hash": "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
						"hash_type": "type",
						"args": "0x"
					},
					"script_type": "lock"
				},
				"order": "desc",
				"limit": 2
			}
		}))
		.await
		.expect("get_transactions with desc order should succeed");

	let first_content = first_result["content"][0]["text"].as_str().unwrap();
	let first_page: serde_json::Value = serde_json::from_str(first_content)
		.expect("First response should be valid JSON");

	// Extract cursor
	let cursor_value = first_page.get("last_cursor")
		.expect("Response should have last_cursor field");

	if cursor_value.is_null() {
		println!("Skipping desc transactions pagination test - no cursor returned (insufficient data)");
		return;
	}

	let cursor = cursor_value.as_str()
		.expect("last_cursor should be a string when not null");

	// Second page with desc order and cursor
	let second_result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_transactions",
			"arguments": {
				"search_key": {
					"script": {
						"code_hash": "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
						"hash_type": "type",
						"args": "0x"
					},
					"script_type": "lock"
				},
				"order": "desc",
				"limit": 2,
				"after_cursor": cursor
			}
		}))
		.await
		.expect("Second get_transactions request with desc order should succeed");

	let second_content = second_result["content"][0]["text"].as_str().unwrap();
	let second_page: serde_json::Value = serde_json::from_str(second_content)
		.expect("Second response should be valid JSON");

	// Validate pagination
	let first_objects = first_page["objects"].as_array()
		.expect("First page should have objects array");
	let second_objects = second_page["objects"].as_array()
		.expect("Second page should have objects array");

	assert!(!second_objects.is_empty(), "Second page should contain results");

	// Ensure no overlap between pages
	for first_tx in first_objects.iter() {
		let first_tx_hash = &first_tx["tx_hash"];
		for second_tx in second_objects.iter() {
			let second_tx_hash = &second_tx["tx_hash"];
			assert_ne!(
				first_tx_hash, second_tx_hash,
				"Pages should not have overlapping transactions"
			);
		}
	}
}

#[tokio::test]
async fn test_get_transactions_pagination_multiple_pages_grouped() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let mut all_tx_hashes = Vec::new();
	let mut cursor: Option<String> = None;
	let max_pages = 5;

	for page_num in 0..max_pages {
		let mut args = json!({
			"search_key": {
				"script": {
					"code_hash": "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
					"hash_type": "type",
					"args": "0x"
				},
				"script_type": "lock"
			},
			"order": "asc",
			"limit": 2,
			"group_by_transaction": true
		});

		if let Some(ref c) = cursor {
			args["after_cursor"] = json!(c);
		}

		let result = ctx
			.mcp_call("tools/call", json!({"name": "get_transactions", "arguments": args}))
			.await
			.expect(&format!("Page {} request should succeed", page_num + 1));

		let content = result["content"][0]["text"].as_str().unwrap();
		let page: serde_json::Value = serde_json::from_str(content)
			.expect("Response should be valid JSON");

		let objects = page["objects"].as_array()
			.expect("Response should have objects array");

		if objects.is_empty() {
			println!("Reached end of results at page {}", page_num + 1);
			break;
		}

		// Collect transaction hashes from this page
		for obj in objects {
			let tx_hash = obj["tx_hash"].clone();

			// With group_by_transaction=true, each tx_hash should appear only once
			assert!(
				!all_tx_hashes.contains(&tx_hash),
				"Duplicate transaction found across pages: {:?}",
				tx_hash
			);

			all_tx_hashes.push(tx_hash);
		}

		// Get cursor for next page
		let cursor_value = page.get("last_cursor")
			.expect("Response should have last_cursor field");

		if cursor_value.is_null() {
			println!("No more pages after page {}", page_num + 1);
			break;
		}

		cursor = Some(cursor_value.as_str()
			.expect("last_cursor should be a string")
			.to_string());
	}

	// Should have traversed at least one page
	assert!(!all_tx_hashes.is_empty(), "Should have collected at least some results");
}

#[tokio::test]
async fn test_get_transactions_pagination_multiple_pages_ungrouped() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let mut all_entries = Vec::new();
	let mut cursor: Option<String> = None;
	let max_pages = 5;

	for page_num in 0..max_pages {
		let mut args = json!({
			"search_key": {
				"script": {
					"code_hash": "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
					"hash_type": "type",
					"args": "0x"
				},
				"script_type": "lock"
			},
			"order": "asc",
			"limit": 2,
			"group_by_transaction": false
		});

		if let Some(ref c) = cursor {
			args["after_cursor"] = json!(c);
		}

		let result = ctx
			.mcp_call("tools/call", json!({"name": "get_transactions", "arguments": args}))
			.await
			.expect(&format!("Page {} request should succeed", page_num + 1));

		let content = result["content"][0]["text"].as_str().unwrap();
		let page: serde_json::Value = serde_json::from_str(content)
			.expect("Response should be valid JSON");

		let objects = page["objects"].as_array()
			.expect("Response should have objects array");

		if objects.is_empty() {
			println!("Reached end of results at page {}", page_num + 1);
			break;
		}

		// Collect transaction entries (tx_hash + io_type + io_index) from this page
		for obj in objects {
			let entry = json!({
				"tx_hash": obj["tx_hash"],
				"io_type": obj["io_type"],
				"io_index": obj["io_index"]
			});

			// With group_by_transaction=false, each entry (not just tx_hash) should be unique
			assert!(
				!all_entries.contains(&entry),
				"Duplicate transaction entry found across pages: {:?}",
				entry
			);

			all_entries.push(entry);
		}

		// Get cursor for next page
		let cursor_value = page.get("last_cursor")
			.expect("Response should have last_cursor field");

		if cursor_value.is_null() {
			println!("No more pages after page {}", page_num + 1);
			break;
		}

		cursor = Some(cursor_value.as_str()
			.expect("last_cursor should be a string")
			.to_string());
	}

	// Should have traversed at least one page
	assert!(!all_entries.is_empty(), "Should have collected at least some results");
}

#[tokio::test]
async fn test_get_transactions_pagination_malformed_cursor() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_transactions",
			"arguments": {
				"search_key": {
					"script": {
						"code_hash": "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
						"hash_type": "type",
						"args": "0x"
					},
					"script_type": "lock"
				},
				"order": "asc",
				"limit": 2,
				"after_cursor": "invalid_cursor_format"
			}
		}))
		.await;

	// Verify CKB RPC rejects malformed cursor with error
	assert!(result.is_err(), "Malformed cursor should be rejected with error");
}

#[tokio::test]
async fn test_get_transactions_empty_results() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_transactions",
			"arguments": {
				"search_key": {
					"script": {
						"code_hash": "0x0000000000000000000000000000000000000000000000000000000000000000",
						"hash_type": "type",
						"args": "0xdeadbeef"
					},
					"script_type": "lock"
				},
				"limit": 10
			}
		}))
		.await
		.expect("get_transactions should succeed even with no results");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
}

// Additional Indexer Error Tests
#[tokio::test]
async fn test_get_cells_invalid_search_key() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_cells",
			"arguments": {
				"search_key": "invalid"
			}
		}))
		.await;

	assert!(result.is_err(), "Should fail for invalid search_key format");
}

#[tokio::test]
async fn test_get_transactions_missing_search_key() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_transactions", "arguments": {}}))
		.await;

	assert!(result.is_err(), "Should fail when search_key is missing");
}

// Chain Methods - Advanced

#[tokio::test]
async fn test_estimate_cycles() {
	// This test validates the estimate_cycles RPC method works correctly.
	// Since estimate_cycles requires resolving transaction inputs which may not exist
	// on a fresh devnet, this test may skip if no suitable transactions are found.
	// The error case is tested separately in test_estimate_cycles_invalid_tx.

	use reqwest::Client;
	let ctx = TestContext::new(RPC_SERVER_PORT);

	// Use direct CKB RPC to find a transaction with real, resolvable inputs
	let client = Client::new();
	let ckb_rpc_url = TestContext::get_ckb_rpc_url().expect("CKB_RPC_URL must be set");

	// Get a recent block with transactions
	let tip_response = client
		.post(&ckb_rpc_url)
		.json(&json!({
			"jsonrpc": "2.0",
			"id": 1,
			"method": "get_tip_block_number",
			"params": []
		}))
		.send()
		.await
		.expect("Should get tip block number");

	let tip_body: Value = tip_response.json().await.expect("Should parse JSON");
	let tip_number_hex = tip_body["result"].as_str().expect("Should have tip number");
	let tip_number = u64::from_str_radix(&tip_number_hex[2..], 16).expect("Should parse hex");

	// Search backwards for a block with non-cellbase transactions
	let mut found_tx = None;
	for offset in 1..std::cmp::min(100, tip_number) {
		let block_number = tip_number - offset;
		let block_response = client
			.post(&ckb_rpc_url)
			.json(&json!({
				"jsonrpc": "2.0",
				"id": 1,
				"method": "get_block_by_number",
				"params": [format!("{:#x}", block_number)]
			}))
			.send()
			.await
			.expect("Should get block");

		let block_body: Value = block_response.json().await.expect("Should parse JSON");
		let transactions = block_body["result"]["transactions"].as_array()
			.expect("Should have transactions");

		// Skip cellbase (first tx), look for regular transactions
		if transactions.len() > 1 {
			found_tx = Some(transactions[1].clone());
			break;
		}
	}

	let tx = match found_tx {
		Some(t) => t,
		None => {
			eprintln!("No suitable transactions found for estimate_cycles test - skipping");
			eprintln!("This is normal on a fresh devnet with no user transactions");
			return;
		}
	};

	// Call estimate_cycles via MCP
	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "estimate_cycles",
			"arguments": {
				"tx": tx
			}
		}))
		.await
		.expect("estimate_cycles should succeed for real transaction");

	let content = result["content"][0]["text"].as_str().unwrap();
	let cycles_result: serde_json::Value = serde_json::from_str(content)
		.expect("Response should be valid JSON");

	// Verify the response has the expected structure
	assert!(cycles_result.get("cycles").is_some(), "Response should have 'cycles' field");

	// Verify cycles is a valid hex number
	let cycles_str = cycles_result["cycles"].as_str().expect("cycles should be a string");
	assert!(cycles_str.starts_with("0x"), "cycles should be in hex format");

	// Parse to verify it's a valid number
	let _cycles_value = u64::from_str_radix(&cycles_str[2..], 16)
		.expect("cycles should be valid hex number");
}

#[tokio::test]
async fn test_estimate_cycles_missing_tx() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "estimate_cycles", "arguments": {}}))
		.await;

	assert!(result.is_err(), "Should fail when tx parameter is missing");
	let error_msg = result.unwrap_err();
	assert!(error_msg.contains("Missing tx"), "Error should mention missing tx parameter");
}

#[tokio::test]
async fn test_estimate_cycles_invalid_tx() {
	let ctx = TestContext::new(RPC_SERVER_PORT);
	let shared_data = SharedTestData::get_or_init_async().await;

	// Use genesis cellbase which has unresolvable inputs (null outpoint)
	let genesis_cellbase = &shared_data.genesis_block["transactions"][0];

	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "estimate_cycles",
			"arguments": {
				"tx": genesis_cellbase
			}
		}))
		.await;

	// Should fail because genesis cellbase references null outpoint
	assert!(result.is_err(), "Should fail for transaction with unresolvable inputs");
	let error_msg = result.unwrap_err();
	// Error can be either TransactionFailedToResolve or just contain "error"
	assert!(error_msg.to_lowercase().contains("error") || error_msg.contains("Failed"),
		"Error should indicate failure, got: {}", error_msg);
}

// Pool Methods

#[tokio::test]
async fn test_send_transaction_missing_tx() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "send_transaction", "arguments": {}}))
		.await;

	assert!(result.is_err(), "Should fail when tx parameter is missing");
	let error_msg = result.unwrap_err();
	assert!(error_msg.contains("Missing tx"), "Error should mention missing tx parameter");
}

#[tokio::test]
async fn test_send_transaction_invalid() {
	let ctx = TestContext::new(RPC_SERVER_PORT);
	let shared_data = SharedTestData::get_or_init_async().await;

	// Try to send genesis cellbase which has unresolvable inputs
	let genesis_cellbase = &shared_data.genesis_block["transactions"][0];

	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "send_transaction",
			"arguments": {
				"tx": genesis_cellbase
			}
		}))
		.await;

	// Should fail - genesis cellbase cannot be sent
	assert!(result.is_err(), "Should fail for invalid transaction");
	let error_msg = result.unwrap_err();
	assert!(error_msg.to_lowercase().contains("error") || error_msg.contains("Failed"),
		"Error should indicate failure, got: {}", error_msg);
}

// Stats Methods

#[tokio::test]
async fn test_get_blockchain_info() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_blockchain_info", "arguments": {}}))
		.await
		.expect("get_blockchain_info should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let info: serde_json::Value = serde_json::from_str(content)
		.expect("Response should be valid JSON");

	// Verify expected fields exist
	assert!(info.get("chain").is_some(), "Response should have 'chain' field");
	assert!(info.get("difficulty").is_some(), "Response should have 'difficulty' field");
	assert!(info.get("epoch").is_some(), "Response should have 'epoch' field");
	assert!(info.get("is_initial_block_download").is_some(), "Response should have 'is_initial_block_download' field");
	assert!(info.get("median_time").is_some(), "Response should have 'median_time' field");

	// Verify chain field is a string (e.g. "ckb", "ckb_testnet", "ckb_dev")
	let chain = info["chain"].as_str().expect("chain should be a string");
	assert!(!chain.is_empty(), "chain should not be empty");

	// Verify difficulty is in hex format
	let difficulty = info["difficulty"].as_str().expect("difficulty should be a string");
	assert!(difficulty.starts_with("0x"), "difficulty should be in hex format");

	// Verify epoch is in hex format
	let epoch = info["epoch"].as_str().expect("epoch should be a string");
	assert!(epoch.starts_with("0x"), "epoch should be in hex format");
}

#[tokio::test]
async fn test_get_consensus() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_consensus", "arguments": {}}))
		.await
		.expect("get_consensus should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let consensus: serde_json::Value = serde_json::from_str(content)
		.expect("Response should be valid JSON");

	// Verify key consensus parameters exist
	assert!(consensus.get("id").is_some(), "Response should have 'id' field");
	assert!(consensus.get("genesis_hash").is_some(), "Response should have 'genesis_hash' field");
	assert!(consensus.get("dao_type_hash").is_some(), "Response should have 'dao_type_hash' field");
	assert!(consensus.get("epoch_duration_target").is_some(), "Response should have 'epoch_duration_target' field");
	assert!(consensus.get("hardfork_features").is_some(), "Response should have 'hardfork_features' field");

	// Verify genesis_hash format
	let genesis_hash = consensus["genesis_hash"].as_str().expect("genesis_hash should be a string");
	assert!(genesis_hash.starts_with("0x"), "genesis_hash should be in hex format");
	assert_eq!(genesis_hash.len(), 66, "genesis_hash should be 66 characters (0x + 64 hex digits)");

	// Verify id field (chain identifier)
	let id = consensus["id"].as_str().expect("id should be a string");
	assert!(!id.is_empty(), "id should not be empty");

	// Verify hardfork_features is an array
	assert!(consensus["hardfork_features"].is_array(), "hardfork_features should be an array");
}

#[tokio::test]
async fn test_tx_pool_info() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "tx_pool_info", "arguments": {}}))
		.await
		.expect("tx_pool_info should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let pool_info: serde_json::Value = serde_json::from_str(content)
		.expect("Response should be valid JSON");

	// Verify key pool info fields exist
	assert!(pool_info.get("tip_hash").is_some(), "Response should have 'tip_hash' field");
	assert!(pool_info.get("tip_number").is_some(), "Response should have 'tip_number' field");
	assert!(pool_info.get("pending").is_some(), "Response should have 'pending' field");
	assert!(pool_info.get("proposed").is_some(), "Response should have 'proposed' field");
	assert!(pool_info.get("orphan").is_some(), "Response should have 'orphan' field");
	assert!(pool_info.get("total_tx_size").is_some(), "Response should have 'total_tx_size' field");
	assert!(pool_info.get("total_tx_cycles").is_some(), "Response should have 'total_tx_cycles' field");
	assert!(pool_info.get("min_fee_rate").is_some(), "Response should have 'min_fee_rate' field");
	assert!(pool_info.get("max_tx_pool_size").is_some(), "Response should have 'max_tx_pool_size' field");

	// Verify tip_hash format
	let tip_hash = pool_info["tip_hash"].as_str().expect("tip_hash should be a string");
	assert!(tip_hash.starts_with("0x"), "tip_hash should be in hex format");

	// Verify numeric fields are in hex format
	let tip_number = pool_info["tip_number"].as_str().expect("tip_number should be a string");
	assert!(tip_number.starts_with("0x"), "tip_number should be in hex format");

	let pending = pool_info["pending"].as_str().expect("pending should be a string");
	assert!(pending.starts_with("0x"), "pending should be in hex format");
}

#[tokio::test]
async fn test_get_raw_tx_pool_non_verbose() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_raw_tx_pool",
			"arguments": { "verbose": false }
		}))
		.await
		.expect("get_raw_tx_pool should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let pool: serde_json::Value = serde_json::from_str(content)
		.expect("Response should be valid JSON");

	// Non-verbose returns object with pending and proposed arrays
	assert!(pool.get("pending").is_some(), "Response should have 'pending' field");
	assert!(pool.get("proposed").is_some(), "Response should have 'proposed' field");

	// Verify pending is an array
	let pending = pool["pending"].as_array().expect("pending should be an array");

	// Each entry should be a tx hash string (if any exist)
	for tx_hash in pending {
		let hash_str = tx_hash.as_str().expect("tx hash should be a string");
		assert!(hash_str.starts_with("0x"), "tx hash should be in hex format");
		assert_eq!(hash_str.len(), 66, "tx hash should be 66 characters");
	}
}

#[tokio::test]
async fn test_get_raw_tx_pool_verbose() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_raw_tx_pool",
			"arguments": { "verbose": true }
		}))
		.await
		.expect("get_raw_tx_pool should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let pool: serde_json::Value = serde_json::from_str(content)
		.expect("Response should be valid JSON");

	// Verbose returns object with pending and proposed as objects (not arrays)
	assert!(pool.get("pending").is_some(), "Response should have 'pending' field");
	assert!(pool.get("proposed").is_some(), "Response should have 'proposed' field");
	assert!(pool.get("conflicted").is_some(), "Response should have 'conflicted' field");

	// Verify pending is an object
	let pending = pool["pending"].as_object().expect("pending should be an object");

	// Each entry maps tx_hash -> tx details with cycles, size, fee, etc
	for (_tx_hash, tx_info) in pending {
		assert!(tx_info.get("cycles").is_some(), "tx info should have 'cycles' field");
		assert!(tx_info.get("size").is_some(), "tx info should have 'size' field");
		assert!(tx_info.get("fee").is_some(), "tx info should have 'fee' field");
		assert!(tx_info.get("ancestors_size").is_some(), "tx info should have 'ancestors_size' field");
		assert!(tx_info.get("ancestors_cycles").is_some(), "tx info should have 'ancestors_cycles' field");
		assert!(tx_info.get("ancestors_count").is_some(), "tx info should have 'ancestors_count' field");
	}
}

#[tokio::test]
async fn test_get_raw_tx_pool_default() {
	// Test default behavior (no verbose parameter)
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_raw_tx_pool",
			"arguments": {}
		}))
		.await
		.expect("get_raw_tx_pool should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let pool: serde_json::Value = serde_json::from_str(content)
		.expect("Response should be valid JSON");

	// Default is non-verbose (arrays)
	assert!(pool.get("pending").is_some(), "Response should have 'pending' field");
	assert!(pool.get("proposed").is_some(), "Response should have 'proposed' field");

	// Verify pending is an array (non-verbose)
	pool["pending"].as_array().expect("pending should be an array in default mode");
}

#[tokio::test]
async fn test_get_pool_tx_detail_info() {
	// This test attempts to query detailed pool info for a transaction.
	// On a fresh/empty devnet, the pool may be empty, so we gracefully skip.
	let ctx = TestContext::new(RPC_SERVER_PORT);

	// First, get raw tx pool to find a transaction
	let pool_result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_raw_tx_pool",
			"arguments": { "verbose": false }
		}))
		.await
		.expect("get_raw_tx_pool should succeed");

	let pool_content = pool_result["content"][0]["text"].as_str().unwrap();
	let pool: serde_json::Value = serde_json::from_str(pool_content)
		.expect("Pool response should be valid JSON");

	let pending = pool["pending"].as_array().expect("pending should be an array");

	if pending.is_empty() {
		eprintln!("No pending transactions in pool - skipping test (normal on empty devnet)");
		return;
	}

	let tx_hash = pending[0].as_str().expect("tx hash should be a string");

	// Query detailed info for this transaction
	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_pool_tx_detail_info",
			"arguments": { "tx_hash": tx_hash }
		}))
		.await
		.expect("get_pool_tx_detail_info should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let detail: serde_json::Value = serde_json::from_str(content)
		.expect("Response should be valid JSON");

	// Verify expected fields
	assert!(detail.get("entry_status").is_some(), "Response should have 'entry_status' field");
	assert!(detail.get("pending_count").is_some(), "Response should have 'pending_count' field");
	assert!(detail.get("proposed_count").is_some(), "Response should have 'proposed_count' field");
	assert!(detail.get("ancestors_count").is_some(), "Response should have 'ancestors_count' field");
	assert!(detail.get("descendants_count").is_some(), "Response should have 'descendants_count' field");
	assert!(detail.get("timestamp").is_some(), "Response should have 'timestamp' field");

	// Verify entry_status is a valid string
	let entry_status = detail["entry_status"].as_str().expect("entry_status should be a string");
	assert!(["pending", "proposed", "conflicted"].contains(&entry_status), "entry_status should be one of: pending, proposed, conflicted");

	// Verify score_sortkey exists if status is pending
	if entry_status == "pending" {
		assert!(detail.get("score_sortkey").is_some(), "pending tx should have 'score_sortkey' field");
		let score_sortkey = &detail["score_sortkey"];
		assert!(score_sortkey.get("fee").is_some(), "score_sortkey should have 'fee' field");
		assert!(score_sortkey.get("weight").is_some(), "score_sortkey should have 'weight' field");
	}
}

#[tokio::test]
async fn test_get_pool_tx_detail_info_missing_tx_hash() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_pool_tx_detail_info",
			"arguments": {}
		}))
		.await;

	assert!(result.is_err(), "Should fail when tx_hash is missing");
	let error = result.unwrap_err();
	assert!(error.to_string().contains("Missing tx_hash") || error.to_string().contains("required"),
		"Error should mention missing tx_hash");
}

#[tokio::test]
async fn test_get_pool_tx_detail_info_invalid_tx_hash() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_pool_tx_detail_info",
			"arguments": { "tx_hash": "0xinvalid" }
		}))
		.await;

	// Either fails or returns None
	if let Ok(response) = result {
		let content = response["content"][0]["text"].as_str().unwrap();

		// Check if it's a null response (transaction not in pool)
		let value: serde_json::Value = serde_json::from_str(content)
			.expect("Response should be valid JSON");

		// CKB returns null for transaction not in pool
		assert!(value.is_null(), "Should return null for transaction not in pool");
	}
	// Error is also acceptable (invalid format)
}

#[tokio::test]
async fn test_tx_pool_ready() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "tx_pool_ready", "arguments": {}}))
		.await
		.expect("tx_pool_ready should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let ready: serde_json::Value = serde_json::from_str(content)
		.expect("Response should be valid JSON");

	// Should be a boolean
	assert!(ready.is_boolean(), "Response should be a boolean");

	// For a running node, pool should typically be ready
	let ready_bool = ready.as_bool().expect("Should be a boolean value");
	assert!(ready_bool, "tx-pool service should be ready on running node");
}

#[tokio::test]
async fn test_sync_state() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "sync_state", "arguments": {}}))
		.await
		.expect("sync_state should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let sync_state: serde_json::Value = serde_json::from_str(content)
		.expect("Response should be valid JSON");

	// Verify key sync state fields
	assert!(sync_state.get("ibd").is_some(), "Response should have 'ibd' field");
	assert!(sync_state.get("tip_hash").is_some(), "Response should have 'tip_hash' field");
	assert!(sync_state.get("tip_number").is_some(), "Response should have 'tip_number' field");
	assert!(sync_state.get("best_known_block_number").is_some(), "Response should have 'best_known_block_number' field");
	assert!(sync_state.get("best_known_block_timestamp").is_some(), "Response should have 'best_known_block_timestamp' field");
	assert!(sync_state.get("orphan_blocks_count").is_some(), "Response should have 'orphan_blocks_count' field");
	assert!(sync_state.get("inflight_blocks_count").is_some(), "Response should have 'inflight_blocks_count' field");
	assert!(sync_state.get("fast_time").is_some(), "Response should have 'fast_time' field");
	assert!(sync_state.get("normal_time").is_some(), "Response should have 'normal_time' field");
	assert!(sync_state.get("low_time").is_some(), "Response should have 'low_time' field");

	// Verify ibd is boolean
	sync_state["ibd"].as_bool().expect("ibd should be a boolean");

	// Verify tip_hash format
	let tip_hash = sync_state["tip_hash"].as_str().expect("tip_hash should be a string");
	assert!(tip_hash.starts_with("0x"), "tip_hash should be in hex format");
	assert_eq!(tip_hash.len(), 66, "tip_hash should be 66 characters");

	// Verify numeric fields are in hex format
	let tip_number = sync_state["tip_number"].as_str().expect("tip_number should be a string");
	assert!(tip_number.starts_with("0x"), "tip_number should be in hex format");
}

#[tokio::test]
async fn test_get_peers() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_peers", "arguments": {}}))
		.await
		.expect("get_peers should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let peers: serde_json::Value = serde_json::from_str(content)
		.expect("Response should be valid JSON");

	// Should be an array
	let peers_array = peers.as_array().expect("Response should be an array");

	// If there are peers, verify structure
	if !peers_array.is_empty() {
		let peer = &peers_array[0];

		// Verify key peer fields
		assert!(peer.get("node_id").is_some(), "Peer should have 'node_id' field");
		assert!(peer.get("addresses").is_some(), "Peer should have 'addresses' field");
		assert!(peer.get("is_outbound").is_some(), "Peer should have 'is_outbound' field");
		assert!(peer.get("connected_duration").is_some(), "Peer should have 'connected_duration' field");
		assert!(peer.get("protocols").is_some(), "Peer should have 'protocols' field");
		assert!(peer.get("version").is_some(), "Peer should have 'version' field");

		// Verify node_id is a string
		peer["node_id"].as_str().expect("node_id should be a string");

		// Verify is_outbound is boolean
		peer["is_outbound"].as_bool().expect("is_outbound should be a boolean");

		// Verify addresses is an array
		let addresses = peer["addresses"].as_array().expect("addresses should be an array");
		if !addresses.is_empty() {
			assert!(addresses[0].get("address").is_some(), "Address should have 'address' field");
			assert!(addresses[0].get("score").is_some(), "Address should have 'score' field");
		}

		// Verify protocols is an array
		let protocols = peer["protocols"].as_array().expect("protocols should be an array");
		if !protocols.is_empty() {
			assert!(protocols[0].get("id").is_some(), "Protocol should have 'id' field");
			assert!(protocols[0].get("version").is_some(), "Protocol should have 'version' field");
		}
	}
}

#[tokio::test]
async fn test_get_deployments_info() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_deployments_info", "arguments": {}}))
		.await
		.expect("get_deployments_info should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let deployments_info: serde_json::Value = serde_json::from_str(content)
		.expect("Response should be valid JSON");

	// Verify key deployment info fields
	assert!(deployments_info.get("hash").is_some(), "Response should have 'hash' field");
	assert!(deployments_info.get("epoch").is_some(), "Response should have 'epoch' field");
	assert!(deployments_info.get("deployments").is_some(), "Response should have 'deployments' field");

	// Verify hash format
	let hash = deployments_info["hash"].as_str().expect("hash should be a string");
	assert!(hash.starts_with("0x"), "hash should be in hex format");
	assert_eq!(hash.len(), 66, "hash should be 66 characters");

	// Verify epoch format
	let epoch = deployments_info["epoch"].as_str().expect("epoch should be a string");
	assert!(epoch.starts_with("0x"), "epoch should be in hex format");

	// Verify deployments is an object
	let deployments = deployments_info["deployments"].as_object().expect("deployments should be an object");

	// If there are deployments, verify structure
	for (deployment_name, deployment_info) in deployments {
		// Deployment should have state and bit fields at minimum
		assert!(deployment_info.get("state").is_some(), "Deployment '{}' should have 'state' field", deployment_name);
		assert!(deployment_info.get("bit").is_some(), "Deployment '{}' should have 'bit' field", deployment_name);

		// Verify state is a string
		deployment_info["state"].as_str().expect(&format!("Deployment '{}' state should be a string", deployment_name));

		// Verify bit is a number (in hex format)
		deployment_info["bit"].as_u64().or_else(|| deployment_info["bit"].as_str().and_then(|s| u64::from_str_radix(&s[2..], 16).ok()))
			.expect(&format!("Deployment '{}' bit should be a number", deployment_name));
	}
}

#[tokio::test]
async fn test_calculate_dao_maximum_withdraw_missing_params() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	// Test missing out_point
	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "calculate_dao_maximum_withdraw",
			"arguments": {
				"kind": "0xa5f5c85987a15de25661e5a214f2c1449cd803f071acc7999820f25246471f40"
			}
		}))
		.await;

	assert!(result.is_err(), "Should fail when out_point is missing");

	// Test missing kind
	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "calculate_dao_maximum_withdraw",
			"arguments": {
				"out_point": {
					"tx_hash": "0xa4037a893eb48e18ed4ef61034ce26eba9c585f15c9cee102ae58505565eccc3",
					"index": "0x0"
				}
			}
		}))
		.await;

	assert!(result.is_err(), "Should fail when kind is missing");
}

#[tokio::test]
async fn test_estimate_fee_rate() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "estimate_fee_rate", "arguments": {}}))
		.await
		.expect("estimate_fee_rate should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let fee_rate: serde_json::Value = serde_json::from_str(content)
		.expect("Response should be valid JSON");

	// Should be a hex string representing shannons per KB
	let fee_rate_str = fee_rate.as_str().expect("fee_rate should be a string");
	assert!(fee_rate_str.starts_with("0x"), "fee_rate should be in hex format");

	// Parse as u64 to verify it's a valid number
	let fee_value = u64::from_str_radix(&fee_rate_str[2..], 16)
		.expect("fee_rate should be valid hex number");
	assert!(fee_value > 0, "fee_rate should be greater than 0");
}

#[tokio::test]
async fn test_estimate_fee_rate_with_params() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "estimate_fee_rate",
			"arguments": {
				"estimate_mode": "no_priority",
				"enable_fallback": true
			}
		}))
		.await
		.expect("estimate_fee_rate should succeed with params");

	let content = result["content"][0]["text"].as_str().unwrap();
	let fee_rate: serde_json::Value = serde_json::from_str(content)
		.expect("Response should be valid JSON");

	// Verify it's a hex string
	let fee_rate_str = fee_rate.as_str().expect("fee_rate should be a string");
	assert!(fee_rate_str.starts_with("0x"), "fee_rate should be in hex format");
}

#[tokio::test]
async fn test_test_tx_pool_accept_missing_tx() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "test_tx_pool_accept", "arguments": {}}))
		.await;

	assert!(result.is_err(), "Should fail when tx parameter is missing");
	let error_msg = result.unwrap_err();
	assert!(error_msg.contains("Missing tx"), "Error should mention missing tx parameter");
}

#[tokio::test]
async fn test_test_tx_pool_accept_invalid() {
	let ctx = TestContext::new(RPC_SERVER_PORT);
	let shared_data = SharedTestData::get_or_init_async().await;

	// Try to test genesis cellbase which has unresolvable inputs
	let genesis_cellbase = &shared_data.genesis_block["transactions"][0];

	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "test_tx_pool_accept",
			"arguments": {
				"tx": genesis_cellbase
			}
		}))
		.await;

	// Should fail - genesis cellbase has invalid inputs
	assert!(result.is_err(), "Should fail for invalid transaction");
	let error_msg = result.unwrap_err();
	assert!(error_msg.to_lowercase().contains("error") || error_msg.contains("Failed"),
		"Error should indicate failure, got: {}", error_msg);
}
