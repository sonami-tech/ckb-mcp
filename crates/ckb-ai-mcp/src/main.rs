//! CKB AI MCP Server - Unified MCP server for CKB blockchain development.
//!
//! This server implements MCP protocol version 2025-11-25 with Streamable HTTP transport,
//! providing tools, resources, and prompts for CKB development.

use anyhow::Result;
use clap::Parser;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::EnvFilter;

mod capabilities;
mod ckb;
mod dev;
mod docs;
mod jsonrpc;
mod middleware;
mod prompts;
mod rpc;
mod search;
mod server;
mod util;

/// Default test private key for development (DO NOT USE IN PRODUCTION).
const DEFAULT_TEST_PRIVATE_KEY: &str =
	"0x6109170b275a09ad54877b82f7d9930f88cab5717d484fb4741c9f0c0571c078";

/// CKB AI MCP Server - Unified MCP server for CKB blockchain development.
#[derive(Parser, Debug, Clone)]
#[command(name = "ckb-ai-mcp")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Unified MCP server for CKB blockchain development")]
pub struct Args {
	/// Server port.
	#[arg(long, default_value = "3112")]
	pub port: u16,

	/// Server host.
	#[arg(long, default_value = "0.0.0.0")]
	pub host: String,

	/// CKB node RPC URL.
	#[arg(long, default_value = "http://127.0.0.1:8114")]
	pub ckb_rpc: String,

	/// Private key for signing transactions (hex-encoded with 0x prefix).
	#[arg(long, default_value = DEFAULT_TEST_PRIVATE_KEY)]
	pub private_key: String,

	/// Path to documentation directory.
	#[arg(long, default_value = "./docs")]
	pub docs_path: PathBuf,

	/// Path to stats database.
	#[arg(long, default_value = "./data/ckb-ai-mcp-stats.redb")]
	pub stats_db: PathBuf,

	/// Log level (trace, debug, info, warn, error).
	#[arg(long, default_value = "info")]
	pub log_level: String,

	/// Enable only documentation and prompts (no CKB node required).
	#[arg(long, default_value = "false")]
	pub docs_only: bool,

	/// Enable only RPC tools (requires CKB node).
	#[arg(long, default_value = "false")]
	pub rpc_only: bool,

	/// Enable only development tools (requires CKB node).
	#[arg(long, default_value = "false")]
	pub tools_only: bool,

	/// Disable prompts feature.
	#[arg(long, default_value = "false")]
	pub no_prompts: bool,
}

impl Args {
	/// Check if CKB node connection is required based on enabled features.
	pub fn requires_ckb_node(&self) -> bool {
		// If docs_only is enabled, no CKB node is required.
		if self.docs_only {
			return false;
		}
		// Otherwise, RPC or tools features need CKB node.
		true
	}

	/// Check if RPC tools are enabled.
	pub fn rpc_enabled(&self) -> bool {
		// RPC is enabled when: not docs_only, and not exclusively tools_only.
		!self.docs_only && !self.tools_only
	}

	/// Check if development tools are enabled.
	pub fn tools_enabled(&self) -> bool {
		// Tools are enabled when: not docs_only, and not exclusively rpc_only.
		!self.docs_only && !self.rpc_only
	}

	/// Check if documentation resources are enabled.
	pub fn docs_enabled(&self) -> bool {
		// Docs are enabled if: docs_only set, or no exclusive flags set
		self.docs_only || (!self.rpc_only && !self.tools_only)
	}

	/// Check if prompts are enabled.
	pub fn prompts_enabled(&self) -> bool {
		!self.no_prompts && self.docs_enabled()
	}
}

/// Server configuration derived from CLI arguments.
#[derive(Clone)]
pub struct ServerConfig {
	pub args: Args,
	pub stats: Arc<shared::stats::Stats>,
}

#[tokio::main]
async fn main() -> Result<()> {
	let args = Args::parse();

	// Initialize logging.
	let filter = EnvFilter::try_new(&args.log_level).unwrap_or_else(|_| {
		EnvFilter::new(format!(
			"ckb_ai_mcp={},tower_http={}",
			args.log_level, args.log_level
		))
	});

	tracing_subscriber::fmt()
		.with_env_filter(filter)
		.with_target(true)
		.init();

	info!("Starting CKB AI MCP Server v{}", env!("CARGO_PKG_VERSION"));
	info!("Port: {}", args.port);
	info!("Host: {}", args.host);

	// Log enabled features.
	info!("Features:");
	info!("  RPC tools: {}", args.rpc_enabled());
	info!("  Dev tools: {}", args.tools_enabled());
	info!("  Documentation: {}", args.docs_enabled());
	info!("  Prompts: {}", args.prompts_enabled());

	if args.requires_ckb_node() {
		info!("CKB RPC: {}", args.ckb_rpc);
	} else {
		info!("CKB RPC: Not required (docs-only mode)");
	}

	// Initialize stats database.
	info!("Stats database: {:?}", args.stats_db);
	let stats = shared::stats::Stats::open(&args.stats_db)?;
	let stats = Arc::new(stats);

	let config = ServerConfig {
		args: args.clone(),
		stats,
	};

	// Build and run the server.
	let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;
	info!("Listening on {}", addr);

	server::run(addr, config).await?;

	Ok(())
}
