use test_common::{SharedTestData, TestContext};
use serde_json::json;




const TOOLS_SERVER_PORT: u16 = 8003;

// Account Info Tests
#[tokio::test]
async fn test_get_default_account_info() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GetDefaultAccountInfo", "arguments": {}}))
		.await
		.expect("GetDefaultAccountInfo should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
}

#[tokio::test]
async fn test_get_default_account_info_no_private_key_exposed() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GetDefaultAccountInfo", "arguments": {}}))
		.await
		.expect("GetDefaultAccountInfo should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	// GetDefaultAccountInfo intentionally hides private key for security.
	// See test_generate_lock_info_returns_private_key for contrast - GenerateLockInfo
	// returns private key for educational purposes when generating new keys.
	assert!(!content.contains("\"private_key\""), "Should not expose private_key field");
	// Verify we have expected fields but not the private key
	assert!(content.contains("public_key"), "Should contain public_key");
	assert!(content.contains("address_testnet"), "Should contain address_testnet");
}

#[tokio::test]
async fn test_get_default_account_info_has_balance() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GetDefaultAccountInfo", "arguments": {}}))
		.await
		.expect("GetDefaultAccountInfo should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(content.contains("capacity_shannons") || content.contains("capacity_ckb"));
}

#[tokio::test]
async fn test_get_default_account_info_has_addresses() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GetDefaultAccountInfo", "arguments": {}}))
		.await
		.expect("GetDefaultAccountInfo should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(content.contains("address_testnet"));
	assert!(content.contains("address_mainnet"));
}
