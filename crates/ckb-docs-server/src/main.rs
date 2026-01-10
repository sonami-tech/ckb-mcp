use clap::Parser;
use shared::{
	error::Result,
	server::{HasMcpHandler, McpHandlerTrait, McpServerConfig},
	stats::Stats,
};
use std::{path::PathBuf, sync::Arc};

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

	/// Stats database path
	#[arg(long, default_value = "./data/ckb-docs-stats.redb")]
	stats_db: String,
}

#[derive(Clone)]
struct AppState {
	handler: Arc<McpHandler>,
	stats: Arc<Stats>,
}

impl HasMcpHandler for AppState {
	type Handler = McpHandler;

	fn handler(&self) -> &Arc<Self::Handler> {
		&self.handler
	}

	fn server_info_json(&self) -> serde_json::Value {
		serde_json::json!({
			"name": "ckb-docs-server",
			"version": env!("CARGO_PKG_VERSION"),
			"description": "CKB Documentation MCP Server",
			"endpoints": {
				"mcp": "/mcp",
				"sse": "/sse",
				"health": "/health",
				"stats": "/stats"
			},
			"transport": ["http"]
		})
	}

	fn stats(&self) -> Option<&Arc<Stats>> {
		Some(&self.stats)
	}
}

#[axum::async_trait]
impl McpHandlerTrait for McpHandler {
	async fn handle_request(&self, request: shared::mcp::McpRequest) -> Result<shared::mcp::McpResponse> {
		self.handle_request(request).await
	}
}

#[tokio::main]
async fn main() -> Result<()> {
	let args = Args::parse();

	// Initialize docs provider
	let docs_provider = DocsProvider::new(args.docs_path)?;

	// Initialize stats tracking
	let stats = Arc::new(Stats::open(&args.stats_db)?);

	// Initialize MCP handler
	let handler = Arc::new(McpHandler::new(docs_provider));

	let state = AppState { handler, stats };

	// Configure and start server
	let config = McpServerConfig::new(
		args.host,
		args.port,
		"CKB Docs MCP server".to_string(),
		args.log_level,
	);

	config.serve(state).await
}
