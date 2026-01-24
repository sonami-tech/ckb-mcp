//! CKB composite tool tests for ckb-ai-mcp unified server.
//!
//! Tests the 5 CKB composite tools (ckb_* prefix) that combine multiple
//! RPC calls into high-level operations.

mod common;

use common::{SharedTestData, TestContext};
use serde_json::json;

// =============================================================================
// ckb_query_chain_status Tests
// =============================================================================

#[tokio::test]
async fn test_ckb_query_chain_status_basic() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool("ckb_query_chain_status", json!({}))
		.await
		.expect("ckb_query_chain_status should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let status: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	// Verify all expected fields from the composite query.
	assert!(status.get("tip").is_some(), "Should have tip field");
	assert!(status.get("sync").is_some(), "Should have sync field");
	assert!(status.get("indexer").is_some(), "Should have indexer field");
	assert!(status.get("mempool").is_some(), "Should have mempool field");
}

#[tokio::test]
async fn test_ckb_query_chain_status_tip_structure() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool("ckb_query_chain_status", json!({}))
		.await
		.expect("ckb_query_chain_status should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let status: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	let tip = &status["tip"];
	assert!(tip.get("number").is_some(), "Tip should have number");
	assert!(tip.get("hash").is_some(), "Tip should have hash");
	assert!(tip.get("timestamp").is_some(), "Tip should have timestamp");
}

#[tokio::test]
async fn test_ckb_query_chain_status_indexer_structure() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool("ckb_query_chain_status", json!({}))
		.await
		.expect("ckb_query_chain_status should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let status: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	let indexer = &status["indexer"];
	assert!(
		indexer.get("block_number").is_some(),
		"Indexer should have block_number"
	);
	assert!(
		indexer.get("block_hash").is_some(),
		"Indexer should have block_hash"
	);
}

// =============================================================================
// ckb_query_address Tests
// =============================================================================

#[tokio::test]
async fn test_ckb_query_address_requires_address() {
	let ctx = TestContext::new();

	// Should fail without address parameter.
	let result = ctx.call_tool("ckb_query_address", json!({})).await;

	// The tool should return an error (either as error result or error in content).
	match result {
		Ok(value) => {
			let content = value["content"][0]["text"].as_str().unwrap_or("");
			assert!(
				content.contains("address") || content.contains("required"),
				"Should indicate address is required"
			);
		}
		Err(e) => {
			assert!(
				e.contains("address") || e.contains("required"),
				"Error should indicate address is required"
			);
		}
	}
}

#[tokio::test]
async fn test_ckb_query_address_with_testnet_address() {
	let ctx = TestContext::new();
	let shared_data = SharedTestData::get_or_init_async().await;

	// Skip on mainnet to avoid using mainnet addresses.
	if shared_data.chain_type == "mainnet" {
		println!("Skipping testnet address test on mainnet");
		return;
	}

	// Use a known testnet address format.
	let address = "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqwgx292hnvmn68xf779vmzrshpmm6epn4c0cgwga";

	let result = ctx
		.call_tool("ckb_query_address", json!({"address": address}))
		.await
		.expect("ckb_query_address should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let query_result: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	assert!(
		query_result.get("address").is_some(),
		"Should have address field"
	);
	assert!(
		query_result.get("lock_script").is_some(),
		"Should have lock_script field"
	);
	assert!(
		query_result.get("capacity").is_some(),
		"Should have capacity field"
	);
}

