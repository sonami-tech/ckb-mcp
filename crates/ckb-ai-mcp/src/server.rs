//! HTTP server setup with rmcp Streamable HTTP transport.

use anyhow::Result;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::StreamableHttpService;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::capabilities::CkbMcpServer;
use crate::ServerConfig;

/// Application state shared across handlers.
#[derive(Clone)]
pub struct AppState {
	pub config: ServerConfig,
}

/// Run the MCP server.
pub async fn run(addr: SocketAddr, config: ServerConfig) -> Result<()> {
	let state = AppState {
		config: config.clone(),
	};

	// Create the MCP service with StreamableHttpService.
	let mcp_service = StreamableHttpService::new(
		{
			let config = config.clone();
			move || Ok(CkbMcpServer::new(config.clone()))
		},
		LocalSessionManager::default().into(),
		Default::default(),
	);

	// Build the router.
	let app = Router::new()
		// Health endpoint.
		.route("/health", get(health_handler))
		// Stats endpoint.
		.route("/stats", get(stats_handler))
		// MCP endpoint via StreamableHttpService.
		.nest_service("/mcp", mcp_service)
		// Shared state.
		.with_state(Arc::new(state))
		// CORS configuration.
		.layer(
			CorsLayer::new()
				.allow_origin(Any)
				.allow_methods(Any)
				.allow_headers(Any),
		)
		// Request tracing.
		.layer(TraceLayer::new_for_http());

	info!("Server starting on {}", addr);

	let listener = tokio::net::TcpListener::bind(addr).await?;
	axum::serve(listener, app)
		.with_graceful_shutdown(shutdown_signal())
		.await?;

	info!("Server shutdown complete");
	Ok(())
}

/// Health check endpoint.
async fn health_handler() -> impl IntoResponse {
	(StatusCode::OK, "OK")
}

/// Stats endpoint with format query parameter.
async fn stats_handler(
	State(state): State<Arc<AppState>>,
	axum::extract::Query(params): axum::extract::Query<StatsQueryParams>,
) -> impl IntoResponse {
	let format = params.format.as_deref().unwrap_or("human");

	let result = match format {
		"json" => state.config.stats.format_json(),
		"prometheus" => state.config.stats.format_prometheus(),
		_ => state.config.stats.format_human(),
	};

	match result {
		Ok(body) => {
			let content_type = match format {
				"json" => "application/json",
				"prometheus" => "text/plain; charset=utf-8",
				_ => "text/plain; charset=utf-8",
			};
			(
				StatusCode::OK,
				[(axum::http::header::CONTENT_TYPE, content_type)],
				body,
			)
				.into_response()
		}
		Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
	}
}

#[derive(Debug, serde::Deserialize)]
struct StatsQueryParams {
	format: Option<String>,
}

/// Shutdown signal handler.
async fn shutdown_signal() {
	let ctrl_c = async {
		tokio::signal::ctrl_c()
			.await
			.expect("Failed to install Ctrl+C handler");
	};

	#[cfg(unix)]
	let terminate = async {
		tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
			.expect("Failed to install signal handler")
			.recv()
			.await;
	};

	#[cfg(not(unix))]
	let terminate = std::future::pending::<()>();

	tokio::select! {
		_ = ctrl_c => {
			info!("Received Ctrl+C, shutting down");
		}
		_ = terminate => {
			info!("Received SIGTERM, shutting down");
		}
	}
}
