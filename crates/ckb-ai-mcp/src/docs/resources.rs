//! Documentation resource definitions.
//!
//! This module defines all 98 documentation resources with the new URI scheme:
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
	let bytes = file_path.as_bytes();
	let len = bytes.len();
	let mime_type =
		if len >= 3 && bytes[len - 3] == b'.' && bytes[len - 2] == b'p' && bytes[len - 1] == b'y' {
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
			// =================================================================
			// quickstart/ - 2 files
			// =================================================================
			make_resource(
				"ckb://docs/quickstart/ai-quick-reference",
				"AI Quick Reference",
				"quickstart/ai-quick-reference.md",
			),
			make_resource(
				"ckb://docs/quickstart/getting-started",
				"Getting Started",
				"quickstart/getting-started.md",
			),
			// =================================================================
			// concepts/ - 11 files
			// =================================================================
			make_resource(
				"ckb://docs/concepts/cell-lifecycle",
				"Cell Lifecycle",
				"concepts/cell-lifecycle.md",
			),
			make_resource(
				"ckb://docs/concepts/cell-model",
				"Cell Model",
				"concepts/cell-model.md",
			),
			make_resource(
				"ckb://docs/concepts/header-dependencies",
				"Header Dependencies",
				"concepts/header-dependencies.md",
			),
			make_resource(
				"ckb://docs/concepts/lock-values",
				"Lock Values",
				"concepts/lock-values.md",
			),
			make_resource(
				"ckb://docs/concepts/molecule-serialization",
				"Molecule Serialization",
				"concepts/molecule-serialization.md",
			),
			make_resource(
				"ckb://docs/concepts/network-history",
				"Network History",
				"concepts/network-history.md",
			),
			make_resource(
				"ckb://docs/concepts/programming-model",
				"CKB Programming Model",
				"concepts/programming-model.md",
			),
			make_resource(
				"ckb://docs/concepts/script-groups",
				"Script Groups",
				"concepts/script-groups.md",
			),
			make_resource(
				"ckb://docs/concepts/syscalls",
				"Syscalls",
				"concepts/syscalls.md",
			),
			make_resource(
				"ckb://docs/concepts/transaction-lifecycle",
				"Transaction Lifecycle",
				"concepts/transaction-lifecycle.md",
			),
			make_resource(
				"ckb://docs/concepts/transaction-structure",
				"Transaction Structure",
				"concepts/transaction-structure.md",
			),
			// =================================================================
			// scripts/ - 11 files
			// =================================================================
			make_resource(
				"ckb://docs/scripts/c-migration",
				"C to Rust Migration",
				"scripts/c-migration.md",
			),
			make_resource(
				"ckb://docs/scripts/lock-script-minimal",
				"Minimal Lock Script",
				"scripts/lock-script-minimal.md",
			),
			make_resource(
				"ckb://docs/scripts/operation-detection",
				"Operation Detection",
				"scripts/operation-detection.md",
			),
			make_resource(
				"ckb://docs/scripts/patterns",
				"Script Development Patterns",
				"scripts/patterns.md",
			),
			make_resource(
				"ckb://docs/scripts/rust-testing",
				"Rust Script Testing",
				"scripts/rust-testing.md",
			),
			make_resource(
				"ckb://docs/scripts/script-sources",
				"Script Sources",
				"scripts/script-sources.md",
			),
			make_resource(
				"ckb://docs/scripts/seed-cell",
				"Seed Cell Pattern",
				"scripts/seed-cell.md",
			),
			make_resource(
				"ckb://docs/scripts/system-scripts",
				"System Scripts",
				"scripts/system-scripts.md",
			),
			make_resource(
				"ckb://docs/scripts/system-security",
				"System Security",
				"scripts/system-security.md",
			),
			make_resource(
				"ckb://docs/scripts/type-id",
				"Type ID Pattern",
				"scripts/type-id.md",
			),
			make_resource(
				"ckb://docs/scripts/type-script-minimal",
				"Minimal Type Script",
				"scripts/type-script-minimal.md",
			),
			// =================================================================
			// tokens/ - 5 files
			// =================================================================
			make_resource(
				"ckb://docs/tokens/cell-collection",
				"Cell Collection",
				"tokens/cell-collection.md",
			),
			make_resource(
				"ckb://docs/tokens/simple-transfers",
				"Simple Transfers",
				"tokens/simple-transfers.md",
			),
			make_resource(
				"ckb://docs/tokens/token-creation",
				"Token Creation",
				"tokens/token-creation.md",
			),
			make_resource(
				"ckb://docs/tokens/udt-overview",
				"UDT Overview",
				"tokens/udt-overview.md",
			),
			make_resource(
				"ckb://docs/tokens/xudt-minting",
				"xUDT Minting",
				"tokens/xudt-minting.md",
			),
			// =================================================================
			// omnilock/ - 6 files
			// =================================================================
			make_resource(
				"ckb://docs/omnilock/api-examples",
				"Omnilock API Examples",
				"omnilock/api-examples.md",
			),
			make_resource(
				"ckb://docs/omnilock/development",
				"Omnilock Development",
				"omnilock/development.md",
			),
			make_resource(
				"ckb://docs/omnilock/errors",
				"Omnilock Errors",
				"omnilock/errors.md",
			),
			make_resource(
				"ckb://docs/omnilock/ethereum-example",
				"Omnilock Ethereum Example",
				"omnilock/ethereum-example.md",
			),
			make_resource(
				"ckb://docs/omnilock/interoperability",
				"Omnilock Interoperability",
				"omnilock/interoperability.md",
			),
			make_resource(
				"ckb://docs/omnilock/protocol",
				"Omnilock Protocol",
				"omnilock/protocol.md",
			),
			// =================================================================
			// spore/ - 5 files
			// =================================================================
			make_resource(
				"ckb://docs/spore/development",
				"Spore Development",
				"spore/development.md",
			),
			make_resource(
				"ckb://docs/spore/digital-objects",
				"Spore Digital Objects",
				"spore/digital-objects.md",
			),
			make_resource("ckb://docs/spore/errors", "Spore Errors", "spore/errors.md"),
			make_resource(
				"ckb://docs/spore/protocol",
				"Spore Protocol",
				"spore/protocol.md",
			),
			make_resource(
				"ckb://docs/spore/sdk-examples",
				"Spore SDK Examples",
				"spore/sdk-examples.md",
			),
			// =================================================================
			// cota/ - 4 files
			// =================================================================
			make_resource(
				"ckb://docs/cota/development",
				"CoTA Development",
				"cota/development.md",
			),
			make_resource(
				"ckb://docs/cota/infrastructure",
				"CoTA Infrastructure",
				"cota/infrastructure.md",
			),
			make_resource(
				"ckb://docs/cota/protocol",
				"CoTA Protocol",
				"cota/protocol.md",
			),
			make_resource(
				"ckb://docs/cota/sdk-examples",
				"CoTA SDK Examples",
				"cota/sdk-examples.md",
			),
			// =================================================================
			// dao/ - 2 files
			// =================================================================
			make_resource(
				"ckb://docs/dao/patterns-advanced",
				"DAO Advanced Patterns",
				"dao/patterns-advanced.md",
			),
			make_resource(
				"ckb://docs/dao/patterns-basic",
				"DAO Basic Patterns",
				"dao/patterns-basic.md",
			),
			// =================================================================
			// ickb/ - 4 files
			// =================================================================
			make_resource(
				"ckb://docs/ickb/debugging",
				"iCKB Debugging",
				"ickb/debugging.md",
			),
			make_resource(
				"ckb://docs/ickb/development",
				"iCKB Development",
				"ickb/development.md",
			),
			make_resource(
				"ckb://docs/ickb/liquidity",
				"iCKB Liquidity",
				"ickb/liquidity.md",
			),
			make_resource(
				"ckb://docs/ickb/protocol",
				"iCKB Protocol",
				"ickb/protocol.md",
			),
			// =================================================================
			// protocols/ - 5 files
			// =================================================================
			make_resource(
				"ckb://docs/protocols/ckbfs",
				"CKBFS Protocol",
				"protocols/ckbfs.md",
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
				"ckb://docs/protocols/rgb-plus-plus",
				"RGB++ Protocol",
				"protocols/rgb-plus-plus.md",
			),
			make_resource(
				"ckb://docs/protocols/ssri",
				"SSRI Protocol",
				"protocols/ssri.md",
			),
			// =================================================================
			// sdk/ - 9 files
			// =================================================================
			make_resource("ckb://docs/sdk/ccc-api", "CCC API", "sdk/ccc-api.md"),
			make_resource(
				"ckb://docs/sdk/ccc-cross-chain",
				"CCC Cross-Chain",
				"sdk/ccc-cross-chain.md",
			),
			make_resource("ckb://docs/sdk/ccc-ssri", "CCC SSRI", "sdk/ccc-ssri.md"),
			make_resource("ckb://docs/sdk/ickb-sdk", "iCKB SDK", "sdk/ickb-sdk.md"),
			make_resource(
				"ckb://docs/sdk/lumos-patterns",
				"Lumos Patterns",
				"sdk/lumos-patterns.md",
			),
			make_resource(
				"ckb://docs/sdk/molecule-api",
				"Molecule API",
				"sdk/molecule-api.md",
			),
			make_resource(
				"ckb://docs/sdk/molecule-schema",
				"Molecule Schema",
				"sdk/molecule-schema.md",
			),
			make_resource(
				"ckb://docs/sdk/rust-sdk-advanced",
				"Rust SDK Advanced",
				"sdk/rust-sdk-advanced.md",
			),
			make_resource(
				"ckb://docs/sdk/rust-sdk-basic",
				"Rust SDK Basic",
				"sdk/rust-sdk-basic.md",
			),
			// =================================================================
			// transactions/ - 5 files
			// =================================================================
			make_resource(
				"ckb://docs/transactions/building-patterns",
				"Transaction Building Patterns",
				"transactions/building-patterns.md",
			),
			make_resource(
				"ckb://docs/transactions/cobuild-integration",
				"CoBuild Integration",
				"transactions/cobuild-integration.md",
			),
			make_resource(
				"ckb://docs/transactions/proxy-locks",
				"Proxy Locks",
				"transactions/proxy-locks.md",
			),
			make_resource(
				"ckb://docs/transactions/proxy-lock-testing",
				"Proxy Lock Testing",
				"transactions/proxy-lock-testing.md",
			),
			make_resource(
				"ckb://docs/transactions/ssri-implementation",
				"SSRI Implementation",
				"transactions/ssri-implementation.md",
			),
			// =================================================================
			// reference/ - 5 files
			// =================================================================
			make_resource(
				"ckb://docs/reference/lock-script-hashes",
				"Lock Script Hashes",
				"reference/lock-script-hashes.md",
			),
			make_resource(
				"ckb://docs/reference/protocol-script-hashes",
				"Protocol Script Hashes",
				"reference/protocol-script-hashes.md",
			),
			make_resource(
				"ckb://docs/reference/syscalls-quick-ref",
				"Syscalls Quick Reference",
				"reference/syscalls-quick-ref.md",
			),
			make_resource(
				"ckb://docs/reference/system-script-hashes",
				"System Script Hashes",
				"reference/system-script-hashes.md",
			),
			make_resource(
				"ckb://docs/reference/token-script-hashes",
				"Token Script Hashes",
				"reference/token-script-hashes.md",
			),
			// =================================================================
			// troubleshooting/ - 4 files
			// =================================================================
			make_resource(
				"ckb://docs/troubleshooting/common-script-errors",
				"Common Script Errors",
				"troubleshooting/common-script-errors.md",
			),
			make_resource(
				"ckb://docs/troubleshooting/rust-development",
				"Rust Development Issues",
				"troubleshooting/rust-development.md",
			),
			make_resource(
				"ckb://docs/troubleshooting/transaction-errors",
				"Transaction Errors",
				"troubleshooting/transaction-errors.md",
			),
			make_resource(
				"ckb://docs/troubleshooting/xudt-errors",
				"xUDT Errors",
				"troubleshooting/xudt-errors.md",
			),
			// =================================================================
			// tools/ - 5 files
			// =================================================================
			make_resource(
				"ckb://docs/tools/binary-deployment",
				"Binary Deployment",
				"tools/binary-deployment.md",
			),
			make_resource(
				"ckb://docs/tools/contract-workspace",
				"Contract Workspace",
				"tools/contract-workspace.md",
			),
			make_resource(
				"ckb://docs/tools/development-tools",
				"Development Tools",
				"tools/development-tools.md",
			),
			make_resource(
				"ckb://docs/tools/offckb-workflow",
				"OffCKB Workflow",
				"tools/offckb-workflow.md",
			),
			make_resource(
				"ckb://docs/tools/ssri-server",
				"SSRI Server",
				"tools/ssri-server.md",
			),
			// =================================================================
			// ecosystem/ - 3 files
			// =================================================================
			make_resource(
				"ckb://docs/ecosystem/file-storage",
				"File Storage",
				"ecosystem/file-storage.md",
			),
			make_resource(
				"ckb://docs/ecosystem/interactive-courses",
				"Interactive Courses",
				"ecosystem/interactive-courses.md",
			),
			make_resource(
				"ckb://docs/ecosystem/project-directory",
				"Project Directory",
				"ecosystem/project-directory.md",
			),
			// =================================================================
			// examples/ - 3 files
			// =================================================================
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
			make_resource(
				"ckb://docs/examples/dob-development",
				"DOB Development",
				"examples/dob-development.md",
			),
			// =================================================================
			// fiber/ - 9 files
			// =================================================================
			make_resource(
				"ckb://docs/fiber/overview",
				"Fiber Network Overview",
				"fiber/overview.md",
			),
			make_resource(
				"ckb://docs/fiber/node-setup",
				"Fiber Node Setup",
				"fiber/node-setup.md",
			),
			make_resource(
				"ckb://docs/fiber/rpc-reference",
				"Fiber RPC Reference",
				"fiber/rpc-reference.md",
			),
			make_resource(
				"ckb://docs/fiber/channels",
				"Fiber Channels",
				"fiber/channels.md",
			),
			make_resource(
				"ckb://docs/fiber/invoices-and-payments",
				"Fiber Invoices and Payments",
				"fiber/invoices-and-payments.md",
			),
			make_resource(
				"ckb://docs/fiber/routing-and-graph",
				"Fiber Routing and Graph",
				"fiber/routing-and-graph.md",
			),
			make_resource(
				"ckb://docs/fiber/on-chain-scripts",
				"Fiber On-Chain Scripts",
				"fiber/on-chain-scripts.md",
			),
			make_resource(
				"ckb://docs/fiber/udt-channels",
				"Fiber UDT Channels",
				"fiber/udt-channels.md",
			),
			make_resource(
				"ckb://docs/fiber/cross-chain-hub",
				"Fiber Cross-Chain Hub",
				"fiber/cross-chain-hub.md",
			),
		]
	}
}
