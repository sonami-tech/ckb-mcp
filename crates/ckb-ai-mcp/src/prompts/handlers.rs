//! Prompt handlers for generating workflow guidance.

use rmcp::model::{GetPromptResult, PromptMessage, PromptMessageRole};
use serde_json::Value;
use shared::error::{CkbMcpError, Result};
use tracing::debug;

/// Handlers for workflow prompts.
#[derive(Clone)]
pub struct PromptsHandlers;

impl PromptsHandlers {
	/// Create new PromptsHandlers instance.
	pub fn new() -> Self {
		Self
	}

	/// Check if a prompt name is a valid workflow prompt.
	pub fn is_prompt(name: &str) -> bool {
		matches!(
			name,
			"create_script" | "deploy_script" | "query_blockchain" | "transfer_ckb"
		)
	}

	/// Handle a prompt request.
	pub fn handle(&self, name: &str, args: &Value) -> Result<GetPromptResult> {
		debug!("Handling prompt: {} with args: {:?}", name, args);

		match name {
			"create_script" => self.create_script_prompt(args),
			"deploy_script" => self.deploy_script_prompt(args),
			"query_blockchain" => self.query_blockchain_prompt(args),
			"transfer_ckb" => self.transfer_ckb_prompt(args),
			_ => Err(CkbMcpError::InvalidParameter(format!(
				"Unknown prompt: {}",
				name
			))),
		}
	}

	/// Generate the create_script workflow prompt.
	fn create_script_prompt(&self, args: &Value) -> Result<GetPromptResult> {
		let script_type = args
			.get("script_type")
			.and_then(|v| v.as_str())
			.ok_or_else(|| {
				CkbMcpError::InvalidParameter("Missing required argument: script_type".to_string())
			})?;

		let script_name = args
			.get("script_name")
			.and_then(|v| v.as_str())
			.ok_or_else(|| {
				CkbMcpError::InvalidParameter("Missing required argument: script_name".to_string())
			})?;

		let description = args
			.get("description")
			.and_then(|v| v.as_str())
			.unwrap_or("A CKB script");

		let type_explanation = match script_type {
			"lock" => "Lock scripts control who can spend a cell. They define ownership and authorization rules.",
			"type" => "Type scripts validate state transitions. They define what data can be stored and how it changes.",
			_ => "Script type should be 'lock' or 'type'.",
		};

		let content = format!(
			r#"# Create CKB Script: {script_name}

## Script Type: {script_type}
{type_explanation}

## Description
{description}

## Workflow Steps

### Step 1: Set Up Project
Use Capsule or ckb-script-templates to create a new script project:
```bash
capsule new {script_name} --template rust
cd {script_name}
```

### Step 2: Review Project Structure
Key files to understand:
- `contracts/{script_name}/src/main.rs` - Main script logic
- `contracts/{script_name}/src/error.rs` - Custom error codes
- `Capsule.toml` - Build configuration
- `deployment.toml` - Deployment settings

### Step 3: Implement Script Logic
Read relevant documentation resources:
- `ckb://docs/concepts/cell-model` - Understand the cell model
- `ckb://docs/patterns/{script_type}-script-development` - {script_type} script patterns
- `ckb://docs/api-reference/ckb-syscalls` - Available syscalls

### Step 4: Write Tests
Create tests in `tests/src/tests.rs` using the CKB testing framework.

### Step 5: Build
```bash
capsule build --release
```

### Step 6: Deploy
Use the `deploy_script` prompt for deployment guidance.

## Recommended Tools
- `dev_get_chain_type` - Verify target network
- `dev_get_address_balance` - Check deployment account balance
- `search_resources` - Find relevant documentation

## Next Steps
After implementing your script, use the `deploy_script` prompt to deploy it to the blockchain."#
		);

		Ok(GetPromptResult {
			description: Some(format!(
				"Workflow for creating a {} script named '{}'",
				script_type, script_name
			)),
			messages: vec![PromptMessage::new_text(PromptMessageRole::User, content)],
		})
	}

