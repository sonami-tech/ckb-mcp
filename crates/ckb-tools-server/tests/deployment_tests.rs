use test_common::TestContext;
use serde_json::json;




const TOOLS_SERVER_PORT: u16 = 8003;

/// Extract tx_hash from deployment result JSON and wait for confirmation and indexer sync.
/// Fails if tx_hash is missing or invalid format (must be 0x + 64 hex chars).
async fn wait_for_deployment_confirmation(result: &serde_json::Value) {
	// Extract tx_hash.
	let tx_hash = result
		.get("tx_hash")
		.and_then(|v| v.as_str())
		.expect("Deploy result must contain tx_hash field.");

	// Validate format: "0x" + 64 hex characters = 66 chars total.
	assert!(
		tx_hash.starts_with("0x") && tx_hash.len() == 66 && tx_hash[2..].chars().all(|c| c.is_ascii_hexdigit()),
		"tx_hash must be valid format (0x + 64 hex chars), got: {}",
		tx_hash
	);

	// Wait for transaction to be confirmed and get the block number.
	let block_number = TestContext::wait_for_tx_confirmation(tx_hash)
		.await
		.expect("Transaction should confirm.");

	// Wait for indexer to sync past the confirmation block.
	TestContext::wait_for_indexer_sync(block_number)
		.await
		.expect("Indexer should sync.");
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
		.expect("Empty data should be accepted (decodes to zero bytes).");

	let content = result["content"][0]["text"].as_str().unwrap();

	// Verify deployment succeeded
	assert!(content.contains("tx_hash"), "Should return transaction hash");

	// Parse and validate response structure
	let response: serde_json::Value = serde_json::from_str(content)
		.expect("Response should be valid JSON.");

	let tx_hash = response["tx_hash"].as_str()
		.expect("tx_hash should be present.");
	assert!(tx_hash.starts_with("0x"), "tx_hash should be hex format");
	assert_eq!(tx_hash.len(), 66, "tx_hash should be 66 chars (0x + 64 hex digits)");

	// Verify capacity is present (cells with empty data still need capacity)
	assert!(response.get("capacity").is_some(), "Should include capacity information");

	// Wait for confirmation.
	let deploy_json: serde_json::Value = serde_json::from_str(content)
		.expect("Deploy response should be valid JSON.");
	wait_for_deployment_confirmation(&deploy_json).await;
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
		.expect("DeployCellData should succeed with valid hex.");
	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
	assert!(content.contains("tx_hash"));

	let deploy_json: serde_json::Value = serde_json::from_str(content)
		.expect("Deploy response should be valid JSON.");
	wait_for_deployment_confirmation(&deploy_json).await;
}

#[tokio::test]
async fn test_deploy_cell_data_with_0x_prefix() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let unique_data = format!("0x{:x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
	let result = ctx
		.mcp_call("tools/call", json!({"name": "DeployCellData", "arguments": {"data": unique_data}}))
		.await
		.expect("DeployCellData should succeed with 0x prefix.");
	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());

	let deploy_json: serde_json::Value = serde_json::from_str(content)
		.expect("Deploy response should be valid JSON.");
	wait_for_deployment_confirmation(&deploy_json).await;
}

#[tokio::test]
async fn test_deploy_cell_data_without_0x_prefix() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let unique_data = format!("{:#x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
	let result = ctx
		.mcp_call("tools/call", json!({"name": "DeployCellData", "arguments": {"data": unique_data}}))
		.await
		.expect("DeployCellData should succeed without 0x prefix.");
	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());

	let deploy_json: serde_json::Value = serde_json::from_str(content)
		.expect("Deploy response should be valid JSON.");
	wait_for_deployment_confirmation(&deploy_json).await;
}

#[tokio::test]
async fn test_deploy_cell_data_large_payload() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	// Create a larger data payload (512 bytes) with unique timestamp prefix
	let timestamp = format!("{:#x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
	let large_data = format!("{}{}", timestamp, "00".repeat(512 - (timestamp.len() - 2) / 2));

	let result = ctx
		.mcp_call("tools/call", json!({"name": "DeployCellData", "arguments": {"data": large_data}}))
		.await
		.expect("DeployCellData should succeed with large payload.");
	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());

	let deploy_json: serde_json::Value = serde_json::from_str(content)
		.expect("Deploy response should be valid JSON.");
	wait_for_deployment_confirmation(&deploy_json).await;
}

#[tokio::test]
async fn test_deploy_cell_data_returns_tx_hash() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let unique_data = format!("{:#x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
	let result = ctx
		.mcp_call("tools/call", json!({"name": "DeployCellData", "arguments": {"data": unique_data}}))
		.await
		.expect("DeployCellData should succeed.");
	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(content.contains("tx_hash"), "Should return transaction hash");
	assert!(content.contains("0x"), "Transaction hash should be in hex format");

	let deploy_json: serde_json::Value = serde_json::from_str(content)
		.expect("Deploy response should be valid JSON.");
	wait_for_deployment_confirmation(&deploy_json).await;
}

