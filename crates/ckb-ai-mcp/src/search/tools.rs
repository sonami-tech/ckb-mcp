//! Search tool definitions.

use crate::util::{make_tool_annotated, ToolHints};
use rmcp::model::Tool;
use serde_json::json;
use std::sync::LazyLock;

/// Search tools for finding tools and resources.
pub static SEARCH_TOOLS: LazyLock<Vec<Tool>> = LazyLock::new(|| {
	vec![
		make_tool_annotated(
			"search_tools",
			"Search Tools",
			"Search available MCP tools by keyword. Returns matching tools with names and descriptions.",
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
		),
		make_tool_annotated(
			"search_resources",
			"Search Resources",
			"Search available documentation resources by keyword. Returns matching resources with URIs and descriptions.",
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
		),
	]
});
