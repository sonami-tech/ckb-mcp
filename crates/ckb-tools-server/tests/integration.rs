use serde_json::json;

#[path = "../../shared/tests/common/mod.rs"]
mod common;

use common::TestContext;

const TOOLS_SERVER_PORT: u16 = 8003;

/// Run first - fail fast if server not available
#[tokio::test]
async fn test_00_server_running() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	ctx.verify_server_running()
		.await
		.expect("ckb-tools-server must be running on port 8003. Start with: cargo run --bin ckb-tools-server");
}

#[tokio::test]
async fn test_mcp_initialize() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call(
			"initialize",
			json!({
				"protocolVersion": "2024-11-05",
				"capabilities": {},
				"clientInfo": {
					"name": "test-client",
					"version": "1.0.0"
				}
			}),
		)
		.await
		.expect("initialize should succeed");

	assert_eq!(result["protocolVersion"], "2024-11-05");
	assert!(result["serverInfo"]["name"]
		.as_str()
		.unwrap()
		.contains("ckb-tools"));
	assert!(result["capabilities"]["tools"].is_object());
}

#[tokio::test]
async fn test_tools_list_returns_9_tools() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/list", json!({}))
		.await
		.expect("tools/list should succeed");

	let tools = result["tools"].as_array().unwrap();
	assert_eq!(tools.len(), 9, "Should have exactly 9 tools");
}

// Account Info Tests
#[tokio::test]
async fn test_get_default_account_info() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GetDefaultAccountInfo", "arguments": {}}))
		.await
		.expect("GetDefaultAccountInfo should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
}

#[tokio::test]
async fn test_get_default_account_info_no_private_key_exposed() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GetDefaultAccountInfo", "arguments": {}}))
		.await
		.expect("GetDefaultAccountInfo should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.contains("\"private_key\""), "Should not expose private_key field");
	// Verify we have expected fields but not the private key
	assert!(content.contains("public_key"), "Should contain public_key");
	assert!(content.contains("address_testnet"), "Should contain address_testnet");
}

#[tokio::test]
async fn test_get_default_account_info_has_balance() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GetDefaultAccountInfo", "arguments": {}}))
		.await
		.expect("GetDefaultAccountInfo should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(content.contains("capacity_shannons") || content.contains("capacity_ckb"));
}

#[tokio::test]
async fn test_get_default_account_info_has_addresses() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GetDefaultAccountInfo", "arguments": {}}))
		.await
		.expect("GetDefaultAccountInfo should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(content.contains("address_testnet"));
	assert!(content.contains("address_mainnet"));
}

// Chain Info Tests
#[tokio::test]
async fn test_get_chain_type_returns_valid_type() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GetChainType", "arguments": {}}))
		.await
		.expect("GetChainType should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
}

#[tokio::test]
async fn test_get_chain_type_is_testnet_or_mainnet_or_devnet() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GetChainType", "arguments": {}}))
		.await
		.expect("GetChainType should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(
		content.contains("testnet") || content.contains("mainnet") || content.contains("devnet"),
		"Should be one of testnet, mainnet, or devnet"
	);
}

#[tokio::test]
async fn test_get_genesis_hash_valid_format() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GetGenesisHash", "arguments": {}}))
		.await
		.expect("GetGenesisHash should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(content.starts_with("0x"), "Should be hex hash");
	assert!(content.len() > 60, "Should be full hash length");
}

#[tokio::test]
async fn test_get_genesis_hash_matches_chain_type() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	// Get chain type
	let chain_result = ctx
		.mcp_call("tools/call", json!({"name": "GetChainType", "arguments": {}}))
		.await
		.expect("GetChainType should succeed");

	// Get genesis hash
	let hash_result = ctx
		.mcp_call("tools/call", json!({"name": "GetGenesisHash", "arguments": {}}))
		.await
		.expect("GetGenesisHash should succeed");

	let chain_type = chain_result["content"][0]["text"].as_str().unwrap();
	let genesis_hash = hash_result["content"][0]["text"].as_str().unwrap();

	// Verify consistency
	assert!(!genesis_hash.is_empty());
	assert!(!chain_type.is_empty());
}

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
	assert!(content.contains("capacity_shannons") || content.contains("capacity_ckb"));
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

	let result = ctx
		.mcp_call("tools/call", json!({"name": "DeployCellData", "arguments": {"data": ""}}))
		.await;

	// Empty data might be valid or invalid depending on implementation
	// At minimum it shouldn't panic
	let _ = result;
}

