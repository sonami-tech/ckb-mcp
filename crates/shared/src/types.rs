use serde::{Deserialize, Serialize};

/// CKB blockchain types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHash(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionHash(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockNumber(pub u64);

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

/// MCP tool parameter types
#[derive(Debug, Serialize, Deserialize)]
pub struct GetBlockParams {
	pub block_hash: Option<String>,
	pub block_number: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTransactionParams {
	pub tx_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetCellParams {
	pub out_point: OutPoint,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OutPoint {
	pub tx_hash: String,
	pub index: u32,
}

/// Common server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
	pub host: String,
	pub port: u16,
	pub ckb_rpc_url: Option<String>,
	pub log_level: String,
}