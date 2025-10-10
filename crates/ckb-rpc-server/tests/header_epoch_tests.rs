use serde_json::json;
use test_common::{SharedTestData, TestContext};


const RPC_SERVER_PORT: u16 = 8001;

#[tokio::test]
async fn test_get_tip_header() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_tip_header", "arguments": {}}))
		.await
		.expect("get_tip_header should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();

	// Parse as JSON object to validate structure
	let _header: serde_json::Value = serde_json::from_str(content)
		.expect("Response should be valid JSON object");
	assert!(_header.is_object(), "Response should be a JSON object");
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
async fn test_get_current_epoch() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_current_epoch", "arguments": {}}))
		.await
		.expect("get_current_epoch should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();

	// Parse as JSON object to validate structure
	let _epoch: serde_json::Value = serde_json::from_str(content)
		.expect("Response should be valid JSON object");
	assert!(_epoch.is_object(), "Response should be a JSON object");
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
async fn test_get_header_invalid_hash() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_header", "arguments": {"block_hash": "not_a_hash"}}))
		.await;

	assert!(result.is_err(), "Should fail for invalid hash");
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

