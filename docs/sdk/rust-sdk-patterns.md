# Rust SDK Patterns

## Description

CKB Rust SDK patterns for transaction building, cell collection, multi-signature operations, and DAO interactions. SDK setup and configuration for mainnet/testnet/devnet. Cell management with lock scripts and UDT cells. Advanced transaction operations including transfers, Omnilock integration, and Anyone-Can-Pay. Nervos DAO deposit/withdrawal handling with compensation calculation. Error handling patterns and network abstraction.

## Basic SDK Setup and Configuration

```rust
use ckb_sdk::{
    CkbRpcClient, HttpRpcClient,
    traits::{DefaultCellCollector, DefaultHeaderDepResolver, DefaultTransactionDependencyProvider},
    tx_builder::*,
    unlock::*,
    Address, AddressPayload, NetworkType,
};
use ckb_types::{
    core::{TransactionBuilder, TransactionView},
    packed::*,
    prelude::*,
    H256,
};

pub struct CkbSdkManager {
    rpc_client: HttpRpcClient,
    cell_collector: DefaultCellCollector,
    header_dep_resolver: DefaultHeaderDepResolver,
    tx_dep_provider: DefaultTransactionDependencyProvider,
    network_type: NetworkType,
}

impl CkbSdkManager {
    pub fn new(ckb_uri: &str, network_type: NetworkType) -> Self {
        let rpc_client = HttpRpcClient::new(ckb_uri.to_string());
        let cell_collector = DefaultCellCollector::new(ckb_uri);
        let header_dep_resolver = DefaultHeaderDepResolver::new(ckb_uri);
        let tx_dep_provider = DefaultTransactionDependencyProvider::new(ckb_uri, 10);

        Self {
            rpc_client,
            cell_collector,
            header_dep_resolver,
            tx_dep_provider,
            network_type,
        }
    }

    // Network configurations
    pub fn mainnet() -> Self {
        Self::new("https://mainnet.ckb.dev", NetworkType::Mainnet)
    }

    pub fn testnet() -> Self {
        Self::new("https://testnet.ckb.dev", NetworkType::Testnet)
    }

    pub fn devnet(port: u16) -> Self {
        let uri = format!("http://localhost:{}", port);
        Self::new(&uri, NetworkType::Dev)
    }
}
```

**Reference:** `resources/ckb-sdk-rust/examples/`

## Cell Collection and Management

```rust
use ckb_sdk::traits::{CellCollector, CellQueryOptions, LiveCell};

impl CkbSdkManager {
    // Collect cells by lock script
    pub async fn collect_cells_by_lock(
        &mut self,
        lock_script: Script,
        capacity_needed: Option<u64>,
    ) -> Result<Vec<LiveCell>, Box<dyn std::error::Error>> {
        let query = CellQueryOptions::new_lock(lock_script);

        if let Some(capacity) = capacity_needed {
            query.min_total_capacity(capacity);
        }

        let cells = self.cell_collector.collect_live_cells(&query, true)?;
        Ok(cells)
    }

    // Collect UDT cells
    pub async fn collect_udt_cells(
        &mut self,
        type_script: Script,
        owner_lock: Script,
        min_amount: Option<u128>,
    ) -> Result<Vec<(LiveCell, u128)>, Box<dyn std::error::Error>> {
        let mut query = CellQueryOptions::new_type(type_script);
        query.secondary_script(Some(owner_lock));

        let cells = self.cell_collector.collect_live_cells(&query, true)?;
        let mut udt_cells = Vec::new();

        for cell in cells {
            if cell.output_data.len() >= 16 {
                let amount_bytes: [u8; 16] = cell.output_data[0..16].try_into()?;
                let amount = u128::from_le_bytes(amount_bytes);

                if min_amount.map_or(true, |min| amount >= min) {
                    udt_cells.push((cell, amount));
                }
            }
        }

        Ok(udt_cells)
    }

    // Get cell details
    pub async fn get_cell_details(
        &self,
        out_point: &OutPoint,
    ) -> Result<Option<(CellOutput, Bytes)>, Box<dyn std::error::Error>> {
        let cell_with_status = self.rpc_client.get_live_cell(out_point.clone(), true)?;

        match cell_with_status.status {
            CellStatus::Live => {
                let cell = cell_with_status.cell.unwrap();
                Ok(Some((cell.output, cell.data.unwrap_or_default().content)))
            },
            _ => Ok(None),
        }
    }
}
```

