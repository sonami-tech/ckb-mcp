//! Development tool handlers.

use ckb_hash::blake2b_256;

/// Known genesis hashes for network detection.
const MAINNET_GENESIS: &str = "0x92b197aa1fba0f63633922c61c92375c9c074a93e85963554f5499fe1450d0e5";
const TESTNET_GENESIS: &str = "0x10639e0895502b5688a6be8cf69460d76541bfa4821629d86d62ba0aae3f9606";
use ckb_sdk::{
	Address, AddressPayload, CkbRpcClient as SdkCkbRpcClient, HumanCapacity, NetworkInfo,
	rpc::ckb_indexer::{ScriptType, SearchKey, SearchKeyFilter},
	transaction::{
		TransactionBuilderConfiguration,
		builder::{CkbTransactionBuilder, SimpleTransactionBuilder},
		input::InputIterator,
		signer::{SignContexts, TransactionSigner},
	},
};
use ckb_types::{
	H256,
	bytes::Bytes,
	core::Capacity,
	packed::{CellDep, CellOutput, OutPoint, Script},
	prelude::*,
};
use rmcp::model::{CallToolResult, Content};
use secp256k1::SecretKey;
use serde::{Deserialize, Serialize};
use shared::ckb_client::CkbRpcClient;
use shared::error::{CkbMcpError, Result};
use std::str::FromStr;
use tracing::{debug, info};

