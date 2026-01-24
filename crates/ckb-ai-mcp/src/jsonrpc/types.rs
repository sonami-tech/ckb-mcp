//! JSON-RPC 2.0 request and response types.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 request.
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
	pub jsonrpc: String,
	pub id: Value,
	pub method: String,
	#[serde(default)]
	pub params: Value,
}

/// JSON-RPC 2.0 response (success or error).
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
	pub jsonrpc: &'static str,
	pub id: Value,
	#[serde(flatten)]
	pub payload: ResponsePayload,
}

/// Response payload - either success result or error.
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ResponsePayload {
	Success { result: Value },
	Error { error: JsonRpcError },
}

/// JSON-RPC 2.0 error object.
#[derive(Debug, Serialize)]
pub struct JsonRpcError {
	pub code: i32,
	pub message: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub data: Option<Value>,
}

// Standard JSON-RPC 2.0 error codes.
pub const PARSE_ERROR: i32 = -32700;
pub const INVALID_REQUEST: i32 = -32600;
pub const METHOD_NOT_FOUND: i32 = -32601;
pub const INVALID_PARAMS: i32 = -32602;
pub const INTERNAL_ERROR: i32 = -32603;

impl JsonRpcResponse {
	/// Create a success response.
	pub fn success(id: Value, result: Value) -> Self {
		Self {
			jsonrpc: "2.0",
			id,
			payload: ResponsePayload::Success { result },
		}
	}

	/// Create an error response.
	pub fn error(id: Value, code: i32, message: impl Into<String>) -> Self {
		Self {
			jsonrpc: "2.0",
			id,
			payload: ResponsePayload::Error {
				error: JsonRpcError {
					code,
					message: message.into(),
					data: None,
				},
			},
		}
	}

	/// Create an error response with additional data.
	#[allow(dead_code)]
	pub fn error_with_data(id: Value, code: i32, message: impl Into<String>, data: Value) -> Self {
		Self {
			jsonrpc: "2.0",
			id,
			payload: ResponsePayload::Error {
				error: JsonRpcError {
					code,
					message: message.into(),
					data: Some(data),
				},
			},
		}
	}

	/// Create a parse error response.
	pub fn parse_error() -> Self {
		Self::error(Value::Null, PARSE_ERROR, "Parse error")
	}

	/// Create an invalid request response.
	pub fn invalid_request(id: Value) -> Self {
		Self::error(id, INVALID_REQUEST, "Invalid Request")
	}

	/// Create a method not found response.
	pub fn method_not_found(id: Value, method: &str) -> Self {
		Self::error(
			id,
			METHOD_NOT_FOUND,
			format!("Method not found: {}", method),
		)
	}

	/// Create an invalid params response.
	pub fn invalid_params(id: Value, message: impl Into<String>) -> Self {
		Self::error(id, INVALID_PARAMS, message)
	}

	/// Create an internal error response.
	pub fn internal_error(id: Value, message: impl Into<String>) -> Self {
		Self::error(id, INTERNAL_ERROR, message)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_parse_request() {
		let json = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}"#;
		let req: JsonRpcRequest = serde_json::from_str(json).unwrap();
		assert_eq!(req.jsonrpc, "2.0");
		assert_eq!(req.method, "tools/list");
		assert_eq!(req.id, Value::Number(1.into()));
	}

	#[test]
	fn test_parse_request_string_id() {
		let json = r#"{"jsonrpc":"2.0","id":"abc","method":"ping","params":{}}"#;
		let req: JsonRpcRequest = serde_json::from_str(json).unwrap();
		assert_eq!(req.id, Value::String("abc".to_string()));
	}

	#[test]
	fn test_parse_request_no_params() {
		let json = r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#;
		let req: JsonRpcRequest = serde_json::from_str(json).unwrap();
		assert_eq!(req.params, Value::Null);
	}

	#[test]
	fn test_success_response() {
		let resp =
			JsonRpcResponse::success(Value::Number(1.into()), serde_json::json!({"tools": []}));
		let json = serde_json::to_string(&resp).unwrap();
		assert!(json.contains(r#""jsonrpc":"2.0""#));
		assert!(json.contains(r#""id":1"#));
		assert!(json.contains(r#""result":{"tools":[]}"#));
	}

	#[test]
	fn test_error_response() {
		let resp = JsonRpcResponse::method_not_found(Value::Number(1.into()), "unknown/method");
		let json = serde_json::to_string(&resp).unwrap();
		assert!(json.contains(r#""code":-32601"#));
		assert!(json.contains("unknown/method"));
	}

	#[test]
	fn test_parse_error_null_id() {
		let resp = JsonRpcResponse::parse_error();
		let json = serde_json::to_string(&resp).unwrap();
		assert!(json.contains(r#""id":null"#));
		assert!(json.contains(r#""code":-32700"#));
	}
}
