use thiserror::Error;

/// JSON-RPC 2.0 error codes.
pub mod codes {
	/// Parse error - Invalid JSON was received.
	pub const PARSE_ERROR: i64 = -32700;
	/// Invalid Request - The JSON sent is not a valid Request object.
	pub const INVALID_REQUEST: i64 = -32600;
	/// Method not found - The method does not exist / is not available.
	pub const METHOD_NOT_FOUND: i64 = -32601;
	/// Invalid params - Invalid method parameter(s).
	pub const INVALID_PARAMS: i64 = -32602;
	/// Internal error - Internal JSON-RPC error.
	pub const INTERNAL_ERROR: i64 = -32603;

	// Server error codes (-32000 to -32099 reserved for implementation-defined errors)

	/// CKB RPC error - Error from CKB node RPC.
	pub const CKB_RPC_ERROR: i64 = -32000;
	/// Resource not found - Requested resource does not exist.
	pub const RESOURCE_NOT_FOUND: i64 = -32001;
	/// Network error - Error in network communication.
	pub const NETWORK_ERROR: i64 = -32002;
}

#[derive(Error, Debug)]
pub enum CkbMcpError {
	#[error("CKB RPC error: {0}")]
	CkbRpc(String),

	#[error("JSON serialization error: {0}")]
	Json(#[from] serde_json::Error),

	#[error("HTTP request error: {0}")]
	Http(String),

	#[error("MCP protocol error: {0}")]
	Mcp(String),

	#[error("Invalid parameter: {0}")]
	InvalidParameter(String),

	#[error("Resource not found: {0}")]
	NotFound(String),

	#[error("Internal server error: {0}")]
	Internal(String),

	#[error("IO error: {0}")]
	Io(#[from] std::io::Error),

	#[error("Network error: {0}")]
	Network(#[from] anyhow::Error),
}

impl From<reqwest::Error> for CkbMcpError {
	fn from(err: reqwest::Error) -> Self {
		CkbMcpError::Http(err.to_string())
	}
}

impl From<tokio::task::JoinError> for CkbMcpError {
	fn from(err: tokio::task::JoinError) -> Self {
		CkbMcpError::Internal(format!("Task join error: {}", err))
	}
}

impl CkbMcpError {
	/// Get the JSON-RPC error code for this error.
	pub fn code(&self) -> i64 {
		match self {
			CkbMcpError::CkbRpc(_) => codes::CKB_RPC_ERROR,
			CkbMcpError::Json(_) => codes::PARSE_ERROR,
			CkbMcpError::Http(_) | CkbMcpError::Network(_) => codes::NETWORK_ERROR,
			CkbMcpError::Mcp(_) => codes::INVALID_REQUEST,
			CkbMcpError::InvalidParameter(_) => codes::INVALID_PARAMS,
			CkbMcpError::NotFound(_) => codes::RESOURCE_NOT_FOUND,
			CkbMcpError::Internal(_) | CkbMcpError::Io(_) => codes::INTERNAL_ERROR,
		}
	}
}

pub type Result<T> = std::result::Result<T, CkbMcpError>;