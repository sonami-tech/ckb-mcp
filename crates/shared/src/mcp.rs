use serde::{Deserialize, Serialize};

/// MCP protocol types
#[derive(Debug, Serialize, Deserialize)]
pub struct McpRequest {
	pub jsonrpc: String,
	pub method: String,
	pub params: Option<serde_json::Value>,
	pub id: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpResponse {
	pub jsonrpc: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub result: Option<serde_json::Value>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub error: Option<McpError>,
	pub id: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpError {
	pub code: i32,
	pub message: String,
	pub data: Option<serde_json::Value>,
}

/// MCP capabilities
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolDefinition {
	pub name: String,
	pub description: String,
	#[serde(rename = "inputSchema")]
	pub input_schema: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceDefinition {
	pub uri: String,
	pub name: String,
	pub description: Option<String>,
	#[serde(rename = "mimeType")]
	pub mime_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PromptDefinition {
	pub name: String,
	pub description: String,
	pub arguments: Option<Vec<PromptArgument>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PromptArgument {
	pub name: String,
	pub description: String,
	pub required: bool,
}

/// MCP server capabilities response
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerCapabilities {
	pub tools: Option<ToolsCapability>,
	pub resources: Option<ResourcesCapability>,
	pub prompts: Option<PromptsCapability>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolsCapability {
	#[serde(rename = "listChanged")]
	pub list_changed: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourcesCapability {
	pub subscribe: Option<bool>,
	#[serde(rename = "listChanged")]
	pub list_changed: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PromptsCapability {
	#[serde(rename = "listChanged")]
	pub list_changed: Option<bool>,
}

/// Utility functions for MCP responses
pub fn create_success_response(id: Option<serde_json::Value>, result: serde_json::Value) -> McpResponse {
	McpResponse {
		jsonrpc: "2.0".to_string(),
		result: Some(result),
		error: None,
		id,
	}
}

pub fn create_error_response(id: Option<serde_json::Value>, code: i32, message: String) -> McpResponse {
	McpResponse {
		jsonrpc: "2.0".to_string(),
		result: None,
		error: Some(McpError {
			code,
			message,
			data: None,
		}),
		id,
	}
}