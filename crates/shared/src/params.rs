//! Parameter extraction helpers for MCP tool handlers.
//!
//! This module provides utility functions to extract and validate parameters
//! from JSON-RPC tool call arguments, reducing boilerplate in handler code.

use crate::error::{CkbMcpError, Result};
use serde_json::Value;

/// Extract a required string parameter.
///
/// Returns an error if the field is missing or not a string.
pub fn extract_str<'a>(args: &'a Value, field: &str) -> Result<&'a str> {
	args.get(field)
		.and_then(|v| v.as_str())
		.ok_or_else(|| CkbMcpError::InvalidParameter(format!("Missing required field: {}", field)))
}

/// Extract an optional string parameter.
///
/// Returns `None` if the field is missing or not a string.
pub fn extract_str_opt<'a>(args: &'a Value, field: &str) -> Option<&'a str> {
	args.get(field).and_then(|v| v.as_str())
}

/// Extract a required u64 parameter.
///
/// Returns an error if the field is missing or not a valid u64.
pub fn extract_u64(args: &Value, field: &str) -> Result<u64> {
	args.get(field)
		.and_then(|v| v.as_u64())
		.ok_or_else(|| CkbMcpError::InvalidParameter(format!("Missing required field: {}", field)))
}

/// Extract an optional u64 parameter.
///
/// Returns `None` if the field is missing or not a valid u64.
pub fn extract_u64_opt(args: &Value, field: &str) -> Option<u64> {
	args.get(field).and_then(|v| v.as_u64())
}

/// Extract a boolean parameter with a default value.
///
/// Returns the default if the field is missing or not a boolean.
pub fn extract_bool(args: &Value, field: &str, default: bool) -> bool {
	args.get(field).and_then(|v| v.as_bool()).unwrap_or(default)
}

/// Extract a required object parameter.
///
/// Returns an error if the field is missing or not an object.
pub fn extract_object<'a>(args: &'a Value, field: &str) -> Result<&'a Value> {
	args.get(field)
		.filter(|v| v.is_object())
		.ok_or_else(|| CkbMcpError::InvalidParameter(format!("Missing required object: {}", field)))
}

/// Extract a required array parameter.
///
/// Returns an error if the field is missing or not an array.
pub fn extract_array<'a>(args: &'a Value, field: &str) -> Result<&'a Vec<Value>> {
	args.get(field)
		.and_then(|v| v.as_array())
		.ok_or_else(|| CkbMcpError::InvalidParameter(format!("Missing required array: {}", field)))
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::json;

	#[test]
	fn test_extract_str() {
		let args = json!({"name": "test", "count": 42});

		assert_eq!(extract_str(&args, "name").unwrap(), "test");
		assert!(extract_str(&args, "missing").is_err());
		assert!(extract_str(&args, "count").is_err()); // not a string
	}

	#[test]
	fn test_extract_str_opt() {
		let args = json!({"name": "test"});

		assert_eq!(extract_str_opt(&args, "name"), Some("test"));
		assert_eq!(extract_str_opt(&args, "missing"), None);
	}

	#[test]
	fn test_extract_u64() {
		let args = json!({"count": 42, "name": "test"});

		assert_eq!(extract_u64(&args, "count").unwrap(), 42);
		assert!(extract_u64(&args, "missing").is_err());
		assert!(extract_u64(&args, "name").is_err()); // not a number
	}

	#[test]
	fn test_extract_bool() {
		let args = json!({"enabled": true});

		assert!(extract_bool(&args, "enabled", false));
		assert!(!extract_bool(&args, "missing", false));
		assert!(extract_bool(&args, "missing", true));
	}

	#[test]
	fn test_extract_object() {
		let args = json!({"config": {"key": "value"}, "name": "test"});

		assert!(extract_object(&args, "config").is_ok());
		assert!(extract_object(&args, "missing").is_err());
		assert!(extract_object(&args, "name").is_err()); // not an object
	}

	#[test]
	fn test_extract_array() {
		let args = json!({"items": [1, 2, 3], "name": "test"});

		assert_eq!(extract_array(&args, "items").unwrap().len(), 3);
		assert!(extract_array(&args, "missing").is_err());
		assert!(extract_array(&args, "name").is_err()); // not an array
	}
}
