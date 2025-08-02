use axum::{
	extract::State,
	http::{HeaderValue, StatusCode},
	routing::{get, post},
	Json, Router,
};
use clap::Parser;
use shared::{error::Result, types::ServerConfig};
use std::{path::PathBuf, sync::Arc};
use tower_http::cors::CorsLayer;
use tracing::info;

mod handlers;
mod docs;

use handlers::McpHandler;
use docs::DocsProvider;

#[derive(Parser)]
#[command(name = "ckb-docs-server")]
#[command(about = "CKB Documentation MCP Server")]
struct Args {
	/// Port to bind to
	#[arg(short, long, default_value = "8002")]
	port: u16,

	/// Host to bind to
	#[arg(long, default_value = "0.0.0.0")]
	host: String,

	/// Custom docs directory
	#[arg(long)]
	docs_path: Option<PathBuf>,

	/// Log level
	#[arg(long, default_value = "info")]
	log_level: String,
}

#[derive(Clone)]
struct AppState {
	handler: Arc<McpHandler>,
}

#[tokio::main]
async fn main() -> Result<()> {
	let args = Args::parse();

	// Initialize logging
	tracing_subscriber::fmt()
		.with_env_filter(&args.log_level)
		.init();

	let _config = ServerConfig {
		host: args.host.clone(),
		port: args.port,
		ckb_rpc_url: None,
		log_level: args.log_level,
	};

	// Initialize docs provider
	let docs_provider = DocsProvider::new(args.docs_path)?;

	// Initialize MCP handler
	let handler = Arc::new(McpHandler::new(docs_provider));

	let state = AppState { handler };

	// Build router
	let app = Router::new()
		.route("/", get(mcp_info_handler))
		.route("/mcp", post(mcp_handler))
		.route("/health", get(health_handler))
		.layer(
			CorsLayer::new()
				.allow_origin("*".parse::<HeaderValue>().unwrap())
				.allow_methods([axum::http::Method::GET, axum::http::Method::POST])
				.allow_headers([axum::http::header::CONTENT_TYPE]),
		)
		.with_state(state);

	let addr = format!("{}:{}", args.host, args.port);
	info!("Starting CKB Docs MCP server on {}", addr);

	let listener = tokio::net::TcpListener::bind(&addr).await?;
	axum::serve(listener, app).await?;

	Ok(())
}


async fn mcp_handler(
	State(state): State<AppState>,
	Json(request): Json<shared::mcp::McpRequest>,
) -> std::result::Result<Json<shared::mcp::McpResponse>, StatusCode> {
	match state.handler.handle_request(request).await {
		Ok(response) => Ok(Json(response)),
		Err(e) => {
			tracing::warn!("MCP request failed: {}", e);
			Err(StatusCode::INTERNAL_SERVER_ERROR)
		}
	}
}

async fn mcp_info_handler() -> Json<serde_json::Value> {
	Json(serde_json::json!({
		"name": "ckb-docs-server",
		"version": "0.1.0",
		"description": "CKB Documentation MCP Server",
		"endpoints": {
			"mcp": "/mcp",
			"sse": "/sse",
			"health": "/health"
		},
		"transport": ["http"]
	}))
}

async fn health_handler() -> &'static str {
	"OK"
}