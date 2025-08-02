use axum::{
	extract::State,
	http::{HeaderValue, StatusCode},
	routing::{get, post},
	Json, Router,
};
use clap::Parser;
use shared::{error::Result, types::ServerConfig};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};

mod handlers;
mod rpc;

use handlers::McpHandler;
use rpc::CkbRpcClient;

#[derive(Parser)]
#[command(name = "ckb-rpc-server")]
#[command(about = "CKB RPC MCP Server")]
struct Args {
	/// Port to bind to
	#[arg(short, long, default_value = "8001")]
	port: u16,

	/// Host to bind to
	#[arg(long, default_value = "0.0.0.0")]
	host: String,

	/// CKB node RPC URL
	#[arg(long, default_value = "http://127.0.0.1:8114")]
	ckb_rpc: String,

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

	let config = ServerConfig {
		host: args.host.clone(),
		port: args.port,
		ckb_rpc_url: Some(args.ckb_rpc),
		log_level: args.log_level,
	};

	// Initialize CKB RPC client
	let rpc_client = CkbRpcClient::new(config.ckb_rpc_url.as_ref().unwrap())?;

	// Initialize MCP handler
	let handler = Arc::new(McpHandler::new(rpc_client));

	let state = AppState { handler };

	// Build router
	let app = Router::new()
		.route("/", get(mcp_info_handler))
		.route("/register", post(register_handler))
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
	info!("Starting CKB RPC MCP server on {}", addr);

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
			warn!("MCP request failed: {}", e);
			Err(StatusCode::INTERNAL_SERVER_ERROR)
		}
	}
}

async fn mcp_info_handler() -> Json<serde_json::Value> {
	Json(serde_json::json!({
		"name": "ckb-rpc-server",
		"version": "0.1.0",
		"description": "CKB RPC MCP Server",
		"mcp_endpoint": "/mcp",
		"sse_endpoint": "/sse",
		"health_endpoint": "/health",
		"transport": ["http"],
		"capabilities": {
			"tools": true,
			"resources": false,
			"prompts": false
		}
	}))
}

async fn register_handler() -> Json<serde_json::Value> {
	// Simple registration response - no OAuth required for our basic MCP server
	Json(serde_json::json!({
		"client_id": "ckb-mcp-client",
		"client_name": "CKB MCP Client",
		"grant_types": ["client_credentials"],
		"token_endpoint_auth_method": "none"
	}))
}

async fn health_handler() -> &'static str {
	"OK"
}