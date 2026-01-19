//! MCP server capabilities and handler implementation.

use rmcp::handler::server::ServerHandler;
use rmcp::model::{
	CallToolResult, Content, ErrorData, GetPromptResult, Implementation, ListPromptsResult,
	ListResourcesResult, ListToolsResult, PaginatedRequestParam, ProtocolVersion,
	ReadResourceResult, ServerCapabilities, ServerInfo,
};
use rmcp::service::RequestContext;
use shared::ckb_client::CkbRpcClient;
use std::sync::Arc;
use tracing::{debug, info};

use crate::rpc::{RpcHandlers, RPC_TOOLS};
use crate::ServerConfig;

/// Main MCP server implementing all capabilities.
#[derive(Clone)]
pub struct CkbMcpServer {
	config: ServerConfig,
	rpc_handlers: Option<Arc<RpcHandlers>>,
}

impl CkbMcpServer {
	/// Create a new CKB MCP server instance.
	pub fn new(config: ServerConfig) -> Self {
		info!("Creating new CKB MCP server instance");

		// Create RPC handlers if RPC tools are enabled.
		let rpc_handlers = if config.args.rpc_enabled() {
			match CkbRpcClient::new(&config.args.ckb_rpc) {
				Ok(client) => {
					info!("RPC client created for {}", config.args.ckb_rpc);
					Some(Arc::new(RpcHandlers::new(client)))
				}
				Err(e) => {
					tracing::error!("Failed to create RPC client: {}", e);
					None
				}
			}
		} else {
			None
		};

		Self {
			config,
			rpc_handlers,
		}
	}
}

impl ServerHandler for CkbMcpServer {
	fn get_info(&self) -> ServerInfo {
		// Build capabilities based on enabled features.
		// Use a builder approach that works with rmcp's type-state pattern.
		let capabilities = if self.config.args.rpc_enabled() || self.config.args.tools_enabled() {
			if self.config.args.docs_enabled() {
				if self.config.args.prompts_enabled() {
					// All enabled: tools + resources + prompts.
					ServerCapabilities::builder()
						.enable_tools()
						.enable_resources()
						.enable_prompts()
						.build()
				} else {
					// Tools + resources only.
					ServerCapabilities::builder()
						.enable_tools()
						.enable_resources()
						.build()
				}
			} else if self.config.args.prompts_enabled() {
				// Tools + prompts only.
				ServerCapabilities::builder()
					.enable_tools()
					.enable_prompts()
					.build()
			} else {
				// Tools only.
				ServerCapabilities::builder().enable_tools().build()
			}
		} else if self.config.args.docs_enabled() {
			if self.config.args.prompts_enabled() {
				// Resources + prompts.
				ServerCapabilities::builder()
					.enable_resources()
					.enable_prompts()
					.build()
			} else {
				// Resources only.
				ServerCapabilities::builder().enable_resources().build()
			}
		} else if self.config.args.prompts_enabled() {
			// Prompts only.
			ServerCapabilities::builder().enable_prompts().build()
		} else {
			// Nothing enabled.
			ServerCapabilities::builder().build()
		};

		ServerInfo {
			protocol_version: ProtocolVersion::V_2025_03_26,
			capabilities,
			server_info: Implementation {
				name: "ckb-ai-mcp".to_string(),
				version: "1.0.0".to_string(),
				title: None,
				website_url: None,
				icons: None,
			},
			instructions: Some(
				"CKB blockchain development server providing RPC queries, development tools, \
				 documentation, and guided workflows for building CKB smart contracts and applications."
					.to_string(),
			),
		}
	}

	async fn list_tools(
		&self,
		_request: Option<PaginatedRequestParam>,
		_context: RequestContext<rmcp::service::RoleServer>,
	) -> Result<ListToolsResult, ErrorData> {
		debug!("list_tools called");

		let mut tools = Vec::new();

		// Add RPC tools if enabled.
		if self.config.args.rpc_enabled() {
			tools.extend(RPC_TOOLS.iter().cloned());
		}

		// TODO: Add dev tools in Phase 4.
		// TODO: Add search tools in Phase 5.

		// Record stats.
		// Note: We don't record list_tools calls in stats since it's called frequently.

		Ok(ListToolsResult {
			tools,
			next_cursor: None,
			meta: None,
		})
	}

