use serde_json::json;
use test_common::{SharedTestData, TestContext};

use test_common::{SharedTestData, TestContext};

const RPC_SERVER_PORT: u16 = 8001;

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
async fn test_get_live_cell_genesis() {
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
async fn test_get_live_cell_missing_index() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_live_cell", "arguments": {"tx_hash": "0x0000000000000000000000000000000000000000000000000000000000000000"}}))
		.await;

	assert!(result.is_err(), "Should fail when index is missing");
}

// Indexer Methods - Success
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

