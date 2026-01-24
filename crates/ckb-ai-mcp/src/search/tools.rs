//! Search tool definitions.

use crate::util::{make_tool_with_output_schema, ToolHints};
use rmcp::model::Tool;
use serde_json::json;
use std::sync::LazyLock;

/// Search tools for finding tools and resources.
pub static SEARCH_TOOLS: LazyLock<Vec<Tool>> = LazyLock::new(|| {
	vec![
		make_tool_with_output_schema(
			"search_tools",
			"Search Tools",
			"Search available MCP tools by keyword. Returns matching tools with names and descriptions. Use this to discover relevant tools before calling them.",
			json!({
				"type": "object",
				"properties": {
					"query": {
						"type": "string",
						"description": "Search keyword(s) to match against tool names and descriptions."
					},
					"limit": {
						"type": "integer",
						"description": "Maximum number of results to return. Default: 10, max: 50.",
						"default": 10
					}
				},
				"required": ["query"]
			}),
			ToolHints::query_idempotent(),
			Some(json!({
				"type": "object",
				"properties": {
					"query": { "type": "string", "description": "The search query used" },
					"total_matches": { "type": "integer", "description": "Total number of matching tools" },
					"results": {
						"type": "array",
						"description": "Matching tools sorted by relevance",
						"items": {
							"type": "object",
							"properties": {
								"name": { "type": "string", "description": "Tool name for invocation" },
								"description": { "type": "string", "description": "Tool description" },
								"score": { "type": "number", "description": "Relevance score" }
							},
							"required": ["name", "description", "score"]
						}
					}
				},
				"required": ["query", "total_matches", "results"]
			})),
		),
		make_tool_with_output_schema(
			"search_resources",
			"Search Resources",
			"Search available documentation resources by keyword. Returns matching resources with URIs and descriptions. Use this to find relevant CKB development documentation.",
			json!({
				"type": "object",
				"properties": {
					"query": {
						"type": "string",
						"description": "Search keyword(s) to match against resource names and descriptions."
					},
					"limit": {
						"type": "integer",
						"description": "Maximum number of results to return. Default: 10, max: 50.",
						"default": 10
					}
				},
				"required": ["query"]
			}),
			ToolHints::query_idempotent(),
			Some(json!({
				"type": "object",
				"properties": {
					"query": { "type": "string", "description": "The search query used" },
					"total_matches": { "type": "integer", "description": "Total number of matching resources" },
					"results": {
						"type": "array",
						"description": "Matching resources sorted by relevance",
						"items": {
							"type": "object",
							"properties": {
								"uri": { "type": "string", "description": "Resource URI for reading" },
								"name": { "type": "string", "description": "Resource name" },
								"description": { "type": "string", "description": "Resource description" },
								"score": { "type": "number", "description": "Relevance score" }
							},
							"required": ["uri", "name", "description", "score"]
						}
					}
				},
				"required": ["query", "total_matches", "results"]
			})),
		),
	]
});
