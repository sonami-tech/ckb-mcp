use shared::error::{CkbMcpError, Result};
use std::{collections::HashMap, path::PathBuf, process::Command};
use tracing::{debug, info};

#[derive(Clone)]
pub struct ToolsProvider {
	_ckb_rpc_url: Option<String>,
	_workspace: Option<PathBuf>,
	templates: HashMap<String, String>,
}

impl ToolsProvider {
	pub fn new(ckb_rpc_url: Option<String>, workspace: Option<PathBuf>) -> Result<Self> {
		let mut provider = Self {
			_ckb_rpc_url: ckb_rpc_url,
			_workspace: workspace,
			templates: HashMap::new(),
		};

		provider.load_templates();
		info!("Initialized tools provider with {} templates", provider.templates.len());

		Ok(provider)
	}

	pub async fn generate_contract(&self, name: &str, contract_type: &str) -> Result<String> {
		let template = match contract_type {
			"lock" => self.get_lock_template(name),
			"type" => self.get_type_template(name),
			_ => return Err(CkbMcpError::InvalidParameter(
				"Contract type must be 'lock' or 'type'".to_string()
			)),
		};

		info!("Generated {} contract template for: {}", contract_type, name);
		Ok(template)
	}

	pub async fn compile_contract(&self, contract_path: &str) -> Result<String> {
		debug!("Compiling contract at path: {}", contract_path);

		// Check if capsule is available
		let output = Command::new("capsule")
			.args(&["build", "--name", contract_path])
			.output()
			.map_err(|e| CkbMcpError::Internal(format!("Failed to run capsule: {}", e)))?;

		if output.status.success() {
			let stdout = String::from_utf8_lossy(&output.stdout);
			let stderr = String::from_utf8_lossy(&output.stderr);
			Ok(format!("Compilation successful!\n\nSTDOUT:\n{}\n\nSTDERR:\n{}", stdout, stderr))
		} else {
			let stderr = String::from_utf8_lossy(&output.stderr);
			Err(CkbMcpError::Internal(format!("Compilation failed: {}", stderr)))
		}
	}

	pub async fn run_tests(&self, contract_name: Option<&str>) -> Result<String> {
		debug!("Running tests for contract: {:?}", contract_name);

		let mut args = vec!["test"];
		if let Some(name) = contract_name {
			args.extend(&["--name", name]);
		}

		let output = Command::new("capsule")
			.args(&args)
			.output()
			.map_err(|e| CkbMcpError::Internal(format!("Failed to run capsule test: {}", e)))?;

		if output.status.success() {
			let stdout = String::from_utf8_lossy(&output.stdout);
			let stderr = String::from_utf8_lossy(&output.stderr);
			Ok(format!("Tests passed!\n\nSTDOUT:\n{}\n\nSTDERR:\n{}", stdout, stderr))
		} else {
			let stderr = String::from_utf8_lossy(&output.stderr);
			Err(CkbMcpError::Internal(format!("Tests failed: {}", stderr)))
		}
	}

	pub async fn deploy_contract(&self, contract_name: &str, address: &str, env: &str) -> Result<String> {
		debug!("Deploying contract {} to {} using address {}", contract_name, env, address);

		let output = Command::new("capsule")
			.args(&["deploy", "--address", address, "--env", env, "--contract", contract_name])
			.output()
			.map_err(|e| CkbMcpError::Internal(format!("Failed to run capsule deploy: {}", e)))?;

		if output.status.success() {
			let stdout = String::from_utf8_lossy(&output.stdout);
			let stderr = String::from_utf8_lossy(&output.stderr);
			Ok(format!("Deployment successful!\n\nSTDOUT:\n{}\n\nSTDERR:\n{}", stdout, stderr))
		} else {
			let stderr = String::from_utf8_lossy(&output.stderr);
			Err(CkbMcpError::Internal(format!("Deployment failed: {}", stderr)))
		}
	}

	pub async fn format_code(&self, file_path: Option<&str>) -> Result<String> {
		debug!("Formatting code for file: {:?}", file_path);

		let mut args = vec!["fmt"];
		if let Some(path) = file_path {
			args.push(path);
		}

		let output = Command::new("cargo")
			.args(&args)
			.output()
			.map_err(|e| CkbMcpError::Internal(format!("Failed to run cargo fmt: {}", e)))?;

		if output.status.success() {
			let stdout = String::from_utf8_lossy(&output.stdout);
			let stderr = String::from_utf8_lossy(&output.stderr);
			Ok(format!("Code formatting completed!\n\nSTDOUT:\n{}\n\nSTDERR:\n{}", stdout, stderr))
		} else {
			let stderr = String::from_utf8_lossy(&output.stderr);
			Err(CkbMcpError::Internal(format!("Code formatting failed: {}", stderr)))
		}
	}

