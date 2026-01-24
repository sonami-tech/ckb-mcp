//! Prompt definitions for CKB development workflows.

use rmcp::model::{Prompt, PromptArgument};
use std::sync::LazyLock;

/// Helper to create a prompt definition.
fn make_prompt(
	name: &'static str,
	description: &'static str,
	arguments: Vec<(&'static str, &'static str, bool)>,
) -> Prompt {
	let args = if arguments.is_empty() {
		None
	} else {
		Some(
			arguments
				.into_iter()
				.map(|(name, desc, required)| PromptArgument {
					name: name.to_string(),
					title: None,
					description: Some(desc.to_string()),
					required: Some(required),
				})
				.collect(),
		)
	};

	Prompt {
		name: name.to_string(),
		title: None,
		description: Some(description.to_string()),
		arguments: args,
		icons: None,
		meta: None,
	}
}

/// Available workflow prompts.
pub static PROMPTS: LazyLock<Vec<Prompt>> = LazyLock::new(|| {
	vec![
		make_prompt(
			"create_script",
			"Guided workflow for creating a new CKB script (smart contract). \
			Provides step-by-step instructions for setting up a Rust-based CKB script project, \
			including project structure, dependencies, and basic script logic.",
			vec![
				(
					"script_type",
					"Type of script: 'lock' for asset custody or 'type' for state validation",
					true,
				),
				(
					"script_name",
					"Name for the script project (e.g., 'my-token', 'simple-lock')",
					true,
				),
				(
					"description",
					"Brief description of what the script should do",
					false,
				),
			],
		),
		make_prompt(
			"deploy_script",
			"Guided workflow for deploying a compiled CKB script to the blockchain. \
			Covers building the script, creating a deployment transaction, and verifying the deployment.",
			vec![
				(
					"binary_path",
					"Path to the compiled script binary (.so or RISC-V binary)",
					true,
				),
				(
					"network",
					"Target network: 'devnet', 'testnet', or 'mainnet'",
					true,
				),
			],
		),
		make_prompt(
			"query_blockchain",
			"Guided workflow for querying CKB blockchain data. \
			Helps construct appropriate queries for cells, transactions, headers, and other chain data.",
			vec![
				(
					"query_type",
					"Type of query: 'cell', 'transaction', 'block', 'header', or 'tip'",
					true,
				),
				(
					"identifier",
					"Identifier for the query (hash, address, or search criteria)",
					false,
				),
			],
		),
		make_prompt(
			"transfer_ckb",
			"Guided workflow for transferring CKB or tokens between addresses. \
			Covers address validation, transaction construction, fee calculation, and signing.",
			vec![
				("to_address", "Destination CKB address", true),
				(
					"amount",
					"Amount to transfer in CKB (e.g., '100' for 100 CKB)",
					true,
				),
				(
					"token_type",
					"Type of transfer: 'ckb' for native CKB or 'udt' for user-defined tokens",
					false,
				),
			],
		),
	]
});
