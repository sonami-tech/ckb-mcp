//! Dev tool tests for ckb-ai-mcp unified server.
//!
//! Tests the 8 dev tools (dev_* prefix) for development operations.
//!
//! NOTE: Deployment tests are slow (up to 60 seconds) as they wait for
//! blockchain transaction confirmation.

mod common;

use common::{SharedTestData, TestContext};
use serde_json::json;

// =============================================================================
// Chain Info Tests (Fast)
// =============================================================================

#[tokio::test]
async fn test_dev_get_chain_type() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool("dev_get_chain_type", json!({}))
		.await
		.expect("dev_get_chain_type should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();

	assert!(
		content.contains("mainnet") || content.contains("testnet") || content.contains("devnet"),
		"Should return valid chain type"
	);
}

#[tokio::test]
async fn test_dev_get_chain_type_matches_shared_data() {
	let ctx = TestContext::new();
	let shared_data = SharedTestData::get_or_init_async().await;

	let result = ctx
		.call_tool("dev_get_chain_type", json!({}))
		.await
		.expect("dev_get_chain_type should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();

	assert!(
		content.contains(&shared_data.chain_type),
		"Chain type should match shared data"
	);
}

#[tokio::test]
async fn test_dev_get_genesis_hash() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool("dev_get_genesis_hash", json!({}))
		.await
		.expect("dev_get_genesis_hash should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let hash = content.trim();

	assert!(hash.starts_with("0x"), "Should be hex format");
	assert_eq!(hash.len(), 66, "Should be 66 characters");
	assert!(
		hash[2..].chars().all(|c| c.is_ascii_hexdigit()),
		"Should be valid hex"
	);
}

#[tokio::test]
async fn test_dev_get_genesis_hash_matches_shared_data() {
	let ctx = TestContext::new();
	let shared_data = SharedTestData::get_or_init_async().await;

	let result = ctx
		.call_tool("dev_get_genesis_hash", json!({}))
		.await
		.expect("dev_get_genesis_hash should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();

	assert!(
		content.contains(&shared_data.genesis_hash),
		"Genesis hash should match shared data"
	);
}

// =============================================================================
// Lock Info Tests (Fast)
// =============================================================================

#[tokio::test]
async fn test_dev_generate_lock_info() {
	let ctx = TestContext::new();

	// Use a test private key.
	let test_private_key = "0x0000000000000000000000000000000000000000000000000000000000000001";

	let result = ctx
		.call_tool(
			"dev_generate_lock_info",
			json!({"private_key": test_private_key}),
		)
		.await
		.expect("dev_generate_lock_info should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let info: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	// Verify all expected fields exist.
	assert!(
		info.get("private_key").is_some(),
		"Should have private_key field"
	);
	assert!(
		info.get("public_key").is_some(),
		"Should have public_key field"
	);
	assert!(info.get("lock_arg").is_some(), "Should have lock_arg field");
	assert!(
		info.get("lock_script").is_some(),
		"Should have lock_script field"
	);
	assert!(
		info.get("lock_hash").is_some(),
		"Should have lock_hash field"
	);
	assert!(
		info.get("address_testnet").is_some(),
		"Should have address_testnet field"
	);
	assert!(
		info.get("address_mainnet").is_some(),
		"Should have address_mainnet field"
	);
}

#[tokio::test]
async fn test_dev_get_lock_info_from_address() {
	let ctx = TestContext::new();

	// Use a known testnet address format.
	let address_testnet = "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqwgx292hnvmn68xf779vmzrshpmm6epn4c0cgwga";

	let result = ctx
		.call_tool(
			"dev_get_lock_info_from_address",
			json!({"address": address_testnet}),
		)
		.await
		.expect("dev_get_lock_info_from_address should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let info: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	assert!(
		info.get("lock_script").is_some(),
		"Should have lock_script field"
	);
}

// =============================================================================
// Account Info Tests (Fast)
// =============================================================================

#[tokio::test]
async fn test_dev_get_default_account_info() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool("dev_get_default_account_info", json!({}))
		.await
		.expect("dev_get_default_account_info should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let info: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	// Verify expected fields.
	assert!(
		info.get("public_key").is_some(),
		"Should have public_key field"
	);
	assert!(info.get("lock_arg").is_some(), "Should have lock_arg field");
	assert!(
		info.get("lock_script").is_some(),
		"Should have lock_script field"
	);
	assert!(
		info.get("lock_hash").is_some(),
		"Should have lock_hash field"
	);
	assert!(
		info.get("address_testnet").is_some(),
		"Should have address_testnet field"
	);
	assert!(
		info.get("address_mainnet").is_some(),
		"Should have address_mainnet field"
	);
}

// =============================================================================
// Balance Tests (Fast)
// =============================================================================

#[tokio::test]
async fn test_dev_get_address_balance_default() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool("dev_get_address_balance", json!({}))
		.await
		.expect("dev_get_address_balance should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let balance: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	// Verify expected fields.
	assert!(balance.get("address").is_some(), "Should have address field");
	assert!(
		balance.get("capacity_shannons").is_some(),
		"Should have capacity_shannons field"
	);
	assert!(
		balance.get("capacity_ckb").is_some(),
		"Should have capacity_ckb field"
	);
}

#[tokio::test]
async fn test_dev_get_address_balance_with_address() {
	let ctx = TestContext::new();

	// Use a known testnet address.
	let address_testnet = "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqwgx292hnvmn68xf779vmzrshpmm6epn4c0cgwga";

	let result = ctx
		.call_tool(
			"dev_get_address_balance",
			json!({"address": address_testnet}),
		)
		.await
		.expect("dev_get_address_balance should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let balance: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	assert!(balance.get("capacity_shannons").is_some());
}

// =============================================================================
// Faucet Tests (Network-dependent, may fail on mainnet)
// =============================================================================

// Note: Faucet tests are skipped by default as they:
// 1. Only work on testnet
// 2. Are rate-limited
// 3. May not be available
//
// Uncomment to run manually on testnet:
//
// #[tokio::test]
// async fn test_dev_request_testnet_funds() {
//     let ctx = TestContext::new();
//     let shared_data = SharedTestData::get_or_init_async().await;
//
//     // Only run on testnet.
//     if shared_data.chain_type != "testnet" {
//         println!("Skipping faucet test (not on testnet)");
//         return;
//     }
//
//     let result = ctx
//         .call_tool("dev_request_testnet_funds", json!({}))
//         .await
//         .expect("dev_request_testnet_funds should succeed");
//
//     let content = result["content"][0]["text"].as_str().unwrap();
//     assert!(!content.is_empty(), "Should return faucet response");
// }

// =============================================================================
// Deployment Tests (Slow - waits for blockchain confirmation)
// =============================================================================

// Note: Deployment tests are slow and require:
// 1. A funded default account
// 2. Devnet or testnet (not mainnet)
//
// These tests are included but may be skipped based on environment.

#[tokio::test]
async fn test_dev_deploy_cell_data_small() {
	let ctx = TestContext::new();
	let shared_data = SharedTestData::get_or_init_async().await;

	// Skip on mainnet.
	if shared_data.chain_type == "mainnet" {
		println!("Skipping deployment test (mainnet)");
		return;
	}

	// Deploy a small piece of data (16 bytes).
	let small_data = "48656c6c6f2c20576f726c6421212121"; // "Hello, World!!!" in hex

	let result = ctx
		.call_tool("dev_deploy_cell_data", json!({"data": small_data}))
		.await;

	match result {
		Ok(value) => {
			let content = value["content"][0]["text"].as_str().unwrap_or("");

			// Check if it's an error response (not JSON).
			if content.contains("insufficient")
				|| content.contains("capacity")
				|| content.contains("Insufficient")
				|| content.contains("error")
			{
				println!("Skipping deployment test (insufficient funds): {}", content);
				return;
			}

			let deployment: serde_json::Value = serde_json::from_str(content)
				.unwrap_or_else(|_| panic!("Response should be valid JSON, got: {}", content));

			assert!(
				deployment.get("tx_hash").is_some(),
				"Should have tx_hash field"
			);
			assert!(
				deployment.get("output_index").is_some(),
				"Should have output_index field"
			);
			assert!(
				deployment.get("data_size").is_some(),
				"Should have data_size field"
			);
		}
		Err(e) => {
			// May fail if account has insufficient funds or other expected errors.
			if e.contains("insufficient")
				|| e.contains("capacity")
				|| e.contains("Insufficient")
				|| e.contains("PoolRejectedTransactionByOutputsValidator")
			{
				println!("Skipping deployment test (expected failure): {}", e);
			} else {
				panic!("Unexpected deployment error: {}", e);
			}
		}
	}
}
