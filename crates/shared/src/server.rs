use axum::{
	extract::State,
	http::{HeaderValue, StatusCode},
	routing::{get, post},
	Json, Router,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};

use crate::{
	error::Result,
	mcp::{McpRequest, McpResponse},
};

/// Trait that server state must implement to provide MCP handling.
pub trait HasMcpHandler: Clone + Send + Sync + 'static {
	type Handler: McpHandlerTrait;

	fn handler(&self) -> &Arc<Self::Handler>;

	/// Get the server info JSON for the "/" endpoint.
	fn server_info_json(&self) -> serde_json::Value;
}

/// Trait for MCP request handlers.
#[axum::async_trait]
pub trait McpHandlerTrait: Send + Sync {
	async fn handle_request(&self, request: McpRequest) -> Result<McpResponse>;
}

/// Configuration for starting an MCP server.
pub struct McpServerConfig {
	pub host: String,
	pub port: u16,
	pub server_name: String,
	pub log_level: String,
}

impl McpServerConfig {
	pub fn new(host: String, port: u16, server_name: String, log_level: String) -> Self {
		Self {
			host,
			port,
			server_name,
			log_level,
		}
	}

	/// Initialize logging with the configured log level.
	pub fn init_logging(&self) {
		tracing_subscriber::fmt()
			.with_env_filter(&self.log_level)
			.init();
	}

	/// Start the MCP server with the given state.
	pub async fn serve<S>(self, state: S) -> Result<()>
	where
		S: HasMcpHandler,
	{
		// Initialize logging
		self.init_logging();

		// Build router with standard MCP endpoints
		let app = Router::new()
			.route("/", get(mcp_info_handler::<S>))
			.route("/mcp", post(mcp_handler::<S>))
			.route("/health", get(health_handler))
			.layer(
				CorsLayer::new()
					.allow_origin("*".parse::<HeaderValue>().unwrap())
					.allow_methods([axum::http::Method::GET, axum::http::Method::POST])
					.allow_headers([axum::http::header::CONTENT_TYPE]),
			)
			.with_state(state);

		let addr = format!("{}:{}", self.host, self.port);
		info!("Starting {} on {}", self.server_name, addr);

		let listener = tokio::net::TcpListener::bind(&addr).await?;
		axum::serve(listener, app).await?;

		Ok(())
	}
}

/// Standard MCP info handler.
async fn mcp_info_handler<S>(State(state): State<S>) -> Json<serde_json::Value>
where
	S: HasMcpHandler,
{
	Json(state.server_info_json())
}

/// Standard MCP request handler.
async fn mcp_handler<S>(
	State(state): State<S>,
	Json(request): Json<McpRequest>,
) -> std::result::Result<Json<McpResponse>, StatusCode>
where
	S: HasMcpHandler,
{
	match state.handler().handle_request(request).await {
		Ok(response) => Ok(Json(response)),
		Err(e) => {
			warn!("MCP request failed: {}", e);
			Err(StatusCode::INTERNAL_SERVER_ERROR)
		}
	}
}

/// Standard health check handler.
async fn health_handler() -> &'static str {
	"OK"
}
