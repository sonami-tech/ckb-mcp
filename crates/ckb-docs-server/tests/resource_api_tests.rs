use test_common::TestContext;
use serde_json::json;




const DOCS_SERVER_PORT: u16 = 8002;

#[tokio::test]
async fn test_resources_list() {
	let ctx = TestContext::new(DOCS_SERVER_PORT);

	let result = ctx
		.mcp_call("resources/list", json!({}))
		.await
		.expect("resources/list should succeed");

	assert!(result["resources"].is_array());
	let resources = result["resources"].as_array().unwrap();
	assert!(!resources.is_empty(), "Should have documentation resources");
}

#[tokio::test]
async fn test_resources_list_all_have_descriptions() {
	let ctx = TestContext::new(DOCS_SERVER_PORT);

	let result = ctx
		.mcp_call("resources/list", json!({}))
		.await
		.expect("resources/list should succeed");

	let resources = result["resources"].as_array().unwrap();
	for resource in resources {
		let description = resource["description"].as_str().expect("Should have description");
		assert!(!description.is_empty(), "Description should not be empty");
		assert!(description.len() <= 1024, "Description should be under 1024 characters");
	}
}

#[tokio::test]
async fn test_resources_list_all_use_correct_uri_scheme() {
	let ctx = TestContext::new(DOCS_SERVER_PORT);

	let result = ctx
		.mcp_call("resources/list", json!({}))
		.await
		.expect("resources/list should succeed");

	let resources = result["resources"].as_array().unwrap();
	for resource in resources {
		let uri = resource["uri"].as_str().expect("Should have URI");
		assert!(uri.starts_with("ckb-dev-context://"), "URI should use ckb-dev-context:// scheme");
	}
}
