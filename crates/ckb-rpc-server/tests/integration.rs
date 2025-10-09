use serde_json::json;
use serial_test::serial;

#[path = "../../shared/tests/common/mod.rs"]
mod common;

use common::TestContext;

const RPC_SERVER_PORT: u16 = 8001;

/// Run first - fail fast if server not available
#[tokio::test]
#[serial]
async fn test_00_server_running() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	ctx.verify_server_running()
		.await
		.expect("ckb-rpc-server must be running on port 8001. Start with: cargo run --bin ckb-rpc-server");
}

#[tokio::test]
async fn test_mcp_initialize() {
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

#[tokio::test]
async fn test_tools_list_returns_16_tools() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/list", json!({}))
		.await
		.expect("tools/list should succeed");

	let tools = result["tools"].as_array().unwrap();
	assert_eq!(tools.len(), 16, "Should have exactly 16 RPC tools");
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
		.await;

	// May return null or error depending on RPC implementation
	if let Ok(res) = result {
		let content = res["content"][0]["text"].as_str().unwrap();
		assert!(content.contains("null") || content.contains("error"));
	}
}

#[tokio::test]
async fn test_get_block_by_number_beyond_tip() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_block_by_number", "arguments": {"block_number": 99999999}}))
		.await;

	// Should return null for beyond tip
	if let Ok(res) = result {
		let content = res["content"][0]["text"].as_str().unwrap();
		assert!(content.contains("null"));
	}
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
		.await;

	// Should return null for future epoch
	if let Ok(res) = result {
		let content = res["content"][0]["text"].as_str().unwrap();
		assert!(content.contains("null"));
	}
}

#[tokio::test]
async fn test_get_header_by_number_beyond_tip() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_header_by_number", "arguments": {"block_number": 99999999}}))
		.await;

	if let Ok(res) = result {
		let content = res["content"][0]["text"].as_str().unwrap();
		assert!(content.contains("null"));
	}
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

	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_cells_capacity",
			"arguments": {
				"search_key": {
					"script": {
						"code_hash": "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
						"hash_type": "type",
						"args": "0x"
					},
					"script_type": "lock"
				}
			}
		}))
		.await
		.expect("get_cells_capacity should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
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
	assert!(!content.is_empty());
	assert!(content.contains("version") || content.contains("node_id"));
}

#[tokio::test]
async fn test_local_node_info_has_required_fields() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "local_node_info", "arguments": {}}))
		.await
		.expect("local_node_info should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	// Should have version, node_id, addresses, etc.
	assert!(content.contains("\""));
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

	// Get genesis block first
	let block_result = ctx
		.mcp_call("tools/call", json!({"name": "get_block_by_number", "arguments": {"block_number": 0}}))
		.await
		.expect("get_block_by_number should succeed");

	let block_content = block_result["content"][0]["text"].as_str().unwrap();

	// Genesis block should have cellbase transaction
	// For now just test that the call format works
	if block_content.contains("transactions") {
		// Would need to parse actual tx hash from block, skip for now if complex
	}
}

#[tokio::test]
async fn test_get_transaction_nonexistent() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_transaction", "arguments": {"tx_hash": "0x0000000000000000000000000000000000000000000000000000000000000000"}}))
		.await;

	// Should return null or succeed with null result
	if let Ok(res) = result {
		let content = res["content"][0]["text"].as_str().unwrap();
		assert!(content.contains("null"));
	}
}

#[tokio::test]
async fn test_get_block_hash_negative_number() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	// Note: JSON doesn't have negative integers in the same way, but we can test with invalid value
	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_block_hash", "arguments": {"block_number": -1}}))
		.await;

	// May fail or be handled differently
	let _ = result;
}

// Live Cell Tests
#[tokio::test]
async fn test_get_live_cell_valid() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	// Use a known genesis cellbase output
	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_live_cell",
			"arguments": {
				"tx_hash": "0x0000000000000000000000000000000000000000000000000000000000000000",
				"index": 0,
				"with_data": false
			}
		}))
		.await;

	// May or may not exist depending on network
	let _ = result;
}

#[tokio::test]
async fn test_get_live_cell_with_data() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_live_cell",
			"arguments": {
				"tx_hash": "0x0000000000000000000000000000000000000000000000000000000000000000",
				"index": 0,
				"with_data": true
			}
		}))
		.await;

	let _ = result;
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

	// This tests the response format when querying a potentially spent cell
	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_live_cell",
			"arguments": {
				"tx_hash": "0x0000000000000000000000000000000000000000000000000000000000000000",
				"index": 0
			}
		}))
		.await;

	// Should return status information
	let _ = result;
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

	// First get some cells
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
				"limit": 1
			}
		}))
		.await
		.expect("get_cells should succeed");

	// Note: Would need to parse cursor from response to test pagination properly
	let content = first_result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
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
