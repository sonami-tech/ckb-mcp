//! Documentation resource handlers for list_resources and read_resource.

use rmcp::model::{ReadResourceResult, Resource, ResourceContents};
use shared::error::{CkbMcpError, Result};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, error, info, warn};

use super::resources::DOCS_RESOURCES;

/// Cached content for a documentation resource.
#[derive(Clone)]
struct CachedContent {
	content: String,
	description: Option<String>,
}

/// Documentation handlers for resource operations.
#[derive(Clone)]
pub struct DocsHandlers {
	docs_path: PathBuf,
	/// Cached content: URI -> cached content.
	content_cache: HashMap<String, CachedContent>,
}

impl DocsHandlers {
	/// Create a new DocsHandlers instance.
	///
	/// Loads all documentation files from the specified path.
	pub fn new(docs_path: PathBuf) -> Result<Self> {
		let mut handlers = Self {
			docs_path,
			content_cache: HashMap::new(),
		};

		handlers.load_documentation()?;
		info!(
			"Initialized docs handlers with {} resources",
			handlers.content_cache.len()
		);

		Ok(handlers)
	}

	/// Load all documentation files into the cache.
	fn load_documentation(&mut self) -> Result<()> {
		for doc_resource in DOCS_RESOURCES.iter() {
			let full_path = self.docs_path.join(doc_resource.file_path);

			match fs::read_to_string(&full_path) {
				Ok(content) => {
					let description = Self::extract_description(&content);
					self.content_cache.insert(
						doc_resource.uri.to_string(),
						CachedContent {
							content,
							description,
						},
					);
					debug!("Loaded documentation resource: {}", doc_resource.uri);
				}
				Err(e) => {
					error!(
						"Failed to load documentation file {}: {}",
						full_path.display(),
						e
					);
					warn!("Resource {} will not be available", doc_resource.uri);
					// Continue loading other resources instead of failing completely.
				}
			}
		}

		if self.content_cache.is_empty() {
			return Err(CkbMcpError::Internal(format!(
				"No documentation files could be loaded from: {}",
				self.docs_path.display()
			)));
		}

		Ok(())
	}

	/// List all available documentation resources.
	///
	/// Returns Resource structs with descriptions extracted from file content.
	pub fn list_resources(&self) -> Vec<Resource> {
		DOCS_RESOURCES
			.iter()
			.filter_map(|doc_resource| {
				// Only include resources that were successfully loaded.
				if let Some(cached) = self.content_cache.get(doc_resource.uri) {
					let description = cached.description.clone().unwrap_or_else(|| {
						// Fallback to first 100 characters if no Description section found.
						if cached.content.len() > 100 {
							format!("{}...", &cached.content[..100])
						} else {
							cached.content.clone()
						}
					});

					let size = cached.content.len() as u32;
					Some(doc_resource.to_resource(Some(description), Some(size)))
				} else {
					None
				}
			})
			.collect()
	}

	/// Read a specific documentation resource.
	pub fn read_resource(&self, uri: &str) -> Result<ReadResourceResult> {
		if let Some(cached) = self.content_cache.get(uri) {
			Ok(ReadResourceResult {
				contents: vec![ResourceContents::text(&cached.content, uri)],
			})
		} else {
			Err(CkbMcpError::NotFound(format!(
				"Resource not found: {}",
				uri
			)))
		}
	}

	/// Check if a URI is a documentation resource.
	pub fn is_docs_resource(uri: &str) -> bool {
		uri.starts_with("ckb://docs/")
	}

	/// Extract the Description section from a markdown document.
	fn extract_description(content: &str) -> Option<String> {
		let mut in_description = false;
		let mut description_lines = Vec::new();

		for line in content.lines() {
			// Look for ## Description header.
			if line.trim() == "## Description" {
				in_description = true;
				// Skip the header line and the next empty line if present.
				continue;
			}

			// If we're in the description section.
			if in_description {
				// Stop at the next section header.
				if line.starts_with('#') && !line.starts_with("###") {
					break;
				}

				// Skip empty lines at the beginning.
				if description_lines.is_empty() && line.trim().is_empty() {
					continue;
				}

				// Add non-empty lines to description.
				if !line.trim().is_empty() || !description_lines.is_empty() {
					description_lines.push(line.trim());
				}
			}
		}

		if description_lines.is_empty() {
			None
		} else {
			// Join lines and clean up extra whitespace.
			let description = description_lines
				.join(" ")
				.split_whitespace()
				.collect::<Vec<_>>()
				.join(" ");
			Some(description)
		}
	}
}
