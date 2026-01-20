//! Phase tests for ckb-ai-mcp unified server.
//!
//! These tests run in order (enforced by nextest's test-threads = 1):
//! - Phase 1: Verify server is running
//! - Phase 2: Verify CKB RPC is accessible (direct connection)
//! - Phase 3: Collect shared test data
//! - Phase 4: Verify MCP capabilities

mod common;

use common::{get_ckb_rpc_url, SharedTestData, TestContext};
use serde_json::json;

/// Phase 1: Verify MCP server is running.
#[tokio::test]
async fn test_00_server_running() {
	let ctx = TestContext::new();

	ctx.verify_server_running()
		.await
		.expect("ckb-ai-mcp must be running on port 3112. Start with: ./target/release/ckb-ai-mcp");
}

/// Phase 2: Verify CKB RPC is available (direct connection, not through MCP).
#[tokio::test]
async fn test_01_ckb_rpc_available() {
	use reqwest::Client;

	let ckb_rpc_url = get_ckb_rpc_url().expect("CKB_RPC_URL must be set");

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

	assert!(
		body.get("error").is_none(),
		"CKB RPC should not return error"
	);
	assert!(body.get("result").is_some(), "CKB RPC should return result");
}

/// Phase 3: Collect shared test data from CKB RPC (not through MCP).
#[tokio::test]
async fn test_02_collect_shared_data() {
	SharedTestData::initialize()
		.await
		.expect("Should successfully collect shared test data from CKB RPC");

	let data = SharedTestData::get().expect("Shared data should be initialized");

	assert!(!data.chain_type.is_empty(), "Chain type should not be empty");
	assert!(
		data.genesis_hash.starts_with("0x"),
		"Genesis hash should be hex format"
	);
	assert!(
		data.genesis_block.get("header").is_some(),
		"Genesis block should have header"
	);
}

/// Phase 4: Verify tools/list returns tools.
#[tokio::test]
async fn test_03_tools_list() {
	let ctx = TestContext::new();

	let result = ctx.list_tools().await.expect("tools/list should succeed");

	assert!(result["tools"].is_array(), "Should return tools array");
	let tools = result["tools"].as_array().unwrap();
	assert!(!tools.is_empty(), "Should have at least one tool");

	// Verify we have RPC tools (rpc_* prefix).
	let has_rpc_tools = tools
		.iter()
		.any(|t| t["name"].as_str().map(|n| n.starts_with("rpc_")).unwrap_or(false));
	assert!(has_rpc_tools, "Should have RPC tools");

	// Verify we have dev tools (dev_* prefix).
	let has_dev_tools = tools
		.iter()
		.any(|t| t["name"].as_str().map(|n| n.starts_with("dev_")).unwrap_or(false));
	assert!(has_dev_tools, "Should have dev tools");

	// Verify we have search tools.
	let has_search_tools = tools
		.iter()
		.any(|t| t["name"].as_str().map(|n| n.starts_with("search_")).unwrap_or(false));
	assert!(has_search_tools, "Should have search tools");
}

/// Phase 4: Verify resources/list returns resources.
#[tokio::test]
async fn test_04_resources_list() {
	let ctx = TestContext::new();

	let result = ctx
		.list_resources()
		.await
		.expect("resources/list should succeed");

	assert!(
		result["resources"].is_array(),
		"Should return resources array"
	);
	let resources = result["resources"].as_array().unwrap();
	assert!(!resources.is_empty(), "Should have documentation resources");

	// Verify URI scheme.
	let first_uri = resources[0]["uri"].as_str().unwrap();
	assert!(
		first_uri.starts_with("ckb://docs/"),
		"Should use ckb://docs/ URI scheme"
	);
}

/// Phase 4: Verify prompts/list returns prompts.
#[tokio::test]
async fn test_05_prompts_list() {
	let ctx = TestContext::new();

	let result = ctx
		.list_prompts()
		.await
		.expect("prompts/list should succeed");

	assert!(result["prompts"].is_array(), "Should return prompts array");
	let prompts = result["prompts"].as_array().unwrap();
	assert!(!prompts.is_empty(), "Should have workflow prompts");

	// Verify expected prompts exist.
	let prompt_names: Vec<&str> = prompts
		.iter()
		.filter_map(|p| p["name"].as_str())
		.collect();

	assert!(
		prompt_names.contains(&"create_script"),
		"Should have create_script prompt"
	);
	assert!(
		prompt_names.contains(&"deploy_script"),
		"Should have deploy_script prompt"
	);
	assert!(
		prompt_names.contains(&"query_blockchain"),
		"Should have query_blockchain prompt"
	);
	assert!(
		prompt_names.contains(&"transfer_ckb"),
		"Should have transfer_ckb prompt"
	);
}

/// Phase 4: Verify ping endpoint works.
#[tokio::test]
async fn test_06_ping() {
	let ctx = TestContext::new();

	let result = ctx
		.rpc_call("ping", json!({}))
		.await
		.expect("ping should succeed");

	assert!(result.is_object(), "ping should return empty object");
}
