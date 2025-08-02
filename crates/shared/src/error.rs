use thiserror::Error;

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

pub type Result<T> = std::result::Result<T, CkbMcpError>;