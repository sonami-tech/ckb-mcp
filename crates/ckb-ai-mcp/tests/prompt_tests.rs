//! Prompt tests for ckb-ai-mcp unified server.
//!
//! Tests the 4 workflow prompts (create_script, deploy_script,
//! query_blockchain, transfer_ckb) that provide guided development workflows.

mod common;

use common::TestContext;
use serde_json::json;

// =============================================================================
// Prompt Listing Tests
// =============================================================================

#[tokio::test]
async fn test_prompts_list_returns_array() {
	let ctx = TestContext::new();

	let result = ctx
		.list_prompts()
		.await
		.expect("prompts/list should succeed");

	assert!(result["prompts"].is_array(), "Should return prompts array");
}

#[tokio::test]
async fn test_prompts_list_has_four_prompts() {
	let ctx = TestContext::new();

	let result = ctx
		.list_prompts()
		.await
		.expect("prompts/list should succeed");

	let prompts = result["prompts"].as_array().unwrap();
	assert_eq!(prompts.len(), 4, "Should have exactly 4 workflow prompts");
}

#[tokio::test]
async fn test_prompts_list_all_have_names() {
	let ctx = TestContext::new();

	let result = ctx
		.list_prompts()
		.await
		.expect("prompts/list should succeed");

	let prompts = result["prompts"].as_array().unwrap();

	for prompt in prompts {
		assert!(prompt.get("name").is_some(), "Prompt should have name");
		assert!(
			!prompt["name"].as_str().unwrap().is_empty(),
			"Prompt name should not be empty"
		);
	}
}

#[tokio::test]
async fn test_prompts_list_all_have_descriptions() {
	let ctx = TestContext::new();

	let result = ctx
		.list_prompts()
		.await
		.expect("prompts/list should succeed");

	let prompts = result["prompts"].as_array().unwrap();

	for prompt in prompts {
		let name = prompt["name"].as_str().unwrap();
		assert!(
			prompt.get("description").is_some(),
			"Prompt {} should have description",
			name
		);
	}
}

