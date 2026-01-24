//! RPC tool tests for ckb-ai-mcp unified server.
//!
//! Tests the 36 RPC tools (rpc_* prefix) that query the CKB blockchain.

mod common;

use common::{SharedTestData, TestContext};
use serde_json::json;

// =============================================================================
// Block Query Tests
// =============================================================================

#[tokio::test]
async fn test_rpc_get_tip_block_number() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool("rpc_get_tip_block_number", json!({}))
		.await
		.expect("rpc_get_tip_block_number should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let block_num: String =
		serde_json::from_str(content).expect("Response should be valid JSON string");

	assert!(block_num.starts_with("0x"), "Should be hex format");
	assert!(block_num.len() > 2, "Should have hex digits");
}

#[tokio::test]
async fn test_rpc_get_tip_header() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool("rpc_get_tip_header", json!({}))
		.await
		.expect("rpc_get_tip_header should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let header: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	assert!(header.get("hash").is_some(), "Header should have hash");
	assert!(header.get("number").is_some(), "Header should have number");
	assert!(
		header.get("timestamp").is_some(),
		"Header should have timestamp"
	);
}

#[tokio::test]
async fn test_rpc_get_block_by_number_genesis() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool("rpc_get_block_by_number", json!({"block_number": 0}))
		.await
		.expect("rpc_get_block_by_number should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let block: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	assert!(block.get("header").is_some(), "Block should have header");
	assert!(
		block.get("transactions").is_some(),
		"Block should have transactions"
	);
}

#[tokio::test]
async fn test_rpc_get_block_hash() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool("rpc_get_block_hash", json!({"block_number": 0}))
		.await
		.expect("rpc_get_block_hash should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let hash: String = serde_json::from_str(content).expect("Response should be valid JSON string");

	assert!(hash.starts_with("0x"), "Should be hex format");
	assert_eq!(hash.len(), 66, "Block hash should be 66 characters");
}

#[tokio::test]
async fn test_rpc_get_block_with_hash() {
	let ctx = TestContext::new();
	let shared_data = SharedTestData::get_or_init_async().await;

	let result = ctx
		.call_tool(
			"rpc_get_block",
			json!({"block_hash": shared_data.genesis_hash}),
		)
		.await
		.expect("rpc_get_block should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(content.contains("header"), "Response should contain header");
}

#[tokio::test]
async fn test_rpc_get_header() {
	let ctx = TestContext::new();
	let shared_data = SharedTestData::get_or_init_async().await;

	let result = ctx
		.call_tool(
			"rpc_get_header",
			json!({"block_hash": shared_data.genesis_hash}),
		)
		.await
		.expect("rpc_get_header should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let header: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	assert!(header.get("hash").is_some(), "Header should have hash");
}

#[tokio::test]
async fn test_rpc_get_header_by_number() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool("rpc_get_header_by_number", json!({"block_number": 0}))
		.await
		.expect("rpc_get_header_by_number should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let header: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	assert!(header.get("number").is_some(), "Header should have number");
}

// =============================================================================
// Blockchain Info Tests
// =============================================================================

#[tokio::test]
async fn test_rpc_get_blockchain_info() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool("rpc_get_blockchain_info", json!({}))
		.await
		.expect("rpc_get_blockchain_info should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let info: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	assert!(info.get("chain").is_some(), "Should have chain field");
	assert!(
		info.get("difficulty").is_some(),
		"Should have difficulty field"
	);
	assert!(info.get("epoch").is_some(), "Should have epoch field");
}

#[tokio::test]
async fn test_rpc_get_consensus() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool("rpc_get_consensus", json!({}))
		.await
		.expect("rpc_get_consensus should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let consensus: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	assert!(consensus.get("id").is_some(), "Should have id field");
	assert!(
		consensus.get("genesis_hash").is_some(),
		"Should have genesis_hash field"
	);
}

// =============================================================================
// Epoch Tests
// =============================================================================

#[tokio::test]
async fn test_rpc_get_current_epoch() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool("rpc_get_current_epoch", json!({}))
		.await
		.expect("rpc_get_current_epoch should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let epoch: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	assert!(epoch.get("number").is_some(), "Epoch should have number");
	assert!(
		epoch.get("start_number").is_some(),
		"Epoch should have start_number"
	);
	assert!(epoch.get("length").is_some(), "Epoch should have length");
}

#[tokio::test]
async fn test_rpc_get_epoch_by_number() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool("rpc_get_epoch_by_number", json!({"epoch_number": 0}))
		.await
		.expect("rpc_get_epoch_by_number should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let epoch: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	assert!(epoch.get("number").is_some(), "Epoch should have number");
}

// =============================================================================
// Transaction Pool Tests
// =============================================================================

#[tokio::test]
async fn test_rpc_get_pool_info() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool("rpc_get_pool_info", json!({}))
		.await
		.expect("rpc_get_pool_info should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let pool: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	// Pool info has various fields like tip_hash, pending, proposed.
	assert!(pool.is_object(), "Pool info should be an object");
}

#[tokio::test]
async fn test_rpc_get_pool_ready() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool("rpc_get_pool_ready", json!({}))
		.await
		.expect("rpc_get_pool_ready should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let ready: bool = serde_json::from_str(content).expect("Response should be valid JSON boolean");

	// Pool should typically be ready.
	assert!(ready, "Pool should be ready");
}

// =============================================================================
// Node Info Tests
// =============================================================================

#[tokio::test]
async fn test_rpc_get_node_info() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool("rpc_get_node_info", json!({}))
		.await
		.expect("rpc_get_node_info should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let info: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	assert!(info.get("version").is_some(), "Should have version field");
}

#[tokio::test]
async fn test_rpc_get_peers() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool("rpc_get_peers", json!({}))
		.await
		.expect("rpc_get_peers should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let peers: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	// Peers is an array (may be empty for devnet).
	assert!(peers.is_array(), "Peers should be an array");
}

#[tokio::test]
async fn test_rpc_get_sync_state() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool("rpc_get_sync_state", json!({}))
		.await
		.expect("rpc_get_sync_state should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let state: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	assert!(state.is_object(), "Sync state should be an object");
}

// =============================================================================
// Indexer Tests
// =============================================================================

#[tokio::test]
async fn test_rpc_get_indexer_tip() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool("rpc_get_indexer_tip", json!({}))
		.await
		.expect("rpc_get_indexer_tip should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let tip: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	assert!(
		tip.get("block_number").is_some(),
		"Tip should have block_number"
	);
	assert!(
		tip.get("block_hash").is_some(),
		"Tip should have block_hash"
	);
}

// =============================================================================
// Block Economics Tests
// =============================================================================

#[tokio::test]
async fn test_rpc_get_block_median_time() {
	let ctx = TestContext::new();
	let shared_data = SharedTestData::get_or_init_async().await;

	let result = ctx
		.call_tool(
			"rpc_get_block_median_time",
			json!({"block_hash": shared_data.genesis_hash}),
		)
		.await
		.expect("rpc_get_block_median_time should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let median_time: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	let time_str = median_time.as_str().expect("Should be a string");
	assert!(time_str.starts_with("0x"), "Should be hex format");
}

#[tokio::test]
async fn test_rpc_get_block_economics_genesis() {
	let ctx = TestContext::new();
	let shared_data = SharedTestData::get_or_init_async().await;

	let result = ctx
		.call_tool(
			"rpc_get_block_economics",
			json!({"block_hash": shared_data.genesis_hash}),
		)
		.await
		.expect("rpc_get_block_economics should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let state: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	// Genesis block returns null for economic state.
	assert!(
		state.is_null(),
		"Genesis block should have null economic state"
	);
}
