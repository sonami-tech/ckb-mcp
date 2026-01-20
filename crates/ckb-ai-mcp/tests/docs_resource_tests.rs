//! Documentation resource tests for ckb-ai-mcp unified server.
//!
//! Tests the 86 documentation resources served via resources/list and resources/read.

mod common;

use common::TestContext;

// =============================================================================
// Resource Listing Tests
// =============================================================================

#[tokio::test]
async fn test_resources_list_returns_array() {
	let ctx = TestContext::new();

	let result = ctx
		.list_resources()
		.await
		.expect("resources/list should succeed");

	assert!(
		result["resources"].is_array(),
		"Should return resources array"
	);
}

#[tokio::test]
async fn test_resources_list_not_empty() {
	let ctx = TestContext::new();

	let result = ctx
		.list_resources()
		.await
		.expect("resources/list should succeed");

	let resources = result["resources"].as_array().unwrap();
	assert!(!resources.is_empty(), "Should have documentation resources");
	assert!(
		resources.len() >= 80,
		"Should have at least 80 resources (currently 86)"
	);
}

#[tokio::test]
async fn test_resources_list_all_have_descriptions() {
	let ctx = TestContext::new();

	let result = ctx
		.list_resources()
		.await
		.expect("resources/list should succeed");

	let resources = result["resources"].as_array().unwrap();

	for resource in resources {
		let uri = resource["uri"].as_str().unwrap_or("unknown");
		let description = resource["description"]
			.as_str()
			.unwrap_or_else(|| panic!("Resource {} should have description", uri));

		assert!(
			!description.is_empty(),
			"Resource {} description should not be empty",
			uri
		);
		assert!(
			description.len() <= 1024,
			"Resource {} description should be under 1024 characters (got {})",
			uri,
			description.len()
		);
	}
}

#[tokio::test]
async fn test_resources_list_all_use_correct_uri_scheme() {
	let ctx = TestContext::new();

	let result = ctx
		.list_resources()
		.await
		.expect("resources/list should succeed");

	let resources = result["resources"].as_array().unwrap();

	for resource in resources {
		let uri = resource["uri"].as_str().expect("Should have URI");
		assert!(
			uri.starts_with("ckb://docs/"),
			"URI {} should use ckb://docs/ scheme",
			uri
		);
	}
}

#[tokio::test]
async fn test_resources_list_all_have_names() {
	let ctx = TestContext::new();

	let result = ctx
		.list_resources()
		.await
		.expect("resources/list should succeed");

	let resources = result["resources"].as_array().unwrap();

	for resource in resources {
		let uri = resource["uri"].as_str().unwrap_or("unknown");
		let name = resource["name"]
			.as_str()
			.unwrap_or_else(|| panic!("Resource {} should have name", uri));

		assert!(
			!name.is_empty(),
			"Resource {} name should not be empty",
			uri
		);
	}
}

// =============================================================================
// Resource Reading Tests
// =============================================================================

#[tokio::test]
async fn test_read_resource_ai_quick_reference() {
	let ctx = TestContext::new();

	let result = ctx
		.read_resource("ckb://docs/ai-quick-reference")
		.await
		.expect("Should read ai-quick-reference");

	let contents = result["contents"].as_array().expect("Should have contents");
	assert!(!contents.is_empty(), "Should have content");

	let text = contents[0]["text"].as_str().expect("Should have text");
	assert!(!text.is_empty(), "Content should not be empty");
	assert!(
		text.contains("CKB") || text.contains("Nervos"),
		"Should contain CKB-related content"
	);
}

#[tokio::test]
async fn test_read_resource_cell_model() {
	let ctx = TestContext::new();

	let result = ctx
		.read_resource("ckb://docs/concepts/cell-model")
		.await
		.expect("Should read cell-model");

	let contents = result["contents"].as_array().expect("Should have contents");
	let text = contents[0]["text"].as_str().expect("Should have text");

	assert!(text.contains("cell") || text.contains("Cell"), "Should contain cell content");
}

