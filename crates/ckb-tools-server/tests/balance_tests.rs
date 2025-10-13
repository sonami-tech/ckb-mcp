use test_common::TestContext;
use serde_json::json;




const TOOLS_SERVER_PORT: u16 = 8003;

// Balance Tests
#[tokio::test]
async fn test_get_address_balance_default() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GetAddressBalance", "arguments": {}}))
		.await
		.expect("GetAddressBalance should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
	assert!(content.contains("capacity"));
}

#[tokio::test]
async fn test_get_address_balance_specific_address() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let test_address = "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqwgx292hnvmn68xf779vmzrshpmm6epn4c0cgwga";

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GetAddressBalance", "arguments": {"address": test_address}}))
		.await
		.expect("GetAddressBalance should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
}

#[tokio::test]
async fn test_get_address_balance_invalid_address() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GetAddressBalance", "arguments": {"address": "invalid"}}))
		.await;

	assert!(result.is_err(), "Should fail for invalid address");
}

#[tokio::test]
async fn test_get_address_balance_malformed_address() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GetAddressBalance", "arguments": {"address": "ckt123"}}))
		.await;

	assert!(result.is_err(), "Should fail for malformed address");
}

#[tokio::test]
async fn test_get_address_balance_empty_address() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GetAddressBalance", "arguments": {"address": ""}}))
		.await;

	assert!(result.is_err(), "Should fail for empty address");
}

#[tokio::test]
async fn test_get_address_balance_has_ckb_and_shannon() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GetAddressBalance", "arguments": {}}))
		.await
		.expect("GetAddressBalance should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(content.contains("capacity_shannons"), "Should contain capacity_shannons field");
	assert!(content.contains("capacity_ckb"), "Should contain capacity_ckb field");
}

// Additional Balance Tests
#[tokio::test]
async fn test_get_address_balance_zero_balance() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	// Use an address unlikely to have any CKB (derived from private key 0x01)
	// This address has valid checksum but likely zero balance
	let empty_address = "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqt4z78ng4yutl5u6xsv27ht6q08mhujf8s2r0n40";

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GetAddressBalance", "arguments": {"address": empty_address}}))
		.await
		.expect("GetAddressBalance should succeed for address with zero balance");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
}

#[tokio::test]
async fn test_get_address_balance_has_capacity_breakdown() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GetAddressBalance", "arguments": {}}))
		.await
		.expect("GetAddressBalance should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();

	// Verify all capacity fields are present
	assert!(content.contains("capacity_shannons"), "Should contain total capacity_shannons");
	assert!(content.contains("capacity_ckb"), "Should contain total capacity_ckb");
	assert!(content.contains("free_capacity_shannons"), "Should contain free_capacity_shannons");
	assert!(content.contains("free_capacity_ckb"), "Should contain free_capacity_ckb");
	assert!(content.contains("occupied_capacity_shannons"), "Should contain occupied_capacity_shannons");
	assert!(content.contains("occupied_capacity_ckb"), "Should contain occupied_capacity_ckb");

	// Parse JSON and verify math: total = free + occupied
	let balance_info: serde_json::Value = serde_json::from_str(content).expect("Should parse as JSON");
	let total = balance_info["capacity_shannons"].as_u64().expect("total should be u64");
	let free = balance_info["free_capacity_shannons"].as_u64().expect("free should be u64");
	let occupied = balance_info["occupied_capacity_shannons"].as_u64().expect("occupied should be u64");

	assert_eq!(total, free + occupied, "Total capacity should equal free + occupied");
}
