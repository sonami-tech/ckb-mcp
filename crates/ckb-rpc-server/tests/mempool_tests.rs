use serde_json::{json, Value};
use test_common::{SharedTestData, TestContext};

const RPC_SERVER_PORT: u16 = 8001;

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
async fn test_test_tx_pool_accept_missing_tx() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "test_tx_pool_accept", "arguments": {}}))
		.await;

	assert!(result.is_err(), "Should fail when tx parameter is missing");
	let error_msg = result.unwrap_err();
	assert!(
		error_msg.contains("Missing") && error_msg.contains("tx"),
		"Error should mention missing tx parameter, got: {}",
		error_msg
	);
}

#[tokio::test]
async fn test_test_tx_pool_accept_invalid() {
	let ctx = TestContext::new(RPC_SERVER_PORT);
	let shared_data = SharedTestData::get_or_init_async().await;

	// Try to test genesis cellbase which has unresolvable inputs
	let genesis_cellbase_view = &shared_data.genesis_block["transactions"][0];

	// Convert TransactionView to Transaction by removing the hash field
	// test_tx_pool_accept expects Transaction (without hash), not TransactionView
	let mut genesis_cellbase = genesis_cellbase_view.as_object().unwrap().clone();
	genesis_cellbase.remove("hash");
	let genesis_cellbase = Value::Object(genesis_cellbase);

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

