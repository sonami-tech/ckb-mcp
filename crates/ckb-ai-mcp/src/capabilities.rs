//! MCP server capabilities and handler implementation.

use rmcp::handler::server::ServerHandler;
use rmcp::model::{
	CallToolRequestParams, CallToolResult, Content, ErrorData, GetPromptRequestParams,
	GetPromptResult, Implementation, ListPromptsResult, ListResourcesResult, ListToolsResult,
	PaginatedRequestParams, ProtocolVersion, ReadResourceRequestParams, ReadResourceResult,
	ServerCapabilities, ServerInfo,
};
use rmcp::service::RequestContext;
use shared::ckb_client::CkbRpcClient;
use std::sync::Arc;
use tracing::{debug, info};

use crate::ServerConfig;
use crate::ckb::{CKB_TOOLS, CkbHandlers};
use crate::dev::{DEV_TOOLS, DevHandlers};
use crate::docs::DocsHandlers;
use crate::prompts::{PROMPTS, PromptsHandlers};
use crate::rpc::{RPC_TOOLS, RpcHandlers};
use crate::search::{SEARCH_TOOLS, SearchHandlers};

/// Factory for creating CkbMcpServer instances with shared handlers.
#[derive(Clone)]
pub struct CkbMcpServerFactory {
	config: ServerConfig,
	dev_handlers: Option<Arc<DevHandlers>>,
}

impl CkbMcpServerFactory {
	/// Create a new factory with shared dev handlers.
	pub fn new(config: ServerConfig, dev_handlers: Option<Arc<DevHandlers>>) -> Self {
		Self {
			config,
			dev_handlers,
		}
	}

	/// Create a new CkbMcpServer instance.
	pub fn create(&self) -> Result<CkbMcpServer, std::io::Error> {
		Ok(CkbMcpServer::new_with_handlers(
			self.config.clone(),
			self.dev_handlers.clone(),
		))
	}
}

/// Main MCP server implementing all capabilities.
#[derive(Clone)]
pub struct CkbMcpServer {
	config: ServerConfig,
	rpc_handlers: Option<Arc<RpcHandlers>>,
	ckb_handlers: Option<Arc<CkbHandlers>>,
	dev_handlers: Option<Arc<DevHandlers>>,
	docs_handlers: Option<Arc<DocsHandlers>>,
	search_handlers: SearchHandlers,
	prompts_handlers: Option<PromptsHandlers>,
}

impl CkbMcpServer {
	/// Create a new CKB MCP server instance with pre-built dev handlers.
	/// This is the preferred constructor when dev handlers are shared (e.g., with HTTP endpoints).
	pub fn new_with_handlers(config: ServerConfig, dev_handlers: Option<Arc<DevHandlers>>) -> Self {
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

		// Use provided dev handlers or create new ones if tools are enabled but none provided.
		let dev_handlers = if config.args.tools_enabled() {
			dev_handlers.or_else(|| match CkbRpcClient::new(&config.args.ckb_rpc) {
				Ok(client) => {
					match DevHandlers::new(
						client,
						config.args.ckb_rpc.clone(),
						config.args.private_key.clone(),
					) {
						Ok(handlers) => {
							info!("Dev handlers created for {}", config.args.ckb_rpc);
							Some(Arc::new(handlers))
						}
						Err(e) => {
							tracing::error!("Failed to create dev handlers: {}", e);
							None
						}
					}
				}
				Err(e) => {
					tracing::error!("Failed to create CKB client for dev handlers: {}", e);
					None
				}
			})
		} else {
			None
		};

		// Create docs handlers if docs are enabled.
		let docs_handlers = if config.args.docs_enabled() {
			match DocsHandlers::new(config.args.docs_path.clone()) {
				Ok(handlers) => {
					info!("Docs handlers created from {:?}", config.args.docs_path);
					Some(Arc::new(handlers))
				}
				Err(e) => {
					tracing::error!("Failed to create docs handlers: {}", e);
					None
				}
			}
		} else {
			None
		};

		// Create CKB composite handlers if RPC or tools are enabled.
		let ckb_handlers = if config.args.rpc_enabled() || config.args.tools_enabled() {
			match CkbRpcClient::new(&config.args.ckb_rpc) {
				Ok(client) => {
					info!("CKB handlers created for {}", config.args.ckb_rpc);
					Some(Arc::new(CkbHandlers::new(client)))
				}
				Err(e) => {
					tracing::error!("Failed to create CKB client for CKB handlers: {}", e);
					None
				}
			}
		} else {
			None
		};

		// Create search handlers (always enabled).
		let search_handlers = SearchHandlers::new();

		// Create prompts handlers if prompts are enabled.
		let prompts_handlers = if config.args.prompts_enabled() {
			Some(PromptsHandlers::new())
		} else {
			None
		};

		Self {
			config,
			rpc_handlers,
			ckb_handlers,
			dev_handlers,
			docs_handlers,
			search_handlers,
			prompts_handlers,
		}
	}

