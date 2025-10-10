use test_common::{SharedTestData, TestContext};
use serde_json::json;




const TOOLS_SERVER_PORT: u16 = 8003;

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