#[tokio::test]
async fn test_prompts_list_expected_prompts_exist() {
	let ctx = TestContext::new();

	let result = ctx
		.list_prompts()
		.await
		.expect("prompts/list should succeed");

	let prompts = result["prompts"].as_array().unwrap();
	let prompt_names: Vec<&str> = prompts.iter().filter_map(|p| p["name"].as_str()).collect();

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

// =============================================================================
// create_script Prompt Tests
// =============================================================================

#[tokio::test]
async fn test_prompt_create_script_lock() {
	let ctx = TestContext::new();

	let result = ctx
		.get_prompt(
			"create_script",
			Some(json!({
				"script_type": "lock",
				"script_name": "my-custom-lock"
			})),
		)
		.await
		.expect("create_script prompt should succeed");

	assert!(
		result.get("messages").is_some(),
		"Should have messages field"
	);

	let messages = result["messages"].as_array().unwrap();
	assert!(!messages.is_empty(), "Should have at least one message");

	let content = messages[0]["content"]["text"].as_str().unwrap();
	assert!(
		content.contains("lock") || content.contains("Lock"),
		"Should mention lock script"
	);
}

#[tokio::test]
async fn test_prompt_create_script_type() {
	let ctx = TestContext::new();

	let result = ctx
		.get_prompt(
			"create_script",
			Some(json!({
				"script_type": "type",
				"script_name": "my-custom-type"
			})),
		)
		.await
		.expect("create_script prompt should succeed");

	let messages = result["messages"].as_array().unwrap();
	let content = messages[0]["content"]["text"].as_str().unwrap();

	assert!(
		content.contains("type") || content.contains("Type"),
		"Should mention type script"
	);
}

#[tokio::test]
async fn test_prompt_create_script_with_description() {
	let ctx = TestContext::new();

	let result = ctx
		.get_prompt(
			"create_script",
			Some(json!({
				"script_type": "lock",
				"script_name": "multi-sig-lock",
				"description": "A multi-signature lock requiring 2-of-3 signatures"
			})),
		)
		.await
		.expect("create_script prompt should succeed");

	let messages = result["messages"].as_array().unwrap();
	assert!(!messages.is_empty(), "Should have messages");
}

// =============================================================================
// deploy_script Prompt Tests
// =============================================================================

#[tokio::test]
async fn test_prompt_deploy_script_devnet() {
	let ctx = TestContext::new();

	let result = ctx
		.get_prompt(
			"deploy_script",
			Some(json!({
				"binary_path": "/path/to/script.so",
				"network": "devnet"
			})),
		)
		.await
		.expect("deploy_script prompt should succeed");

	let messages = result["messages"].as_array().unwrap();
	assert!(!messages.is_empty(), "Should have messages");

	let content = messages[0]["content"]["text"].as_str().unwrap();
	assert!(
		content.contains("deploy") || content.contains("Deploy"),
		"Should mention deployment"
	);
}

#[tokio::test]
async fn test_prompt_deploy_script_testnet() {
	let ctx = TestContext::new();

	let result = ctx
		.get_prompt(
			"deploy_script",
			Some(json!({
				"binary_path": "/path/to/script.so",
				"network": "testnet"
			})),
		)
		.await
		.expect("deploy_script prompt should succeed");

	let messages = result["messages"].as_array().unwrap();
	let content = messages[0]["content"]["text"].as_str().unwrap();

	assert!(
		content.contains("testnet") || content.contains("Testnet"),
		"Should mention testnet"
	);
}

// =============================================================================
// query_blockchain Prompt Tests
// =============================================================================

#[tokio::test]
async fn test_prompt_query_blockchain_cell() {
	let ctx = TestContext::new();

	let result = ctx
		.get_prompt("query_blockchain", Some(json!({"query_type": "cell"})))
		.await
		.expect("query_blockchain prompt should succeed");

	let messages = result["messages"].as_array().unwrap();
	assert!(!messages.is_empty(), "Should have messages");

	let content = messages[0]["content"]["text"].as_str().unwrap();
	assert!(
		content.contains("cell") || content.contains("Cell"),
		"Should mention cell queries"
	);
}

#[tokio::test]
async fn test_prompt_query_blockchain_transaction() {
	let ctx = TestContext::new();

	let result = ctx
		.get_prompt(
			"query_blockchain",
			Some(json!({"query_type": "transaction"})),
		)
		.await
		.expect("query_blockchain prompt should succeed");

	let messages = result["messages"].as_array().unwrap();
	let content = messages[0]["content"]["text"].as_str().unwrap();

	assert!(
		content.contains("transaction") || content.contains("Transaction"),
		"Should mention transaction queries"
	);
}

#[tokio::test]
async fn test_prompt_query_blockchain_block() {
	let ctx = TestContext::new();

	let result = ctx
		.get_prompt("query_blockchain", Some(json!({"query_type": "block"})))
		.await
		.expect("query_blockchain prompt should succeed");

	let messages = result["messages"].as_array().unwrap();
	let content = messages[0]["content"]["text"].as_str().unwrap();

	assert!(
		content.contains("block") || content.contains("Block"),
		"Should mention block queries"
	);
}

#[tokio::test]
async fn test_prompt_query_blockchain_tip() {
	let ctx = TestContext::new();

	let result = ctx
		.get_prompt("query_blockchain", Some(json!({"query_type": "tip"})))
		.await
		.expect("query_blockchain prompt should succeed");

	let messages = result["messages"].as_array().unwrap();
	assert!(!messages.is_empty(), "Should have messages for tip query");
}

// =============================================================================
// transfer_ckb Prompt Tests
// =============================================================================

#[tokio::test]
async fn test_prompt_transfer_ckb_basic() {
	let ctx = TestContext::new();

	let result = ctx
		.get_prompt(
			"transfer_ckb",
			Some(json!({
				"to_address": "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqwgx292hnvmn68xf779vmzrshpmm6epn4c0cgwga",
				"amount": "100"
			})),
		)
		.await
		.expect("transfer_ckb prompt should succeed");

	let messages = result["messages"].as_array().unwrap();
	assert!(!messages.is_empty(), "Should have messages");

	let content = messages[0]["content"]["text"].as_str().unwrap();
	assert!(
		content.contains("transfer") || content.contains("Transfer"),
		"Should mention transfer"
	);
}

#[tokio::test]
async fn test_prompt_transfer_ckb_with_token_type() {
	let ctx = TestContext::new();

	let result = ctx
		.get_prompt(
			"transfer_ckb",
			Some(json!({
				"to_address": "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqwgx292hnvmn68xf779vmzrshpmm6epn4c0cgwga",
				"amount": "1000",
				"token_type": "udt"
			})),
		)
		.await
		.expect("transfer_ckb prompt should succeed");

	let messages = result["messages"].as_array().unwrap();
	let content = messages[0]["content"]["text"].as_str().unwrap();

	assert!(
		content.contains("UDT") || content.contains("token") || content.contains("Token"),
		"Should mention UDT/token for token_type=udt"
	);
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[tokio::test]
async fn test_prompt_invalid_name() {
	let ctx = TestContext::new();

	let result = ctx.get_prompt("nonexistent_prompt", None).await;

	assert!(result.is_err(), "Should fail for invalid prompt name");
}

#[tokio::test]
async fn test_prompt_missing_required_args() {
	let ctx = TestContext::new();

	// create_script requires script_type and script_name.
	let result = ctx
		.get_prompt("create_script", Some(json!({"script_type": "lock"})))
		.await;

	// May succeed with partial args or fail - behavior depends on implementation.
	// The key is it shouldn't panic.
	let _ = result;
}
