//! JSON-RPC 2.0 request handler.

use axum::{
	extract::State,
	http::StatusCode,
	response::IntoResponse,
	Json,
};
use serde_json::Value;
use std::sync::Arc;
use tracing::debug;

use super::types::{JsonRpcRequest, JsonRpcResponse, INTERNAL_ERROR, INVALID_PARAMS, METHOD_NOT_FOUND};
use crate::capabilities::CkbMcpServer;
use crate::server::AppState;

/// JSON-RPC endpoint handler.
///
/// Accepts plain JSON-RPC 2.0 requests and returns plain JSON responses.
/// This is a stateless alternative to the SSE-based MCP endpoint.
pub async fn jsonrpc_handler(
	State(state): State<Arc<AppState>>,
	body: String,
) -> impl IntoResponse {
	// Parse JSON-RPC request.
	let request: JsonRpcRequest = match serde_json::from_str(&body) {
		Ok(req) => req,
		Err(e) => {
			debug!("Failed to parse JSON-RPC request: {}", e);
			return (StatusCode::OK, Json(JsonRpcResponse::parse_error()));
		}
	};

	// Validate JSON-RPC version.
	if request.jsonrpc != "2.0" {
		return (
			StatusCode::OK,
			Json(JsonRpcResponse::invalid_request(request.id)),
		);
	}

	debug!("JSON-RPC request: method={}", request.method);

	// Create CkbMcpServer instance for handling the request.
	let server = CkbMcpServer::new_with_handlers(
		state.config.clone(),
		state.dev_handlers.clone(),
	);

	// Route to appropriate handler.
	let result = route_method(&server, &request.method, &request.params).await;

	match result {
		Ok(value) => (StatusCode::OK, Json(JsonRpcResponse::success(request.id, value))),
		Err((code, message)) => (
			StatusCode::OK,
			Json(JsonRpcResponse::error(request.id, code, message)),
		),
	}
}

/// Route a JSON-RPC method to the appropriate handler.
async fn route_method(
	server: &CkbMcpServer,
	method: &str,
	params: &Value,
) -> Result<Value, (i32, String)> {
	match method {
		// Tools
		"tools/list" => {
			let result = server.list_tools_internal()
				.map_err(|e| (INTERNAL_ERROR, e.message.to_string()))?;
			serde_json::to_value(result)
				.map_err(|e| (INTERNAL_ERROR, e.to_string()))
		}

		"tools/call" => {
			let name = get_string_param(params, "name")?;
			let arguments = params
				.get("arguments")
				.cloned()
				.unwrap_or(Value::Object(serde_json::Map::new()));

			let result = server.call_tool_internal(&name, &arguments).await
				.map_err(|e| (INTERNAL_ERROR, e.message.to_string()))?;
			serde_json::to_value(result)
				.map_err(|e| (INTERNAL_ERROR, e.to_string()))
		}

		// Resources
		"resources/list" => {
			let result = server.list_resources_internal()
				.map_err(|e| (INTERNAL_ERROR, e.message.to_string()))?;
			serde_json::to_value(result)
				.map_err(|e| (INTERNAL_ERROR, e.to_string()))
		}

		"resources/read" => {
			let uri = get_string_param(params, "uri")?;

			let result = server.read_resource_internal(&uri)
				.map_err(|e| (INTERNAL_ERROR, e.message.to_string()))?;
			serde_json::to_value(result)
				.map_err(|e| (INTERNAL_ERROR, e.to_string()))
		}

		// Prompts
		"prompts/list" => {
			let result = server.list_prompts_internal()
				.map_err(|e| (INTERNAL_ERROR, e.message.to_string()))?;
			serde_json::to_value(result)
				.map_err(|e| (INTERNAL_ERROR, e.to_string()))
		}

		"prompts/get" => {
			let name = get_string_param(params, "name")?;
			let arguments = params.get("arguments").cloned();

			let result = server.get_prompt_internal(&name, arguments)
				.map_err(|e| (INTERNAL_ERROR, e.message.to_string()))?;
			serde_json::to_value(result)
				.map_err(|e| (INTERNAL_ERROR, e.to_string()))
		}

		// Utility
		"ping" => Ok(Value::Object(serde_json::Map::new())),

		// Unknown method
		_ => Err((METHOD_NOT_FOUND, format!("Method not found: {}", method))),
	}
}

/// Extract a required string parameter from JSON params.
fn get_string_param(params: &Value, key: &str) -> Result<String, (i32, String)> {
	params
		.get(key)
		.and_then(|v| v.as_str())
		.map(|s| s.to_string())
		.ok_or_else(|| (INVALID_PARAMS, format!("Missing required parameter: {}", key)))
}
