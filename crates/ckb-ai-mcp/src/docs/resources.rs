//! Documentation resource definitions.
//!
//! This module defines all 86 documentation resources with the new URI scheme:
//! `ckb://docs/` instead of `ckb-dev-context://`

use rmcp::model::{Annotated, RawResource, Resource};
use std::sync::LazyLock;

/// Static list of all documentation resources.
pub static DOCS_RESOURCES: LazyLock<Vec<DocResource>> = LazyLock::new(DocResourceDefinitions::all);

/// Documentation resource with file path mapping.
#[derive(Clone, Debug)]
pub struct DocResource {
	pub uri: &'static str,
	pub name: &'static str,
	pub file_path: &'static str,
	pub mime_type: &'static str,
}

impl DocResource {
	/// Convert to rmcp Resource type.
	pub fn to_resource(&self, description: Option<String>, size: Option<u32>) -> Resource {
		let mut raw = RawResource::new(self.uri, self.name);
		raw.description = description;
		raw.mime_type = Some(self.mime_type.to_string());
		raw.size = size;
		Annotated::new(raw, None)
	}
}

/// Helper to create a documentation resource definition.
const fn make_resource(
	uri: &'static str,
	name: &'static str,
	file_path: &'static str,
) -> DocResource {
	let mime_type = if matches!(
		file_path.as_bytes().last(),
		Some(b'y') // ends with .py
	) {
		"text/x-python"
	} else {
		"text/markdown"
	};

	DocResource {
		uri,
		name,
		file_path,
		mime_type,
	}
}

/// Documentation resource definitions.
pub struct DocResourceDefinitions;

