//! Middleware for modifying MCP responses.
//!
//! This module provides middleware to inject Anthropic-specific extensions
//! like `defer_loading` into MCP tool responses.

use axum::body::Body;
use axum::http::{Request, Response};
use bytes::Bytes;
use http_body::Body as HttpBody;
use http_body_util::BodyExt;
use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{Layer, Service};
use tracing::debug;

/// Tools that should NOT have defer_loading enabled (always loaded).
const ALWAYS_LOADED_TOOLS: &[&str] = &["search_tools", "search_resources"];

/// Layer that adds `defer_loading` to MCP tool responses.
#[derive(Clone)]
pub struct DeferLoadingLayer;

impl<S> Layer<S> for DeferLoadingLayer {
	type Service = DeferLoadingMiddleware<S>;

	fn layer(&self, inner: S) -> Self::Service {
		DeferLoadingMiddleware { inner }
	}
}

/// Middleware service that injects `defer_loading` into tools/list responses.
#[derive(Clone)]
pub struct DeferLoadingMiddleware<S> {
	inner: S,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for DeferLoadingMiddleware<S>
where
	S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
	S::Future: Send + 'static,
	S::Error: Send + 'static,
	ReqBody: Send + 'static,
	ResBody: HttpBody<Data = Bytes, Error = Infallible> + Send + 'static,
{
	type Response = Response<Body>;
	type Error = S::Error;
	type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

	fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		self.inner.poll_ready(cx)
	}

	fn call(&mut self, request: Request<ReqBody>) -> Self::Future {
		let mut inner = self.inner.clone();

		Box::pin(async move {
			let response = inner.call(request).await?;

			// Check if this could be a tools/list response by content-type.
			let content_type = response
				.headers()
				.get("content-type")
				.and_then(|v| v.to_str().ok())
				.unwrap_or("");

			// Only process JSON responses.
			if !content_type.contains("application/json") {
				// Convert response body to axum::body::Body.
				let (parts, body) = response.into_parts();
				let body = Body::new(body);
				return Ok(Response::from_parts(parts, body));
			}

			// Collect the response body.
			let (parts, body) = response.into_parts();
			let bytes = match body.collect().await {
				Ok(collected) => collected.to_bytes(),
				Err(_) => {
					// If we can't collect, return an empty response.
					return Ok(Response::from_parts(parts, Body::empty()));
				}
			};

			// Try to parse as JSON and check if it's a tools/list response.
			let modified_bytes = match serde_json::from_slice::<serde_json::Value>(&bytes) {
				Ok(mut json) => {
					if let Some(modified) = try_inject_defer_loading(&mut json) {
						debug!("Injected defer_loading into tools/list response");
						modified
					} else {
						bytes.to_vec()
					}
				}
				Err(_) => bytes.to_vec(),
			};

			// Reconstruct the response with modified body.
			let new_body = Body::from(modified_bytes);
			Ok(Response::from_parts(parts, new_body))
		})
	}
}

/// Try to inject defer_loading into a JSON-RPC tools/list response.
/// Returns Some(modified_bytes) if the JSON was a tools/list response and was modified.
fn try_inject_defer_loading(json: &mut serde_json::Value) -> Option<Vec<u8>> {
	// Check if this is a JSON-RPC response with a result containing tools.
	let result = json.get_mut("result")?;

	// Check if result has a "tools" array (tools/list response format).
	let tools = result.get_mut("tools")?.as_array_mut()?;

	// Add defer_loading to each tool.
	for tool in tools.iter_mut() {
		if let Some(obj) = tool.as_object_mut() {
			let name = obj.get("name").and_then(|n| n.as_str()).unwrap_or("");
			let defer = !ALWAYS_LOADED_TOOLS.contains(&name);
			obj.insert("defer_loading".to_string(), serde_json::Value::Bool(defer));
		}
	}

	// Serialize back to bytes.
	serde_json::to_vec(json).ok()
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::json;

	#[test]
	fn test_inject_defer_loading_tools_list() {
		let mut json = json!({
			"jsonrpc": "2.0",
			"id": 1,
			"result": {
				"tools": [
					{"name": "search_tools", "description": "Search tools"},
					{"name": "search_resources", "description": "Search resources"},
					{"name": "rpc_get_block", "description": "Get block"}
				]
			}
		});

		let result = try_inject_defer_loading(&mut json);
		assert!(result.is_some());

		let tools = json["result"]["tools"].as_array().unwrap();
		assert_eq!(tools[0]["defer_loading"], false); // search_tools - always loaded
		assert_eq!(tools[1]["defer_loading"], false); // search_resources - always loaded
		assert_eq!(tools[2]["defer_loading"], true); // rpc_get_block - deferred
	}

	#[test]
	fn test_inject_defer_loading_non_tools_response() {
		let mut json = json!({
			"jsonrpc": "2.0",
			"id": 1,
			"result": {
				"resources": []
			}
		});

		let result = try_inject_defer_loading(&mut json);
		assert!(result.is_none());
	}
}
