use clap::Parser;
use shared::{
	ckb_client::CkbRpcClient,
	error::Result,
	server::{HasMcpHandler, McpHandlerTrait, McpServerConfig},
};
use std::sync::Arc;

mod handlers;
mod session;
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
}

#[derive(Clone)]
struct AppState {
	handler: Arc<McpHandler>,
}

impl HasMcpHandler for AppState {
	type Handler = McpHandler;

	fn handler(&self) -> &Arc<Self::Handler> {
		&self.handler
	}

	fn server_info_json(&self) -> serde_json::Value {
		serde_json::json!({
			"name": "ckb-tools-server",
			"version": "0.1.0",
			"description": "CKB Development Tools MCP Server",
			"endpoints": {
				"mcp": "/mcp",
				"sse": "/sse",
				"health": "/health"
			},
			"transport": ["http"]
		})
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

	// Initialize CKB client
	let ckb_client = CkbRpcClient::new(args.ckb_rpc.clone())?;

	// Initialize tools provider
	let tools_provider = ToolsProvider::new(ckb_client, args.ckb_rpc, args.private_key)?;

	// Initialize MCP handler
	let handler = Arc::new(McpHandler::new(tools_provider));

	let state = AppState { handler };

	// Configure and start server
	let config = McpServerConfig::new(
		args.host,
		args.port,
		"CKB Tools MCP server".to_string(),
		args.log_level,
	);

	config.serve(state).await
}
