use serde_json::json;
use test_common::{SharedTestData, TestContext};

const RPC_SERVER_PORT: u16 = 8001;

#[tokio::test]
async fn test_get_transaction_genesis() {
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
async fn test_get_transactions_missing_search_key() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_transactions", "arguments": {}}))
		.await;

	assert!(result.is_err(), "Should fail when search_key is missing");
}

// Chain Methods - Advanced

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
async fn test_get_transaction_proof() {
	let ctx = TestContext::new(RPC_SERVER_PORT);
	let shared_data = SharedTestData::get_or_init_async().await;

	// Use genesis block transaction
	let genesis_tx_hash = shared_data.genesis_block["transactions"][0]["hash"]
		.as_str()
		.expect("Genesis should have transaction hash");
	let genesis_hash = &shared_data.genesis_hash;

	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_transaction_proof",
			"arguments": {
				"tx_hashes": [genesis_tx_hash],
				"block_hash": genesis_hash
			}
		}))
		.await
		.expect("get_transaction_proof should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let proof: serde_json::Value = serde_json::from_str(content)
		.expect("Response should be valid JSON");

	// Verify proof structure
	assert!(proof["block_hash"].is_string(), "Should have block_hash");
	assert!(proof["witnesses_root"].is_string(), "Should have witnesses_root");
	assert!(proof["proof"]["indices"].is_array(), "Should have proof indices");
	assert!(proof["proof"]["lemmas"].is_array(), "Should have proof lemmas");
}

#[tokio::test]
async fn test_get_transaction_proof_without_block_hash() {
	let ctx = TestContext::new(RPC_SERVER_PORT);
	let shared_data = SharedTestData::get_or_init_async().await;

	// Use genesis block transaction
	let genesis_tx_hash = shared_data.genesis_block["transactions"][0]["hash"]
		.as_str()
		.expect("Genesis should have transaction hash");

	// Without block_hash, should still work (searches for the transaction)
	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_transaction_proof",
			"arguments": {
				"tx_hashes": [genesis_tx_hash]
			}
		}))
		.await
		.expect("get_transaction_proof should succeed without block_hash");

	let content = result["content"][0]["text"].as_str().unwrap();
	let proof: serde_json::Value = serde_json::from_str(content)
		.expect("Response should be valid JSON");

	assert!(proof["block_hash"].is_string(), "Should have block_hash");
}

#[tokio::test]
async fn test_verify_transaction_proof() {
	let ctx = TestContext::new(RPC_SERVER_PORT);
	let shared_data = SharedTestData::get_or_init_async().await;

	// First get a proof
	let genesis_tx_hash = shared_data.genesis_block["transactions"][0]["hash"]
		.as_str()
		.expect("Genesis should have transaction hash");
	let genesis_hash = &shared_data.genesis_hash;

	let proof_result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_transaction_proof",
			"arguments": {
				"tx_hashes": [genesis_tx_hash],
				"block_hash": genesis_hash
			}
		}))
		.await
		.expect("get_transaction_proof should succeed");

	let proof_content = proof_result["content"][0]["text"].as_str().unwrap();
	let proof: serde_json::Value = serde_json::from_str(proof_content)
		.expect("Proof should be valid JSON");

	// Now verify the proof
	let verify_result = ctx
		.mcp_call("tools/call", json!({
			"name": "verify_transaction_proof",
			"arguments": {
				"tx_proof": proof
			}
		}))
		.await
		.expect("verify_transaction_proof should succeed");

	let verify_content = verify_result["content"][0]["text"].as_str().unwrap();
	let tx_hashes: serde_json::Value = serde_json::from_str(verify_content)
		.expect("Response should be valid JSON array");

	// Should return array of transaction hashes
	assert!(tx_hashes.is_array(), "Should return array of tx hashes");
	let tx_array = tx_hashes.as_array().unwrap();
	assert!(!tx_array.is_empty(), "Should have at least one transaction hash");
	assert_eq!(tx_array[0].as_str().unwrap(), genesis_tx_hash, "Should match original tx hash");
}

#[tokio::test]
async fn test_get_transaction_proof_missing_tx_hashes() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_transaction_proof",
			"arguments": {}
		}))
		.await;

	assert!(result.is_err(), "Should fail when tx_hashes is missing");
	let error_msg = result.unwrap_err();
	assert!(error_msg.contains("tx_hashes"), "Error should mention tx_hashes");
}

#[tokio::test]
async fn test_get_transaction_proof_empty_array() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_transaction_proof",
			"arguments": {
				"tx_hashes": []
			}
		}))
		.await;

	assert!(result.is_err(), "Should fail when tx_hashes is empty");
	let error_msg = result.unwrap_err();
	assert!(error_msg.contains("empty"), "Error should mention empty array");
}

#[tokio::test]
async fn test_verify_transaction_proof_missing_proof() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "verify_transaction_proof",
			"arguments": {}
		}))
		.await;

	assert!(result.is_err(), "Should fail when tx_proof is missing");
	let error_msg = result.unwrap_err();
	assert!(error_msg.contains("tx_proof"), "Error should mention tx_proof");
}