## Advanced Transaction Building

```rust
use ckb_sdk::tx_builder::{CapacityBalancer, unlock::UnlockContext};

impl CkbSdkManager {
    // Build complex multi-input/output transaction
    pub async fn build_complex_transaction(
        &mut self,
        operations: Vec<TransactionOperation>,
        fee_rate: u64,
    ) -> Result<TransactionView, Box<dyn std::error::Error>> {
        let mut builder = CapacityBalancer::new_simple(
            &mut self.cell_collector,
            &self.header_dep_resolver,
            &self.tx_dep_provider,
        );

        let mut witness_generators = Vec::new();

        // Process each operation
        for operation in operations {
            match operation {
                TransactionOperation::Transfer { from, to, amount } => {
                    self.add_transfer_to_builder(&mut builder, from, to, amount)?;
                },
                TransactionOperation::UdtTransfer { udt_type, from, to, amount } => {
                    self.add_udt_transfer_to_builder(&mut builder, udt_type, from, to, amount)?;
                },
                TransactionOperation::DaoDeposit { from, amount } => {
                    self.add_dao_deposit_to_builder(&mut builder, from, amount)?;
                },
            }
        }

        // Generate witnesses
        let placeholder_witness = WitnessArgs::new_builder()
            .lock(Some(Bytes::from(vec![0u8; 65])).pack())
            .build();
        let mut witness_generator = PlaceholderWitnessGenerator::new(placeholder_witness);

        // Build transaction
        let balancer = builder.build(&mut witness_generator)?;
        let change_address = Address::new(self.network_type, AddressPayload::from_pubkey_hash(&[0u8; 20]));

        let tx = balancer.finalize(&change_address.payload(), fee_rate)?;

        Ok(tx)
    }

    // Add transfer operation to builder
    fn add_transfer_to_builder(
        &self,
        builder: &mut CapacityBalancer<DefaultCellCollector, DefaultHeaderDepResolver, DefaultTransactionDependencyProvider>,
        from: Address,
        to: Address,
        amount: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let to_lock_script = to.payload().into();

        builder.add_output_and_data(
            CellOutput::new_builder()
                .capacity(amount.pack())
                .lock(to_lock_script)
                .build(),
            Bytes::new(),
        );

        Ok(())
    }
}

pub enum TransactionOperation {
    Transfer {
        from: Address,
        to: Address,
        amount: u64,
    },
    UdtTransfer {
        udt_type: Script,
        from: Address,
        to: Address,
        amount: u128,
    },
    DaoDeposit {
        from: Address,
        amount: u64,
    },
}
```

## Multi-Signature and Omnilock Integration

