//! Documentation resource tests for ckb-ai-mcp unified server.
//!
//! Tests the 89 documentation resources served via resources/list and resources/read.

mod common;

use common::TestContext;
use std::collections::HashSet;

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
		resources.len() >= 85,
		"Should have at least 85 resources (currently 89)"
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

#[tokio::test]
async fn test_resources_list_uris_are_unique() {
	let ctx = TestContext::new();

	let result = ctx
		.list_resources()
		.await
		.expect("resources/list should succeed");

	let resources = result["resources"].as_array().unwrap();

	let uris: Vec<&str> = resources
		.iter()
		.filter_map(|r| r["uri"].as_str())
		.collect();

	let unique: HashSet<&str> = uris.iter().copied().collect();
	assert_eq!(
		uris.len(),
		unique.len(),
		"All resource URIs should be unique (found {} duplicates)",
		uris.len() - unique.len()
	);
}

// =============================================================================
// Resource Reading Tests
// =============================================================================

#[tokio::test]
async fn test_read_resource_ai_quick_reference() {
	let ctx = TestContext::new();

	let result = ctx
		.read_resource("ckb://docs/quickstart/ai-quick-reference")
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

	assert!(
		text.contains("cell") || text.contains("Cell"),
		"Should contain cell content"
	);
}