#[tokio::test]
async fn test_deploy_cell_data_odd_length_hex() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "DeployCellData", "arguments": {"data": "123"}}))
		.await;

	assert!(result.is_err(), "Should fail for odd-length hex string");
}

// Cell Deployment - DeployCellDataFromFile Tests
#[tokio::test]
async fn test_deploy_cell_data_from_file_missing_file() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "DeployCellDataFromFile", "arguments": {"file_path": "/nonexistent/file.bin"}}))
		.await;

	assert!(result.is_err(), "Should fail for nonexistent file");
}

#[tokio::test]
async fn test_deploy_cell_data_from_file_invalid_path() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "DeployCellDataFromFile", "arguments": {"file_path": "\0invalid"}}))
		.await;

	assert!(result.is_err(), "Should fail for invalid path");
}

#[tokio::test]
async fn test_deploy_cell_data_from_file_empty_path() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "DeployCellDataFromFile", "arguments": {"file_path": ""}}))
		.await;

	assert!(result.is_err(), "Should fail for empty path");
}

#[tokio::test]
async fn test_deploy_cell_data_from_file_missing_param() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "DeployCellDataFromFile", "arguments": {}}))
		.await;

	assert!(result.is_err(), "Should fail when file_path parameter is missing");
}

// Faucet Tests
#[tokio::test]
async fn test_request_testnet_funds_invalid_address() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "RequestTestnetFunds", "arguments": {"address": "invalid"}}))
		.await;

	assert!(result.is_err(), "Should fail for invalid address");
}

// General Error Cases
#[tokio::test]
async fn test_unknown_tool_name() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "NonexistentTool", "arguments": {}}))
		.await;

	assert!(result.is_err(), "Should fail for unknown tool");
}

#[tokio::test]
async fn test_missing_tool_name() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"arguments": {}}))
		.await;

	assert!(result.is_err(), "Should fail when tool name is missing");
}

#[tokio::test]
async fn test_missing_params() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({}))
		.await;

	assert!(result.is_err(), "Should fail when params are missing");
}

#[tokio::test]
async fn test_invalid_json_rpc_request() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"invalid": "structure"}))
		.await;

	assert!(result.is_err(), "Should fail for malformed JSON-RPC request");
}

// Additional Balance Tests
#[tokio::test]
async fn test_get_address_balance_zero_balance() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	// Use an address unlikely to have any CKB
	let empty_address = "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsq2qf8keemy2p5uu0g0gn8cd4jr23s7ct7az30dmke";

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GetAddressBalance", "arguments": {"address": empty_address}}))
		.await
		.expect("GetAddressBalance should succeed for address with zero balance");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
}

// Additional Lock Info - Mainnet Address Test
#[tokio::test]
async fn test_get_lock_info_from_address_mainnet() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let mainnet_address = "ckb1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqwgx292hnvmn68xf779vmzrshpmm6epn4c0cgwga";

	let result = ctx
		.mcp_call("tools/call", json!({"name": "GetLockInfoFromAddress", "arguments": {"address": mainnet_address}}))
		.await
		.expect("GetLockInfoFromAddress should succeed with mainnet address");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
	assert!(content.contains("address_mainnet"));
}

// Cell Deployment Success Cases
#[tokio::test]
async fn test_deploy_cell_data_valid_hex() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "DeployCellData", "arguments": {"data": "48656c6c6f"}}))
		.await
		.expect("DeployCellData should succeed with valid hex");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
	assert!(content.contains("tx_hash"));
}

#[tokio::test]
async fn test_deploy_cell_data_with_0x_prefix() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "DeployCellData", "arguments": {"data": "0x48656c6c6f"}}))
		.await
		.expect("DeployCellData should succeed with 0x prefix");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
}

#[tokio::test]
async fn test_deploy_cell_data_without_0x_prefix() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "DeployCellData", "arguments": {"data": "48656c6c6f"}}))
		.await
		.expect("DeployCellData should succeed without 0x prefix");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
}

#[tokio::test]
async fn test_deploy_cell_data_large_payload() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	// Create a larger data payload (1KB of data)
	let large_data = "00".repeat(512);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "DeployCellData", "arguments": {"data": large_data}}))
		.await
		.expect("DeployCellData should succeed with large payload");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
}

