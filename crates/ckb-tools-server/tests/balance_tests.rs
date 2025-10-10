use test_common::{SharedTestData, TestContext};
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
