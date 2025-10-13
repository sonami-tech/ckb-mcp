use serde_json::json;
use test_common::{SharedTestData, TestContext};

const RPC_SERVER_PORT: u16 = 8001;

#[tokio::test]
async fn test_get_tip_block_number() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_tip_block_number", "arguments": {}}))
		.await
		.expect("get_tip_block_number should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();

	// Parse as JSON string and validate hex format
	let block_num: String = serde_json::from_str(content)
		.expect("Response should be valid JSON string");
	assert!(block_num.starts_with("0x"), "Should start with 0x");
	assert!(block_num.len() > 2, "Should have hex digits after 0x");
	assert!(block_num[2..].chars().all(|c| c.is_ascii_hexdigit()), "Should be valid hex number");
}

#[tokio::test]
async fn test_get_block_by_number_genesis() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_block_by_number", "arguments": {"block_number": 0}}))
		.await
		.expect("get_block_by_number genesis should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();

	// Parse as JSON object to validate structure
	let _block: serde_json::Value = serde_json::from_str(content)
		.expect("Response should be valid JSON object");
	assert!(_block.is_object(), "Response should be a JSON object");
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
async fn test_get_block_hash() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_block_hash", "arguments": {"block_number": 0}}))
		.await
		.expect("get_block_hash should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();

	// Parse as JSON string and validate exact block hash format (32 bytes = 64 hex chars)
	let hash: String = serde_json::from_str(content)
		.expect("Response should be valid JSON string");
	assert!(hash.starts_with("0x"), "Should start with 0x");
	assert_eq!(hash.len(), 66, "Block hash should be exactly 66 characters (0x + 64 hex digits)");
	assert!(hash[2..].chars().all(|c| c.is_ascii_hexdigit()), "Should contain only hex digits after 0x");
}

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
async fn test_get_block_economic_state() {
	let ctx = TestContext::new(RPC_SERVER_PORT);
	let shared_data = SharedTestData::get_or_init_async().await;

	// Genesis block returns null for economic state
	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_block_economic_state",
			"arguments": {
				"block_hash": shared_data.genesis_hash
			}
		}))
		.await
		.expect("get_block_economic_state should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let state: serde_json::Value = serde_json::from_str(content)
		.expect("Response should be valid JSON");

	// Genesis block should return null (per documentation)
	assert!(state.is_null(), "Genesis block should have null economic state");
}

#[tokio::test]
async fn test_get_block_median_time() {
	let ctx = TestContext::new(RPC_SERVER_PORT);
	let shared_data = SharedTestData::get_or_init_async().await;

	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_block_median_time",
			"arguments": {
				"block_hash": shared_data.genesis_hash
			}
		}))
		.await
		.expect("get_block_median_time should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let median_time: serde_json::Value = serde_json::from_str(content)
		.expect("Response should be valid JSON");

	// Should return a hex timestamp
	let time_str = median_time.as_str().expect("median_time should be a string");
	assert!(time_str.starts_with("0x"), "median_time should be in hex format");

	// Parse as u64 to verify it's a valid timestamp (genesis has timestamp 0, which is valid)
	let _time_value = u64::from_str_radix(&time_str[2..], 16)
		.expect("median_time should be valid hex number");
}

#[tokio::test]
async fn test_get_block_filter() {
	let ctx = TestContext::new(RPC_SERVER_PORT);
	let shared_data = SharedTestData::get_or_init_async().await;

	// Most blocks return null for block filter (not enabled by default)
	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_block_filter",
			"arguments": {
				"block_hash": shared_data.genesis_hash
			}
		}))
		.await
		.expect("get_block_filter should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let filter: serde_json::Value = serde_json::from_str(content)
		.expect("Response should be valid JSON");

	// Most likely null (block filters not enabled), but could have data/hash fields
	if !filter.is_null() {
		assert!(filter["data"].is_string(), "Filter should have data field");
		assert!(filter["hash"].is_string(), "Filter should have hash field");
	}
}

// Note: Removed mining-related tests (test_get_block_template, test_get_block_template_with_limits)
// These require the Miner RPC module to be enabled on the CKB node, which is typically disabled
// on public devnet/testnet nodes. Mining operations are not relevant for AI-assisted smart contract
// development workflows.