#[tokio::test]
async fn test_ckb_query_address_with_cells() {
	let ctx = TestContext::new();
	let shared_data = SharedTestData::get_or_init_async().await;

	if shared_data.chain_type == "mainnet" {
		println!("Skipping on mainnet");
		return;
	}

	let address = "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqwgx292hnvmn68xf779vmzrshpmm6epn4c0cgwga";

	let result = ctx
		.call_tool(
			"ckb_query_address",
			json!({
				"address": address,
				"include_cells": true,
				"cell_limit": 5
			}),
		)
		.await
		.expect("ckb_query_address should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let query_result: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	// When include_cells is true, should have cells field.
	assert!(
		query_result.get("cells").is_some(),
		"Should have cells field when include_cells=true"
	);
}

#[tokio::test]
async fn test_ckb_query_address_without_cells() {
	let ctx = TestContext::new();
	let shared_data = SharedTestData::get_or_init_async().await;

	if shared_data.chain_type == "mainnet" {
		println!("Skipping on mainnet");
		return;
	}

	let address = "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqwgx292hnvmn68xf779vmzrshpmm6epn4c0cgwga";

	let result = ctx
		.call_tool(
			"ckb_query_address",
			json!({
				"address": address,
				"include_cells": false
			}),
		)
		.await
		.expect("ckb_query_address should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let query_result: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	// Should still have address and capacity but no cells.
	assert!(query_result.get("address").is_some());
	assert!(query_result.get("capacity").is_some());
}

#[tokio::test]
async fn test_ckb_query_address_invalid_address() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool(
			"ckb_query_address",
			json!({"address": "invalid-address-format"}),
		)
		.await;

	// Should fail for invalid address.
	match result {
		Ok(value) => {
			let content = value["content"][0]["text"].as_str().unwrap_or("");
			assert!(
				content.to_lowercase().contains("invalid")
					|| content.to_lowercase().contains("error"),
				"Should indicate invalid address"
			);
		}
		Err(e) => {
			assert!(
				e.to_lowercase().contains("invalid") || e.to_lowercase().contains("address"),
				"Error should indicate invalid address"
			);
		}
	}
}

// =============================================================================
// ckb_query_transaction Tests
// =============================================================================

#[tokio::test]
async fn test_ckb_query_transaction_genesis_tx() {
	let ctx = TestContext::new();
	let shared_data = SharedTestData::get_or_init_async().await;

	// Get a transaction hash from genesis block.
	let genesis_txs = shared_data.genesis_block["transactions"].as_array();
	if let Some(txs) = genesis_txs
		&& let Some(first_tx) = txs.first()
		&& let Some(tx_hash) = first_tx["hash"].as_str()
	{
		let result = ctx
			.call_tool("ckb_query_transaction", json!({"tx_hash": tx_hash}))
			.await
			.expect("ckb_query_transaction should succeed");

		let content = result["content"][0]["text"].as_str().unwrap();
		let query_result: serde_json::Value =
			serde_json::from_str(content).expect("Response should be valid JSON");

		assert!(
			query_result.get("transaction").is_some(),
			"Should have transaction field"
		);
		assert!(
			query_result.get("resolved_inputs").is_some(),
			"Should have resolved_inputs field"
		);
	}
}

#[tokio::test]
async fn test_ckb_query_transaction_not_found() {
	let ctx = TestContext::new();

	// Query with a non-existent transaction hash.
	let fake_hash = "0x0000000000000000000000000000000000000000000000000000000000000000";

	let result = ctx
		.call_tool("ckb_query_transaction", json!({"tx_hash": fake_hash}))
		.await
		.expect("ckb_query_transaction should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let query_result: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	// For non-existent transaction, the tx_status should be "unknown"
	// and inner transaction should be null.
	let tx = &query_result["transaction"];
	let status = tx["tx_status"]["status"].as_str().unwrap_or("");
	let inner_tx = &tx["transaction"];

	assert!(
		status == "unknown" || inner_tx.is_null(),
		"Non-existent transaction should have unknown status or null transaction"
	);
}

#[tokio::test]
async fn test_ckb_query_transaction_requires_hash() {
	let ctx = TestContext::new();

	let result = ctx.call_tool("ckb_query_transaction", json!({})).await;

	// Should fail without tx_hash.
	match result {
		Ok(value) => {
			let content = value["content"][0]["text"].as_str().unwrap_or("");
			assert!(
				content.contains("tx_hash") || content.contains("required"),
				"Should indicate tx_hash is required"
			);
		}
		Err(e) => {
			assert!(
				e.contains("tx_hash") || e.contains("required"),
				"Error should mention tx_hash"
			);
		}
	}
}

// =============================================================================
// ckb_query_script_cells Tests
// =============================================================================

#[tokio::test]
async fn test_ckb_query_script_cells_by_lock() {
	let ctx = TestContext::new();
	let shared_data = SharedTestData::get_or_init_async().await;

	if shared_data.chain_type == "mainnet" {
		println!("Skipping on mainnet");
		return;
	}

	// Query cells by SECP256K1 lock script (common on all networks).
	// Testnet SECP256K1 code hash.
	let code_hash = "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8";

	let result = ctx
		.call_tool(
			"ckb_query_script_cells",
			json!({
				"script_type": "lock",
				"code_hash": code_hash,
				"hash_type": "type",
				"limit": 5
			}),
		)
		.await
		.expect("ckb_query_script_cells should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let query_result: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	assert!(
		query_result.get("search_key").is_some(),
		"Should have search_key field"
	);
	assert!(
		query_result.get("cells").is_some(),
		"Should have cells field"
	);
	assert!(
		query_result.get("total_capacity").is_some(),
		"Should have total_capacity field"
	);
}

#[tokio::test]
async fn test_ckb_query_script_cells_with_args() {
	let ctx = TestContext::new();
	let shared_data = SharedTestData::get_or_init_async().await;

	if shared_data.chain_type == "mainnet" {
		println!("Skipping on mainnet");
		return;
	}

	let code_hash = "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8";
	// Arbitrary lock args prefix.
	let args = "0x1234";

	let result = ctx
		.call_tool(
			"ckb_query_script_cells",
			json!({
				"script_type": "lock",
				"code_hash": code_hash,
				"hash_type": "type",
				"args": args,
				"limit": 5
			}),
		)
		.await
		.expect("ckb_query_script_cells should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let query_result: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	// Should have search_key showing the args.
	let search_key = &query_result["search_key"];
	assert!(search_key.get("script").is_some());
}

#[tokio::test]
async fn test_ckb_query_script_cells_order_asc() {
	let ctx = TestContext::new();
	let shared_data = SharedTestData::get_or_init_async().await;

	if shared_data.chain_type == "mainnet" {
		println!("Skipping on mainnet");
		return;
	}

	let code_hash = "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8";

	let result = ctx
		.call_tool(
			"ckb_query_script_cells",
			json!({
				"script_type": "lock",
				"code_hash": code_hash,
				"hash_type": "type",
				"order": "asc",
				"limit": 3
			}),
		)
		.await
		.expect("ckb_query_script_cells should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let _query_result: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	// Test passes if query succeeds with asc order.
}

#[tokio::test]
async fn test_ckb_query_script_cells_requires_params() {
	let ctx = TestContext::new();

	// Missing required params should fail.
	let result = ctx.call_tool("ckb_query_script_cells", json!({})).await;

	match result {
		Ok(value) => {
			let content = value["content"][0]["text"].as_str().unwrap_or("");
			assert!(
				content.contains("script_type")
					|| content.contains("code_hash")
					|| content.contains("required"),
				"Should indicate missing parameters"
			);
		}
		Err(e) => {
			assert!(
				e.contains("script_type") || e.contains("code_hash") || e.contains("required"),
				"Error should mention required params"
			);
		}
	}
}

// =============================================================================
// ckb_validate_transaction Tests
// =============================================================================

#[tokio::test]
async fn test_ckb_validate_transaction_requires_tx() {
	let ctx = TestContext::new();

	let result = ctx.call_tool("ckb_validate_transaction", json!({})).await;

	// Should fail without tx parameter.
	match result {
		Ok(value) => {
			let content = value["content"][0]["text"].as_str().unwrap_or("");
			assert!(
				content.contains("tx") || content.contains("required"),
				"Should indicate tx is required"
			);
		}
		Err(e) => {
			assert!(
				e.contains("tx") || e.contains("required"),
				"Error should mention tx parameter"
			);
		}
	}
}

#[tokio::test]
async fn test_ckb_validate_transaction_invalid_tx() {
	let ctx = TestContext::new();

	// Pass an invalid transaction structure.
	let invalid_tx = json!({
		"version": "0x0",
		"inputs": [],
		"outputs": [],
		"outputs_data": []
	});

	let result = ctx
		.call_tool("ckb_validate_transaction", json!({"tx": invalid_tx}))
		.await
		.expect("ckb_validate_transaction should return result");

	let content = result["content"][0]["text"].as_str().unwrap();
	let validation: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	// Should have validation results (even if they indicate errors).
	assert!(
		validation.get("dry_run").is_some(),
		"Should have dry_run field"
	);
	assert!(
		validation.get("cycles").is_some(),
		"Should have cycles field"
	);
	assert!(
		validation.get("fee_rate").is_some(),
		"Should have fee_rate field"
	);

	// The dry_run should indicate failure for an invalid transaction.
	let dry_run = &validation["dry_run"];
	// Note: success could be false due to invalid tx.
	assert!(
		dry_run.get("success").is_some(),
		"dry_run should have success field"
	);
}

// =============================================================================
// Tool Discovery Tests (ensures CKB tools are listed)
// =============================================================================

#[tokio::test]
async fn test_ckb_tools_are_listed() {
	let ctx = TestContext::new();

	let result = ctx.list_tools().await.expect("tools/list should succeed");

	let tools = result["tools"].as_array().unwrap();

	let ckb_tool_names: Vec<&str> = tools
		.iter()
		.filter_map(|t| t["name"].as_str())
		.filter(|n| n.starts_with("ckb_"))
		.collect();

	// Should have all 5 CKB composite tools.
	assert!(
		ckb_tool_names.contains(&"ckb_query_address"),
		"Should have ckb_query_address"
	);
	assert!(
		ckb_tool_names.contains(&"ckb_query_chain_status"),
		"Should have ckb_query_chain_status"
	);
	assert!(
		ckb_tool_names.contains(&"ckb_query_transaction"),
		"Should have ckb_query_transaction"
	);
	assert!(
		ckb_tool_names.contains(&"ckb_validate_transaction"),
		"Should have ckb_validate_transaction"
	);
	assert!(
		ckb_tool_names.contains(&"ckb_query_script_cells"),
		"Should have ckb_query_script_cells"
	);
}

#[tokio::test]
async fn test_ckb_tools_have_annotations() {
	let ctx = TestContext::new();

	let result = ctx.list_tools().await.expect("tools/list should succeed");

	let tools = result["tools"].as_array().unwrap();

	for tool in tools.iter().filter(|t| {
		t["name"]
			.as_str()
			.map(|n| n.starts_with("ckb_"))
			.unwrap_or(false)
	}) {
		let name = tool["name"].as_str().unwrap();

		// All CKB tools should have annotations.
		assert!(
			tool.get("annotations").is_some(),
			"CKB tool {} should have annotations",
			name
		);

		// All CKB query tools should be read-only.
		if name.contains("query") || name.contains("validate") {
			let annotations = &tool["annotations"];
			let read_only = annotations.get("readOnlyHint").and_then(|v| v.as_bool());
			assert_eq!(
				read_only,
				Some(true),
				"CKB query tool {} should be read-only",
				name
			);
		}
	}
}

#[tokio::test]
async fn test_ckb_tools_have_output_schemas() {
	let ctx = TestContext::new();

	let result = ctx.list_tools().await.expect("tools/list should succeed");

	let tools = result["tools"].as_array().unwrap();

	for tool in tools.iter().filter(|t| {
		t["name"]
			.as_str()
			.map(|n| n.starts_with("ckb_"))
			.unwrap_or(false)
	}) {
		let name = tool["name"].as_str().unwrap();

		// All CKB tools should have output schemas (added in protocol 2025-06-18).
		assert!(
			tool.get("outputSchema").is_some(),
			"CKB tool {} should have outputSchema",
			name
		);
	}
}