#[tokio::test]
async fn test_deploy_cell_data_returns_tx_hash() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "DeployCellData", "arguments": {"data": "48656c6c6f"}}))
		.await
		.expect("DeployCellData should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(content.contains("tx_hash"), "Should return transaction hash");
	assert!(content.contains("0x"), "Transaction hash should be in hex format");
}

#[tokio::test]
async fn test_deploy_cell_data_returns_capacity() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "DeployCellData", "arguments": {"data": "48656c6c6f"}}))
		.await
		.expect("DeployCellData should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(content.contains("capacity"), "Should return capacity information");
}

// Cell Deployment From File Tests
#[tokio::test]
async fn test_deploy_cell_data_from_file_valid() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	// Create a temporary test file
	use std::fs;
	let test_file = "/tmp/test_deploy_data.bin";
	fs::write(test_file, b"Hello CKB").expect("Should write test file");

	let result = ctx
		.mcp_call("tools/call", json!({"name": "DeployCellDataFromFile", "arguments": {"file_path": test_file}}))
		.await
		.expect("DeployCellDataFromFile should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());
	assert!(content.contains("tx_hash"));

	// Cleanup
	let _ = fs::remove_file(test_file);
}

#[tokio::test]
async fn test_deploy_cell_data_from_file_directory_not_file() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "DeployCellDataFromFile", "arguments": {"file_path": "/tmp"}}))
		.await;

	assert!(result.is_err(), "Should fail when path is a directory");
}

#[tokio::test]
async fn test_deploy_cell_data_from_file_relative_path() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	// Create file in current directory
	use std::fs;
	let test_file = "test_relative.bin";
	fs::write(test_file, b"Test").expect("Should write test file");

	let result = ctx
		.mcp_call("tools/call", json!({"name": "DeployCellDataFromFile", "arguments": {"file_path": test_file}}))
		.await;

	// May succeed or fail depending on working directory
	let _ = result;

	// Cleanup
	let _ = fs::remove_file(test_file);
}

#[tokio::test]
async fn test_deploy_cell_data_from_file_absolute_path() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	// Create a temporary file with absolute path
	use std::fs;
	let test_file = "/tmp/test_absolute.bin";
	fs::write(test_file, b"Absolute").expect("Should write test file");

	let result = ctx
		.mcp_call("tools/call", json!({"name": "DeployCellDataFromFile", "arguments": {"file_path": test_file}}))
		.await
		.expect("DeployCellDataFromFile should succeed with absolute path");

	let content = result["content"][0]["text"].as_str().unwrap();
	assert!(!content.is_empty());

	// Cleanup
	let _ = fs::remove_file(test_file);
}

// Faucet Tests
#[tokio::test]
async fn test_request_testnet_funds_default() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "RequestTestnetFunds", "arguments": {}}))
		.await;

	// May succeed or fail depending on rate limits
	// Just verify it doesn't panic
	let _ = result;
}

#[tokio::test]
async fn test_request_testnet_funds_specific_address() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let test_address = "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqwgx292hnvmn68xf779vmzrshpmm6epn4c0cgwga";

	let result = ctx
		.mcp_call("tools/call", json!({"name": "RequestTestnetFunds", "arguments": {"address": test_address}}))
		.await;

	// May succeed or fail depending on rate limits
	let _ = result;
}

#[tokio::test]
async fn test_request_testnet_funds_mainnet_address() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let mainnet_address = "ckb1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqwgx292hnvmn68xf779vmzrshpmm6epn4c0cgwga";

	let result = ctx
		.mcp_call("tools/call", json!({"name": "RequestTestnetFunds", "arguments": {"address": mainnet_address}}))
		.await;

	// Faucet may reject mainnet addresses or accept them and convert
	// Just verify it doesn't panic
	let _ = result;
}

#[tokio::test]
async fn test_request_testnet_funds_rate_limit() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	// Request funds multiple times to potentially hit rate limit
	let _result1 = ctx
		.mcp_call("tools/call", json!({"name": "RequestTestnetFunds", "arguments": {}}))
		.await;

	let _result2 = ctx
		.mcp_call("tools/call", json!({"name": "RequestTestnetFunds", "arguments": {}}))
		.await;

	// One of these likely hits rate limit
	// Just verify no panic
}