	pub async fn create_project(&self, name: &str, project_type: &str) -> Result<String> {
		debug!("Creating new project: {} of type: {}", name, project_type);

		let output = match project_type {
			"capsule" => {
				Command::new("capsule")
					.args(&["new", name])
					.output()
					.map_err(|e| CkbMcpError::Internal(format!("Failed to run capsule new: {}", e)))?
			},
			_ => return Err(CkbMcpError::InvalidParameter(
				"Project type must be 'capsule'".to_string()
			)),
		};

		if output.status.success() {
			let stdout = String::from_utf8_lossy(&output.stdout);
			let stderr = String::from_utf8_lossy(&output.stderr);
			Ok(format!("Project created successfully!\n\nSTDOUT:\n{}\n\nSTDERR:\n{}", stdout, stderr))
		} else {
			let stderr = String::from_utf8_lossy(&output.stderr);
			Err(CkbMcpError::Internal(format!("Project creation failed: {}", stderr)))
		}
	}

	fn load_templates(&mut self) {
		// Load basic contract templates
		self.templates.insert("lock".to_string(), self.get_default_lock_template());
		self.templates.insert("type".to_string(), self.get_default_type_template());
	}

	fn get_lock_template(&self, name: &str) -> String {
		let template = self.get_default_lock_template();
		template.replace("CONTRACT_NAME", name)
	}

	fn get_type_template(&self, name: &str) -> String {
		let template = self.get_default_type_template();
		template.replace("CONTRACT_NAME", name)
	}

	fn get_default_lock_template(&self) -> String {
		r#"#![no_std]
#![no_main]

use ckb_std::{
	debug,
	default_alloc,
	entry,
	error::SysError,
	high_level::{load_script, load_witness_args},
	ckb_constants::Source,
	ckb_types::{bytes::Bytes, prelude::*},
};

entry!(program_entry);
default_alloc!();

/// CONTRACT_NAME Lock Script
pub fn program_entry() -> i8 {
	match main() {
		Ok(_) => 0,
		Err(err) => err as i8,
	}
}

fn main() -> Result<(), SysError> {
	debug!("Starting CONTRACT_NAME lock script");

	// Load script args
	let script = load_script()?;
	let args: Bytes = script.args().unpack();
	
	if args.len() < 20 {
		return Err(SysError::Encoding);
	}

	// Load witness for signature verification
	let witness_args = load_witness_args(0, Source::GroupInput)?;
	let lock: Bytes = witness_args.lock().to_opt().unwrap().unpack();
	
	if lock.len() < 65 {
		return Err(SysError::Encoding);
	}

	// TODO: Implement signature verification logic
	// This is a placeholder - add your signature verification here
	debug!("Signature verification would happen here");

	debug!("CONTRACT_NAME lock script completed successfully");
	Ok(())
}
"#.to_string()
	}

	fn get_default_type_template(&self) -> String {
		r#"#![no_std]
#![no_main]

use ckb_std::{
	debug,
	default_alloc,
	entry,
	error::SysError,
	high_level::{load_cell_data, load_script, QueryIter},
	ckb_constants::Source,
	ckb_types::{bytes::Bytes, prelude::*},
};

entry!(program_entry);
default_alloc!();

/// CONTRACT_NAME Type Script
pub fn program_entry() -> i8 {
	match main() {
		Ok(_) => 0,
		Err(err) => err as i8,
	}
}

fn main() -> Result<(), SysError> {
	debug!("Starting CONTRACT_NAME type script");

	// Load script args
	let script = load_script()?;
	let args: Bytes = script.args().unpack();
	
	// Validate input cells
	for data in QueryIter::new(load_cell_data, Source::GroupInput) {
		let data: Bytes = data.unpack();
		
		// TODO: Add input validation logic
		if !validate_cell_data(&data) {
			return Err(SysError::Encoding);
		}
	}
	
	// Validate output cells
	for data in QueryIter::new(load_cell_data, Source::GroupOutput) {
		let data: Bytes = data.unpack();
		
		// TODO: Add output validation logic
		if !validate_cell_data(&data) {
			return Err(SysError::Encoding);
		}
	}

	debug!("CONTRACT_NAME type script completed successfully");
	Ok(())
}

fn validate_cell_data(data: &Bytes) -> bool {
	// TODO: Implement your cell data validation logic
	// This is a placeholder
	data.len() >= 4
}
"#.to_string()
	}
}