#[tokio::test]
async fn test_deploy_cell_data_returns_capacity() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let unique_data = format!("{:#x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
	let result = ctx
		.mcp_call("tools/call", json!({"name": "DeployCellData", "arguments": {"data": unique_data}}))
		.await
		.expect("DeployCellData should succeed.");
	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(content.contains("capacity"), "Should return capacity information");

	let deploy_json: serde_json::Value = serde_json::from_str(content)
		.expect("Deploy response should be valid JSON.");
	wait_for_deployment_confirmation(&deploy_json).await;
}

// Size Limit Tests (1KB limit for inline data)
#[tokio::test]
async fn test_deploy_cell_data_at_1kb_limit() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	// Create exactly 1,024 bytes (1KB) with unique timestamp prefix
	let timestamp = format!("{:#x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
	let remaining_bytes = 1024 - ((timestamp.len() - 2) / 2);
	let data_at_limit = format!("{}{}", timestamp, "00".repeat(remaining_bytes));

	let result = ctx
		.mcp_call("tools/call", json!({"name": "DeployCellData", "arguments": {"data": data_at_limit}}))
		.await
		.expect("DeployCellData should succeed at exactly 1KB limit.");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(content.contains("tx_hash"), "Should return transaction hash");

	let deploy_json: serde_json::Value = serde_json::from_str(content)
		.expect("Deploy response should be valid JSON.");
	wait_for_deployment_confirmation(&deploy_json).await;
}

#[tokio::test]
async fn test_deploy_cell_data_just_under_limit() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	// Create 1,023 bytes (just under 1KB limit) with unique timestamp prefix
	let timestamp = format!("{:#x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
	let remaining_bytes = 1023 - ((timestamp.len() - 2) / 2);
	let data_under_limit = format!("{}{}", timestamp, "00".repeat(remaining_bytes));

	let result = ctx
		.mcp_call("tools/call", json!({"name": "DeployCellData", "arguments": {"data": data_under_limit}}))
		.await
		.expect("DeployCellData should succeed just under 1KB limit.");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(content.contains("tx_hash"), "Should return transaction hash");

	let deploy_json: serde_json::Value = serde_json::from_str(content)
		.expect("Deploy response should be valid JSON.");
	wait_for_deployment_confirmation(&deploy_json).await;
}

#[tokio::test]
async fn test_deploy_cell_data_just_over_limit() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	// Create 1,025 bytes (just over 1KB limit)
	let data_over_limit = "00".repeat(1025);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "DeployCellData", "arguments": {"data": data_over_limit}}))
		.await;

	assert!(result.is_err(), "Should fail when data exceeds 1KB limit");
}

#[tokio::test]
async fn test_deploy_cell_data_significantly_over_limit() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	// Create 50KB (significantly over 1KB limit)
	let data_50kb = "00".repeat(51200);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "DeployCellData", "arguments": {"data": data_50kb}}))
		.await;

	assert!(result.is_err(), "Should fail when data significantly exceeds limit");
}

#[tokio::test]
async fn test_deploy_cell_data_error_message_includes_sizes() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	// Create 2KB to trigger error (over 1KB limit)
	let data_2kb = "00".repeat(2048);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "DeployCellData", "arguments": {"data": data_2kb}}))
		.await;

	assert!(result.is_err(), "Should fail for 2KB data");
	let error_msg = format!("{:?}", result.unwrap_err());
	assert!(error_msg.contains("2048"), "Error should include actual size (2048 bytes)");
	assert!(error_msg.contains("1KB"), "Error should mention 1KB limit");
}

#[tokio::test]
async fn test_deploy_cell_data_error_message_suggests_http_endpoint() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	// Create 2KB to trigger error (over 1KB limit)
	let data_2kb = "00".repeat(2048);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "DeployCellData", "arguments": {"data": data_2kb}}))
		.await;

	assert!(result.is_err(), "Should fail for 2KB data");
	let error_msg = format!("{:?}", result.unwrap_err());
	assert!(error_msg.contains("/deploy/file"), "Error should suggest /deploy/file HTTP endpoint");
	assert!(error_msg.contains("curl"), "Error should mention curl as the tool to use");
}

#[tokio::test]
async fn test_deploy_cell_data_768_bytes_under_limit() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	// Create 768 bytes (well under 1KB limit) with unique timestamp prefix
	let timestamp = format!("{:#x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
	let remaining_bytes = 768 - ((timestamp.len() - 2) / 2);
	let data_768 = format!("{}{}", timestamp, "00".repeat(remaining_bytes));

	let result = ctx
		.mcp_call("tools/call", json!({"name": "DeployCellData", "arguments": {"data": data_768}}))
		.await
		.expect("DeployCellData should succeed with 768 bytes data.");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(content.contains("tx_hash"), "Should return transaction hash");

	let deploy_json: serde_json::Value = serde_json::from_str(content)
		.expect("Deploy response should be valid JSON.");
	wait_for_deployment_confirmation(&deploy_json).await;
}

#[tokio::test]
async fn test_deploy_cell_data_10kb_over_limit() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	// Create 10KB (over 1KB limit)
	let data_10kb = "00".repeat(10240);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "DeployCellData", "arguments": {"data": data_10kb}}))
		.await;

	assert!(result.is_err(), "Should fail when data exceeds 1KB limit (10KB test)");
}