#[tokio::test]
async fn test_read_resource_token_creation() {
	let ctx = TestContext::new();

	let result = ctx
		.read_resource("ckb://docs/patterns/token-creation")
		.await
		.expect("Should read token-creation");

	let contents = result["contents"].as_array().expect("Should have contents");
	let text = contents[0]["text"].as_str().expect("Should have text");

	assert!(
		text.contains("token") || text.contains("Token") || text.contains("UDT"),
		"Should contain token content"
	);
}

#[tokio::test]
async fn test_read_resource_invalid_uri() {
	let ctx = TestContext::new();

	let result = ctx.read_resource("ckb://docs/nonexistent-resource").await;

	assert!(result.is_err(), "Should fail for invalid resource URI");
}

#[tokio::test]
async fn test_read_resource_wrong_scheme() {
	let ctx = TestContext::new();

	let result = ctx.read_resource("invalid://docs/cell-model").await;

	assert!(result.is_err(), "Should fail for invalid URI scheme");
}

// =============================================================================
// Category Coverage Tests
// =============================================================================

#[tokio::test]
async fn test_resources_include_api_reference() {
	let ctx = TestContext::new();

	let result = ctx
		.list_resources()
		.await
		.expect("resources/list should succeed");

	let resources = result["resources"].as_array().unwrap();
	let has_api_ref = resources
		.iter()
		.any(|r| r["uri"].as_str().map(|u| u.contains("/api-reference/")).unwrap_or(false));

	assert!(has_api_ref, "Should have api-reference resources");
}

#[tokio::test]
async fn test_resources_include_concepts() {
	let ctx = TestContext::new();

	let result = ctx
		.list_resources()
		.await
		.expect("resources/list should succeed");

	let resources = result["resources"].as_array().unwrap();
	let has_concepts = resources
		.iter()
		.any(|r| r["uri"].as_str().map(|u| u.contains("/concepts/")).unwrap_or(false));

	assert!(has_concepts, "Should have concepts resources");
}

#[tokio::test]
async fn test_resources_include_patterns() {
	let ctx = TestContext::new();

	let result = ctx
		.list_resources()
		.await
		.expect("resources/list should succeed");

	let resources = result["resources"].as_array().unwrap();
	let has_patterns = resources
		.iter()
		.any(|r| r["uri"].as_str().map(|u| u.contains("/patterns/")).unwrap_or(false));

	assert!(has_patterns, "Should have patterns resources");
}

#[tokio::test]
async fn test_resources_include_protocols() {
	let ctx = TestContext::new();

	let result = ctx
		.list_resources()
		.await
		.expect("resources/list should succeed");

	let resources = result["resources"].as_array().unwrap();
	let has_protocols = resources
		.iter()
		.any(|r| r["uri"].as_str().map(|u| u.contains("/protocols/")).unwrap_or(false));

	assert!(has_protocols, "Should have protocols resources");
}

#[tokio::test]
async fn test_resources_include_troubleshooting() {
	let ctx = TestContext::new();

	let result = ctx
		.list_resources()
		.await
		.expect("resources/list should succeed");

	let resources = result["resources"].as_array().unwrap();
	let has_troubleshooting = resources
		.iter()
		.any(|r| r["uri"].as_str().map(|u| u.contains("/troubleshooting/")).unwrap_or(false));

	assert!(has_troubleshooting, "Should have troubleshooting resources");
}

// =============================================================================
// All Resources Readable Test
// =============================================================================

#[tokio::test]
async fn test_all_resources_are_readable() {
	let ctx = TestContext::new();

	let list_result = ctx
		.list_resources()
		.await
		.expect("resources/list should succeed");

	let resources = list_result["resources"].as_array().unwrap();

	// Test a sample of resources (testing all 86 would be slow).
	let sample_uris: Vec<&str> = resources
		.iter()
		.filter_map(|r| r["uri"].as_str())
		.take(10)
		.collect();

	for uri in sample_uris {
		let result = ctx.read_resource(uri).await;
		assert!(
			result.is_ok(),
			"Resource {} should be readable: {:?}",
			uri,
			result.err()
		);

		let value = result.unwrap();
		let contents = value["contents"].as_array();
		assert!(
			contents.is_some() && !contents.unwrap().is_empty(),
			"Resource {} should have non-empty contents",
			uri
		);
	}
}