	/// Create a new CKB MCP server instance (convenience method).
	/// Creates dev handlers internally. Use `new_with_handlers` to share handlers.
	#[allow(dead_code)]
	pub fn new(config: ServerConfig) -> Self {
		Self::new_with_handlers(config, None)
	}

	// =========================================================================
	// Internal methods for JSON-RPC endpoint
	// =========================================================================

	/// List all available tools.
	pub fn list_tools_internal(&self) -> Result<ListToolsResult, ErrorData> {
		debug!("list_tools_internal called");

		let mut tools = Vec::new();

		if self.config.args.rpc_enabled() {
			tools.extend(RPC_TOOLS.iter().cloned());
		}

		if self.config.args.tools_enabled() {
			tools.extend(DEV_TOOLS.iter().cloned());
		}

		// CKB composite tools available when RPC or tools are enabled.
		if self.config.args.rpc_enabled() || self.config.args.tools_enabled() {
			tools.extend(CKB_TOOLS.iter().cloned());
		}

		if !tools.is_empty() || self.config.args.docs_enabled() {
			tools.extend(SEARCH_TOOLS.iter().cloned());
		}

		Ok(ListToolsResult {
			tools,
			next_cursor: None,
			meta: None,
		})
	}

	/// Call a tool by name with arguments.
	pub async fn call_tool_internal(
		&self,
		name: &str,
		arguments: &serde_json::Value,
	) -> Result<CallToolResult, ErrorData> {
		debug!("call_tool_internal called: {}", name);

		let result = if RpcHandlers::is_rpc_tool(name) {
			if !self.config.args.rpc_enabled() {
				return Err(ErrorData::invalid_params("RPC tools are disabled", None));
			}
			if let Some(ref handlers) = self.rpc_handlers {
				handlers.handle(name, arguments).await
			} else {
				return Err(ErrorData::invalid_params(
					"RPC client not initialized",
					None,
				));
			}
		} else if DevHandlers::is_dev_tool(name) {
			if !self.config.args.tools_enabled() {
				return Err(ErrorData::invalid_params("Dev tools are disabled", None));
			}
			if let Some(ref handlers) = self.dev_handlers {
				handlers.handle(name, arguments).await
			} else {
				return Err(ErrorData::invalid_params(
					"Dev handlers not initialized",
					None,
				));
			}
		} else if CkbHandlers::is_ckb_tool(name) {
			if !self.config.args.rpc_enabled() && !self.config.args.tools_enabled() {
				return Err(ErrorData::invalid_params("CKB tools are disabled", None));
			}
			if let Some(ref handlers) = self.ckb_handlers {
				handlers.handle(name, arguments).await
			} else {
				return Err(ErrorData::invalid_params(
					"CKB handlers not initialized",
					None,
				));
			}
		} else if SearchHandlers::is_search_tool(name) {
			let mut tools = Vec::new();
			if self.config.args.rpc_enabled() {
				tools.extend(RPC_TOOLS.iter().cloned());
			}
			if self.config.args.tools_enabled() {
				tools.extend(DEV_TOOLS.iter().cloned());
			}
			if self.config.args.rpc_enabled() || self.config.args.tools_enabled() {
				tools.extend(CKB_TOOLS.iter().cloned());
			}
			tools.extend(SEARCH_TOOLS.iter().cloned());

			let resources = if let Some(ref handlers) = self.docs_handlers {
				handlers.list_resources()
			} else {
				vec![]
			};

			self.search_handlers
				.handle(name, arguments, &tools, &resources)
		} else {
			return Err(ErrorData::invalid_params(
				format!("Unknown tool: {}", name),
				None,
			));
		};

		match result {
			Ok(call_result) => {
				self.config.stats.record_tool_call(name);
				Ok(call_result)
			}
			Err(e) => {
				self.config.stats.record_error(name, &e.to_string());
				Ok(CallToolResult::error(vec![Content::text(e.to_string())]))
			}
		}
	}

	/// List all available resources.
	pub fn list_resources_internal(&self) -> Result<ListResourcesResult, ErrorData> {
		debug!("list_resources_internal called");

		if !self.config.args.docs_enabled() {
			return Err(ErrorData::invalid_params("Resources are disabled", None));
		}

		let resources = if let Some(ref handlers) = self.docs_handlers {
			handlers.list_resources()
		} else {
			return Err(ErrorData::invalid_params(
				"Docs handlers not initialized",
				None,
			));
		};

		Ok(ListResourcesResult {
			resources,
			next_cursor: None,
			meta: None,
		})
	}

