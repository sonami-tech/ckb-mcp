use shared::error::{CkbMcpError, Result};
use std::{collections::HashMap, fs, path::PathBuf};
use tracing::{info, warn, error};

#[derive(Clone)]
pub struct DocsProvider {
	docs_path: PathBuf,
	built_in_docs: HashMap<String, String>,
}

impl DocsProvider {
	pub fn new(docs_path: Option<PathBuf>) -> Result<Self> {
		let docs_path = docs_path.unwrap_or_else(|| {
			// Default to docs directory relative to the workspace root
			std::env::current_dir()
				.unwrap_or_else(|_| PathBuf::from("."))
				.join("docs")
		});

		let mut provider = Self {
			docs_path,
			built_in_docs: HashMap::new(),
		};

		provider.load_built_in_docs()?;
		info!("Initialized docs provider with {} built-in resources", provider.built_in_docs.len());

		Ok(provider)
	}

	pub fn list_resources(&self) -> Vec<(String, String, String)> {
		let mut resources = Vec::new();

		// Add built-in resources
		for (uri, content) in &self.built_in_docs {
			let description = self.extract_description(content).unwrap_or_else(|| {
				// Fallback to first 100 characters if no Description section found
				if content.len() > 100 {
					format!("{}...", &content[..100])
				} else {
					content.clone()
				}
			});
			resources.push((uri.clone(), uri.replace("ckb-dev-context://", ""), description));
		}

		resources
	}

	/// Extract the Description section from a markdown document
	fn extract_description(&self, content: &str) -> Option<String> {
		let lines: Vec<&str> = content.lines().collect();
		let mut in_description = false;
		let mut description_lines = Vec::new();
		
		for line in lines.iter() {
			// Look for ## Description header
			if line.trim() == "## Description" {
				in_description = true;
				// Skip the header line and the next empty line if present
				continue;
			}
			
			// If we're in the description section
			if in_description {
				// Stop at the next section header
				if line.starts_with('#') && !line.starts_with("###") {
					break;
				}
				
				// Skip empty lines at the beginning
				if description_lines.is_empty() && line.trim().is_empty() {
					continue;
				}
				
				// Add non-empty lines to description
				if !line.trim().is_empty() || !description_lines.is_empty() {
					description_lines.push(line.trim());
				}
			}
		}
		
		if description_lines.is_empty() {
			None
		} else {
			// Join lines and clean up extra whitespace
			let description = description_lines.join(" ")
				.split_whitespace()
				.collect::<Vec<_>>()
				.join(" ");
			Some(description)
		}
	}

	pub fn get_resource(&self, uri: &str) -> Result<String> {
		if let Some(content) = self.built_in_docs.get(uri) {
			return Ok(content.clone());
		}

		Err(CkbMcpError::NotFound(format!("Resource not found: {}", uri)))
	}

