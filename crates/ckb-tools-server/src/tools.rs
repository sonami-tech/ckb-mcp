use shared::{
	ckb_client::CkbRpcClient,
	error::{CkbMcpError, Result},
};
use std::str::FromStr;
use tracing::{debug, info};
use serde::{Deserialize, Serialize};
use ckb_sdk::{
	rpc::ckb_indexer::{ScriptType, SearchKey},
	transaction::{
		builder::{CkbTransactionBuilder, SimpleTransactionBuilder},
		input::InputIterator,
		signer::{SignContexts, TransactionSigner},
		TransactionBuilderConfiguration,
	},
	Address, AddressPayload, CkbRpcClient as SdkCkbRpcClient, HumanCapacity, NetworkInfo,
};
use ckb_types::{
	bytes::Bytes,
	core::Capacity,
	packed::{CellDep, CellOutput, OutPoint, Script},
	prelude::*,
	H256,
};
use ckb_hash::blake2b_256;
use secp256k1::SecretKey;

#[derive(Clone)]
pub struct ToolsProvider {
	ckb_client: CkbRpcClient,
	ckb_rpc_url: String,
	private_key: String,
	network_type: ckb_sdk::NetworkType,
	/// Cached sighash cell_dep from genesis block
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
	pub block_number: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LockScriptInfo {
	pub code_hash: String,
	pub hash_type: String,
	pub args: String,
}

impl ToolsProvider {
	pub fn new(ckb_client: CkbRpcClient, ckb_rpc_url: String, private_key: String) -> Result<Self> {
		// Detect network type by querying genesis hash
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

	/// Detect network type by querying genesis block hash
	/// Returns (NetworkType, Optional custom sighash cell_dep for devnet)
	fn detect_network(rpc_url: &str) -> Result<(ckb_sdk::NetworkType, Option<CellDep>)> {
		use ckb_jsonrpc_types::BlockNumber;

		let client = SdkCkbRpcClient::new(rpc_url);
		let genesis_block = client
			.get_block_by_number(BlockNumber::from(0))
			.map_err(|e| CkbMcpError::Internal(format!("Failed to fetch genesis block: {}", e)))?
			.ok_or_else(|| CkbMcpError::Internal("Genesis block not found".to_string()))?;

		let genesis_hash = genesis_block.header.hash.to_string();

		// Known genesis hashes
		const MAINNET_GENESIS: &str = "0x92b197aa1fba0f63633922c61c92375c9c074a93e85963554f5499fe1450d0e5";
		const TESTNET_GENESIS: &str = "0x10639e0895502b5688a6be8cf69460d76541bfa4821629d86d62ba0aae3f9606";

		match genesis_hash.as_str() {
			MAINNET_GENESIS => Ok((ckb_sdk::NetworkType::Mainnet, None)),
			TESTNET_GENESIS => Ok((ckb_sdk::NetworkType::Testnet, None)),
			_ => {
				// Unknown genesis - assume devnet, fetch sighash cell_dep from genesis
				info!("Unknown genesis hash {}, assuming devnet.", genesis_hash);

				// Get second transaction which contains the dep_group
				// In standard CKB genesis: tx[0] = cellbase with scripts, tx[1] = dep_group
				let dep_group_tx_hash = genesis_block.transactions[1].hash.clone();

				// Sighash dep_group is at output index 0 of transaction 1
				let out_point = OutPoint::new_builder()
					.tx_hash(dep_group_tx_hash.pack())
					.index(0u32.pack())
					.build();

				let cell_dep = CellDep::new_builder()
					.out_point(out_point)
					.dep_type(ckb_types::core::DepType::DepGroup.into())
					.build();

				// Use Testnet as the network type since devnet has same structure
				Ok((ckb_sdk::NetworkType::Testnet, Some(cell_dep)))
			}
		}
	}

	/// Parse private key from hex string (with or without 0x prefix)
	fn parse_private_key(&self) -> Result<SecretKey> {
		let key_hex = self.private_key.trim_start_matches("0x");
		let key_bytes = hex::decode(key_hex)
			.map_err(|e| CkbMcpError::InvalidParameter(format!("Invalid private key hex: {}", e)))?;

		SecretKey::from_slice(&key_bytes)
			.map_err(|e| CkbMcpError::InvalidParameter(format!("Invalid private key: {}", e)))
	}

	/// Derive sender address from private key
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

	pub async fn deploy_cell_data(&self, data: Vec<u8>) -> Result<DeploymentResult> {
		info!("Deploying cell with data size: {} bytes", data.len());
		self.deploy_data_internal(data).await
	}

	pub async fn get_genesis_hash(&self) -> Result<String> {
		info!("Fetching genesis block hash.");

		// Get block 0 (genesis block)
		let genesis_block = self.ckb_client.get_block_by_number(0).await?;

		// Extract the hash from the header
		let hash = genesis_block
			.get("header")
			.and_then(|h| h.get("hash"))
			.and_then(|h| h.as_str())
			.ok_or_else(|| CkbMcpError::Internal("Failed to extract genesis hash".to_string()))?
			.to_string();

		debug!("Genesis hash: {}", hash);
		Ok(hash)
	}

	pub async fn get_chain_type(&self) -> Result<String> {
		info!("Determining chain type.");

		let genesis_hash = self.get_genesis_hash().await?;

		// Known genesis hashes for mainnet and testnet
		const MAINNET_GENESIS: &str = "0x92b197aa1fba0f63633922c61c92375c9c074a93e85963554f5499fe1450d0e5";
		const TESTNET_GENESIS: &str = "0x10639e0895502b5688a6be8cf69460d76541bfa4821629d86d62ba0aae3f9606";

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

	pub async fn get_address_balance(&self, address: Option<String>) -> Result<BalanceInfo> {
		info!("Checking address balance.");

		// Use provided address or default to sender address from private key
		let addr = match address {
			Some(a) => Address::from_str(&a)
				.map_err(|e| CkbMcpError::InvalidParameter(format!("Invalid address: {}", e)))?,
			None => self.get_sender_address()?,
		};

		debug!("Querying balance for address: {}", addr);

		// Build search key for the address's lock script
		let lock_script: Script = Script::from(&addr);
		let search_key = SearchKey {
			script: lock_script.into(),
			script_type: ScriptType::Lock,
			script_search_mode: None,
			filter: None,
			with_data: None,
			group_by_transaction: None,
		};

		// Query capacity using indexer RPC
		let ckb_client = SdkCkbRpcClient::new(&self.ckb_rpc_url);

		let cells_capacity_opt = ckb_client
			.get_cells_capacity(search_key)
			.map_err(|e| CkbMcpError::Internal(format!("Failed to query cells capacity: {}", e)))?;

		let (capacity_shannons, capacity_ckb, block_number) = match cells_capacity_opt {
			Some(cells_capacity) => {
				let capacity_shannons = cells_capacity.capacity.value();
				let capacity_ckb = HumanCapacity::from(capacity_shannons).to_string();
				let block_number = cells_capacity.block_number.value();
				(capacity_shannons, capacity_ckb, block_number)
			}
			None => {
				// No cells found - return zero balance with current tip block number
				let tip_header = ckb_client
					.get_tip_header()
					.map_err(|e| CkbMcpError::Internal(format!("Failed to get tip header: {}", e)))?;
				let block_number = tip_header.inner.number.value();
				(0, HumanCapacity::from(0).to_string(), block_number)
			}
		};

		info!(
			"Address {} has {} CKB ({} shannons) at block {}",
			addr, capacity_ckb, capacity_shannons, block_number
		);

		Ok(BalanceInfo {
			address: addr.to_string(),
			capacity_shannons,
			capacity_ckb,
			block_number,
		})
	}

	pub fn generate_lock_info(&self, private_key: Option<String>) -> Result<LockInfo> {
		info!("Generating lock info from private key.");

		// Use provided private key or default to configured private key
		let key_hex = private_key.unwrap_or_else(|| self.private_key.clone());
		let key_hex = key_hex.trim_start_matches("0x");

		// Parse private key
		let key_bytes = hex::decode(key_hex)
			.map_err(|e| CkbMcpError::InvalidParameter(format!("Invalid private key hex: {}", e)))?;
		let secret_key = SecretKey::from_slice(&key_bytes)
			.map_err(|e| CkbMcpError::InvalidParameter(format!("Invalid private key: {}", e)))?;

		// Derive public key
		let secp = secp256k1::Secp256k1::new();
		let pubkey = secp256k1::PublicKey::from_secret_key(&secp, &secret_key);
		let pubkey_bytes = pubkey.serialize();

		// Calculate lock arg (blake2b hash of public key, first 20 bytes)
		let pubkey_hash = blake2b_256(&pubkey_bytes[..])[0..20].to_vec();

		// Build lock script (secp256k1_blake160_sighash_all)
		let mut hash_bytes = [0u8; 20];
		hash_bytes.copy_from_slice(&pubkey_hash);
		let payload = AddressPayload::from_pubkey_hash(hash_bytes.into());

		// Generate addresses for both networks
		let address_testnet = Address::new(ckb_sdk::NetworkType::Testnet, payload.clone(), true);
		let address_mainnet = Address::new(ckb_sdk::NetworkType::Mainnet, payload.clone(), true);

		// Get lock script
		let lock_script: Script = Script::from(&address_testnet);

		// Calculate lock hash (blake2b of molecule-serialized script)
		let lock_hash_bytes = lock_script.calc_script_hash();

		Ok(LockInfo {
			private_key: format!("0x{}", key_hex),
			public_key: format!("0x{}", hex::encode(pubkey_bytes)),
			lock_arg: format!("0x{}", hex::encode(&pubkey_hash)),
			lock_script: LockScriptInfo {
				code_hash: format!("{:#x}", lock_script.code_hash()),
				hash_type: format!("{:?}", lock_script.hash_type()),
				args: format!("0x{}", hex::encode(lock_script.args().raw_data())),
			},
			lock_hash: format!("{:#x}", lock_hash_bytes),
			address_testnet: address_testnet.to_string(),
			address_mainnet: address_mainnet.to_string(),
		})
	}

	pub fn get_lock_info_from_address(&self, address: String) -> Result<LockInfo> {
		info!("Extracting lock info from address.");

		// Parse address
		let addr = Address::from_str(&address)
			.map_err(|e| CkbMcpError::InvalidParameter(format!("Invalid address: {}", e)))?;

		// Get lock script from address
		let lock_script: Script = Script::from(&addr);

		// Calculate lock hash (blake2b of molecule-serialized script)
		let lock_hash_bytes = lock_script.calc_script_hash();

		// Extract lock arg
		let lock_arg = lock_script.args().raw_data();

		// Determine network type and generate counterpart address
		let network_type = addr.network();
		let payload = addr.payload();
		let (address_testnet, address_mainnet) = match network_type {
			ckb_sdk::NetworkType::Testnet => {
				let mainnet_addr = Address::new(ckb_sdk::NetworkType::Mainnet, payload.clone(), true);
				(address.clone(), mainnet_addr.to_string())
			}
			ckb_sdk::NetworkType::Mainnet => {
				let testnet_addr = Address::new(ckb_sdk::NetworkType::Testnet, payload.clone(), true);
				(testnet_addr.to_string(), address.clone())
			}
			_ => {
				// For dev network, just use the same address for both
				(address.clone(), address.clone())
			}
		};

		Ok(LockInfo {
			private_key: "N/A - Cannot derive from address".to_string(),
			public_key: "N/A - Cannot derive from address".to_string(),
			lock_arg: format!("0x{}", hex::encode(&lock_arg)),
			lock_script: LockScriptInfo {
				code_hash: format!("{:#x}", lock_script.code_hash()),
				hash_type: format!("{:?}", lock_script.hash_type()),
				args: format!("0x{}", hex::encode(lock_script.args().raw_data())),
			},
			lock_hash: format!("{:#x}", lock_hash_bytes),
			address_testnet,
			address_mainnet,
		})
	}

	pub async fn request_testnet_funds(&self, address: Option<String>) -> Result<String> {
		info!("Requesting testnet funds from faucet.");

		let addr = match address {
			Some(a) => Address::from_str(&a)
				.map_err(|e| CkbMcpError::InvalidParameter(format!("Invalid address: {}", e)))?,
			None => self.get_sender_address()?,
		};

		// Call the Nervos testnet faucet API.
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
		let body = response.text().await
			.map_err(|e| CkbMcpError::Http(format!("Failed to read faucet response: {}", e)))?;

		if !status.is_success() {
			return Err(CkbMcpError::Http(format!(
				"Faucet request failed with status {}: {}",
				status, body
			)));
		}

		info!("Faucet request successful for address: {}", addr);
		Ok(format!("Successfully requested testnet funds for address: {}\nResponse: {}", addr, body))
	}

	pub async fn get_default_account_info(&self) -> Result<DefaultAccountInfo> {
		info!("Getting default account information.");

		// Generate lock info from the configured private key
		let lock_info = self.generate_lock_info(None)?;

		// Get balance for the default address
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
			block_number: balance_info.block_number,
		})
	}

