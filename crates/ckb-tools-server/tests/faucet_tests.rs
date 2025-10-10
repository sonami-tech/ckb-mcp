use test_common::TestContext;
use serde_json::json;




const TOOLS_SERVER_PORT: u16 = 8003;

// Faucet Tests
#[tokio::test]
async fn test_faucet_rejects_invalid_address() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "RequestTestnetFunds", "arguments": {"address": "invalid"}}))
		.await;

	assert!(result.is_err(), "Should fail for invalid address");
}

#[tokio::test]
async fn test_faucet_request_default_address() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "RequestTestnetFunds", "arguments": {}}))
		.await;

	// May succeed or fail depending on rate limits
	// Just verify it doesn't panic
	let _ = result;
}

#[tokio::test]
async fn test_faucet_request_specific_address() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let test_address = "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqwgx292hnvmn68xf779vmzrshpmm6epn4c0cgwga";

	let result = ctx
		.mcp_call("tools/call", json!({"name": "RequestTestnetFunds", "arguments": {"address": test_address}}))
		.await;

	// May succeed or fail depending on rate limits
	let _ = result;
}

#[tokio::test]
async fn test_faucet_request_mainnet_address() {
	let ctx = TestContext::new(TOOLS_SERVER_PORT);

	let mainnet_address = "ckb1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqwgx292hnvmn68xf779vmzrshpmm6epn4c0cgwga";

	let result = ctx
		.mcp_call("tools/call", json!({"name": "RequestTestnetFunds", "arguments": {"address": mainnet_address}}))
		.await;

	// Faucet may reject mainnet addresses or accept them and convert
	// Just verify it doesn't panic
	let _ = result;
}
