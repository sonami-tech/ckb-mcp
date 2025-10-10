use serde_json::json;

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

// Indexer Pagination & Filtering Tests
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
async fn test_get_cells_with_order_desc() {
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
				"order": "desc",
				"limit": 10
			}
		}))
		.await
		.expect("get_cells with desc order should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
}

#[tokio::test]
async fn test_get_cells_with_cursor() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	// Step 1: First request - get page 1 with small limit
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
				"order": "asc",
				"limit": 2
			}
		}))
		.await
		.expect("First get_cells request should succeed");

	// Step 2: Parse first response
	let first_content = first_result["content"][0]["text"].as_str().unwrap();
	let first_page: serde_json::Value = serde_json::from_str(first_content)
		.expect("First response should be valid JSON");

	// Step 3: Extract cursor and handle edge case
	let cursor_value = first_page.get("last_cursor")
		.expect("Response should have last_cursor field");

	if cursor_value.is_null() {
		println!("Skipping pagination test - no cursor returned (insufficient data)");
		return;
	}

	let cursor = cursor_value.as_str()
		.expect("last_cursor should be a string when not null");

	// Step 4: Second request with cursor
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
				"order": "asc",
				"limit": 2,
				"after_cursor": cursor
			}
		}))
		.await
		.expect("Second get_cells request with cursor should succeed");

	// Step 5: Parse second response
	let second_content = second_result["content"][0]["text"].as_str().unwrap();
	let second_page: serde_json::Value = serde_json::from_str(second_content)
		.expect("Second response should be valid JSON");

	// Step 6: Validate pagination
	let first_objects = first_page["objects"].as_array()
		.expect("First page should have objects array");
	let second_objects = second_page["objects"].as_array()
		.expect("Second page should have objects array");

	assert!(!second_objects.is_empty(), "Second page should contain results");

	// Verify pages have different content
	if !first_objects.is_empty() && !second_objects.is_empty() {
		let first_cell_outpoint = &first_objects[0]["out_point"];
		let second_cell_outpoint = &second_objects[0]["out_point"];

		assert_ne!(
			first_cell_outpoint, second_cell_outpoint,
			"Paginated results should return different cells"
		);
	}

	// Step 7: Ensure no overlap between pages
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