	async fn call_tool(
		&self,
		request: rmcp::model::CallToolRequestParam,
		_context: RequestContext<rmcp::service::RoleServer>,
	) -> Result<CallToolResult, ErrorData> {
		debug!("call_tool called: {}", request.name);

		let name: &str = &request.name;
		let arguments = serde_json::Value::Object(request.arguments.unwrap_or_default());

		// Route to appropriate handler.
		let result = if RpcHandlers::is_rpc_tool(name) {
			if !self.config.args.rpc_enabled() {
				return Err(ErrorData::invalid_params("RPC tools are disabled", None));
			}
			if let Some(ref handlers) = self.rpc_handlers {
				handlers.handle(name, &arguments).await
			} else {
				return Err(ErrorData::invalid_params(
					"RPC client not initialized",
					None,
				));
			}
		} else {
			// TODO: Route to dev tools in Phase 4.
			// TODO: Route to search tools in Phase 5.
			return Err(ErrorData::invalid_params(
				format!("Unknown tool: {}", name),
				None,
			));
		};

		match result {
			Ok(call_result) => {
				// Record successful tool call in stats.
				self.config.stats.record_tool_call(name);
				Ok(call_result)
			}
			Err(e) => {
				// Record error in stats.
				self.config.stats.record_error();
				Ok(CallToolResult::error(vec![Content::text(e.to_string())]))
			}
		}
	}

	async fn list_resources(
		&self,
		_request: Option<PaginatedRequestParam>,
		_context: RequestContext<rmcp::service::RoleServer>,
	) -> Result<ListResourcesResult, ErrorData> {
		debug!("list_resources called");

		if !self.config.args.docs_enabled() {
			return Err(ErrorData::invalid_params("Resources are disabled", None));
		}

		// Phase 1: Return empty resources list.
		// Phase 3 will add actual resources.
		Ok(ListResourcesResult {
			resources: vec![],
			next_cursor: None,
			meta: None,
		})
	}

	async fn read_resource(
		&self,
		request: rmcp::model::ReadResourceRequestParam,
		_context: RequestContext<rmcp::service::RoleServer>,
	) -> Result<ReadResourceResult, ErrorData> {
		debug!("read_resource called: {}", request.uri);

		if !self.config.args.docs_enabled() {
			return Err(ErrorData::invalid_params("Resources are disabled", None));
		}

		// Phase 1: Return resource not found.
		// Phase 3 will implement resource reading.
		Err(ErrorData::invalid_params(
			format!("Resource '{}' not implemented yet", request.uri),
			None,
		))
	}

	async fn list_prompts(
		&self,
		_request: Option<PaginatedRequestParam>,
		_context: RequestContext<rmcp::service::RoleServer>,
	) -> Result<ListPromptsResult, ErrorData> {
		debug!("list_prompts called");

		if !self.config.args.prompts_enabled() {
			return Err(ErrorData::invalid_params("Prompts are disabled", None));
		}

		// Phase 1: Return empty prompts list.
		// Phase 6 will add actual prompts.
		Ok(ListPromptsResult {
			prompts: vec![],
			next_cursor: None,
			meta: None,
		})
	}

	async fn get_prompt(
		&self,
		request: rmcp::model::GetPromptRequestParam,
		_context: RequestContext<rmcp::service::RoleServer>,
	) -> Result<GetPromptResult, ErrorData> {
		debug!("get_prompt called: {}", request.name);

		if !self.config.args.prompts_enabled() {
			return Err(ErrorData::invalid_params("Prompts are disabled", None));
		}

		// Phase 1: Return prompt not found.
		// Phase 6 will implement prompt handling.
		Err(ErrorData::invalid_params(
			format!("Prompt '{}' not implemented yet", request.name),
			None,
		))
	}
}