```rust
use ckb_sdk::unlock::{MultisigConfig, OmnilockConfig, OmnilockUnlocker};

impl CkbSdkManager {
    // Create multisig configuration
    pub fn create_multisig_config(
        &self,
        public_keys: Vec<[u8; 33]>,
        threshold: u8,
    ) -> Result<MultisigConfig, Box<dyn std::error::Error>> {
        let config = MultisigConfig::new_with_pubkeys(public_keys, threshold)?;
        Ok(config)
    }

    // Create Omnilock configuration for Ethereum compatibility
    pub fn create_omnilock_ethereum_config(
        &self,
        ethereum_address: [u8; 20],
    ) -> Result<OmnilockConfig, Box<dyn std::error::Error>> {
        let config = OmnilockConfig::new_ethereum(ethereum_address);
        Ok(config)
    }

    // Sign transaction with Omnilock
    pub async fn sign_with_omnilock(
        &self,
        tx: &TransactionView,
        config: &OmnilockConfig,
        signature: [u8; 65], // Ethereum signature
    ) -> Result<TransactionView, Box<dyn std::error::Error>> {
        let unlocker = OmnilockUnlocker::new(config.clone());
        let signed_tx = unlocker.unlock_transaction(tx, signature)?;
        Ok(signed_tx)
    }

    // Build Anyone-Can-Pay transaction
    pub async fn build_acp_transaction(
        &mut self,
        acp_cell: OutPoint,
        contribution: u64,
        contributor_key: [u8; 32],
    ) -> Result<TransactionView, Box<dyn std::error::Error>> {
        let mut builder = TransactionBuilder::default();

        // Add ACP cell input
        builder = builder.input(CellInput::new(acp_cell, 0));

        // Get current cell state
        let (current_output, current_data) = self.get_cell_details(&acp_cell)
            .await?
            .ok_or("ACP cell not found")?;

        // Calculate new capacity
        let current_capacity = current_output.capacity().unpack();
        let new_capacity = current_capacity + contribution;

        // Create updated output
        builder = builder.output(
            CellOutput::new_builder()
                .capacity(new_capacity.pack())
                .lock(current_output.lock())
                .type_(current_output.type_())
                .build()
        );
        builder = builder.output_data(current_data.pack());

        // Add ACP witness
        let witness = WitnessArgs::new_builder()
            .lock(Some(Bytes::from(vec![0u8; 65])).pack())
            .build();
        builder = builder.witness(witness.as_bytes().pack());

        let tx = builder.build();

        // Sign with ACP unlocker
        let acp_unlocker = AnyoneCanPayUnlocker::new();
        let signed_tx = acp_unlocker.sign_transaction(&tx, &contributor_key)?;

        Ok(signed_tx)
    }
}
```

**Reference:** `resources/ckb-sdk-rust/examples/transfer_from_omnilock.rs`

## DAO (Nervos DAO) Operations

```rust
use ckb_sdk::constants::{DAO_TYPE_HASH, SIGHASH_TYPE_HASH};

impl CkbSdkManager {
    // Calculate DAO compensation
    pub async fn calculate_dao_compensation(
        &self,
        deposit_out_point: &OutPoint,
        withdraw_block_number: u64,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        // Get deposit transaction and block
        let deposit_tx_with_status = self.rpc_client.get_transaction(deposit_out_point.tx_hash())?;
        let deposit_tx = deposit_tx_with_status.transaction.unwrap();
        let deposit_block_hash = deposit_tx_with_status.tx_status.block_hash.unwrap();
        let deposit_header = self.rpc_client.get_header(deposit_block_hash)?;

        // Get withdraw block header
        let withdraw_block_hash = self.rpc_client.get_block_hash(withdraw_block_number)?;
        let withdraw_header = self.rpc_client.get_header(withdraw_block_hash)?;

        // Calculate compensation using DAO formula
        let deposit_capacity = deposit_tx.inner.outputs[deposit_out_point.index().unpack() as usize]
            .capacity().unpack();

        let deposit_ar = extract_dao_ar(&deposit_header.inner.dao);
        let withdraw_ar = extract_dao_ar(&withdraw_header.inner.dao);

        let compensation = deposit_capacity * withdraw_ar / deposit_ar;

        Ok(compensation)
    }

    // Complete DAO withdrawal (phase 2)
    pub async fn complete_dao_withdrawal(
        &mut self,
        withdraw_out_point: &OutPoint,
        to_address: &Address,
        private_key: [u8; 32],
    ) -> Result<TransactionView, Box<dyn std::error::Error>> {
        // Get withdrawal cell
        let (withdraw_output, withdraw_data) = self.get_cell_details(withdraw_out_point)
            .await?
            .ok_or("Withdrawal cell not found")?;

        // Parse withdrawal data to get deposit block number
        let deposit_block_number = u64::from_le_bytes(
            withdraw_data[0..8].try_into().unwrap()
        );
        let withdraw_block_number = u64::from_le_bytes(
            withdraw_data[8..16].try_into().unwrap()
        );

        // Calculate final compensation
        let compensation = self.calculate_dao_compensation(
            withdraw_out_point,
            withdraw_block_number,
        ).await?;

        let mut builder = TransactionBuilder::default();

        // Add withdrawal input
        builder = builder.input(CellInput::new(withdraw_out_point.clone(), 0));

        // Add compensation output
        builder = builder.output(
            CellOutput::new_builder()
                .capacity(compensation.pack())
                .lock(to_address.payload().into())
                .build()
        );
        builder = builder.output_data(Bytes::new().pack());

        // Add required header deps
        let deposit_block_hash = self.rpc_client.get_block_hash(deposit_block_number)?;
        let withdraw_block_hash = self.rpc_client.get_block_hash(withdraw_block_number)?;

        builder = builder.header_dep(deposit_block_hash);
        builder = builder.header_dep(withdraw_block_hash);

        // Add witness
        let witness = WitnessArgs::new_builder()
            .lock(Some(Bytes::from(vec![0u8; 65])).pack())
            .build();
        builder = builder.witness(witness.as_bytes().pack());

        let tx = builder.build();

        // Sign transaction
        let signer = SecpCkbRawKeySigner::new_with_secret_keys(vec![private_key]);
        let tx_with_groups = TransactionWithScriptGroups::get_script_groups(&tx)?;
        let signed_tx = signer.sign_transaction(&tx_with_groups)?;

        Ok(signed_tx)
    }
}

// Helper function to extract AR (accumulation rate) from DAO field
fn extract_dao_ar(dao: &Byte32) -> u64 {
    let dao_bytes = dao.as_slice();
    u64::from_le_bytes(dao_bytes[8..16].try_into().unwrap())
}
```

