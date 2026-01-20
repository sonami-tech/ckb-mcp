//! Utility functions shared across modules.

use rmcp::model::Tool;

/// Helper to create a tool definition with standard fields.
pub fn make_tool(
	name: &'static str,
	description: &'static str,
	input_schema: serde_json::Value,
) -> Tool {
	Tool {
		name: name.into(),
		description: Some(description.into()),
		input_schema: input_schema.as_object().unwrap().clone().into(),
		annotations: None,
		output_schema: None,
		title: None,
		icons: None,
		meta: None,
	}
}