#[tokio::test]
async fn test_read_resource_token_creation() {
	let ctx = TestContext::new();

	let result = ctx
		.read_resource("ckb://docs/tokens/token-creation")
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
async fn test_read_resource_programming_model() {
	let ctx = TestContext::new();

	let result = ctx
		.read_resource("ckb://docs/concepts/programming-model")
		.await
		.expect("Should read programming-model");

	let contents = result["contents"].as_array().expect("Should have contents");
	let text = contents[0]["text"].as_str().expect("Should have text");

	assert!(
		text.contains("UTXO") || text.contains("cell") || text.contains("transaction"),
		"Should contain CKB programming model content"
	);
}

#[tokio::test]
async fn test_read_resource_patterns() {
	let ctx = TestContext::new();

	let result = ctx
		.read_resource("ckb://docs/scripts/patterns")
		.await
		.expect("Should read patterns");

	let contents = result["contents"].as_array().expect("Should have contents");
	let text = contents[0]["text"].as_str().expect("Should have text");

	assert!(
		text.contains("script") || text.contains("Script") || text.contains("pattern"),
		"Should contain script pattern content"
	);
}

#[tokio::test]
async fn test_read_resource_getting_started() {
	let ctx = TestContext::new();

	let result = ctx
		.read_resource("ckb://docs/quickstart/getting-started")
		.await
		.expect("Should read getting-started");

	let contents = result["contents"].as_array().expect("Should have contents");
	let text = contents[0]["text"].as_str().expect("Should have text");

	assert!(
		text.contains("CCC") || text.contains("SDK") || text.contains("Rust"),
		"Should contain getting-started content"
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

/// Verify all expected resource categories are present.
#[tokio::test]
async fn test_resources_include_all_categories() {
	let ctx = TestContext::new();

	let result = ctx
		.list_resources()
		.await
		.expect("resources/list should succeed");

	let resources = result["resources"].as_array().unwrap();

	let expected_categories = [
		("quickstart", 2),
		("concepts", 11),
		("scripts", 11),
		("tokens", 5),
		("omnilock", 6),
		("spore", 5),
		("cota", 4),
		("dao", 2),
		("ickb", 4),
		("protocols", 5),
		("sdk", 9),
		("transactions", 5),
		("reference", 5),
		("troubleshooting", 4),
		("tools", 5),
		("ecosystem", 3),
		("examples", 3),
	];

	for (category, min_count) in &expected_categories {
		let pattern = format!("/{}/", category);
		let count = resources
			.iter()
			.filter(|r| {
				r["uri"]
					.as_str()
					.map(|u| u.contains(&pattern))
					.unwrap_or(false)
			})
			.count();

		assert!(
			count >= *min_count,
			"Category '{}' should have at least {} resources, found {}",
			category,
			min_count,
			count
		);
	}
}

// =============================================================================
// All Resources Readable Test
// =============================================================================

/// Verify every registered resource can be read with non-empty content.
#[tokio::test]
async fn test_all_resources_are_readable() {
	let ctx = TestContext::new();

	let list_result = ctx
		.list_resources()
		.await
		.expect("resources/list should succeed");

	let resources = list_result["resources"].as_array().unwrap();

	let all_uris: Vec<&str> = resources
		.iter()
		.filter_map(|r| r["uri"].as_str())
		.collect();

	assert!(
		all_uris.len() >= 85,
		"Should have at least 85 URIs to test"
	);

	let mut failures: Vec<String> = Vec::new();

	for uri in &all_uris {
		match ctx.read_resource(uri).await {
			Ok(value) => {
				let contents = value["contents"].as_array();
				if contents.is_none() || contents.unwrap().is_empty() {
					failures.push(format!("{}: empty contents", uri));
				} else {
					let text = contents.unwrap()[0]["text"].as_str().unwrap_or("");
					if text.is_empty() {
						failures.push(format!("{}: empty text", uri));
					}
				}
			}
			Err(e) => {
				failures.push(format!("{}: read failed: {}", uri, e));
			}
		}
	}

	assert!(
		failures.is_empty(),
		"Failed resources ({}/{}):\n  {}",
		failures.len(),
		all_uris.len(),
		failures.join("\n  ")
	);
}

/// Verify every resource has a description that contains meaningful words.
///
/// Descriptions are the primary search vector for AI discovery. They should
/// contain substantive keywords, not just boilerplate.
#[tokio::test]
async fn test_all_descriptions_contain_substantive_keywords() {
	let ctx = TestContext::new();

	let result = ctx
		.list_resources()
		.await
		.expect("resources/list should succeed");

	let resources = result["resources"].as_array().unwrap();

	let mut failures: Vec<String> = Vec::new();

	for resource in resources {
		let uri = resource["uri"].as_str().unwrap_or("unknown");
		let description = resource["description"].as_str().unwrap_or("");

		// Description should have at least 10 words for meaningful search matching.
		let word_count = description.split_whitespace().count();
		if word_count < 10 {
			failures.push(format!(
				"{}: description has only {} words (minimum 10)",
				uri, word_count
			));
		}
	}

	assert!(
		failures.is_empty(),
		"Resources with insufficient descriptions ({}):\n  {}",
		failures.len(),
		failures.join("\n  ")
	);
}

/// Verify all resource content contains CKB-relevant keywords.
///
/// Every documentation resource should contain at least one domain-relevant
/// keyword to ensure it has substantive CKB development content.
#[tokio::test]
async fn test_all_resources_have_relevant_content() {
	let ctx = TestContext::new();

	let list_result = ctx
		.list_resources()
		.await
		.expect("resources/list should succeed");

	let resources = list_result["resources"].as_array().unwrap();

	// Domain-relevant keywords that should appear in any CKB documentation.
	let domain_keywords = [
		"CKB", "ckb", "cell", "Cell", "script", "Script", "transaction", "Transaction",
		"token", "Token", "lock", "Lock", "type", "capacity", "Nervos", "blockchain",
		"UTXO", "hash", "block", "witness", "deploy", "Rust", "SDK", "UDT", "xUDT",
		"sUDT", "Spore", "Omnilock", "DAO", "iCKB", "CoTA", "RGB", "SSRI", "CoBuild",
		"molecule", "Molecule", "RISC-V", "syscall", "args", "code_hash", "hash_type",
		"def ", "import ", "class ",  // Python file markers
	];

	let mut failures: Vec<String> = Vec::new();

	for resource in resources {
		let uri = resource["uri"].as_str().unwrap_or("unknown");

		if let Ok(value) = ctx.read_resource(uri).await {
			if let Some(contents) = value["contents"].as_array() {
				if let Some(text) = contents.first().and_then(|c| c["text"].as_str()) {
					let has_keyword = domain_keywords
						.iter()
						.any(|kw| text.contains(kw));

					if !has_keyword {
						failures.push(format!(
							"{}: no CKB-relevant keywords found (first 100 chars: {:?})",
							uri,
							&text[..text.len().min(100)]
						));
					}
				}
			}
		}
	}

	assert!(
		failures.is_empty(),
		"Resources without CKB-relevant content ({}):\n  {}",
		failures.len(),
		failures.join("\n  ")
	);
}

/// Verify all resource content starts with a Description section.
#[tokio::test]
async fn test_all_resources_start_with_description() {
	let ctx = TestContext::new();

	let list_result = ctx
		.list_resources()
		.await
		.expect("resources/list should succeed");

	let resources = list_result["resources"].as_array().unwrap();

	let mut failures: Vec<String> = Vec::new();

	for resource in resources {
		let uri = resource["uri"].as_str().unwrap_or("unknown");

		// Skip Python files — they don't follow markdown format.
		if uri.contains("/examples/calculate_file_hashes")
			|| uri.contains("/examples/consolidate_cells")
		{
			continue;
		}

		if let Ok(value) = ctx.read_resource(uri).await {
			if let Some(contents) = value["contents"].as_array() {
				if let Some(text) = contents.first().and_then(|c| c["text"].as_str()) {
					if !text.trim_start().starts_with("## Description") {
						failures.push(format!(
							"{}: does not start with '## Description'",
							uri
						));
					}
				}
			}
		}
	}

	assert!(
		failures.is_empty(),
		"Resources missing '## Description' header ({}):\n  {}",
		failures.len(),
		failures.join("\n  ")
	);
}
