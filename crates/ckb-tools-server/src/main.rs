use axum::{
	extract::{Multipart, Query, State},
	http::{HeaderValue, StatusCode},
	response::{IntoResponse, Response},
	routing::{get, post},
	Json, Router,
};
use clap::Parser;
use serde::{Deserialize, Serialize};
use shared::{
	ckb_client::CkbRpcClient,
	error::Result,
	mcp::{McpRequest, McpResponse},
	stats::Stats,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::{error, info};

mod handlers;
mod tools;
use handlers::McpHandler;
use tools::ToolsProvider;

#[derive(Parser)]
#[command(name = "ckb-tools-server")]
#[command(about = "CKB Development Tools MCP Server")]
struct Args {
	/// Port to bind to
	#[arg(short, long, default_value = "8003")]
	port: u16,

	/// Host to bind to
	#[arg(long, default_value = "0.0.0.0")]
	host: String,

	/// CKB node RPC URL
	#[arg(long, default_value = "http://127.0.0.1:8114")]
	ckb_rpc: String,

	/// Private key for signing transactions (hex format with or without 0x prefix)
	/// Default: Test key for development - DO NOT USE IN PRODUCTION
	/// Default key: 0xd7a9c7138ff3963efdd222033c90d7241d99122beeefd9bfbca17dd12d39c9ca
	#[arg(long, default_value = "0xd7a9c7138ff3963efdd222033c90d7241d99122beeefd9bfbca17dd12d39c9ca")]
	private_key: String,

	/// Log level
	#[arg(long, default_value = "info")]
	log_level: String,

	/// Stats database path
	#[arg(long, default_value = "./data/ckb-tools-stats.redb")]
	stats_db: String,
}

#[derive(Clone)]
struct AppState {
	handler: Arc<McpHandler>,
	tools_provider: Arc<ToolsProvider>,
	stats: Arc<Stats>,
}

/// Response from the file upload endpoint.
#[derive(Serialize)]
struct UploadResponse {
	tx_hash: String,
	output_index: u32,
	data_size: usize,
	capacity: u64,
}

/// Error response from the file upload endpoint.
#[derive(Serialize)]
struct UploadError {
	error: String,
}

#[tokio::main]
async fn main() -> Result<()> {
	let args = Args::parse();

	// Initialize logging
	tracing_subscriber::fmt()
		.with_env_filter(&args.log_level)
		.init();

	// Initialize CKB client
	let ckb_client = CkbRpcClient::new(args.ckb_rpc.clone())?;

	// Initialize tools provider
	let tools_provider = ToolsProvider::new(ckb_client, args.ckb_rpc, args.private_key)?;
	let tools_provider = Arc::new(tools_provider);

	// Initialize stats tracking
	let stats = Arc::new(Stats::open(&args.stats_db)?);

	// Initialize MCP handler
	let handler = Arc::new(McpHandler::new((*tools_provider).clone()));

	let state = AppState {
		handler,
		tools_provider,
		stats,
	};

	// Build router with MCP and HTTP endpoints
	let app = Router::new()
		.route("/", get(server_info_handler))
		.route("/mcp", post(mcp_handler))
		.route("/health", get(health_handler))
		.route("/stats", get(stats_handler))
		.route("/deploy/file", post(upload_file_handler))
		.layer(
			CorsLayer::new()
				.allow_origin("*".parse::<HeaderValue>().unwrap())
				.allow_methods([axum::http::Method::GET, axum::http::Method::POST])
				.allow_headers([axum::http::header::CONTENT_TYPE]),
		)
		.with_state(state);

	let addr = format!("{}:{}", args.host, args.port);
	info!("Starting CKB Tools MCP server on {}", addr);

	let listener = tokio::net::TcpListener::bind(&addr).await?;
	axum::serve(listener, app).await?;

	Ok(())
}

/// Server info handler.
async fn server_info_handler() -> Json<serde_json::Value> {
	Json(serde_json::json!({
		"name": "ckb-tools-server",
		"version": env!("CARGO_PKG_VERSION"),
		"description": "CKB Development Tools MCP Server",
		"endpoints": {
			"mcp": "/mcp",
			"deploy_file": "/deploy/file",
			"health": "/health",
			"stats": "/stats"
		},
		"transport": ["http"]
	}))
}

/// MCP request handler.
async fn mcp_handler(
	State(state): State<AppState>,
	Json(request): Json<McpRequest>,
) -> std::result::Result<Json<McpResponse>, StatusCode> {
	match state.handler.handle_request(request).await {
		Ok(response) => Ok(Json(response)),
		Err(e) => {
			error!("MCP request failed: {}", e);
			Err(StatusCode::INTERNAL_SERVER_ERROR)
		}
	}
}

/// Health check handler.
async fn health_handler() -> &'static str {
	"OK"
}

/// Query parameters for stats endpoint.
#[derive(Debug, Deserialize)]
struct StatsQuery {
	/// Output format: human (default), json, prometheus
	#[serde(default)]
	format: Option<String>,
}

/// Stats endpoint handler.
async fn stats_handler(
	State(state): State<AppState>,
	Query(query): Query<StatsQuery>,
) -> Response {
	let format = query.format.as_deref().unwrap_or("human");

	match format {
		"json" => match state.stats.format_json() {
			Ok(json) => (
				StatusCode::OK,
				[("content-type", "application/json")],
				json,
			)
				.into_response(),
			Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
		},
		"prometheus" => match state.stats.format_prometheus() {
			Ok(prom) => (
				StatusCode::OK,
				[("content-type", "text/plain; version=0.0.4")],
				prom,
			)
				.into_response(),
			Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
		},
		_ => match state.stats.format_human() {
			Ok(human) => (StatusCode::OK, [("content-type", "text/plain")], human).into_response(),
			Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
		},
	}
}

/// File upload handler for deploying large files.
async fn upload_file_handler(
	State(state): State<AppState>,
	mut multipart: Multipart,
) -> std::result::Result<Json<UploadResponse>, (StatusCode, Json<UploadError>)> {
	// Extract file from multipart form
	let mut file_data: Option<Vec<u8>> = None;

	while let Some(field) = multipart.next_field().await.map_err(|e| {
		(
			StatusCode::BAD_REQUEST,
			Json(UploadError {
				error: format!("Failed to read multipart field: {}", e),
			}),
		)
	})? {
		let name = field.name().unwrap_or("").to_string();
		if name == "file" {
			let data = field.bytes().await.map_err(|e| {
				(
					StatusCode::BAD_REQUEST,
					Json(UploadError {
						error: format!("Failed to read file data: {}", e),
					}),
				)
			})?;
			file_data = Some(data.to_vec());
			break;
		}
	}

	let data = file_data.ok_or_else(|| {
		(
			StatusCode::BAD_REQUEST,
			Json(UploadError {
				error: "No 'file' field found in multipart form. Use: curl -F 'file=@/path/to/file' <url>/deploy/file".to_string(),
			}),
		)
	})?;

	info!("Received file upload: {} bytes", data.len());

	// Deploy using tools provider
	let result = state.tools_provider.deploy_cell_data(data).await.map_err(|e| {
		error!("File deployment failed: {}", e);
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(UploadError {
				error: format!("Deployment failed: {}", e),
			}),
		)
	})?;

	info!(
		"File deployed successfully: tx_hash={}, output_index={}",
		result.tx_hash, result.output_index
	);

	Ok(Json(UploadResponse {
		tx_hash: result.tx_hash,
		output_index: result.output_index,
		data_size: result.data_size,
		capacity: result.capacity,
	}))
}
