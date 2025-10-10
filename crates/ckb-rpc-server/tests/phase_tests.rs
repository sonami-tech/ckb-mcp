use serde_json::json;

#[path = "../../shared/tests/common/mod.rs"]
mod common;

use common::{SharedTestData, TestContext};

const RPC_SERVER_PORT: u16 = 8001;

/// Phase 1: Verify MCP server is running
#[tokio::test]
async fn test_00_server_running() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	ctx.verify_server_running()
		.await
		.expect("ckb-rpc-server must be running on port 8001. Start with: cargo run --bin ckb-rpc-server");
}

/// Phase 2: Verify CKB RPC is available (direct connection, not through MCP)
#[tokio::test]
async fn test_01_ckb_rpc_available() {
	use reqwest::Client;

	let ckb_rpc_url = TestContext::get_ckb_rpc_url()
		.expect("CKB_RPC_URL must be set");

	let client = Client::new();
	let response = client
		.post(&ckb_rpc_url)
		.json(&json!({
			"jsonrpc": "2.0",
			"id": 1,
			"method": "get_tip_block_number",
			"params": []
		}))
		.send()
		.await
		.expect("CKB RPC should be accessible");

	let body: serde_json::Value = response.json().await.expect("Should parse JSON response");

	assert!(body.get("error").is_none(), "CKB RPC should not return error");
	assert!(body.get("result").is_some(), "CKB RPC should return result");
}

/// Phase 3: Collect shared test data from CKB RPC (not through MCP)
#[tokio::test]
async fn test_02_collect_shared_data() {
	SharedTestData::initialize()
		.await
		.expect("Should successfully collect shared test data from CKB RPC");

	let data = SharedTestData::get().expect("Shared data should be initialized");

	// Verify data was collected correctly
	assert!(!data.chain_type.is_empty(), "Chain type should not be empty");
	assert!(data.genesis_hash.starts_with("0x"), "Genesis hash should be hex format");
	assert!(data.genesis_block.get("header").is_some(), "Genesis block should have header");
}
