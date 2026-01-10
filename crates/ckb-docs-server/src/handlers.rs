use shared::{
	error::Result,
	mcp::{create_error_response, create_success_response, McpRequest, McpResponse, ResourceDefinition},
};
use serde_json::{json, Value};
use tracing::{debug, error, info};

use crate::docs::DocsProvider;

pub struct McpHandler {
	docs_provider: DocsProvider,
}

impl McpHandler {
	pub fn new(docs_provider: DocsProvider) -> Self {
		Self { docs_provider }
	}

	pub async fn handle_request(&self, request: McpRequest) -> Result<McpResponse> {
		debug!("Handling MCP request: {}", request.method);

		match request.method.as_str() {
			"initialize" => self.handle_initialize(request.id).await,
			"resources/list" => self.handle_resources_list(request.id).await,
			"resources/read" => self.handle_resources_read(request.params, request.id).await,
			_ => Ok(create_error_response(
				request.id,
				-32601,
				format!("Method not found: {}", request.method),
			)),
		}
	}

	async fn handle_initialize(&self, id: Option<Value>) -> Result<McpResponse> {
		let result = json!({
			"protocolVersion": "2024-11-05",
			"capabilities": {
				"resources": {
					"subscribe": false,
					"listChanged": false
				}
			},
			"serverInfo": {
				"name": "ckb-docs-server",
				"version": env!("CARGO_PKG_VERSION")
			}
		});

		Ok(create_success_response(id, result))
	}

	async fn handle_resources_list(&self, id: Option<Value>) -> Result<McpResponse> {
		let resources_data = self.docs_provider.list_resources();
		
		let resources: Vec<ResourceDefinition> = resources_data
			.into_iter()
			.map(|(uri, name, description)| ResourceDefinition {
				uri,
				name,
				description: Some(description),
				mime_type: Some("text/markdown".to_string()),
			})
			.collect();

		let result = json!({ "resources": resources });
		Ok(create_success_response(id, result))
	}

	async fn handle_resources_read(&self, params: Option<Value>, id: Option<Value>) -> Result<McpResponse> {
		let params = params.ok_or_else(|| {
			shared::error::CkbMcpError::InvalidParameter("Missing parameters".to_string())
		})?;

		let uri = params
			.get("uri")
			.and_then(|v| v.as_str())
			.ok_or_else(|| {
				shared::error::CkbMcpError::InvalidParameter("Missing URI".to_string())
			})?;

		info!("Reading resource: {}", uri);

		match self.docs_provider.get_resource(uri) {
			Ok(content) => {
				let result = json!({
					"contents": [{
						"uri": uri,
						"mimeType": "text/markdown",
						"text": content
					}]
				});
				Ok(create_success_response(id, result))
			}
			Err(e) => {
				error!("Failed to read resource {}: {}", uri, e);
				Ok(create_error_response(id, -32602, e.to_string()))
			}
		}
	}
}