	/// Read a resource by URI.
	pub fn read_resource_internal(&self, uri: &str) -> Result<ReadResourceResult, ErrorData> {
		debug!("read_resource_internal called: {}", uri);

		if !self.config.args.docs_enabled() {
			return Err(ErrorData::invalid_params("Resources are disabled", None));
		}

		if !DocsHandlers::is_docs_resource(uri) {
			return Err(ErrorData::invalid_params(
				format!("Invalid resource URI: {}", uri),
				None,
			));
		}

		if let Some(ref handlers) = self.docs_handlers {
			match handlers.read_resource(uri) {
				Ok(result) => {
					self.config.stats.record_resource_read(uri);
					Ok(result)
				}
				Err(e) => {
					self.config.stats.record_error(uri, &e.to_string());
					Err(ErrorData::invalid_params(e.to_string(), None))
				}
			}
		} else {
			Err(ErrorData::invalid_params(
				"Docs handlers not initialized",
				None,
			))
		}
	}

	/// List all available prompts.
	pub fn list_prompts_internal(&self) -> Result<ListPromptsResult, ErrorData> {
		debug!("list_prompts_internal called");

		if !self.config.args.prompts_enabled() {
			return Err(ErrorData::invalid_params("Prompts are disabled", None));
		}

		Ok(ListPromptsResult {
			prompts: PROMPTS.iter().cloned().collect(),
			next_cursor: None,
			meta: None,
		})
	}

	/// Get a prompt by name with optional arguments.
	pub fn get_prompt_internal(
		&self,
		name: &str,
		arguments: Option<serde_json::Value>,
	) -> Result<GetPromptResult, ErrorData> {
		debug!("get_prompt_internal called: {}", name);

		if !self.config.args.prompts_enabled() {
			return Err(ErrorData::invalid_params("Prompts are disabled", None));
		}

		if !PromptsHandlers::is_prompt(name) {
			return Err(ErrorData::invalid_params(
				format!("Unknown prompt: {}", name),
				None,
			));
		}

		if let Some(ref handlers) = self.prompts_handlers {
			let args = arguments.unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

			match handlers.handle(name, &args) {
				Ok(result) => {
					self.config
						.stats
						.record_tool_call(&format!("prompt:{}", name));
					Ok(result)
				}
				Err(e) => {
					self.config.stats.record_error(name, &e.to_string());
					Err(ErrorData::invalid_params(e.to_string(), None))
				}
			}
		} else {
			Err(ErrorData::invalid_params(
				"Prompts handlers not initialized",
				None,
			))
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
			protocol_version: ProtocolVersion::V_2025_06_18,
			capabilities,
			server_info: Implementation {
				name: "ckb-ai-mcp".to_string(),
				version: env!("CARGO_PKG_VERSION").to_string(),
				title: None,
				website_url: None,
				icons: None,
			},
			instructions: Some(
				"CKB blockchain development server providing RPC queries, development tools, \
				 documentation, and guided workflows for building CKB smart contracts and applications.\n\n\
				 Tool Discovery: This server uses deferred loading. The 'search_tools' and 'search_resources' \
				 tools are always available. Use them to discover relevant tools before calling them. \
				 Other tools are loaded on-demand when invoked."
					.to_string(),
			),
		}
	}

	async fn list_tools(
		&self,
		_request: Option<PaginatedRequestParams>,
		_context: RequestContext<rmcp::service::RoleServer>,
	) -> Result<ListToolsResult, ErrorData> {
		self.list_tools_internal()
	}

	async fn call_tool(
		&self,
		request: CallToolRequestParams,
		_context: RequestContext<rmcp::service::RoleServer>,
	) -> Result<CallToolResult, ErrorData> {
		let name: &str = &request.name;
		let arguments = serde_json::Value::Object(request.arguments.unwrap_or_default());
		self.call_tool_internal(name, &arguments).await
	}

	async fn list_resources(
		&self,
		_request: Option<PaginatedRequestParams>,
		_context: RequestContext<rmcp::service::RoleServer>,
	) -> Result<ListResourcesResult, ErrorData> {
		self.list_resources_internal()
	}

	async fn read_resource(
		&self,
		request: ReadResourceRequestParams,
		_context: RequestContext<rmcp::service::RoleServer>,
	) -> Result<ReadResourceResult, ErrorData> {
		self.read_resource_internal(&request.uri)
	}

	async fn list_prompts(
		&self,
		_request: Option<PaginatedRequestParams>,
		_context: RequestContext<rmcp::service::RoleServer>,
	) -> Result<ListPromptsResult, ErrorData> {
		self.list_prompts_internal()
	}

	async fn get_prompt(
		&self,
		request: GetPromptRequestParams,
		_context: RequestContext<rmcp::service::RoleServer>,
	) -> Result<GetPromptResult, ErrorData> {
		let args = request
			.arguments
			.map(|a| serde_json::Value::Object(a.into_iter().collect()));
		self.get_prompt_internal(&request.name, args)
	}
}
