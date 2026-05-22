//! Utility functions shared across modules.

use ckb_types::packed::Byte;
use rmcp::model::{Tool, ToolAnnotations};

/// Convert a CKB script hash_type byte to its string representation.
///
/// The hash type encoding in CKB uses:
/// - Low bit 1 = "type" (matches by type script hash)
/// - Low bit 0 = "data" variant (matches by data hash)
///
/// For data variants, the high 7 bits encode the VM version:
/// - 0x00 = "data" (v0 CKB VM)
/// - 0x02 = "data1" (v1 CKB VM)
/// - 0x04 = "data2" (v2 CKB VM)
/// - ...up to 0xFE = "data127" (v127 CKB VM)
pub fn hash_type_to_string(hash_type: Byte) -> String {
	let byte_value: u8 = hash_type.into();
	if byte_value == 1 {
		"type".to_string()
	} else if byte_value == 0 {
		"data".to_string()
	} else {
		// Data variants: byte value = version * 2, so version = byte / 2
		let version = byte_value / 2;
		format!("data{}", version)
	}
}

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
	make_tool_with_output_schema(name, title, description, input_schema, hints, None)
}

/// Helper to create a tool definition with annotations and output schema.
pub fn make_tool_with_output_schema(
	name: &'static str,
	title: &'static str,
	description: &'static str,
	input_schema: serde_json::Value,
	hints: ToolHints,
	output_schema: Option<serde_json::Value>,
) -> Tool {
	let tool = Tool::new(
		name,
		description,
		input_schema
			.as_object()
			.expect("input_schema must be a JSON object")
			.clone(),
	)
	.with_title(title)
	.with_annotations(
		ToolAnnotations::new()
			.read_only(hints.read_only)
			.destructive(hints.destructive)
			.idempotent(hints.idempotent)
			.open_world(hints.open_world),
	);

	if let Some(output_schema) = output_schema {
		tool.with_raw_output_schema(
			output_schema
				.as_object()
				.expect("output_schema must be a JSON object")
				.clone()
				.into(),
		)
	} else {
		tool
	}
}
