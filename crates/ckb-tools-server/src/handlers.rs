use shared::{
	error::Result,
	mcp::{create_error_response, create_success_response, McpRequest, McpResponse, ToolDefinition},
};
use serde_json::{json, Value};
use tracing::{debug, error, info};

use crate::tools::ToolsProvider;

pub struct McpHandler {
	tools_provider: ToolsProvider,
}

impl McpHandler {
	pub fn new(tools_provider: ToolsProvider) -> Self {
		Self { tools_provider }
	}

	pub async fn handle_request(&self, request: McpRequest) -> Result<McpResponse> {
		debug!("Handling MCP request: {}", request.method);

		match request.method.as_str() {
			"initialize" => self.handle_initialize(request.id).await,
			"tools/list" => self.handle_tools_list(request.id).await,
			"tools/call" => self.handle_tools_call(request.params, request.id).await,
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
				"tools": {
					"listChanged": false
				}
			},
			"serverInfo": {
				"name": "ckb-tools-server",
				"version": "0.1.0"
			}
		});

		Ok(create_success_response(id, result))
	}

	async fn handle_tools_list(&self, id: Option<Value>) -> Result<McpResponse> {
		let tools = vec![
			ToolDefinition {
				name: "generate_contract".to_string(),
				description: "Generate a new CKB smart contract from template".to_string(),
				input_schema: json!({
					"type": "object",
					"properties": {
						"name": {
							"type": "string",
							"description": "Contract name"
						},
						"contract_type": {
							"type": "string",
							"enum": ["lock", "type"],
							"description": "Type of contract to generate"
						}
					},
					"required": ["name", "contract_type"]
				}),
			},
			ToolDefinition {
				name: "compile_contract".to_string(),
				description: "Compile a CKB smart contract using Capsule".to_string(),
				input_schema: json!({
					"type": "object",
					"properties": {
						"contract_path": {
							"type": "string",
							"description": "Path or name of the contract to compile"
						}
					},
					"required": ["contract_path"]
				}),
			},
			ToolDefinition {
				name: "run_tests".to_string(),
				description: "Run tests for CKB contracts".to_string(),
				input_schema: json!({
					"type": "object",
					"properties": {
						"contract_name": {
							"type": "string",
							"description": "Specific contract to test (optional)"
						}
					}
				}),
			},
			ToolDefinition {
				name: "deploy_contract".to_string(),
				description: "Deploy a contract to CKB network".to_string(),
				input_schema: json!({
					"type": "object",
					"properties": {
						"contract_name": {
							"type": "string",
							"description": "Name of the contract to deploy"
						},
						"address": {
							"type": "string",
							"description": "Deployment address"
						},
						"env": {
							"type": "string",
							"enum": ["testnet", "mainnet"],
							"description": "Target environment"
						}
					},
					"required": ["contract_name", "address", "env"]
				}),
			},
			ToolDefinition {
				name: "format_code".to_string(),
				description: "Format Rust code using cargo fmt".to_string(),
				input_schema: json!({
					"type": "object",
					"properties": {
						"file_path": {
							"type": "string",
							"description": "Specific file to format (optional)"
						}
					}
				}),
			},
			ToolDefinition {
				name: "create_project".to_string(),
				description: "Create a new CKB project".to_string(),
				input_schema: json!({
					"type": "object",
					"properties": {
						"name": {
							"type": "string",
							"description": "Project name"
						},
						"project_type": {
							"type": "string",
							"enum": ["capsule"],
							"description": "Type of project to create"
						}
					},
					"required": ["name", "project_type"]
				}),
			},
		];

		let result = json!({ "tools": tools });
		Ok(create_success_response(id, result))
	}

	async fn handle_tools_call(&self, params: Option<Value>, id: Option<Value>) -> Result<McpResponse> {
		let params = params.ok_or_else(|| {
			shared::error::CkbMcpError::InvalidParameter("Missing parameters".to_string())
		})?;

		let tool_name = params
			.get("name")
			.and_then(|v| v.as_str())
			.ok_or_else(|| {
				shared::error::CkbMcpError::InvalidParameter("Missing tool name".to_string())
			})?;

		let default_args = json!({});
		let arguments = params.get("arguments").unwrap_or(&default_args);

		info!("Calling tool: {} with arguments: {}", tool_name, arguments);

		let result = match tool_name {
			"generate_contract" => self.call_generate_contract(arguments).await,
			"compile_contract" => self.call_compile_contract(arguments).await,
			"run_tests" => self.call_run_tests(arguments).await,
			"deploy_contract" => self.call_deploy_contract(arguments).await,
			"format_code" => self.call_format_code(arguments).await,
			"create_project" => self.call_create_project(arguments).await,
			_ => {
				return Ok(create_error_response(
					id,
					-32602,
					format!("Unknown tool: {}", tool_name),
				))
			}
		};

		match result {
			Ok(data) => Ok(create_success_response(
				id,
				json!({
					"content": [{
						"type": "text",
						"text": data
					}]
				}),
			)),
			Err(e) => {
				error!("Tool call failed: {}", e);
				Ok(create_error_response(id, -32603, e.to_string()))
			}
		}
	}

	async fn call_generate_contract(&self, args: &Value) -> Result<String> {
		let name = args
			.get("name")
			.and_then(|v| v.as_str())
			.ok_or_else(|| {
				shared::error::CkbMcpError::InvalidParameter("Missing contract name".to_string())
			})?;

		let contract_type = args
			.get("contract_type")
			.and_then(|v| v.as_str())
			.ok_or_else(|| {
				shared::error::CkbMcpError::InvalidParameter("Missing contract type".to_string())
			})?;

		self.tools_provider.generate_contract(name, contract_type).await
	}

	async fn call_compile_contract(&self, args: &Value) -> Result<String> {
		let contract_path = args
			.get("contract_path")
			.and_then(|v| v.as_str())
			.ok_or_else(|| {
				shared::error::CkbMcpError::InvalidParameter("Missing contract path".to_string())
			})?;

		self.tools_provider.compile_contract(contract_path).await
	}

	async fn call_run_tests(&self, args: &Value) -> Result<String> {
		let contract_name = args.get("contract_name").and_then(|v| v.as_str());
		self.tools_provider.run_tests(contract_name).await
	}

	async fn call_deploy_contract(&self, args: &Value) -> Result<String> {
		let contract_name = args
			.get("contract_name")
			.and_then(|v| v.as_str())
			.ok_or_else(|| {
				shared::error::CkbMcpError::InvalidParameter("Missing contract name".to_string())
			})?;

		let address = args
			.get("address")
			.and_then(|v| v.as_str())
			.ok_or_else(|| {
				shared::error::CkbMcpError::InvalidParameter("Missing address".to_string())
			})?;

		let env = args
			.get("env")
			.and_then(|v| v.as_str())
			.ok_or_else(|| {
				shared::error::CkbMcpError::InvalidParameter("Missing environment".to_string())
			})?;

		self.tools_provider.deploy_contract(contract_name, address, env).await
	}

	async fn call_format_code(&self, args: &Value) -> Result<String> {
		let file_path = args.get("file_path").and_then(|v| v.as_str());
		self.tools_provider.format_code(file_path).await
	}

	async fn call_create_project(&self, args: &Value) -> Result<String> {
		let name = args
			.get("name")
			.and_then(|v| v.as_str())
			.ok_or_else(|| {
				shared::error::CkbMcpError::InvalidParameter("Missing project name".to_string())
			})?;

		let project_type = args
			.get("project_type")
			.and_then(|v| v.as_str())
			.ok_or_else(|| {
				shared::error::CkbMcpError::InvalidParameter("Missing project type".to_string())
			})?;

		self.tools_provider.create_project(name, project_type).await
	}
}