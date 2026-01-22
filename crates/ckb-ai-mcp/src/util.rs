//! Utility functions shared across modules.

use rmcp::model::{Tool, ToolAnnotations};

/// Tool annotation hints for different tool categories.
#[derive(Debug, Clone, Copy)]
pub struct ToolHints {
	pub read_only: bool,
	pub destructive: bool,
	pub idempotent: bool,
	pub open_world: bool,
}

impl ToolHints {
	/// Read-only query tool that returns the same result for the same input (historical data).
	pub const fn query_idempotent() -> Self {
		Self {
			read_only: true,
			destructive: false,
			idempotent: true,
			open_world: false,
		}
	}

	/// Read-only query tool where results may change over time (live state).
	pub const fn query_live() -> Self {
		Self {
			read_only: true,
			destructive: false,
			idempotent: false,
			open_world: false,
		}
	}

	/// Tool that submits data to the blockchain (destructive, open world).
	pub const fn submit() -> Self {
		Self {
			read_only: false,
			destructive: true,
			idempotent: false,
			open_world: true,
		}
	}

	/// Tool that modifies external state but is not destructive.
	pub const fn write_non_destructive() -> Self {
		Self {
			read_only: false,
			destructive: false,
			idempotent: false,
			open_world: false,
		}
	}
}

/// Helper to create a tool definition with annotations.
pub fn make_tool_annotated(
	name: &'static str,
	title: &'static str,
	description: &'static str,
	input_schema: serde_json::Value,
	hints: ToolHints,
) -> Tool {
	Tool {
		name: name.into(),
		description: Some(description.into()),
		input_schema: input_schema
			.as_object()
			.expect("input_schema must be a JSON object")
			.clone()
			.into(),
		annotations: Some(
			ToolAnnotations::new()
				.read_only(hints.read_only)
				.destructive(hints.destructive)
				.idempotent(hints.idempotent)
				.open_world(hints.open_world),
		),
		output_schema: None,
		title: Some(title.to_string()),
		icons: None,
		meta: None,
	}
}
