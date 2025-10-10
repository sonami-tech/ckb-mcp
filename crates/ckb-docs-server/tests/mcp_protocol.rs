use test_common::TestContext;
use serde_json::json;




const DOCS_SERVER_PORT: u16 = 8002;

#[tokio::test]
async fn test_03_mcp_initialize() {
	let ctx = TestContext::new(DOCS_SERVER_PORT);

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
		.contains("ckb-docs"));
	assert!(result["capabilities"]["resources"].is_object());
}
