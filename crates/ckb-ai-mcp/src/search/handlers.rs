//! Search tool handlers.

use rmcp::model::{CallToolResult, Content, Resource, Tool};
use serde::{Deserialize, Serialize};
use shared::error::{CkbMcpError, Result};
use std::collections::HashMap;
use std::sync::LazyLock;
use tracing::debug;

/// Domain-specific synonyms for CKB development.
/// Maps keywords to their synonyms for improved search discoverability.
/// Note: Bidirectional mappings are needed for proper search (e.g., utxo->cell and cell->utxo).
static SYNONYMS: LazyLock<HashMap<&'static str, &'static [&'static str]>> = LazyLock::new(|| {
	HashMap::from([
		// Action synonyms
		(
			"deploy",
			&["submit", "upload", "publish", "send", "broadcast"][..],
		),
		("submit", &["deploy", "send", "broadcast", "push"][..]),
		(
			"query",
			&["get", "fetch", "read", "retrieve", "search", "find"][..],
		),
		("get", &["query", "fetch", "read", "retrieve"][..]),
		("create", &["make", "build", "generate", "new", "mint"][..]),
		("validate", &["verify", "check", "test"][..]),
		("transfer", &["send", "move", "pay"][..]),
		// CKB domain synonyms - bidirectional mappings
		("cell", &["utxo", "output", "coin"][..]),
		("cells", &["utxos", "outputs", "coins"][..]),
		("utxo", &["cell", "output"][..]),
		("utxos", &["cells", "outputs"][..]),
		("output", &["cell", "utxo"][..]),
		("lock", &["script", "guard", "owner"][..]),
		("type", &["script", "validator", "contract"][..]),
		("capacity", &["balance", "amount", "ckb", "value"][..]),
		("balance", &["capacity", "amount", "funds"][..]),
		("ckb", &["capacity", "balance", "nervos"][..]),
		("transaction", &["tx", "transfer"][..]),
		("tx", &["transaction", "transfer"][..]),
		("address", &["account", "wallet"][..]),
		("account", &["address", "wallet"][..]),
		("wallet", &["address", "account"][..]),
		("block", &["height"][..]),
		("height", &["block"][..]),
		("epoch", &["era", "period"][..]),
		("dao", &["nervos dao", "deposit", "stake"][..]),
		("deposit", &["dao", "stake"][..]),
		("token", &["udt", "sudt", "xudt", "asset", "coin"][..]),
		("udt", &["token", "sudt", "xudt", "asset"][..]),
		("sudt", &["token", "udt", "asset"][..]),
		("xudt", &["token", "udt", "asset"][..]),
		("asset", &["token", "udt"][..]),
		("script", &["contract", "program", "code"][..]),
		("contract", &["script", "program"][..]),
		("indexer", &["index", "search"][..]),
		("pool", &["mempool", "pending"][..]),
		("mempool", &["pool", "pending"][..]),
		("fee", &["cost", "rate", "gas"][..]),
		("gas", &["fee", "cost"][..]),
		("proof", &["merkle", "verify"][..]),
		// Protocol synonyms - bidirectional mappings
		("omnilock", &["universal", "multi-sig", "ethereum"][..]),
		("spore", &["nft", "digital object", "dob"][..]),
		("nft", &["spore", "cota", "digital object"][..]),
		("cota", &["nft", "compact"][..]),
		("rgb", &["rgb++", "bitcoin"][..]),
		("rgb++", &["rgb", "bitcoin"][..]),
		("molecule", &["serialization", "encoding", "schema"][..]),
		("serialization", &["molecule", "encoding"][..]),
	])
});

/// Search result for tools.
#[derive(Serialize, Deserialize, Debug)]
pub struct ToolSearchResult {
	pub name: String,
	pub description: String,
	pub score: f32,
}

/// Search result for resources.
#[derive(Serialize, Deserialize, Debug)]
pub struct ResourceSearchResult {
	pub uri: String,
	pub name: String,
	pub description: String,
	pub score: f32,
}

/// Search results container.
#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResults<T> {
	pub query: String,
	pub total_matches: usize,
	pub results: Vec<T>,
}

/// Search handlers for finding tools and resources.
#[derive(Clone)]
pub struct SearchHandlers;

impl SearchHandlers {
	/// Create new SearchHandlers instance.
	pub fn new() -> Self {
		Self
	}

	/// Check if a tool name is a search tool.
	pub fn is_search_tool(name: &str) -> bool {
		name == "search_tools" || name == "search_resources"
	}

	/// Handle a search tool call.
	pub fn handle(
		&self,
		name: &str,
		args: &serde_json::Value,
		tools: &[Tool],
		resources: &[Resource],
	) -> Result<CallToolResult> {
		match name {
			"search_tools" => self.search_tools(args, tools),
			"search_resources" => self.search_resources(args, resources),
			_ => Err(CkbMcpError::InvalidParameter(format!(
				"Unknown search tool: {}",
				name
			))),
		}
	}