impl DocResourceDefinitions {
	/// Get all documentation resource definitions.
	pub fn all() -> Vec<DocResource> {
		vec![
			// Root level
			make_resource(
				"ckb://docs/ai-quick-reference",
				"AI Quick Reference",
				"ai-quick-reference.md",
			),
			// API Reference
			make_resource(
				"ckb://docs/api-reference/ccc-api-patterns",
				"CCC API Patterns",
				"api-reference/ccc-api-patterns.md",
			),
			make_resource(
				"ckb://docs/api-reference/ckb-rust-sdk-practical-examples",
				"CKB Rust SDK Practical Examples",
				"api-reference/ckb-rust-sdk-practical-examples.md",
			),
			make_resource(
				"ckb://docs/api-reference/cota-sdk-examples",
				"CoTA SDK Examples",
				"api-reference/cota-sdk-examples.md",
			),
			make_resource(
				"ckb://docs/api-reference/ickb-sdk-examples",
				"iCKB SDK Examples",
				"api-reference/ickb-sdk-examples.md",
			),
			make_resource(
				"ckb://docs/api-reference/molecule-api-examples",
				"Molecule API Examples",
				"api-reference/molecule-api-examples.md",
			),
			make_resource(
				"ckb://docs/api-reference/omnilock-api-examples",
				"Omnilock API Examples",
				"api-reference/omnilock-api-examples.md",
			),
			make_resource(
				"ckb://docs/api-reference/sdk-examples-and-patterns",
				"SDK Examples and Patterns",
				"api-reference/sdk-examples-and-patterns.md",
			),
			make_resource(
				"ckb://docs/api-reference/spore-sdk-examples",
				"Spore SDK Examples",
				"api-reference/spore-sdk-examples.md",
			),
			make_resource(
				"ckb://docs/api-reference/syscalls-quick-ref",
				"Syscalls Quick Reference",
				"api-reference/syscalls-quick-ref.md",
			),
			make_resource(
				"ckb://docs/api-reference/well-known-hashes",
				"Well-Known Hashes",
				"api-reference/well-known-hashes.md",
			),
			make_resource(
				"ckb://docs/api-reference/ccc-sdk-cross-chain",
				"CCC SDK Cross-Chain",
				"api-reference/ccc-sdk-cross-chain.md",
			),
			make_resource(
				"ckb://docs/api-reference/ccc-sdk-ssri",
				"CCC SDK SSRI",
				"api-reference/ccc-sdk-ssri.md",
			),
			make_resource(
				"ckb://docs/api-reference/omnilock-ethereum-example",
				"Omnilock Ethereum Example",
				"api-reference/omnilock-ethereum-example.md",
			),
			make_resource(
				"ckb://docs/api-reference/xudt-minting-example",
				"xUDT Minting Example",
				"api-reference/xudt-minting-example.md",
			),
			// Concepts
			make_resource(
				"ckb://docs/concepts/advanced-cell-concepts",
				"Advanced Cell Concepts",
				"concepts/advanced-cell-concepts.md",
			),
			make_resource(
				"ckb://docs/concepts/cell-model",
				"Cell Model",
				"concepts/cell-model.md",
			),
			make_resource(
				"ckb://docs/concepts/ckb-syscalls-and-sources",
				"CKB Syscalls and Sources",
				"concepts/ckb-syscalls-and-sources.md",
			),
			make_resource(
				"ckb://docs/concepts/ckb-network-history",
				"CKB Network History",
				"concepts/ckb-network-history.md",
			),
			make_resource(
				"ckb://docs/concepts/molecule-serialization",
				"Molecule Serialization",
				"concepts/molecule-serialization.md",
			),
			make_resource(
				"ckb://docs/concepts/script-groups-and-execution",
				"Script Groups and Execution",
				"concepts/script-groups-and-execution.md",
			),
			make_resource(
				"ckb://docs/concepts/transaction-structure",
				"Transaction Structure",
				"concepts/transaction-structure.md",
			),
			make_resource(
				"ckb://docs/concepts/header-dependencies-and-time-access",
				"Header Dependencies and Time Access",
				"concepts/header-dependencies-and-time-access.md",
			),
			make_resource(
				"ckb://docs/concepts/lock-value-relationships",
				"Lock Value Relationships",
				"concepts/lock-value-relationships.md",
			),
			// Concepts for Coding
			make_resource(
				"ckb://docs/concepts-for-coding/cell-lifecycle",
				"Cell Lifecycle",
				"concepts-for-coding/cell-lifecycle.md",
			),
			make_resource(
				"ckb://docs/concepts-for-coding/transaction-lifecycle",
				"Transaction Lifecycle",
				"concepts-for-coding/transaction-lifecycle.md",
			),
			// Deployment
			make_resource(
				"ckb://docs/deployment/binary-deployment",
				"Binary Deployment",
				"deployment/binary-deployment.md",
			),
			make_resource(
				"ckb://docs/deployment/cota-infrastructure",
				"CoTA Infrastructure",
				"deployment/cota-infrastructure.md",
			),
			// Ecosystem
			make_resource(
				"ckb://docs/ecosystem/project-directory",
				"Project Directory",
				"ecosystem/project-directory.md",
			),
			// Education
			make_resource(
				"ckb://docs/education/interactive-courses",
				"Interactive Courses",
				"education/interactive-courses.md",
			),
			// Getting Started
			make_resource(
				"ckb://docs/getting-started/developer-resources-and-tooling",
				"Developer Resources and Tooling",
				"getting-started/developer-resources-and-tooling.md",
			),
			make_resource(
				"ckb://docs/getting-started/offckb-development-workflow",
				"OffCKB Development Workflow",
				"getting-started/offckb-development-workflow.md",
			),
			make_resource(
				"ckb://docs/getting-started/tool-recommendations",
				"Tool Recommendations",
				"getting-started/tool-recommendations.md",
			),
			// Integration Examples
			make_resource(
				"ckb://docs/integration-examples/cell-collection-automation",
				"Cell Collection Automation",
				"integration-examples/cell-collection-automation.md",
			),
			// Patterns
			make_resource(
				"ckb://docs/patterns/c-to-rust-script-migration",
				"C to Rust Script Migration",
				"patterns/c-to-rust-script-migration.md",
			),
			make_resource(
				"ckb://docs/patterns/cota-nft-development",
				"CoTA NFT Development",
				"patterns/cota-nft-development.md",
			),
			make_resource(
				"ckb://docs/patterns/dao-development-patterns",
				"DAO Development Patterns",
				"patterns/dao-development-patterns.md",
			),
			make_resource(
				"ckb://docs/patterns/development-tools-and-templates",
				"Development Tools and Templates",
				"patterns/development-tools-and-templates.md",
			),
			make_resource(
				"ckb://docs/patterns/file-storage-patterns",
				"File Storage Patterns",
				"patterns/file-storage-patterns.md",
			),
			make_resource(
				"ckb://docs/patterns/ickb-development",
				"iCKB Development",
				"patterns/ickb-development.md",
			),
			make_resource(
				"ckb://docs/patterns/ickb-liquidity-patterns",
				"iCKB Liquidity Patterns",
				"patterns/ickb-liquidity-patterns.md",
			),
			make_resource(
				"ckb://docs/patterns/minimal-lock-script",
				"Minimal Lock Script",
				"patterns/minimal-lock-script.md",
			),
			make_resource(
				"ckb://docs/patterns/minimal-type-script",
				"Minimal Type Script",
				"patterns/minimal-type-script.md",
			),
			make_resource(
				"ckb://docs/patterns/molecule-schema-development",
				"Molecule Schema Development",
				"patterns/molecule-schema-development.md",
			),
			make_resource(
				"ckb://docs/patterns/omnilock-development",
				"Omnilock Development",
				"patterns/omnilock-development.md",
			),
			make_resource(
				"ckb://docs/patterns/omnilock-interoperability",
				"Omnilock Interoperability",
				"patterns/omnilock-interoperability.md",
			),
			make_resource(
				"ckb://docs/patterns/operation-detection",
				"Operation Detection",
				"patterns/operation-detection.md",
			),
			make_resource(
				"ckb://docs/patterns/rust-script-development-patterns",
				"Rust Script Development Patterns",
				"patterns/rust-script-development-patterns.md",
			),
			make_resource(
				"ckb://docs/patterns/script-development-patterns",
				"Script Development Patterns",
				"patterns/script-development-patterns.md",
			),
			make_resource(
				"ckb://docs/patterns/script-source-patterns",
				"Script Source Patterns",
				"patterns/script-source-patterns.md",
			),
			make_resource(
				"ckb://docs/patterns/seed-cell-pattern",
				"Seed Cell Pattern",
				"patterns/seed-cell-pattern.md",
			),
			make_resource(
				"ckb://docs/patterns/simple-transfer",
				"Simple Transfer",
				"patterns/simple-transfer.md",
			),
			make_resource(
				"ckb://docs/patterns/spore-development",
				"Spore Development",
				"patterns/spore-development.md",
			),
			make_resource(
				"ckb://docs/patterns/system-scripts-and-core-patterns",
				"System Scripts and Core Patterns",
				"patterns/system-scripts-and-core-patterns.md",
			),
			make_resource(
				"ckb://docs/patterns/token-creation",
				"Token Creation",
				"patterns/token-creation.md",
			),
			make_resource(
				"ckb://docs/patterns/transaction-building-patterns",
				"Transaction Building Patterns",
				"patterns/transaction-building-patterns.md",
			),
			make_resource(
				"ckb://docs/patterns/type-id-pattern",
				"Type ID Pattern",
				"patterns/type-id-pattern.md",
			),
			make_resource(
				"ckb://docs/patterns/udt-tokens",
				"UDT Tokens",
				"patterns/udt-tokens.md",
			),
			make_resource(
				"ckb://docs/patterns/cobuild-integration",
				"CoBuild Integration",
				"patterns/cobuild-integration.md",
			),
			make_resource(
				"ckb://docs/patterns/ssri-implementation",
				"SSRI Implementation",
				"patterns/ssri-implementation.md",
			),
			make_resource(
				"ckb://docs/patterns/dob-development",
				"DOB Development",
				"patterns/dob-development.md",
			),
			make_resource(
				"ckb://docs/patterns/proxy-lock-patterns",
				"Proxy Lock Patterns",
				"patterns/proxy-lock-patterns.md",
			),
			make_resource(
				"ckb://docs/patterns/proxy-lock-testing-patterns",
				"Proxy Lock Testing Patterns",
				"patterns/proxy-lock-testing-patterns.md",
			),
			make_resource(
				"ckb://docs/patterns/contract-workspace-development",
				"Contract Workspace Development",
				"patterns/contract-workspace-development.md",
			),
			// Protocols
			make_resource(
				"ckb://docs/protocols/ckbfs-protocol",
				"CKBFS Protocol",
				"protocols/ckbfs-protocol.md",
			),
			make_resource(
				"ckb://docs/protocols/cota-protocol",
				"CoTA Protocol",
				"protocols/cota-protocol.md",
			),
			make_resource(
				"ckb://docs/protocols/ickb-protocol",
				"iCKB Protocol",
				"protocols/ickb-protocol.md",
			),
			make_resource(
				"ckb://docs/protocols/omnilock-protocol",
				"Omnilock Protocol",
				"protocols/omnilock-protocol.md",
			),
			make_resource(
				"ckb://docs/protocols/rgb-plus-plus",
				"RGB++ Protocol",
				"protocols/rgb-plus-plus.md",
			),
			make_resource(
				"ckb://docs/protocols/spore-digital-objects",
				"Spore Digital Objects",
				"protocols/spore-digital-objects.md",
			),
			make_resource(
				"ckb://docs/protocols/spore-protocol",
				"Spore Protocol",
				"protocols/spore-protocol.md",
			),
			make_resource(
				"ckb://docs/protocols/cobuild",
				"CoBuild Protocol",
				"protocols/cobuild.md",
			),
			make_resource(
				"ckb://docs/protocols/open-transaction",
				"Open Transaction",
				"protocols/open-transaction.md",
			),
			make_resource(
				"ckb://docs/protocols/ssri",
				"SSRI Protocol",
				"protocols/ssri.md",
			),
			make_resource(
				"ckb://docs/protocols/xudt-protocol",
				"xUDT Protocol",
				"protocols/xudt-protocol.md",
			),
			// Troubleshooting
			make_resource(
				"ckb://docs/troubleshooting/common-script-errors",
				"Common Script Errors",
				"troubleshooting/common-script-errors.md",
			),
			make_resource(
				"ckb://docs/troubleshooting/ickb-debugging",
				"iCKB Debugging",
				"troubleshooting/ickb-debugging.md",
			),
			make_resource(
				"ckb://docs/troubleshooting/rust-script-development-issues",
				"Rust Script Development Issues",
				"troubleshooting/rust-script-development-issues.md",
			),
			make_resource(
				"ckb://docs/troubleshooting/omnilock-errors",
				"Omnilock Errors",
				"troubleshooting/omnilock-errors.md",
			),
			make_resource(
				"ckb://docs/troubleshooting/xudt-errors",
				"xUDT Errors",
				"troubleshooting/xudt-errors.md",
			),
			make_resource(
				"ckb://docs/troubleshooting/transaction-building-errors",
				"Transaction Building Errors",
				"troubleshooting/transaction-building-errors.md",
			),
			make_resource(
				"ckb://docs/troubleshooting/spore-errors",
				"Spore Errors",
				"troubleshooting/spore-errors.md",
			),
			// Tools
			make_resource(
				"ckb://docs/tools/ssri-server",
				"SSRI Server",
				"tools/ssri-server.md",
			),
			// Examples (Python scripts)
			make_resource(
				"ckb://docs/examples/calculate_file_hashes",
				"Calculate File Hashes",
				"examples/calculate_file_hashes.py",
			),
			make_resource(
				"ckb://docs/examples/consolidate_cells",
				"Consolidate Cells",
				"examples/consolidate_cells.py",
			),
		]
	}
}
