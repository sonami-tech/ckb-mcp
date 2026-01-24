use serde::{Deserialize, Serialize};

/// CKB RPC request/response types
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
	pub jsonrpc: String,
	pub method: String,
	pub params: serde_json::Value,
	pub id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
	pub jsonrpc: String,
	pub result: Option<serde_json::Value>,
	pub error: Option<JsonRpcError>,
	pub id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
	pub code: i32,
	pub message: String,
	pub data: Option<serde_json::Value>,
}