**Reference:** `resources/ckb-sdk-rust/examples/dao_operations.rs`

## Error Handling Patterns

```rust
#[derive(Debug, thiserror::Error)]
pub enum CkbSdkError {
    #[error("RPC error: {0}")]
    RpcError(#[from] ckb_jsonrpc_types::Error),

    #[error("Transaction building failed: {0}")]
    TransactionBuildError(String),

    #[error("Insufficient capacity: required {required}, available {available}")]
    InsufficientCapacity { required: u64, available: u64 },

    #[error("Cell not found: {0}")]
    CellNotFound(String),

    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    #[error("SMT error: {0}")]
    SmtError(String),
}

pub type CkbSdkResult<T> = Result<T, CkbSdkError>;
```

## Network Abstraction

```rust
pub trait NetworkProvider {
    fn get_tip_header(&self) -> CkbSdkResult<HeaderView>;
    fn get_transaction(&self, hash: &Byte32) -> CkbSdkResult<TransactionWithStatus>;
    fn send_transaction(&self, tx: &TransactionView) -> CkbSdkResult<Byte32>;
    fn get_live_cell(&self, out_point: &OutPoint) -> CkbSdkResult<CellWithStatus>;
}

pub struct CkbNetworkProvider {
    rpc_client: HttpRpcClient,
}

impl NetworkProvider for CkbNetworkProvider {
    fn get_tip_header(&self) -> CkbSdkResult<HeaderView> {
        Ok(self.rpc_client.get_tip_header()?)
    }

    // ... implement other methods
}
```

## Configuration Management

```rust
#[derive(Debug, Clone)]
pub struct CkbConfig {
    pub network_type: NetworkType,
    pub ckb_rpc_url: String,
    pub ckb_indexer_url: String,
    pub scripts: ScriptConfig,
    pub tx_pool_config: TxPoolConfig,
}

impl CkbConfig {
    pub fn mainnet() -> Self {
        Self {
            network_type: NetworkType::Mainnet,
            ckb_rpc_url: "https://mainnet.ckb.dev".to_string(),
            ckb_indexer_url: "https://mainnet.ckb.dev/indexer".to_string(),
            scripts: ScriptConfig::mainnet(),
            tx_pool_config: TxPoolConfig::default(),
        }
    }

    pub fn testnet() -> Self {
        Self {
            network_type: NetworkType::Testnet,
            ckb_rpc_url: "https://testnet.ckb.dev".to_string(),
            ckb_indexer_url: "https://testnet.ckb.dev/indexer".to_string(),
            scripts: ScriptConfig::testnet(),
            tx_pool_config: TxPoolConfig::default(),
        }
    }
}
```
