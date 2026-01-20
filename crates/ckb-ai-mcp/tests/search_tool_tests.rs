//! Search tool tests for ckb-ai-mcp unified server.
//!
//! Tests the 2 search tools (search_tools, search_resources) that provide
//! keyword-based discovery of tools and documentation.

mod common;

use common::TestContext;
use serde_json::json;

// =============================================================================
// search_tools Tests
// =============================================================================

#[tokio::test]
async fn test_search_tools_basic() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool("search_tools", json!({"query": "block"}))
		.await
		.expect("search_tools should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let search_result: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	assert!(search_result.get("query").is_some(), "Should have query field");
	assert!(
		search_result.get("total_matches").is_some(),
		"Should have total_matches field"
	);
	assert!(
		search_result.get("results").is_some(),
		"Should have results field"
	);
}

#[tokio::test]
async fn test_search_tools_finds_rpc_tools() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool("search_tools", json!({"query": "rpc block"}))
		.await
		.expect("search_tools should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let search_result: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	let results = search_result["results"].as_array().unwrap();

	// Should find block-related RPC tools.
	let has_block_tool = results
		.iter()
		.any(|r| r["name"].as_str().map(|n| n.contains("block")).unwrap_or(false));

	assert!(has_block_tool, "Should find block-related tools");
}

#[tokio::test]
async fn test_search_tools_finds_dev_tools() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool("search_tools", json!({"query": "deploy"}))
		.await
		.expect("search_tools should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let search_result: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	let results = search_result["results"].as_array().unwrap();

	// Should find deployment tool.
	let has_deploy_tool = results
		.iter()
		.any(|r| r["name"].as_str().map(|n| n.contains("deploy")).unwrap_or(false));

	assert!(has_deploy_tool, "Should find deploy tool");
}

#[tokio::test]
async fn test_search_tools_with_limit() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool("search_tools", json!({"query": "get", "limit": 5}))
		.await
		.expect("search_tools should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let search_result: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	let results = search_result["results"].as_array().unwrap();

	assert!(results.len() <= 5, "Should respect limit parameter");
}

#[tokio::test]
async fn test_search_tools_results_have_score() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool("search_tools", json!({"query": "transaction"}))
		.await
		.expect("search_tools should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let search_result: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	let results = search_result["results"].as_array().unwrap();

	if !results.is_empty() {
		// Each result should have a score.
		for result in results {
			assert!(result.get("score").is_some(), "Result should have score");
			assert!(result.get("name").is_some(), "Result should have name");
			assert!(
				result.get("description").is_some(),
				"Result should have description"
			);
		}
	}
}

#[tokio::test]
async fn test_search_tools_no_results() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool(
			"search_tools",
			json!({"query": "xyznonexistentquery123"}),
		)
		.await
		.expect("search_tools should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let search_result: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	let total_matches = search_result["total_matches"].as_u64().unwrap();
	assert_eq!(total_matches, 0, "Should have zero matches for nonsense query");
}

// =============================================================================
// search_resources Tests
// =============================================================================

#[tokio::test]
async fn test_search_resources_basic() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool("search_resources", json!({"query": "cell"}))
		.await
		.expect("search_resources should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let search_result: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	assert!(search_result.get("query").is_some(), "Should have query field");
	assert!(
		search_result.get("total_matches").is_some(),
		"Should have total_matches field"
	);
	assert!(
		search_result.get("results").is_some(),
		"Should have results field"
	);
}

#[tokio::test]
async fn test_search_resources_finds_cell_model() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool("search_resources", json!({"query": "cell model"}))
		.await
		.expect("search_resources should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let search_result: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	let results = search_result["results"].as_array().unwrap();

	// Should find cell-model resource.
	let has_cell_model = results
		.iter()
		.any(|r| r["uri"].as_str().map(|u| u.contains("cell-model")).unwrap_or(false));

	assert!(has_cell_model, "Should find cell-model resource");
}

#[tokio::test]
async fn test_search_resources_finds_token_docs() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool("search_resources", json!({"query": "token"}))
		.await
		.expect("search_resources should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let search_result: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	let results = search_result["results"].as_array().unwrap();

	// Should find token-related resources.
	let has_token_doc = results
		.iter()
		.any(|r| r["uri"].as_str().map(|u| u.contains("token")).unwrap_or(false));

	assert!(has_token_doc, "Should find token-related documentation");
}

#[tokio::test]
async fn test_search_resources_with_limit() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool("search_resources", json!({"query": "script", "limit": 3}))
		.await
		.expect("search_resources should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let search_result: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	let results = search_result["results"].as_array().unwrap();

	assert!(results.len() <= 3, "Should respect limit parameter");
}

#[tokio::test]
async fn test_search_resources_results_have_uri() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool("search_resources", json!({"query": "omnilock"}))
		.await
		.expect("search_resources should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let search_result: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	let results = search_result["results"].as_array().unwrap();

	if !results.is_empty() {
		for result in results {
			assert!(result.get("uri").is_some(), "Result should have uri");
			assert!(result.get("name").is_some(), "Result should have name");
			assert!(result.get("score").is_some(), "Result should have score");
		}
	}
}

#[tokio::test]
async fn test_search_resources_finds_protocol_docs() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool("search_resources", json!({"query": "protocol spore"}))
		.await
		.expect("search_resources should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let search_result: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	let results = search_result["results"].as_array().unwrap();

	// Should find spore protocol resource.
	let has_spore = results
		.iter()
		.any(|r| r["uri"].as_str().map(|u| u.contains("spore")).unwrap_or(false));

	assert!(has_spore, "Should find spore-related documentation");
}

#[tokio::test]
async fn test_search_resources_no_results() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool(
			"search_resources",
			json!({"query": "xyznonexistentquery456"}),
		)
		.await
		.expect("search_resources should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let search_result: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	let total_matches = search_result["total_matches"].as_u64().unwrap();
	assert_eq!(total_matches, 0, "Should have zero matches for nonsense query");
}

// =============================================================================
// Search Quality Tests
// =============================================================================

#[tokio::test]
async fn test_search_tools_relevance_ordering() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool("search_tools", json!({"query": "rpc_get_block"}))
		.await
		.expect("search_tools should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let search_result: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	let results = search_result["results"].as_array().unwrap();

	if results.len() >= 2 {
		// First result should have higher or equal score than second.
		let first_score = results[0]["score"].as_f64().unwrap();
		let second_score = results[1]["score"].as_f64().unwrap();
		assert!(
			first_score >= second_score,
			"Results should be ordered by score descending"
		);
	}
}

#[tokio::test]
async fn test_search_resources_multi_keyword() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool(
			"search_resources",
			json!({"query": "lock script development"}),
		)
		.await
		.expect("search_resources should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let search_result: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	let total_matches = search_result["total_matches"].as_u64().unwrap();
	assert!(
		total_matches > 0,
		"Should find results for multi-keyword query"
	);
}
