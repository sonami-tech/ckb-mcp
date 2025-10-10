use test_common::{SharedTestData, TestContext};
use serde_json::json;




const TOOLS_SERVER_PORT: u16 = 8003;

// Lock Info - GenerateLockInfo Tests
#[tokio::test]
async fn test_generate_lock_info_missing_key() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GenerateLockInfo", "arguments": {}}))
		.await;

	assert!(result.is_err(), "Should fail when private_key is missing");
}

#[tokio::test]
async fn test_generate_lock_info_with_valid_key() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let test_key = "0xd00c06bfd800d27397002dca6fb0993d5ba6399b4238b2f29ee9deb97593d2bc";

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GenerateLockInfo", "arguments": {"private_key": test_key}}))
		.await
		.expect("GenerateLockInfo should succeed with valid key");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
	assert!(content.contains("lock_script"));
	assert!(content.contains("address_testnet"));
	assert!(content.contains("address_mainnet"));
}

#[tokio::test]
async fn test_generate_lock_info_returns_private_key() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let test_key = "0xd00c06bfd800d27397002dca6fb0993d5ba6399b4238b2f29ee9deb97593d2bc";

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GenerateLockInfo", "arguments": {"private_key": test_key}}))
		.await
		.expect("GenerateLockInfo should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	// GenerateLockInfo intentionally returns private key for educational purposes when
	// users are generating/analyzing new keys. This contrasts with GetDefaultAccountInfo
	// which hides the private key for security. See test_get_default_account_info_no_private_key_exposed.
	assert!(content.contains("private_key"), "GenerateLockInfo returns private key for educational purposes");
}

#[tokio::test]
async fn test_generate_lock_info_invalid_hex() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GenerateLockInfo", "arguments": {"private_key": "not_hex"}}))
		.await;

	assert!(result.is_err(), "Should fail for invalid hex");
}

#[tokio::test]
async fn test_generate_lock_info_invalid_key_length() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GenerateLockInfo", "arguments": {"private_key": "0x1234"}}))
		.await;

	assert!(result.is_err(), "Should fail for invalid key length");
}

#[tokio::test]
async fn test_generate_lock_info_with_0x_prefix() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let test_key = "0xd00c06bfd800d27397002dca6fb0993d5ba6399b4238b2f29ee9deb97593d2bc";

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GenerateLockInfo", "arguments": {"private_key": test_key}}))
		.await
		.expect("GenerateLockInfo should succeed with 0x prefix");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
}

#[tokio::test]
async fn test_generate_lock_info_without_0x_prefix() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let test_key = "d00c06bfd800d27397002dca6fb0993d5ba6399b4238b2f29ee9deb97593d2bc";

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GenerateLockInfo", "arguments": {"private_key": test_key}}))
		.await
		.expect("GenerateLockInfo should succeed without 0x prefix");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
}

#[tokio::test]
async fn test_generate_lock_info_both_addresses_present() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let test_key = "0xd00c06bfd800d27397002dca6fb0993d5ba6399b4238b2f29ee9deb97593d2bc";

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GenerateLockInfo", "arguments": {"private_key": test_key}}))
		.await
		.expect("GenerateLockInfo should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(content.contains("address_testnet"));
	assert!(content.contains("address_mainnet"));
	assert!(content.contains("ckt") || content.contains("ckb"));
}

// Lock Info - GetLockInfoFromAddress Tests
#[tokio::test]
async fn test_get_lock_info_from_address_testnet() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let test_address = "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqwgx292hnvmn68xf779vmzrshpmm6epn4c0cgwga";

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GetLockInfoFromAddress", "arguments": {"address": test_address}}))
		.await
		.expect("GetLockInfoFromAddress should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
}

#[tokio::test]
async fn test_get_lock_info_from_address_no_private_key() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let test_address = "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqwgx292hnvmn68xf779vmzrshpmm6epn4c0cgwga";

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GetLockInfoFromAddress", "arguments": {"address": test_address}}))
		.await
		.expect("GetLockInfoFromAddress should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(content.contains("N/A") || content.contains("Cannot derive"));
}

#[tokio::test]
async fn test_get_lock_info_from_address_invalid_address() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GetLockInfoFromAddress", "arguments": {"address": "invalid_address"}}))
		.await;

	assert!(result.is_err(), "Should fail for invalid address");
}

#[tokio::test]
async fn test_get_lock_info_from_address_malformed() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GetLockInfoFromAddress", "arguments": {"address": "ckt1234"}}))
		.await;

	assert!(result.is_err(), "Should fail for malformed address");
}

#[tokio::test]
async fn test_get_lock_info_from_address_empty() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GetLockInfoFromAddress", "arguments": {"address": ""}}))
		.await;

	assert!(result.is_err(), "Should fail for empty address");
}

#[tokio::test]
async fn test_get_lock_info_from_address_missing_param() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GetLockInfoFromAddress", "arguments": {}}))
		.await;

	assert!(result.is_err(), "Should fail when address parameter is missing");
}

#[tokio::test]
async fn test_get_lock_info_from_address_generates_both_networks() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let test_address = "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqwgx292hnvmn68xf779vmzrshpmm6epn4c0cgwga";

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GetLockInfoFromAddress", "arguments": {"address": test_address}}))
		.await
		.expect("GetLockInfoFromAddress should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(content.contains("address_testnet"));
	assert!(content.contains("address_mainnet"));
}

// Additional Lock Info - Mainnet Address Test
#[tokio::test]
async fn test_get_lock_info_from_address_mainnet() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	// Valid mainnet address derived from private key 0x01
	let mainnet_address = "ckb1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqt4z78ng4yutl5u6xsv27ht6q08mhujf8sy3yulh";

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GetLockInfoFromAddress", "arguments": {"address": mainnet_address}}))
		.await
		.expect("GetLockInfoFromAddress should succeed with mainnet address");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
	assert!(content.contains("address_mainnet"));
}
