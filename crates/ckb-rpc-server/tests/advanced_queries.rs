use serde_json::json;
use test_common::TestContext;


const RPC_SERVER_PORT: u16 = 8001;

#[tokio::test]
async fn test_get_fork_block() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	// Use a random hash that's unlikely to be a fork block
	let random_hash = "0x0000000000000000000000000000000000000000000000000000000000000000";

	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_fork_block",
			"arguments": {
				"block_hash": random_hash
			}
		}))
		.await
		.expect("get_fork_block should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let fork_block: serde_json::Value = serde_json::from_str(content)
		.expect("Response should be valid JSON");

	// Most likely null (no fork at this hash)
	// If not null, it would be a block object
	if !fork_block.is_null() {
		assert!(fork_block["header"].is_object(), "Fork block should have header");
	}
}

#[tokio::test]
async fn test_get_fork_block_with_verbosity() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let random_hash = "0x0000000000000000000000000000000000000000000000000000000000000000";

	// Test with verbosity = 0 (hex string format)
	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_fork_block",
			"arguments": {
				"block_hash": random_hash,
				"verbosity": 0
			}
		}))
		.await
		.expect("get_fork_block should succeed with verbosity 0");

	let content = result["content"][0]["text"].as_str().unwrap();
	let fork_block: serde_json::Value = serde_json::from_str(content)
		.expect("Response should be valid JSON");

	// Should be null or a hex string
	if !fork_block.is_null() {
		let block_str = fork_block.as_str().expect("Should be hex string with verbosity 0");
		assert!(block_str.starts_with("0x"), "Should be hex format");
	}
}

#[tokio::test]
async fn test_get_fork_block_missing_hash() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "get_fork_block",
			"arguments": {}
		}))
		.await;

	assert!(result.is_err(), "Should fail when block_hash is missing");
	let error_msg = result.unwrap_err();
	assert!(error_msg.contains("block_hash"), "Error should mention block_hash");
}

