use test_common::TestContext;
use serde_json::json;




const TOOLS_SERVER_PORT: u16 = 8003;

/// Extract tx_hash from deployment result and wait for confirmation and indexer sync
async fn wait_for_deployment_confirmation(content: &str) {
	if let Some(start) = content.find("\"tx_hash\": \"") {
		let hash_start = start + 12;
		if let Some(end) = content[hash_start..].find('"') {
			let tx_hash = &content[hash_start..hash_start + end];

			// Wait for transaction to be confirmed and get the block number
			let block_number = TestContext::wait_for_tx_confirmation(tx_hash)
				.await
				.expect("Transaction should confirm");

			// Wait for indexer to sync past the confirmation block
			TestContext::wait_for_indexer_sync(block_number)
				.await
				.expect("Indexer should sync");
		}
	}
}

// Cell Deployment - DeployCellData Tests
#[tokio::test]
async fn test_deploy_cell_data_invalid_hex() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "DeployCellData", "arguments": {"data": "not_hex"}}))
		.await;

	assert!(result.is_err(), "Should fail for invalid hex");
}

#[tokio::test]
async fn test_deploy_cell_data_missing_data_param() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "DeployCellData", "arguments": {}}))
		.await;

	assert!(result.is_err(), "Should fail when data parameter is missing");
}

#[tokio::test]
async fn test_deploy_cell_data_empty_data() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	// Empty string "" decodes to zero bytes via hex::decode, which is valid
	let result = ctx
		.mcp_call("tools/call", json!({"name": "DeployCellData", "arguments": {"data": ""}}))
		.await
		.expect("Empty data should be accepted (decodes to zero bytes)");

	let content = result["content"][0]["text"].as_str().unwrap();

	// Verify deployment succeeded
	assert!(content.contains("tx_hash"), "Should return transaction hash");

	// Parse and validate response structure
	let response: serde_json::Value = serde_json::from_str(content)
		.expect("Response should be valid JSON");

	let tx_hash = response["tx_hash"].as_str()
		.expect("tx_hash should be present");
	assert!(tx_hash.starts_with("0x"), "tx_hash should be hex format");
	assert_eq!(tx_hash.len(), 66, "tx_hash should be 66 chars (0x + 64 hex digits)");

	// Verify capacity is present (cells with empty data still need capacity)
	assert!(response.get("capacity").is_some(), "Should include capacity information");

	// Wait for confirmation
	wait_for_deployment_confirmation(content).await;
}

#[tokio::test]
async fn test_deploy_cell_data_odd_length_hex() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "DeployCellData", "arguments": {"data": "123"}}))
		.await;

	assert!(result.is_err(), "Should fail for odd-length hex string");
}

// Cell Deployment Success Cases
#[tokio::test]
async fn test_deploy_cell_data_valid_hex() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let unique_data = format!("{:#x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
	let result = ctx
		.mcp_call("tools/call", json!({"name": "DeployCellData", "arguments": {"data": unique_data}}))
		.await
		.expect("DeployCellData should succeed with valid hex");
	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
	assert!(content.contains("tx_hash"));

	wait_for_deployment_confirmation(content).await;
}

#[tokio::test]
async fn test_deploy_cell_data_with_0x_prefix() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let unique_data = format!("0x{:x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
	let result = ctx
		.mcp_call("tools/call", json!({"name": "DeployCellData", "arguments": {"data": unique_data}}))
		.await
		.expect("DeployCellData should succeed with 0x prefix");
	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());

	wait_for_deployment_confirmation(content).await;
}

#[tokio::test]
async fn test_deploy_cell_data_without_0x_prefix() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let unique_data = format!("{:#x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
	let result = ctx
		.mcp_call("tools/call", json!({"name": "DeployCellData", "arguments": {"data": unique_data}}))
		.await
		.expect("DeployCellData should succeed without 0x prefix");
	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());

	wait_for_deployment_confirmation(content).await;
}

#[tokio::test]
async fn test_deploy_cell_data_large_payload() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	// Create a larger data payload (1KB of data) with unique timestamp prefix
	let timestamp = format!("{:#x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
	let large_data = format!("{}{}", timestamp, "00".repeat(512 - timestamp.len() / 2));

	let result = ctx
		.mcp_call("tools/call", json!({"name": "DeployCellData", "arguments": {"data": large_data}}))
		.await
		.expect("DeployCellData should succeed with large payload");
	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());

	wait_for_deployment_confirmation(content).await;
}

#[tokio::test]
async fn test_deploy_cell_data_returns_tx_hash() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let unique_data = format!("{:#x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
	let result = ctx
		.mcp_call("tools/call", json!({"name": "DeployCellData", "arguments": {"data": unique_data}}))
		.await
		.expect("DeployCellData should succeed");
	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(content.contains("tx_hash"), "Should return transaction hash");
	assert!(content.contains("0x"), "Transaction hash should be in hex format");

	wait_for_deployment_confirmation(content).await;
}

#[tokio::test]
async fn test_deploy_cell_data_returns_capacity() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let unique_data = format!("{:#x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
	let result = ctx
		.mcp_call("tools/call", json!({"name": "DeployCellData", "arguments": {"data": unique_data}}))
		.await
		.expect("DeployCellData should succeed");
	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(content.contains("capacity"), "Should return capacity information");

	wait_for_deployment_confirmation(content).await;
}