	async fn deploy_data_internal(&self, data: Vec<u8>) -> Result<DeploymentResult> {
		let data_size = data.len();
		info!("Building transaction to deploy {} bytes of data", data_size);

		// Parse private key and get sender address
		let secret_key = self.parse_private_key()?;
		let sender_address = self.get_sender_address()?;

		debug!("Sender address: {}", sender_address);

		// Setup network info using detected network type
		let network_info = NetworkInfo::new(self.network_type, self.ckb_rpc_url.clone());
		let mut configuration = TransactionBuilderConfiguration::new_with_network(network_info.clone())
			.map_err(|e| CkbMcpError::Internal(format!("Failed to create transaction configuration: {}", e)))?;

		// For devnet, override the sighash cell_dep with the one from actual genesis
		if let Some(ref custom_cell_dep) = self.sighash_cell_dep {
			// Replace the hardcoded testnet sighash cell_dep with our devnet one
			configuration.script_handlers[0] = Box::new(
				ckb_sdk::transaction::handler::sighash::Secp256k1Blake160SighashAllScriptHandler::new_with_customize(
					vec![custom_cell_dep.clone()]
				)
			);
			debug!("Overrode sighash cell_dep with devnet genesis cell_dep.");
		}

		// Adjust fee configuration for more reliable transaction acceptance
		// Default estimate_tx_size is 128000 which causes massive fee overestimation
		// We use a more realistic 10000 bytes AND increase the fee rate to 30000 shannons/KB
		// to ensure transactions can replace any pending ones in the pool (RBF requires ~2.3x)
		configuration.estimate_tx_size = 10000;
		configuration.fee_rate = 30000; // 30000 shannons per KB (30x default of 1000, ensures RBF replacement)

		// Calculate required capacity using proper occupied_capacity method
		// This accounts for cell header (8 bytes capacity + 32 bytes data hash + 1 byte data length),
		// lock script, and data
		let lock_script: Script = Script::from(&sender_address);
		let data_bytes = Bytes::from(data.clone());

		// First create a temporary output to calculate occupied capacity
		let temp_output = CellOutput::new_builder()
			.capacity(Capacity::zero().pack())
			.lock(lock_script.clone())
			.build();

		// Calculate the actual occupied capacity including all overhead
		let output_capacity = temp_output
			.occupied_capacity(Capacity::bytes(data.len()).unwrap())
			.map_err(|e| CkbMcpError::Internal(format!("Capacity calculation error: {}", e)))?;

		debug!("Output capacity needed: {} shannons", output_capacity.as_u64());

		// Build final output cell with correct capacity
		let output = CellOutput::new_builder()
			.capacity(output_capacity.pack())
			.lock(lock_script.clone())
			.build();

		// Create input iterator - this will automatically collect cells as needed
		let iterator = InputIterator::new_with_address(&[sender_address.clone()], &network_info);

		// Build transaction using SimpleTransactionBuilder
		// The builder will automatically collect enough inputs to cover outputs + fees
		let mut builder = SimpleTransactionBuilder::new(configuration, iterator);
		builder.add_output_and_data(output, data_bytes.pack());

		let mut tx_with_groups = builder
			.build(&Default::default())
			.map_err(|e| CkbMcpError::Internal(format!("Failed to build transaction: {}", e)))?;

		let input_count = tx_with_groups.get_tx_view().inputs().len();
		debug!("Transaction built successfully with {} inputs", input_count);

		// Sign transaction
		let private_keys = vec![H256::from_slice(secret_key.as_ref())
			.map_err(|e| CkbMcpError::Internal(format!("Invalid private key format: {}", e)))?];

		TransactionSigner::new(&network_info)
			.sign_transaction(
				&mut tx_with_groups,
				&SignContexts::new_sighash_h256(private_keys)
					.map_err(|e| CkbMcpError::Internal(format!("Failed to create sign context: {}", e)))?,
			)
			.map_err(|e| CkbMcpError::Internal(format!("Failed to sign transaction: {}", e)))?;

		debug!("Transaction signed successfully.");

		// Convert to JSON-RPC format and send
		let tx_json = ckb_jsonrpc_types::TransactionView::from(tx_with_groups.get_tx_view().clone());

		// Debug: Log cell_deps and inputs
		for (i, cell_dep) in tx_json.inner.cell_deps.iter().enumerate() {
			debug!("CellDep {}: tx_hash={}, index={}", i, cell_dep.out_point.tx_hash, cell_dep.out_point.index);
		}
		for (i, input) in tx_json.inner.inputs.iter().enumerate() {
			debug!("Input {}: previous_output tx_hash={}, index={}", i, input.previous_output.tx_hash, input.previous_output.index);
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
