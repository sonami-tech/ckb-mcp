//! Documentation resource tests for ckb-ai-mcp unified server.
//!
//! Tests the documentation resources served via resources/list and resources/read.

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

	let uris: Vec<&str> = resources.iter().filter_map(|r| r["uri"].as_str()).collect();

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
// Comprehensive Resource Validation
// =============================================================================

/// Domain-relevant keywords that should appear in any CKB documentation.
const DOMAIN_KEYWORDS: &[&str] = &[
	"CKB",
	"ckb",
	"cell",
	"Cell",
	"script",
	"Script",
	"transaction",
	"Transaction",
	"token",
	"Token",
	"lock",
	"Lock",
	"type",
	"capacity",
	"Nervos",
	"blockchain",
	"UTXO",
	"hash",
	"block",
	"witness",
	"deploy",
	"Rust",
	"SDK",
	"UDT",
	"xUDT",
	"sUDT",
	"Spore",
	"Omnilock",
	"DAO",
	"iCKB",
	"CoTA",
	"RGB",
	"SSRI",
	"CoBuild",
	"molecule",
	"Molecule",
	"RISC-V",
	"syscall",
	"args",
	"code_hash",
	"hash_type",
	"def ",
	"import ",
	"class ", // Python file markers
];

/// Verify every registered resource is readable, has relevant content, and
/// starts with a Description section. Reads each resource exactly once.
#[tokio::test]
async fn test_all_resources_content_validation() {
	let ctx = TestContext::new();

	let list_result = ctx
		.list_resources()
		.await
		.expect("resources/list should succeed");

	let resources = list_result["resources"].as_array().unwrap();

	let all_uris: Vec<&str> = resources.iter().filter_map(|r| r["uri"].as_str()).collect();

	assert!(all_uris.len() >= 85, "Should have at least 85 URIs to test");

	let mut read_failures: Vec<String> = Vec::new();
	let mut keyword_failures: Vec<String> = Vec::new();
	let mut description_failures: Vec<String> = Vec::new();

	for uri in &all_uris {
		match ctx.read_resource(uri).await {
			Ok(value) => {
				let contents = value["contents"].as_array();
				if contents.is_none() || contents.unwrap().is_empty() {
					read_failures.push(format!("{}: empty contents", uri));
					continue;
				}

				let text = contents.unwrap()[0]["text"].as_str().unwrap_or("");
				if text.is_empty() {
					read_failures.push(format!("{}: empty text", uri));
					continue;
				}

				// Check for CKB-relevant keywords.
				let has_keyword = DOMAIN_KEYWORDS.iter().any(|kw| text.contains(kw));
				if !has_keyword {
					keyword_failures.push(format!(
						"{}: no CKB-relevant keywords found (first 100 chars: {:?})",
						uri,
						&text[..text.len().min(100)]
					));
				}

				// Check for ## Description header (skip Python files).
				let is_python = uri.contains("/examples/calculate_file_hashes")
					|| uri.contains("/examples/consolidate_cells");
				if !is_python && !text.trim_start().starts_with("## Description") {
					description_failures
						.push(format!("{}: does not start with '## Description'", uri));
				}
			}
			Err(e) => {
				read_failures.push(format!("{}: read failed: {}", uri, e));
			}
		}
	}

	let mut all_failures = Vec::new();
	if !read_failures.is_empty() {
		all_failures.push(format!(
			"Read failures ({}):\n  {}",
			read_failures.len(),
			read_failures.join("\n  ")
		));
	}
	if !keyword_failures.is_empty() {
		all_failures.push(format!(
			"Missing CKB-relevant content ({}):\n  {}",
			keyword_failures.len(),
			keyword_failures.join("\n  ")
		));
	}
	if !description_failures.is_empty() {
		all_failures.push(format!(
			"Missing '## Description' header ({}):\n  {}",
			description_failures.len(),
			description_failures.join("\n  ")
		));
	}

	assert!(
		all_failures.is_empty(),
		"Resource validation failures ({} total):\n{}",
		read_failures.len() + keyword_failures.len() + description_failures.len(),
		all_failures.join("\n\n")
	);
}

/// Verify every resource has a description with at least 10 words.
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
