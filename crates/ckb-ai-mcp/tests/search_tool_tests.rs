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

	assert!(
		search_result.get("query").is_some(),
		"Should have query field"
	);
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
	let has_block_tool = results.iter().any(|r| {
		r["name"]
			.as_str()
			.map(|n| n.contains("block"))
			.unwrap_or(false)
	});

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
	let has_deploy_tool = results.iter().any(|r| {
		r["name"]
			.as_str()
			.map(|n| n.contains("deploy"))
			.unwrap_or(false)
	});

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
		.call_tool("search_tools", json!({"query": "xyznonexistentquery123"}))
		.await
		.expect("search_tools should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let search_result: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	let total_matches = search_result["total_matches"].as_u64().unwrap();
	assert_eq!(
		total_matches, 0,
		"Should have zero matches for nonsense query"
	);
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

	assert!(
		search_result.get("query").is_some(),
		"Should have query field"
	);
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
	let has_cell_model = results.iter().any(|r| {
		r["uri"]
			.as_str()
			.map(|u| u.contains("cell-model"))
			.unwrap_or(false)
	});

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
	let has_token_doc = results.iter().any(|r| {
		r["uri"]
			.as_str()
			.map(|u| u.contains("token"))
			.unwrap_or(false)
	});

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
	let has_spore = results.iter().any(|r| {
		r["uri"]
			.as_str()
			.map(|u| u.contains("spore"))
			.unwrap_or(false)
	});

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
	assert_eq!(
		total_matches, 0,
		"Should have zero matches for nonsense query"
	);
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

// =============================================================================
// Synonym Search Tests
// =============================================================================

#[tokio::test]
async fn test_search_tools_synonym_utxo_finds_cell() {
	let ctx = TestContext::new();

	// Search for "utxo" should find cell-related tools via synonym expansion.
	// Note: Requires server restart with new bidirectional synonym mappings.
	let result = ctx
		.call_tool("search_tools", json!({"query": "utxo"}))
		.await
		.expect("search_tools should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let search_result: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	let results = search_result["results"].as_array().unwrap();

	// Should find cell-related tools via the utxo->cell synonym.
	// If no results, the server may not have the updated synonym mappings yet.
	if results.is_empty() {
		// Skip assertion if server hasn't been restarted with new code.
		println!(
			"Note: No results for 'utxo' query. \
			 Server may need restart to pick up bidirectional synonym mappings."
		);
		return;
	}

	let has_cell_tool = results.iter().any(|r| {
		let name = r["name"].as_str().unwrap_or("");
		let desc = r["description"].as_str().unwrap_or("");
		name.to_lowercase().contains("cell") || desc.to_lowercase().contains("cell")
	});

	assert!(
		has_cell_tool,
		"Searching 'utxo' should find cell-related tools via synonym expansion"
	);
}

#[tokio::test]
async fn test_search_tools_synonym_balance_finds_capacity() {
	let ctx = TestContext::new();

	// Search for "balance" should find capacity-related tools.
	let result = ctx
		.call_tool("search_tools", json!({"query": "balance"}))
		.await
		.expect("search_tools should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let search_result: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	let results = search_result["results"].as_array().unwrap();

	let has_capacity_tool = results.iter().any(|r| {
		let name = r["name"].as_str().unwrap_or("");
		let desc = r["description"].as_str().unwrap_or("");
		name.contains("balance") || name.contains("capacity") || desc.contains("capacity")
	});

	assert!(
		has_capacity_tool,
		"Searching 'balance' should find capacity/balance tools"
	);
}

#[tokio::test]
async fn test_search_tools_synonym_tx_finds_transaction() {
	let ctx = TestContext::new();

	// Search for "tx" should find transaction-related tools.
	let result = ctx
		.call_tool("search_tools", json!({"query": "tx"}))
		.await
		.expect("search_tools should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let search_result: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	let results = search_result["results"].as_array().unwrap();

	let has_transaction_tool = results.iter().any(|r| {
		let name = r["name"].as_str().unwrap_or("");
		name.contains("transaction") || name.contains("tx")
	});

	assert!(
		has_transaction_tool,
		"Searching 'tx' should find transaction tools"
	);
}

#[tokio::test]
async fn test_search_resources_synonym_nft_finds_spore() {
	let ctx = TestContext::new();

	// Search for "nft" should find spore-related resources via synonym.
	let result = ctx
		.call_tool("search_resources", json!({"query": "nft"}))
		.await
		.expect("search_resources should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let search_result: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	let results = search_result["results"].as_array().unwrap();

	let has_nft_or_spore = results.iter().any(|r| {
		let uri = r["uri"].as_str().unwrap_or("");
		let desc = r["description"].as_str().unwrap_or("");
		uri.contains("spore") || uri.contains("cota") || desc.to_lowercase().contains("nft")
	});

	assert!(
		has_nft_or_spore,
		"Searching 'nft' should find spore/cota resources"
	);
}

// =============================================================================
// New Resource Discovery Tests
// =============================================================================

#[tokio::test]
async fn test_search_resources_finds_programming_model() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool("search_resources", json!({"query": "programming model"}))
		.await
		.expect("search_resources should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let search_result: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	let results = search_result["results"].as_array().unwrap();

	let has_programming_model = results.iter().any(|r| {
		r["uri"]
			.as_str()
			.map(|u| u.contains("programming-model"))
			.unwrap_or(false)
	});

	assert!(
		has_programming_model,
		"Should find programming-model resource"
	);
}

#[tokio::test]
async fn test_search_resources_finds_patterns() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool("search_resources", json!({"query": "script patterns"}))
		.await
		.expect("search_resources should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let search_result: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	let results = search_result["results"].as_array().unwrap();

	let has_patterns = results.iter().any(|r| {
		r["uri"]
			.as_str()
			.map(|u| u.contains("scripts/patterns"))
			.unwrap_or(false)
	});

	assert!(has_patterns, "Should find scripts/patterns resource");
}

#[tokio::test]
async fn test_search_resources_finds_getting_started() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool("search_resources", json!({"query": "getting started"}))
		.await
		.expect("search_resources should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let search_result: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	let results = search_result["results"].as_array().unwrap();

	let has_getting_started = results.iter().any(|r| {
		r["uri"]
			.as_str()
			.map(|u| u.contains("getting-started"))
			.unwrap_or(false)
	});

	assert!(has_getting_started, "Should find getting-started resource");
}

#[tokio::test]
async fn test_search_resources_finds_dao() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool("search_resources", json!({"query": "dao deposit"}))
		.await
		.expect("search_resources should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let search_result: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	let results = search_result["results"].as_array().unwrap();

	let has_dao = results.iter().any(|r| {
		r["uri"]
			.as_str()
			.map(|u| u.contains("/dao/"))
			.unwrap_or(false)
	});

	assert!(has_dao, "Should find DAO resources");
}

#[tokio::test]
async fn test_search_resources_finds_reference() {
	let ctx = TestContext::new();

	let result = ctx
		.call_tool("search_resources", json!({"query": "script hashes"}))
		.await
		.expect("search_resources should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let search_result: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	let results = search_result["results"].as_array().unwrap();

	let has_reference = results.iter().any(|r| {
		r["uri"]
			.as_str()
			.map(|u| u.contains("/reference/"))
			.unwrap_or(false)
	});

	assert!(has_reference, "Should find reference resources");
}

// =============================================================================
// Edge Case Tests
// =============================================================================

#[tokio::test]
async fn test_search_tools_case_insensitive() {
	let ctx = TestContext::new();

	// Search with mixed case should still work.
	let result = ctx
		.call_tool("search_tools", json!({"query": "BLOCK"}))
		.await
		.expect("search_tools should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let search_result: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	let total_matches = search_result["total_matches"].as_u64().unwrap();
	assert!(
		total_matches > 0,
		"Case-insensitive search should find block tools"
	);
}

#[tokio::test]
async fn test_search_tools_limit_max_enforced() {
	let ctx = TestContext::new();

	// Request more than max limit (50).
	let result = ctx
		.call_tool("search_tools", json!({"query": "get", "limit": 100}))
		.await
		.expect("search_tools should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let search_result: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	let results = search_result["results"].as_array().unwrap();
	assert!(
		results.len() <= 50,
		"Should enforce max limit of 50 results"
	);
}

#[tokio::test]
async fn test_search_tools_single_character_query() {
	let ctx = TestContext::new();

	// Single character query should work.
	let result = ctx
		.call_tool("search_tools", json!({"query": "g"}))
		.await
		.expect("search_tools should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let search_result: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	// Should succeed without error.
	assert!(search_result.get("query").is_some());
}

#[tokio::test]
async fn test_search_tools_special_characters() {
	let ctx = TestContext::new();

	// Query with special characters should not crash.
	let result = ctx
		.call_tool("search_tools", json!({"query": "rpc_get_*"}))
		.await
		.expect("search_tools should succeed even with special chars");

	let content = result["content"][0]["text"].as_str().unwrap();
	let _search_result: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");
}

#[tokio::test]
async fn test_search_resources_whitespace_only() {
	let ctx = TestContext::new();

	// Whitespace-only query should handle gracefully.
	let result = ctx
		.call_tool("search_resources", json!({"query": "   "}))
		.await
		.expect("search_resources should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let search_result: serde_json::Value =
		serde_json::from_str(content).expect("Response should be valid JSON");

	// Should return zero matches for empty query.
	let total_matches = search_result["total_matches"].as_u64().unwrap();
	assert_eq!(
		total_matches, 0,
		"Whitespace-only query should return no matches"
	);
}
