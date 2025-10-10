use serde_json::json;

#[path = "../../shared/tests/common/mod.rs"]
mod common;

use common::TestContext;

const RPC_SERVER_PORT: u16 = 8001;

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
async fn test_get_block_economic_state_missing_hash() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_block_economic_state",
			"arguments": {}
		}))
		.await;

	assert!(result.is_err(), "Should fail when block_hash is missing");
	let error_msg = result.unwrap_err();
	assert!(error_msg.contains("block_hash"), "Error should mention block_hash");
}

#[tokio::test]
async fn test_get_block_median_time_missing_hash() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_block_median_time",
			"arguments": {}
		}))
		.await;

	assert!(result.is_err(), "Should fail when block_hash is missing");
	let error_msg = result.unwrap_err();
	assert!(error_msg.contains("block_hash"), "Error should mention block_hash");
}

#[tokio::test]
async fn test_get_block_filter_missing_hash() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_block_filter",
			"arguments": {}
		}))
		.await;

	assert!(result.is_err(), "Should fail when block_hash is missing");
	let error_msg = result.unwrap_err();
	assert!(error_msg.contains("block_hash"), "Error should mention block_hash");
}