	/// Generate the deploy_script workflow prompt.
	fn deploy_script_prompt(&self, args: &Value) -> Result<GetPromptResult> {
		let binary_path = args
			.get("binary_path")
			.and_then(|v| v.as_str())
			.ok_or_else(|| {
				CkbMcpError::InvalidParameter("Missing required argument: binary_path".to_string())
			})?;

		let network = args
			.get("network")
			.and_then(|v| v.as_str())
			.ok_or_else(|| {
				CkbMcpError::InvalidParameter("Missing required argument: network".to_string())
			})?;

		let network_info = match network {
			"devnet" => ("Devnet", "Development network - fast confirmation, free CKB"),
			"testnet" => ("Testnet (Aggron)", "Public test network - request faucet funds"),
			"mainnet" => ("Mainnet (Lina)", "Production network - use real CKB with caution"),
			_ => ("Unknown", "Specify 'devnet', 'testnet', or 'mainnet'"),
		};

		let content = format!(
			r#"# Deploy CKB Script

## Binary Path
`{binary_path}`

## Target Network: {network}
{}: {}

## Deployment Workflow

### Step 1: Verify Prerequisites
1. Check the binary exists and is properly compiled:
   - For Capsule projects: `capsule build --release`
   - Verify binary: `file {binary_path}`

2. Verify network connection:
   - Use `dev_get_chain_type` to confirm connected to {network}

3. Check account balance:
   - Use `dev_get_default_account_info` to see deployer address
   - Use `dev_get_address_balance` to verify sufficient CKB
   - Deployment requires CKB for cell capacity (minimum: binary size + 65 bytes overhead)

### Step 2: Deploy the Script
Use the `dev_deploy_cell_data` tool for small binaries (< 1KB inline hex).

For larger binaries, use the HTTP upload endpoint:
```bash
curl -X POST http://localhost:3112/deploy/file \
  -F "file=@{binary_path}" \
  -F "network={network}"
```

### Step 3: Record Deployment Info
After successful deployment, save:
- **Transaction hash**: The deployment transaction ID
- **Cell out point**: `tx_hash:index` identifying the deployed cell
- **Code hash**: Hash of the script code (use for script references)
- **Hash type**: Usually "data1" for direct code hash

### Step 4: Verify Deployment
1. Use `rpc_get_transaction` with the transaction hash to check confirmation
2. Use `rpc_get_live_cell` to verify the cell exists on-chain

### Step 5: Generate Script Reference
Use `dev_generate_lock_info` or create the script structure:
```json
{{
  "code_hash": "<code_hash_from_deployment>",
  "hash_type": "data1",
  "args": "<your_script_args>"
}}
```

## Recommended Tools
- `dev_get_chain_type` - Verify network
- `dev_get_address_balance` - Check balance
- `dev_deploy_cell_data` - Deploy binary
- `rpc_get_transaction` - Verify deployment
- `rpc_get_live_cell` - Confirm cell exists

## Network-Specific Notes
{}

## Security Reminder
- For {network}, ensure you're using appropriate private keys
- Never deploy untested code to mainnet
- Keep deployment records for future reference"#,
			network_info.0,
			network_info.1,
			match network {
				"devnet" => "Devnet resets frequently. Re-deploy after each reset.",
				"testnet" => "Use `dev_request_testnet_funds` if balance is low.",
				"mainnet" => "Double-check all parameters. Mainnet deployments are permanent.",
				_ => "",
			}
		);

		Ok(GetPromptResult {
			description: Some(format!("Workflow for deploying a script to {}", network)),
			messages: vec![PromptMessage::new_text(PromptMessageRole::User, content)],
		})
	}

