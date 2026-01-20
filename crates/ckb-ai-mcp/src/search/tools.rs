//! Search tool definitions.

use crate::util::make_tool;
use rmcp::model::Tool;
use serde_json::json;
use std::sync::LazyLock;

/// Search tools for finding tools and resources.
pub static SEARCH_TOOLS: LazyLock<Vec<Tool>> = LazyLock::new(|| {
	vec![
		make_tool(
			"search_tools",
			"Search available MCP tools by keyword. Returns matching tools with their names \
			and descriptions. Use this to discover relevant tools for a task.",
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
		),
		make_tool(
			"search_resources",
			"Search available documentation resources by keyword. Returns matching resources \
			with their URIs and descriptions. Use this to find relevant CKB development documentation.",
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
		),
	]
});