	/// Search tools by keyword.
	fn search_tools(&self, args: &serde_json::Value, tools: &[Tool]) -> Result<CallToolResult> {
		let query = args
			.get("query")
			.and_then(|v| v.as_str())
			.ok_or_else(|| CkbMcpError::InvalidParameter("Missing query parameter".to_string()))?
			.to_lowercase();

		let limit = args
			.get("limit")
			.and_then(|v| v.as_u64())
			.unwrap_or(10)
			.min(50) as usize;

		debug!("Searching tools for: {} (limit: {})", query, limit);

		let keywords: Vec<&str> = query.split_whitespace().collect();

		let mut results: Vec<ToolSearchResult> = tools
			.iter()
			.filter_map(|tool| {
				let name = tool.name.to_string().to_lowercase();
				let description = tool
					.description
					.as_ref()
					.map(|d| d.to_string().to_lowercase())
					.unwrap_or_default();

				let score = calculate_match_score(&name, &description, &keywords);
				if score > 0.0 {
					Some(ToolSearchResult {
						name: tool.name.to_string(),
						description: tool
							.description
							.as_ref()
							.map(|d| d.to_string())
							.unwrap_or_default(),
						score,
					})
				} else {
					None
				}
			})
			.collect();

		// Sort by score descending.
		results.sort_by(|a, b| {
			b.score
				.partial_cmp(&a.score)
				.unwrap_or(std::cmp::Ordering::Equal)
		});

		let total_matches = results.len();
		results.truncate(limit);

		let search_results = SearchResults {
			query: query.clone(),
			total_matches,
			results,
		};

		let json = serde_json::to_string_pretty(&search_results)?;
		Ok(CallToolResult::success(vec![Content::text(json)]))
	}

	/// Search resources by keyword.
	fn search_resources(
		&self,
		args: &serde_json::Value,
		resources: &[Resource],
	) -> Result<CallToolResult> {
		let query = args
			.get("query")
			.and_then(|v| v.as_str())
			.ok_or_else(|| CkbMcpError::InvalidParameter("Missing query parameter".to_string()))?
			.to_lowercase();

		let limit = args
			.get("limit")
			.and_then(|v| v.as_u64())
			.unwrap_or(10)
			.min(50) as usize;

		debug!("Searching resources for: {} (limit: {})", query, limit);

		let keywords: Vec<&str> = query.split_whitespace().collect();

		let mut results: Vec<ResourceSearchResult> = resources
			.iter()
			.filter_map(|resource| {
				let uri = resource.raw.uri.to_lowercase();
				let name = resource.raw.name.to_lowercase();
				let description = resource
					.raw
					.description
					.as_ref()
					.map(|d| d.to_lowercase())
					.unwrap_or_default();

				// Combine name, uri path components, and description for matching.
				let searchable = format!("{} {} {}", name, uri, description);
				let score = calculate_match_score(&name, &searchable, &keywords);

				if score > 0.0 {
					Some(ResourceSearchResult {
						uri: resource.raw.uri.clone(),
						name: resource.raw.name.clone(),
						description: resource.raw.description.clone().unwrap_or_default(),
						score,
					})
				} else {
					None
				}
			})
			.collect();

		// Sort by score descending.
		results.sort_by(|a, b| {
			b.score
				.partial_cmp(&a.score)
				.unwrap_or(std::cmp::Ordering::Equal)
		});

		let total_matches = results.len();
		results.truncate(limit);

		let search_results = SearchResults {
			query: query.clone(),
			total_matches,
			results,
		};

		let json = serde_json::to_string_pretty(&search_results)?;
		Ok(CallToolResult::success(vec![Content::text(json)]))
	}
}

impl Default for SearchHandlers {
	fn default() -> Self {
		Self::new()
	}
}

/// Expand keywords with their synonyms.
/// Returns a vector of (keyword, is_original) tuples.
fn expand_with_synonyms(keywords: &[&str]) -> Vec<(String, bool)> {
	let mut expanded = Vec::new();
	for keyword in keywords {
		// Add original keyword.
		expanded.push((keyword.to_string(), true));
		// Add synonyms if they exist.
		if let Some(synonyms) = SYNONYMS.get(*keyword) {
			for synonym in *synonyms {
				expanded.push((synonym.to_string(), false));
			}
		}
	}
	expanded
}

/// Calculate a match score for a search query.
/// Higher scores indicate better matches.
/// Uses synonym expansion to improve discoverability.
fn calculate_match_score(name: &str, text: &str, keywords: &[&str]) -> f32 {
	if keywords.is_empty() {
		return 0.0;
	}

	// Expand keywords with synonyms.
	let expanded = expand_with_synonyms(keywords);

	let mut total_score: f32 = 0.0;
	let mut matched_original = 0;
	let mut matched_synonym = 0;

	for (keyword, is_original) in &expanded {
		if keyword.is_empty() {
			continue;
		}

		let mut keyword_score: f32 = 0.0;

		// Weight: original keywords score higher than synonyms.
		let weight = if *is_original { 1.0 } else { 0.5 };

		// Exact match in name (highest weight).
		if name.contains(keyword.as_str()) {
			keyword_score += 3.0 * weight;
			// Bonus for exact word match.
			if name
				.split(|c: char| !c.is_alphanumeric() && c != '_')
				.any(|word| word == keyword)
			{
				keyword_score += 2.0 * weight;
			}
		}

		// Match in description/text.
		if text.contains(keyword.as_str()) {
			keyword_score += 1.0 * weight;
		}

		if keyword_score > 0.0 {
			if *is_original {
				matched_original += 1;
			} else {
				matched_synonym += 1;
			}
			total_score += keyword_score;
		}
	}

	// Require at least one match (original or synonym).
	if matched_original == 0 && matched_synonym == 0 {
		return 0.0;
	}

	// Coverage bonus based on original keywords matched.
	let coverage_bonus = if matched_original > 0 {
		matched_original as f32 / keywords.len() as f32
	} else {
		// Synonym-only matches get lower coverage bonus.
		0.5 * (matched_synonym as f32 / keywords.len() as f32)
	};

	total_score * coverage_bonus
}
