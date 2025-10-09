use clap::Parser;
use shared::{
	error::Result,
	server::{HasMcpHandler, McpHandlerTrait, McpServerConfig},
};
use std::sync::Arc;

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

impl HasMcpHandler for AppState {
	type Handler = McpHandler;

	fn handler(&self) -> &Arc<Self::Handler> {
		&self.handler
	}

	fn server_info_json(&self) -> serde_json::Value {
		serde_json::json!({
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

	// Initialize CKB RPC client
	let rpc_client = CkbRpcClient::new(&args.ckb_rpc)?;

	// Initialize MCP handler
	let handler = Arc::new(McpHandler::new(rpc_client));

	let state = AppState { handler };

	// Configure and start server
	let config = McpServerConfig::new(
		args.host,
		args.port,
		"CKB RPC MCP server".to_string(),
		args.log_level,
	);

	config.serve(state).await
}
