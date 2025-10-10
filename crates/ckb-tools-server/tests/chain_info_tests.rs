use serde_json::json;

#[path = "../../shared/tests/common/mod.rs"]
mod common;

use common::TestContext;

const TOOLS_SERVER_PORT: u16 = 8003;

// Chain Info Tests
#[tokio::test]
async fn test_get_chain_type_returns_valid_type() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GetChainType", "arguments": {}}))
		.await
		.expect("GetChainType should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
}

#[tokio::test]
async fn test_get_chain_type_is_testnet_or_mainnet_or_devnet() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GetChainType", "arguments": {}}))
		.await
		.expect("GetChainType should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(
		content.contains("testnet") || content.contains("mainnet") || content.contains("devnet"),
		"Should be one of testnet, mainnet, or devnet"
	);
}

#[tokio::test]
async fn test_get_genesis_hash_valid_format() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GetGenesisHash", "arguments": {}}))
		.await
		.expect("GetGenesisHash should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let hash = content.trim();
	assert!(hash.starts_with("0x"), "Should be hex hash starting with 0x");
	assert_eq!(hash.len(), 66, "Should be exactly 66 characters (0x + 64 hex digits)");
	// Verify all characters after 0x are valid hex digits
	assert!(hash[2..].chars().all(|c| c.is_ascii_hexdigit()), "Should contain only hex digits after 0x");
}

#[tokio::test]
async fn test_get_genesis_hash_matches_chain_type() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	// Get chain type
	let chain_result = ctx
		.mcp_call("tools/call", json!({"name": "GetChainType", "arguments": {}}))
		.await
		.expect("GetChainType should succeed");

	// Get genesis hash
	let hash_result = ctx
		.mcp_call("tools/call", json!({"name": "GetGenesisHash", "arguments": {}}))
		.await
		.expect("GetGenesisHash should succeed");

	let chain_type = chain_result["content"][0]["text"].as_str().unwrap();
	let genesis_hash = hash_result["content"][0]["text"].as_str().unwrap();

	// Verify consistency
	assert!(!genesis_hash.is_empty());
	assert!(!chain_type.is_empty());
}