	fn load_built_in_docs(&mut self) -> Result<()> {
		// Define the mapping of URIs to file paths
		let resource_mappings = [
			("ckb-dev-context://ai-quick-reference", "ai-quick-reference.md"),
			("ckb-dev-context://api-reference/ccc-api-patterns", "api-reference/ccc-api-patterns.md"),
			("ckb-dev-context://api-reference/ckb-rust-sdk-practical-examples", "api-reference/ckb-rust-sdk-practical-examples.md"),
			("ckb-dev-context://api-reference/cota-sdk-examples", "api-reference/cota-sdk-examples.md"),
			("ckb-dev-context://api-reference/ickb-sdk-examples", "api-reference/ickb-sdk-examples.md"),
			("ckb-dev-context://api-reference/molecule-api-examples", "api-reference/molecule-api-examples.md"),
			("ckb-dev-context://api-reference/omnilock-api-examples", "api-reference/omnilock-api-examples.md"),
			("ckb-dev-context://api-reference/sdk-examples-and-patterns", "api-reference/sdk-examples-and-patterns.md"),
			("ckb-dev-context://api-reference/spore-sdk-examples", "api-reference/spore-sdk-examples.md"),
			("ckb-dev-context://api-reference/syscalls-quick-ref", "api-reference/syscalls-quick-ref.md"),
			("ckb-dev-context://api-reference/ccc-sdk-cross-chain", "api-reference/ccc-sdk-cross-chain.md"),
			("ckb-dev-context://api-reference/ccc-sdk-ssri", "api-reference/ccc-sdk-ssri.md"),
			("ckb-dev-context://concepts/advanced-cell-concepts", "concepts/advanced-cell-concepts.md"),
			("ckb-dev-context://concepts/cell-model", "concepts/cell-model.md"),
			("ckb-dev-context://concepts/ckb-syscalls-and-sources", "concepts/ckb-syscalls-and-sources.md"),
			("ckb-dev-context://concepts/molecule-serialization", "concepts/molecule-serialization.md"),
			("ckb-dev-context://concepts/script-groups-and-execution", "concepts/script-groups-and-execution.md"),
			("ckb-dev-context://concepts/transaction-structure", "concepts/transaction-structure.md"),
			("ckb-dev-context://concepts/header-dependencies-and-time-access", "concepts/header-dependencies-and-time-access.md"),
			("ckb-dev-context://concepts-for-coding/cell-lifecycle", "concepts-for-coding/cell-lifecycle.md"),
			("ckb-dev-context://concepts-for-coding/transaction-lifecycle", "concepts-for-coding/transaction-lifecycle.md"),
			("ckb-dev-context://deployment/binary-deployment", "deployment/binary-deployment.md"),
			("ckb-dev-context://deployment/cota-infrastructure", "deployment/cota-infrastructure.md"),
			("ckb-dev-context://ecosystem/project-directory", "ecosystem/project-directory.md"),
			("ckb-dev-context://education/interactive-courses", "education/interactive-courses.md"),
			("ckb-dev-context://getting-started/developer-resources-and-tooling", "getting-started/developer-resources-and-tooling.md"),
			("ckb-dev-context://getting-started/offckb-development-workflow", "getting-started/offckb-development-workflow.md"),
			("ckb-dev-context://getting-started/tool-recommendations", "getting-started/tool-recommendations.md"),
			("ckb-dev-context://integration-examples/cell-collection-automation", "integration-examples/cell-collection-automation.md"),
			("ckb-dev-context://patterns/c-to-rust-script-migration", "patterns/c-to-rust-script-migration.md"),
			("ckb-dev-context://patterns/cota-nft-development", "patterns/cota-nft-development.md"),
			("ckb-dev-context://patterns/dao-development-patterns", "patterns/dao-development-patterns.md"),
			("ckb-dev-context://patterns/development-tools-and-templates", "patterns/development-tools-and-templates.md"),
			("ckb-dev-context://patterns/file-storage-patterns", "patterns/file-storage-patterns.md"),
			("ckb-dev-context://patterns/ickb-development", "patterns/ickb-development.md"),
			("ckb-dev-context://patterns/ickb-liquidity-patterns", "patterns/ickb-liquidity-patterns.md"),
			("ckb-dev-context://patterns/minimal-lock-script", "patterns/minimal-lock-script.md"),
			("ckb-dev-context://patterns/minimal-type-script", "patterns/minimal-type-script.md"),
			("ckb-dev-context://patterns/molecule-schema-development", "patterns/molecule-schema-development.md"),
			("ckb-dev-context://patterns/omnilock-development", "patterns/omnilock-development.md"),
			("ckb-dev-context://patterns/omnilock-interoperability", "patterns/omnilock-interoperability.md"),
			("ckb-dev-context://patterns/operation-detection", "patterns/operation-detection.md"),
			("ckb-dev-context://patterns/rust-script-development-patterns", "patterns/rust-script-development-patterns.md"),
			("ckb-dev-context://patterns/script-development-patterns", "patterns/script-development-patterns.md"),
			("ckb-dev-context://patterns/seed-cell-pattern", "patterns/seed-cell-pattern.md"),
			("ckb-dev-context://patterns/simple-transfer", "patterns/simple-transfer.md"),
			("ckb-dev-context://patterns/spore-development", "patterns/spore-development.md"),
			("ckb-dev-context://patterns/system-scripts-and-core-patterns", "patterns/system-scripts-and-core-patterns.md"),
			("ckb-dev-context://patterns/token-creation", "patterns/token-creation.md"),
			("ckb-dev-context://patterns/transaction-building-patterns", "patterns/transaction-building-patterns.md"),
			("ckb-dev-context://patterns/type-id-pattern", "patterns/type-id-pattern.md"),
			("ckb-dev-context://patterns/udt-tokens", "patterns/udt-tokens.md"),
			("ckb-dev-context://patterns/cobuild-integration", "patterns/cobuild-integration.md"),
			("ckb-dev-context://patterns/ssri-implementation", "patterns/ssri-implementation.md"),
			("ckb-dev-context://patterns/dob-development", "patterns/dob-development.md"),
			("ckb-dev-context://patterns/proxy-lock-patterns", "patterns/proxy-lock-patterns.md"),
			("ckb-dev-context://patterns/contract-workspace-development", "patterns/contract-workspace-development.md"),
			("ckb-dev-context://protocols/ckbfs-protocol", "protocols/ckbfs-protocol.md"),
			("ckb-dev-context://protocols/cota-protocol", "protocols/cota-protocol.md"),
			("ckb-dev-context://protocols/ickb-protocol", "protocols/ickb-protocol.md"),
			("ckb-dev-context://protocols/omnilock-protocol", "protocols/omnilock-protocol.md"),
			("ckb-dev-context://protocols/rgb-plus-plus", "protocols/rgb-plus-plus.md"),
			("ckb-dev-context://protocols/spore-digital-objects", "protocols/spore-digital-objects.md"),
			("ckb-dev-context://protocols/spore-protocol", "protocols/spore-protocol.md"),
			("ckb-dev-context://protocols/cobuild", "protocols/cobuild.md"),
			("ckb-dev-context://protocols/open-transaction", "protocols/open-transaction.md"),
			("ckb-dev-context://protocols/ssri", "protocols/ssri.md"),
			("ckb-dev-context://protocols/xudt-protocol", "protocols/xudt-protocol.md"),
			("ckb-dev-context://troubleshooting/common-script-errors", "troubleshooting/common-script-errors.md"),
			("ckb-dev-context://troubleshooting/ickb-debugging", "troubleshooting/ickb-debugging.md"),
			("ckb-dev-context://troubleshooting/rust-script-development-issues", "troubleshooting/rust-script-development-issues.md"),
			("ckb-dev-context://tools/ssri-server", "tools/ssri-server.md"),
		];

		for (uri, file_path) in resource_mappings.iter() {
			let full_path = self.docs_path.join(file_path);
			
			match fs::read_to_string(&full_path) {
				Ok(content) => {
					self.built_in_docs.insert(uri.to_string(), content);
					info!("Loaded documentation resource: {}", uri);
				}
				Err(e) => {
					error!("Failed to load documentation file {}: {}", full_path.display(), e);
					warn!("Resource {} will not be available", uri);
					// Continue loading other resources instead of failing completely
				}
			}
		}

		if self.built_in_docs.is_empty() {
			return Err(CkbMcpError::Internal(format!(
				"No documentation files could be loaded from: {}",
				self.docs_path.display()
			)));
		}

		Ok(())
	}
}