	/// Generate the query_blockchain workflow prompt.
	fn query_blockchain_prompt(&self, args: &Value) -> Result<GetPromptResult> {
		let query_type = args
			.get("query_type")
			.and_then(|v| v.as_str())
			.ok_or_else(|| {
				CkbMcpError::InvalidParameter("Missing required argument: query_type".to_string())
			})?;

		let identifier = args
			.get("identifier")
			.and_then(|v| v.as_str())
			.unwrap_or("<not specified>");

		let (tools, examples) = match query_type {
			"cell" => (
				vec![
					"rpc_get_live_cell - Get a specific cell by out point",
					"rpc_get_cells - Search cells by lock/type script",
					"rpc_get_cells_capacity - Get total capacity of matching cells",
				],
				r#"
### Example: Get cells by lock script
```json
{
  "search_key": {
    "script": {
      "code_hash": "0x9bd7...",
      "hash_type": "type",
      "args": "0x..."
    },
    "script_type": "lock"
  },
  "limit": 10
}
```

### Example: Get live cell
```json
{
  "out_point": {
    "tx_hash": "0x...",
    "index": "0x0"
  },
  "with_data": true
}
```"#,
			),
			"transaction" => (
				vec![
					"rpc_get_transaction - Get transaction by hash",
					"rpc_get_transactions - Search transactions",
					"rpc_get_transaction_proof - Get merkle proof",
				],
				r#"
### Example: Get transaction
```json
{
  "tx_hash": "0x..."
}
```

### Example: Search transactions by script
```json
{
  "search_key": {
    "script": {
      "code_hash": "0x...",
      "hash_type": "type",
      "args": "0x..."
    },
    "script_type": "lock"
  }
}
```"#,
			),
			"block" => (
				vec![
					"rpc_get_block - Get block by hash",
					"rpc_get_block_by_number - Get block by height",
					"rpc_get_tip_block_number - Get current height",
				],
				r#"
### Example: Get block by number
```json
{
  "block_number": "0x100"
}
```

### Example: Get block by hash
```json
{
  "block_hash": "0x..."
}
```"#,
			),
			"header" => (
				vec![
					"rpc_get_header - Get header by hash",
					"rpc_get_header_by_number - Get header by height",
					"rpc_get_tip_header - Get current tip header",
				],
				r#"
### Example: Get header by number
```json
{
  "block_number": "0x100"
}
```"#,
			),
			"tip" => (
				vec![
					"rpc_get_tip_header - Get tip header",
					"rpc_get_tip_block_number - Get tip block number",
					"rpc_get_blockchain_info - Get chain info",
				],
				r#"
### Example: Get chain tip
No parameters needed - just call the tool directly.
```"#,
			),
			_ => (
				vec!["Use a valid query_type: cell, transaction, block, header, or tip"],
				"",
			),
		};

		let content = format!(
			r#"# Query CKB Blockchain

## Query Type: {query_type}
Identifier: {identifier}

## Available Tools for {query_type} Queries
{}

## Query Examples
{examples}

## General Query Tips

### Working with Hashes
- All hashes are 32 bytes, represented as 66-character hex strings (0x + 64 chars)
- Block hashes, transaction hashes, and code hashes follow this format

### Working with Numbers
- CKB RPC uses hex-encoded numbers: "0x64" = 100
- Use the `0x` prefix for all numeric parameters

### Pagination
- Use `limit` to control result count
- Use `after_cursor` for pagination through large result sets
- Default limit is usually 10-20 items

### Script Queries
Scripts are identified by:
- `code_hash`: Hash of the script code
- `hash_type`: "type" (by type script hash) or "data1" (by data hash)
- `args`: Script-specific arguments

## Recommended Workflow
1. Use `rpc_get_blockchain_info` to verify connection
2. Use the appropriate query tool based on your needs
3. For cell queries, start with `rpc_get_cells` to search
4. For specific cells, use `rpc_get_live_cell` with the out point

## Documentation Resources
- `ckb://docs/concepts/cell-model` - Understanding cells
- `ckb://docs/concepts/transaction-structure` - Transaction format
- `ckb://docs/api-reference/ckb-rpc` - Full RPC reference"#,
			tools.join("\n- ")
		);

		Ok(GetPromptResult {
			description: Some(format!("Workflow for querying {} data from CKB", query_type)),
			messages: vec![PromptMessage::new_text(PromptMessageRole::User, content)],
		})
	}

	/// Generate the transfer_ckb workflow prompt.
	fn transfer_ckb_prompt(&self, args: &Value) -> Result<GetPromptResult> {
		let to_address = args
			.get("to_address")
			.and_then(|v| v.as_str())
			.ok_or_else(|| {
				CkbMcpError::InvalidParameter("Missing required argument: to_address".to_string())
			})?;

		let amount = args.get("amount").and_then(|v| v.as_str()).ok_or_else(|| {
			CkbMcpError::InvalidParameter("Missing required argument: amount".to_string())
		})?;

		let token_type = args
			.get("token_type")
			.and_then(|v| v.as_str())
			.unwrap_or("ckb");

		let transfer_type = match token_type {
			"ckb" => "Native CKB Transfer",
			"udt" => "User-Defined Token (UDT) Transfer",
			_ => "Transfer (specify 'ckb' or 'udt')",
		};

		let content = format!(
			r#"# {transfer_type}

## Transfer Details
- **To Address**: `{to_address}`
- **Amount**: {amount} {token_type}

## Transfer Workflow

### Step 1: Validate Destination Address
1. Verify the address format is correct for CKB
2. Use `dev_get_lock_info_from_address` to decode the address:
```json
{{"address": "{to_address}"}}
```
3. Confirm the lock script type (secp256k1, multisig, etc.)

### Step 2: Check Sender Balance
1. Use `dev_get_default_account_info` to get sender address
2. Use `dev_get_address_balance` to verify sufficient funds:
   - For CKB: Need {amount} CKB + transaction fee (~0.001 CKB)
   - For UDT: Need UDT amount + CKB for cell capacity + fee

### Step 3: Prepare Transaction
{}

### Step 4: Sign and Send
The transaction will be signed using the configured private key.
Monitor for confirmation using `rpc_get_transaction`.

### Step 5: Verify Transfer
1. Check transaction status: `rpc_get_transaction`
2. Verify recipient balance: `dev_get_address_balance`
3. Confirm cell creation: `rpc_get_cells`

## Important Notes

### Minimum Cell Capacity
- Minimum cell: 61 CKB (for basic secp256k1 lock)
- UDT cells require additional capacity for token data

### Transaction Fees
- Typical fee: 0.0001 - 0.001 CKB
- Fees are calculated per byte of transaction size

### Security Checklist
- [ ] Verified destination address is correct
- [ ] Confirmed sufficient balance for transfer + fees
- [ ] Double-checked amount (especially for large transfers)
- [ ] Using correct network (devnet/testnet/mainnet)

## Recommended Tools
- `dev_get_address_balance` - Check balances
- `dev_get_lock_info_from_address` - Validate addresses
- `dev_get_default_account_info` - Get sender info
- `rpc_get_transaction` - Verify transaction
- `rpc_get_cells` - Search cells

## Documentation Resources
- `ckb://docs/concepts/cell-model` - Cell capacity rules
- `ckb://docs/patterns/token-creation-guide` - UDT patterns
- `ckb://docs/concepts/transaction-structure` - Transaction format"#,
			match token_type {
				"ckb" => format!(
					r#"For native CKB transfer:
1. Collect input cells from sender with sufficient capacity
2. Create output cell for recipient:
   - Lock: Recipient's lock script
   - Capacity: {} CKB (in shannons: {} * 10^8)
3. Create change cell returning excess to sender"#,
					amount, amount
				),
				"udt" => format!(
					r#"For UDT transfer:
1. Collect UDT cells from sender with sufficient token amount
2. Collect CKB cells for capacity and fees
3. Create output UDT cell for recipient:
   - Lock: Recipient's lock script
   - Type: UDT type script
   - Data: Token amount ({} tokens as u128 little-endian)
4. Create change cells for remaining tokens and CKB"#,
					amount
				),
				_ => "Specify token_type as 'ckb' or 'udt'".to_string(),
			}
		);

		Ok(GetPromptResult {
			description: Some(format!("Workflow for transferring {} {}", amount, token_type)),
			messages: vec![PromptMessage::new_text(PromptMessageRole::User, content)],
		})
	}
}

impl Default for PromptsHandlers {
	fn default() -> Self {
		Self::new()
	}
}