/// Development tool handlers for CKB operations.
#[derive(Clone)]
pub struct DevHandlers {
	ckb_client: CkbRpcClient,
	ckb_rpc_url: String,
	private_key: String,
	network_type: ckb_sdk::NetworkType,
	/// Cached sighash cell_dep from genesis block.
	sighash_cell_dep: Option<CellDep>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeploymentResult {
	pub tx_hash: String,
	pub output_index: u32,
	pub data_size: usize,
	pub capacity: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BalanceInfo {
	pub address: String,
	pub capacity_shannons: u64,
	pub capacity_ckb: String,
	pub free_capacity_shannons: u64,
	pub free_capacity_ckb: String,
	pub occupied_capacity_shannons: u64,
	pub occupied_capacity_ckb: String,
	pub block_number: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LockInfo {
	pub private_key: String,
	pub public_key: String,
	pub lock_arg: String,
	pub lock_script: LockScriptInfo,
	pub lock_hash: String,
	pub address_testnet: String,
	pub address_mainnet: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DefaultAccountInfo {
	pub public_key: String,
	pub lock_arg: String,
	pub lock_script: LockScriptInfo,
	pub lock_hash: String,
	pub address_testnet: String,
	pub address_mainnet: String,
	pub capacity_shannons: u64,
	pub capacity_ckb: String,
	pub free_capacity_shannons: u64,
	pub free_capacity_ckb: String,
	pub occupied_capacity_shannons: u64,
	pub occupied_capacity_ckb: String,
	pub block_number: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LockScriptInfo {
	pub code_hash: String,
	pub hash_type: String,
	pub args: String,
}

impl DevHandlers {
	/// Create new DevHandlers instance.
	pub fn new(ckb_client: CkbRpcClient, ckb_rpc_url: String, private_key: String) -> Result<Self> {
		// Detect network type by querying genesis hash.
		let (network_type, sighash_cell_dep) = Self::detect_network(&ckb_rpc_url)?;

		info!("Detected network type: {:?}.", network_type);
		if sighash_cell_dep.is_some() {
			debug!("Using custom sighash cell_dep from genesis.");
		}

		Ok(Self {
			ckb_client,
			ckb_rpc_url,
			private_key,
			network_type,
			sighash_cell_dep,
		})
	}

	/// Check if a tool name is a dev tool.
	pub fn is_dev_tool(name: &str) -> bool {
		name.starts_with("dev_")
	}

	/// Handle a dev tool call.
	pub async fn handle(&self, name: &str, args: &serde_json::Value) -> Result<CallToolResult> {
		match name {
			"dev_deploy_cell_data" => self.handle_deploy_cell_data(args).await,
			"dev_get_address_balance" => self.handle_get_address_balance(args).await,
			"dev_get_chain_type" => self.handle_get_chain_type().await,
			"dev_get_genesis_hash" => self.handle_get_genesis_hash().await,
			"dev_generate_lock_info" => self.handle_generate_lock_info(args),
			"dev_get_lock_info_from_address" => self.handle_get_lock_info_from_address(args),
			"dev_request_testnet_funds" => self.handle_request_testnet_funds(args).await,
			"dev_get_default_account_info" => self.handle_get_default_account_info().await,
			_ => Err(CkbMcpError::InvalidParameter(format!(
				"Unknown dev tool: {}",
				name
			))),
		}
	}

	/// Deploy cell data from hex string.
	pub async fn deploy_cell_data(&self, data: Vec<u8>) -> Result<DeploymentResult> {
		info!("Deploying cell with data size: {} bytes", data.len());
		self.deploy_data_internal(data).await
	}

	/// Detect network type by querying genesis block hash.
	/// Returns (NetworkType, Optional custom sighash cell_dep for devnet).
	fn detect_network(rpc_url: &str) -> Result<(ckb_sdk::NetworkType, Option<CellDep>)> {
		use ckb_jsonrpc_types::BlockNumber;

		let client = SdkCkbRpcClient::new(rpc_url);
		let genesis_block = client
			.get_block_by_number(BlockNumber::from(0))
			.map_err(|e| CkbMcpError::Internal(format!("Failed to fetch genesis block: {}", e)))?
			.ok_or_else(|| CkbMcpError::Internal("Genesis block not found".to_string()))?;

		let raw_hash = genesis_block.header.hash.to_string();
		// Normalize hash to have 0x prefix for comparison with constants.
		let genesis_hash = if raw_hash.starts_with("0x") {
			raw_hash
		} else {
			format!("0x{}", raw_hash)
		};

		match genesis_hash.as_str() {
			MAINNET_GENESIS => Ok((ckb_sdk::NetworkType::Mainnet, None)),
			TESTNET_GENESIS => Ok((ckb_sdk::NetworkType::Testnet, None)),
			_ => {
				// Unknown genesis - assume devnet, fetch sighash cell_dep from genesis.
				info!("Unknown genesis hash {}, assuming devnet.", genesis_hash);

				// Get second transaction which contains the dep_group.
				// In standard CKB genesis: tx[0] = cellbase with scripts, tx[1] = dep_group.
				let dep_group_tx_hash = genesis_block.transactions[1].hash.clone();

				// Sighash dep_group is at output index 0 of transaction 1.
				let out_point = OutPoint::new_builder()
					.tx_hash(dep_group_tx_hash.pack())
					.index(0u32)
					.build();

				let cell_dep = CellDep::new_builder()
					.out_point(out_point)
					.dep_type(ckb_types::core::DepType::DepGroup)
					.build();

				// Use Testnet as the network type since devnet has same structure.
				Ok((ckb_sdk::NetworkType::Testnet, Some(cell_dep)))
			}
		}
	}

	/// Parse private key from hex string (with or without 0x prefix).
	fn parse_private_key(&self) -> Result<SecretKey> {
		let key_hex = self.private_key.trim_start_matches("0x");
		let key_bytes = hex::decode(key_hex).map_err(|e| {
			CkbMcpError::InvalidParameter(format!("Invalid private key hex: {}", e))
		})?;

		SecretKey::from_slice(&key_bytes)
			.map_err(|e| CkbMcpError::InvalidParameter(format!("Invalid private key: {}", e)))
	}

	/// Derive sender address from private key.
	fn get_sender_address(&self) -> Result<Address> {
		let secret_key = self.parse_private_key()?;
		let secp = secp256k1::Secp256k1::new();
		let pubkey = secp256k1::PublicKey::from_secret_key(&secp, &secret_key);
		let pubkey_hash = blake2b_256(&pubkey.serialize()[..])[0..20].to_vec();

		let mut hash_bytes = [0u8; 20];
		hash_bytes.copy_from_slice(&pubkey_hash);
		let payload = AddressPayload::from_pubkey_hash(hash_bytes.into());
		Ok(Address::new(ckb_sdk::NetworkType::Testnet, payload, true))
	}

	// Tool handlers.

	async fn handle_deploy_cell_data(&self, args: &serde_json::Value) -> Result<CallToolResult> {
		let data_hex = args
			.get("data")
			.and_then(|v| v.as_str())
			.ok_or_else(|| CkbMcpError::InvalidParameter("Missing data parameter".to_string()))?;

		// Decode hex string to bytes.
		let data = hex::decode(data_hex.trim_start_matches("0x"))
			.map_err(|e| CkbMcpError::InvalidParameter(format!("Invalid hex data: {}", e)))?;

		// Enforce 1KB limit for inline data - larger files should use HTTP upload.
		const MAX_DATA_SIZE: usize = 1024;
		if data.len() > MAX_DATA_SIZE {
			return Err(CkbMcpError::InvalidParameter(format!(
				"Data size ({} bytes) exceeds 1KB limit. For larger files, use: \
				curl -F 'file=@/path/to/file' <base_url>/deploy/file",
				data.len()
			)));
		}

		let result = self.deploy_cell_data(data).await?;
		let json = serde_json::to_string_pretty(&result)?;
		Ok(CallToolResult::success(vec![Content::text(json)]))
	}

	async fn handle_get_address_balance(&self, args: &serde_json::Value) -> Result<CallToolResult> {
		let address = args
			.get("address")
			.and_then(|v| v.as_str())
			.map(|s| s.to_string());

		let result = self.get_address_balance(address).await?;
		let json = serde_json::to_string_pretty(&result)?;
		Ok(CallToolResult::success(vec![Content::text(json)]))
	}

	async fn handle_get_chain_type(&self) -> Result<CallToolResult> {
		let chain_type = self.get_chain_type().await?;
		Ok(CallToolResult::success(vec![Content::text(chain_type)]))
	}

	async fn handle_get_genesis_hash(&self) -> Result<CallToolResult> {
		let genesis_hash = self.get_genesis_hash().await?;
		Ok(CallToolResult::success(vec![Content::text(genesis_hash)]))
	}

	fn handle_generate_lock_info(&self, args: &serde_json::Value) -> Result<CallToolResult> {
		let private_key = args
			.get("private_key")
			.and_then(|v| v.as_str())
			.ok_or_else(|| {
				CkbMcpError::InvalidParameter(
					"private_key parameter is required. To get info about the default account, \
					use dev_get_default_account_info instead."
						.to_string(),
				)
			})?
			.to_string();

		let result = self.generate_lock_info(Some(private_key))?;
		let json = serde_json::to_string_pretty(&result)?;
		Ok(CallToolResult::success(vec![Content::text(json)]))
	}

	fn handle_get_lock_info_from_address(
		&self,
		args: &serde_json::Value,
	) -> Result<CallToolResult> {
		let address = args
			.get("address")
			.and_then(|v| v.as_str())
			.ok_or_else(|| CkbMcpError::InvalidParameter("Missing address parameter".to_string()))?
			.to_string();

		let result = self.get_lock_info_from_address(address)?;
		let json = serde_json::to_string_pretty(&result)?;
		Ok(CallToolResult::success(vec![Content::text(json)]))
	}

	async fn handle_request_testnet_funds(
		&self,
		args: &serde_json::Value,
	) -> Result<CallToolResult> {
		let address = args
			.get("address")
			.and_then(|v| v.as_str())
			.map(|s| s.to_string());

		let result = self.request_testnet_funds(address).await?;
		Ok(CallToolResult::success(vec![Content::text(result)]))
	}

	async fn handle_get_default_account_info(&self) -> Result<CallToolResult> {
		let result = self.get_default_account_info().await?;
		let json = serde_json::to_string_pretty(&result)?;
		Ok(CallToolResult::success(vec![Content::text(json)]))
	}

	// Internal tool implementations.

	async fn get_genesis_hash(&self) -> Result<String> {
		debug!("Fetching genesis block hash.");

		let genesis_block = self.ckb_client.get_block_by_number(0).await?;
		let hash = genesis_block
			.get("header")
			.and_then(|h| h.get("hash"))
			.and_then(|h| h.as_str())
			.ok_or_else(|| CkbMcpError::Internal("Failed to extract genesis hash".to_string()))?
			.to_string();

		debug!("Genesis hash: {}", hash);
		Ok(hash)
	}

	async fn get_chain_type(&self) -> Result<String> {
		debug!("Determining chain type.");

		let genesis_hash = self.get_genesis_hash().await?;

		let chain_type = if genesis_hash == MAINNET_GENESIS {
			"mainnet"
		} else if genesis_hash == TESTNET_GENESIS {
			"testnet"
		} else {
			"devnet"
		};

		info!("Chain type: {}", chain_type);
		Ok(chain_type.to_string())
	}

	async fn get_address_balance(&self, address: Option<String>) -> Result<BalanceInfo> {
		debug!("Checking address balance.");

		let addr = match address {
			Some(a) => Address::from_str(&a)
				.map_err(|e| CkbMcpError::InvalidParameter(format!("Invalid address: {}", e)))?,
			None => self.get_sender_address()?,
		};

		debug!("Querying balance for address: {}", addr);

		let lock_script: Script = Script::from(&addr);

		let search_key_total = SearchKey {
			script: lock_script.clone().into(),
			script_type: ScriptType::Lock,
			script_search_mode: None,
			filter: None,
			with_data: None,
			group_by_transaction: None,
		};

		let search_key_free = SearchKey {
			script: lock_script.into(),
			script_type: ScriptType::Lock,
			script_search_mode: None,
			filter: Some(SearchKeyFilter {
				script: None,
				script_len_range: Some([0u64.into(), 1u64.into()]),
				output_data: None,
				output_data_filter_mode: None,
				output_data_len_range: Some([0u64.into(), 1u64.into()]),
				output_capacity_range: None,
				block_range: None,
			}),
			with_data: None,
			group_by_transaction: None,
		};

		let ckb_client = SdkCkbRpcClient::new(&self.ckb_rpc_url);

		let cells_capacity_total =
			ckb_client
				.get_cells_capacity(search_key_total)
				.map_err(|e| {
					CkbMcpError::Internal(format!("Failed to query total cells capacity: {}", e))
				})?;

		let cells_capacity_free = ckb_client
			.get_cells_capacity(search_key_free)
			.map_err(|e| {
				CkbMcpError::Internal(format!("Failed to query free cells capacity: {}", e))
			})?;

		let (
			capacity_shannons,
			capacity_ckb,
			free_capacity_shannons,
			free_capacity_ckb,
			occupied_capacity_shannons,
			occupied_capacity_ckb,
			block_number,
		) = match cells_capacity_total {
			Some(total_capacity) => {
				let total_shannons = total_capacity.capacity.value();
				let total_ckb = HumanCapacity::from(total_shannons).to_string();
				let block_number = total_capacity.block_number.value();

				let free_shannons = cells_capacity_free
					.map(|fc| fc.capacity.value())
					.unwrap_or(0);
				let free_ckb = HumanCapacity::from(free_shannons).to_string();

				let occupied_shannons = total_shannons.saturating_sub(free_shannons);
				let occupied_ckb = HumanCapacity::from(occupied_shannons).to_string();

				(
					total_shannons,
					total_ckb,
					free_shannons,
					free_ckb,
					occupied_shannons,
					occupied_ckb,
					block_number,
				)
			}
			None => {
				let tip_header = ckb_client.get_tip_header().map_err(|e| {
					CkbMcpError::Internal(format!("Failed to get tip header: {}", e))
				})?;
				let block_number = tip_header.inner.number.value();
				let zero_ckb = HumanCapacity::from(0).to_string();
				(
					0,
					zero_ckb.clone(),
					0,
					zero_ckb.clone(),
					0,
					zero_ckb,
					block_number,
				)
			}
		};

		info!(
			"Address {} has {} CKB total, {} CKB free, {} CKB occupied at block {}",
			addr, capacity_ckb, free_capacity_ckb, occupied_capacity_ckb, block_number
		);

		Ok(BalanceInfo {
			address: addr.to_string(),
			capacity_shannons,
			capacity_ckb,
			free_capacity_shannons,
			free_capacity_ckb,
			occupied_capacity_shannons,
			occupied_capacity_ckb,
			block_number,
		})
	}

	fn generate_lock_info(&self, private_key: Option<String>) -> Result<LockInfo> {
		debug!("Generating lock info from private key.");

		let key_hex = private_key.unwrap_or_else(|| self.private_key.clone());
		let key_hex = key_hex.trim_start_matches("0x");

		let key_bytes = hex::decode(key_hex).map_err(|e| {
			CkbMcpError::InvalidParameter(format!("Invalid private key hex: {}", e))
		})?;
		let secret_key = SecretKey::from_slice(&key_bytes)
			.map_err(|e| CkbMcpError::InvalidParameter(format!("Invalid private key: {}", e)))?;

		let secp = secp256k1::Secp256k1::new();
		let pubkey = secp256k1::PublicKey::from_secret_key(&secp, &secret_key);
		let pubkey_bytes = pubkey.serialize();

		let pubkey_hash = blake2b_256(&pubkey_bytes[..])[0..20].to_vec();

		let mut hash_bytes = [0u8; 20];
		hash_bytes.copy_from_slice(&pubkey_hash);
		let payload = AddressPayload::from_pubkey_hash(hash_bytes.into());

		let address_testnet = Address::new(ckb_sdk::NetworkType::Testnet, payload.clone(), true);
		let address_mainnet = Address::new(ckb_sdk::NetworkType::Mainnet, payload, true);

		let lock_script: Script = Script::from(&address_testnet);
		let lock_hash_bytes = lock_script.calc_script_hash();

		Ok(LockInfo {
			private_key: format!("0x{}", key_hex),
			public_key: format!("0x{}", hex::encode(pubkey_bytes)),
			lock_arg: format!("0x{}", hex::encode(&pubkey_hash)),
			lock_script: LockScriptInfo {
				code_hash: format!("{:#x}", lock_script.code_hash()),
				hash_type: crate::util::hash_type_to_string(lock_script.hash_type()),
				args: format!("0x{}", hex::encode(lock_script.args().raw_data())),
			},
			lock_hash: format!("{:#x}", lock_hash_bytes),
			address_testnet: address_testnet.to_string(),
			address_mainnet: address_mainnet.to_string(),
		})
	}

	fn get_lock_info_from_address(&self, address: String) -> Result<LockInfo> {
		debug!("Extracting lock info from address.");

		let addr = Address::from_str(&address)
			.map_err(|e| CkbMcpError::InvalidParameter(format!("Invalid address: {}", e)))?;

		let lock_script: Script = Script::from(&addr);
		let lock_hash_bytes = lock_script.calc_script_hash();
		let lock_arg = lock_script.args().raw_data();

		let network_type = addr.network();
		let payload = addr.payload();
		let (address_testnet, address_mainnet) = match network_type {
			ckb_sdk::NetworkType::Testnet => {
				let mainnet_addr =
					Address::new(ckb_sdk::NetworkType::Mainnet, payload.clone(), true);
				(address.clone(), mainnet_addr.to_string())
			}
			ckb_sdk::NetworkType::Mainnet => {
				let testnet_addr =
					Address::new(ckb_sdk::NetworkType::Testnet, payload.clone(), true);
				(testnet_addr.to_string(), address.clone())
			}
			_ => (address.clone(), address.clone()),
		};

		Ok(LockInfo {
			private_key: "N/A - Cannot derive from address".to_string(),
			public_key: "N/A - Cannot derive from address".to_string(),
			lock_arg: format!("0x{}", hex::encode(&lock_arg)),
			lock_script: LockScriptInfo {
				code_hash: format!("{:#x}", lock_script.code_hash()),
				hash_type: crate::util::hash_type_to_string(lock_script.hash_type()),
				args: format!("0x{}", hex::encode(lock_script.args().raw_data())),
			},
			lock_hash: format!("{:#x}", lock_hash_bytes),
			address_testnet,
			address_mainnet,
		})
	}

	async fn request_testnet_funds(&self, address: Option<String>) -> Result<String> {
		info!("Requesting testnet funds from faucet.");

		let addr = match address {
			Some(a) => Address::from_str(&a)
				.map_err(|e| CkbMcpError::InvalidParameter(format!("Invalid address: {}", e)))?,
			None => self.get_sender_address()?,
		};

		let client = reqwest::Client::builder()
			.timeout(std::time::Duration::from_secs(30))
			.connect_timeout(std::time::Duration::from_secs(5))
			.build()
			.map_err(|e| CkbMcpError::Internal(format!("Failed to build HTTP client: {}", e)))?;

		let response = client
			.post("https://faucet-api.nervos.org/claim_events")
			.header("accept", "application/json, text/plain, */*")
			.header("content-type", "application/json")
			.json(&serde_json::json!({
				"claim_event": {
					"address_hash": addr.to_string(),
					"amount": "100000"
				}
			}))
			.send()
			.await
			.map_err(|e| CkbMcpError::Http(format!("Failed to request faucet funds: {}", e)))?;

		let status = response.status();
		let body = response
			.text()
			.await
			.map_err(|e| CkbMcpError::Http(format!("Failed to read faucet response: {}", e)))?;

		if !status.is_success() {
			return Err(CkbMcpError::Http(format!(
				"Faucet request failed with status {}: {}",
				status, body
			)));
		}

		info!("Faucet request successful for address: {}", addr);
		Ok(format!(
			"Successfully requested testnet funds for address: {}\nResponse: {}",
			addr, body
		))
	}

	async fn get_default_account_info(&self) -> Result<DefaultAccountInfo> {
		debug!("Getting default account information.");

		let lock_info = self.generate_lock_info(None)?;
		let balance_info = self.get_address_balance(None).await?;

		Ok(DefaultAccountInfo {
			public_key: lock_info.public_key,
			lock_arg: lock_info.lock_arg,
			lock_script: lock_info.lock_script,
			lock_hash: lock_info.lock_hash,
			address_testnet: lock_info.address_testnet,
			address_mainnet: lock_info.address_mainnet,
			capacity_shannons: balance_info.capacity_shannons,
			capacity_ckb: balance_info.capacity_ckb,
			free_capacity_shannons: balance_info.free_capacity_shannons,
			free_capacity_ckb: balance_info.free_capacity_ckb,
			occupied_capacity_shannons: balance_info.occupied_capacity_shannons,
			occupied_capacity_ckb: balance_info.occupied_capacity_ckb,
			block_number: balance_info.block_number,
		})
	}

	async fn deploy_data_internal(&self, data: Vec<u8>) -> Result<DeploymentResult> {
		let data_size = data.len();
		info!("Building transaction to deploy {} bytes of data", data_size);

		let secret_key = self.parse_private_key()?;
		let sender_address = self.get_sender_address()?;

		debug!("Sender address: {}", sender_address);

		let network_info = NetworkInfo::new(self.network_type, self.ckb_rpc_url.clone());
		let mut configuration = TransactionBuilderConfiguration::new_with_network(
			network_info.clone(),
		)
		.map_err(|e| {
			CkbMcpError::Internal(format!("Failed to create transaction configuration: {}", e))
		})?;

		// For devnet, override the sighash cell_dep with the one from actual genesis.
		if let Some(ref custom_cell_dep) = self.sighash_cell_dep {
			configuration.script_handlers[0] = Box::new(
				ckb_sdk::transaction::handler::sighash::Secp256k1Blake160SighashAllScriptHandler::new_with_customize(
					vec![custom_cell_dep.clone()]
				)
			);
			debug!("Overrode sighash cell_dep with devnet genesis cell_dep.");
		}

		configuration.estimate_tx_size = 10000;
		configuration.fee_rate = 30000;

		let lock_script: Script = Script::from(&sender_address);
		let data_bytes = Bytes::from(data.clone());

		let temp_output = CellOutput::new_builder()
			.capacity(Capacity::zero().pack())
			.lock(lock_script.clone())
			.build();

		let data_capacity = Capacity::bytes(data.len())
			.map_err(|e| CkbMcpError::InvalidParameter(format!("Data size too large: {}", e)))?;

		let output_capacity = temp_output
			.occupied_capacity(data_capacity)
			.map_err(|e| CkbMcpError::Internal(format!("Capacity calculation error: {}", e)))?;

		debug!(
			"Output capacity needed: {} shannons",
			output_capacity.as_u64()
		);

		let output = CellOutput::new_builder()
			.capacity(output_capacity.pack())
			.lock(lock_script)
			.build();

		let iterator = InputIterator::new_with_address(&[sender_address], &network_info);

		let mut builder = SimpleTransactionBuilder::new(configuration, iterator);
		builder.add_output_and_data(output, data_bytes.pack());

		let mut tx_with_groups = builder
			.build(&Default::default())
			.map_err(|e| CkbMcpError::Internal(format!("Failed to build transaction: {}", e)))?;

		let input_count = tx_with_groups.get_tx_view().inputs().len();
		debug!("Transaction built successfully with {} inputs", input_count);

		let private_keys =
			vec![H256::from_slice(secret_key.as_ref()).map_err(|e| {
				CkbMcpError::Internal(format!("Invalid private key format: {}", e))
			})?];

		TransactionSigner::new(&network_info)
			.sign_transaction(
				&mut tx_with_groups,
				&SignContexts::new_sighash_h256(private_keys).map_err(|e| {
					CkbMcpError::Internal(format!("Failed to create sign context: {}", e))
				})?,
			)
			.map_err(|e| CkbMcpError::Internal(format!("Failed to sign transaction: {}", e)))?;

		debug!("Transaction signed successfully.");

		let tx_json =
			ckb_jsonrpc_types::TransactionView::from(tx_with_groups.get_tx_view().clone());

		for (i, cell_dep) in tx_json.inner.cell_deps.iter().enumerate() {
			debug!(
				"CellDep {}: tx_hash={}, index={}",
				i, cell_dep.out_point.tx_hash, cell_dep.out_point.index
			);
		}
		for (i, input) in tx_json.inner.inputs.iter().enumerate() {
			debug!(
				"Input {}: previous_output tx_hash={}, index={}",
				i, input.previous_output.tx_hash, input.previous_output.index
			);
		}

		let tx_hash = SdkCkbRpcClient::new(network_info.url.as_str())
			.send_transaction(tx_json.inner, None)
			.map_err(|e| CkbMcpError::Internal(format!("Failed to send transaction: {}", e)))?;

		info!("Transaction sent successfully: {:#x}", tx_hash);

		Ok(DeploymentResult {
			tx_hash: format!("{:#x}", tx_hash),
			output_index: 0,
			data_size,
			capacity: output_capacity.as_u64(),
		})
	}
}
