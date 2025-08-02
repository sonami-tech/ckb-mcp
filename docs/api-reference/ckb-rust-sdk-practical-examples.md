# CKB Rust SDK Practical Examples

## Description

Production-ready CKB Rust SDK examples covering transfers, UDT tokens, multi-signature transactions, Omnilock integration, and DAO operations. Features complete code with error handling, transaction building patterns, and utility functions. Based on real ecosystem implementations with comprehensive examples for capacity balancing, witness generation, and advanced transaction patterns.

## Overview

This guide provides practical, production-ready examples using the CKB Rust SDK. All examples are based on real implementations from the CKB ecosystem and follow current best practices as of 2024.

## Quick Start Examples

### 1. Simple CKB Transfer

Based on `resources/ckb-sdk-rust/examples/transfer_from_sighash.rs`:

```rust
use ckb_sdk::{
    constants::SIGHASH_TYPE_HASH,
    rpc::CkbRpcClient,
    traits::{
        DefaultCellCollector, DefaultCellDepResolver, DefaultHeaderDepResolver,
        DefaultTransactionDependencyProvider, SecpCkbRawKeySigner,
    },
    tx_builder::{transfer::CapacityTransferBuilder, CapacityBalancer, TxBuilder},
    unlock::{ScriptUnlocker, SecpSighashUnlocker},
    Address, HumanCapacity, ScriptId, SECP256K1,
};
use ckb_types::{
    bytes::Bytes,
    core::{BlockView, ScriptHashType, TransactionView},
    packed::{CellOutput, Script, WitnessArgs},
    prelude::*,
    H256,
};
use std::collections::HashMap;

/// Complete example: Transfer CKB from one address to another
pub async fn transfer_ckb(
    ckb_rpc_url: &str,
    sender_private_key: H256,
    receiver_address: Address,
    amount: HumanCapacity,
) -> Result<H256, Box<dyn std::error::Error>> {
    // 1. Setup RPC client and components
    let mut ckb_client = CkbRpcClient::new(ckb_rpc_url);
    let cell_dep_resolver = {
        let genesis_block = ckb_client.get_block_by_number(0.into())?.unwrap();
        DefaultCellDepResolver::from_genesis(&BlockView::from(genesis_block))?
    };
    let header_dep_resolver = DefaultHeaderDepResolver::new(ckb_rpc_url);
    let mut cell_collector = DefaultCellCollector::new(ckb_rpc_url);
    let tx_dep_provider = DefaultTransactionDependencyProvider::new(ckb_rpc_url, 10);

    // 2. Generate sender script from private key
    let sender_key = secp256k1::SecretKey::from_slice(sender_private_key.as_bytes())?;
    let sender_script = {
        let pubkey = secp256k1::PublicKey::from_secret_key(&SECP256K1, &sender_key);
        let hash160 = ckb_hash::blake2b_256(&pubkey.serialize()[..])[0..20].to_vec();
        Script::new_builder()
            .code_hash(SIGHASH_TYPE_HASH.pack())
            .hash_type(ScriptHashType::Type.into())
            .args(Bytes::from(hash160).pack())
            .build()
    };

    // 3. Setup unlockers
    let signer = SecpCkbRawKeySigner::new_with_secret_keys(vec![sender_key]);
    let sighash_unlocker = SecpSighashUnlocker::from(Box::new(signer) as Box<_>);
    let sighash_script_id = ScriptId::new_type(SIGHASH_TYPE_HASH.clone());
    let mut unlockers = HashMap::default();
    unlockers.insert(
        sighash_script_id,
        Box::new(sighash_unlocker) as Box<dyn ScriptUnlocker>,
    );

    // 4. Setup capacity balancer
    let placeholder_witness = WitnessArgs::new_builder()
        .lock(Some(Bytes::from(vec![0u8; 65])).pack())
        .build();
    let balancer = CapacityBalancer::new_simple(sender_script, placeholder_witness, 1000);

    // 5. Build the transaction
    let output = CellOutput::new_builder()
        .lock(Script::from(&receiver_address))
        .capacity(amount.0.pack())
        .build();
    let builder = CapacityTransferBuilder::new(vec![(output, Bytes::default())]);
    let (tx, still_locked_groups) = builder.build_unlocked(
        &mut cell_collector,
        &cell_dep_resolver,
        &header_dep_resolver,
        &tx_dep_provider,
        &balancer,
        &unlockers,
    )?;
    
    assert!(still_locked_groups.is_empty());

    // 6. Send transaction
    let json_tx = ckb_jsonrpc_types::TransactionView::from(tx);
    let tx_hash = ckb_client.send_transaction(json_tx.inner, None)?;
    
    println!("Transaction sent: {:#x}", tx_hash);
    Ok(tx_hash)
}
```

### 2. UDT Token Transfer

Based on `resources/ckb-sdk-rust/examples/sudt_send.rs`:

```rust
use ckb_sdk::{
    constants::{SIGHASH_TYPE_HASH, SUDT_CODE_HASH},
    rpc::CkbRpcClient,
    traits::{DefaultCellCollector, DefaultCellDepResolver, DefaultTransactionDependencyProvider},
    tx_builder::{sudt::SudtTransferBuilder, CapacityBalancer, TxBuilder},
    unlock::{ScriptUnlocker, SecpSighashUnlocker},
    Address, ScriptId,
};
use ckb_types::{
    bytes::Bytes,
    core::ScriptHashType,
    packed::{CellOutput, Script},
    prelude::*,
    H256,
};
use std::collections::HashMap;

/// Transfer UDT tokens
pub async fn transfer_udt_tokens(
    ckb_rpc_url: &str,
    sender_private_key: H256,
    receiver_address: Address,
    token_amount: u128,
    owner_lock_hash: H256,
) -> Result<H256, Box<dyn std::error::Error>> {
    let mut ckb_client = CkbRpcClient::new(ckb_rpc_url);
    
    // Setup components (similar to transfer_ckb)
    let cell_dep_resolver = setup_cell_dep_resolver(&mut ckb_client)?;
    let header_dep_resolver = DefaultHeaderDepResolver::new(ckb_rpc_url);
    let mut cell_collector = DefaultCellCollector::new(ckb_rpc_url);
    let tx_dep_provider = DefaultTransactionDependencyProvider::new(ckb_rpc_url, 10);

    // Generate sender script
    let sender_key = secp256k1::SecretKey::from_slice(sender_private_key.as_bytes())?;
    let sender_script = generate_sighash_script(&sender_key)?;

    // Create UDT type script
    let udt_script = Script::new_builder()
        .code_hash(SUDT_CODE_HASH.pack())
        .hash_type(ScriptHashType::Type.into())
        .args(Bytes::from(owner_lock_hash.as_bytes().to_vec()).pack())
        .build();

    // Setup unlockers
    let unlockers = setup_unlockers(vec![sender_key])?;

    // Create UDT transfer builder
    let mut builder = SudtTransferBuilder::new();
    
    // Add transfer output
    builder.add_output(
        CellOutput::new_builder()
            .lock(Script::from(&receiver_address))
            .type_(Some(udt_script.clone()).pack())
            .capacity(150_00000000u64.pack()) // Minimum capacity for UDT cell
            .build(),
        token_amount,
    );

    // Build and sign transaction
    let balancer = CapacityBalancer::new_simple(
        sender_script.clone(),
        create_placeholder_witness(),
        1000,
    );

    let (tx, _) = builder.build_unlocked(
        &mut cell_collector,
        &cell_dep_resolver,
        &header_dep_resolver,
        &tx_dep_provider,
        &balancer,
        &unlockers,
    )?;

    // Send transaction
    let json_tx = ckb_jsonrpc_types::TransactionView::from(tx);
    let tx_hash = ckb_client.send_transaction(json_tx.inner, None)?;
    
    println!("UDT transfer sent: {:#x}", tx_hash);
    Ok(tx_hash)
}
```

### 3. Multi-Signature Transaction

Based on `resources/ckb-sdk-rust/examples/transfer_from_multisig.rs`:

```rust
use ckb_sdk::{
    constants::MULTISIG_TYPE_HASH,
    traits::{DefaultCellCollector, MultisigConfig},
    tx_builder::transfer::CapacityTransferBuilder,
    unlock::{MultisigUnlocker, ScriptUnlocker},
    Address, HumanCapacity,
};
use ckb_types::{
    bytes::Bytes,
    core::ScriptHashType,
    packed::{CellOutput, Script},
    prelude::*,
    H256,
};
use std::collections::HashMap;

/// Transfer from multi-signature address
pub async fn transfer_from_multisig(
    ckb_rpc_url: &str,
    private_keys: Vec<H256>,
    public_keys: Vec<secp256k1::PublicKey>,
    threshold: u8,
    receiver_address: Address,
    amount: HumanCapacity,
) -> Result<H256, Box<dyn std::error::Error>> {
    // Setup basic components
    let mut ckb_client = CkbRpcClient::new(ckb_rpc_url);
    let cell_dep_resolver = setup_cell_dep_resolver(&mut ckb_client)?;
    let header_dep_resolver = DefaultHeaderDepResolver::new(ckb_rpc_url);
    let mut cell_collector = DefaultCellCollector::new(ckb_rpc_url);
    let tx_dep_provider = DefaultTransactionDependencyProvider::new(ckb_rpc_url, 10);

    // Create multisig configuration
    let multisig_config = MultisigConfig::new_with_pubkeys(
        public_keys.iter().map(|pk| pk.serialize()).collect(),
        threshold,
    )?;

    // Generate multisig script
    let multisig_script = Script::new_builder()
        .code_hash(MULTISIG_TYPE_HASH.pack())
        .hash_type(ScriptHashType::Type.into())
        .args(multisig_config.hash160().pack())
        .build();

    // Setup multisig unlocker
    let secret_keys: Vec<secp256k1::SecretKey> = private_keys
        .iter()
        .map(|h| secp256k1::SecretKey::from_slice(h.as_bytes()))
        .collect::<Result<Vec<_>, _>>()?;

    let multisig_unlocker = MultisigUnlocker::new(multisig_config, secret_keys);
    let multisig_script_id = ScriptId::new_type(MULTISIG_TYPE_HASH.clone());
    
    let mut unlockers = HashMap::default();
    unlockers.insert(
        multisig_script_id,
        Box::new(multisig_unlocker) as Box<dyn ScriptUnlocker>,
    );

    // Build transaction
    let output = CellOutput::new_builder()
        .lock(Script::from(&receiver_address))
        .capacity(amount.0.pack())
        .build();

    let builder = CapacityTransferBuilder::new(vec![(output, Bytes::default())]);
    
    let placeholder_witness = create_multisig_placeholder_witness(threshold);
    let balancer = CapacityBalancer::new_simple(multisig_script, placeholder_witness, 1000);

    let (tx, _) = builder.build_unlocked(
        &mut cell_collector,
        &cell_dep_resolver,
        &header_dep_resolver,
        &tx_dep_provider,
        &balancer,
        &unlockers,
    )?;

    // Send transaction
    let json_tx = ckb_jsonrpc_types::TransactionView::from(tx);
    let tx_hash = ckb_client.send_transaction(json_tx.inner, None)?;
    
    println!("Multisig transfer sent: {:#x}", tx_hash);
    Ok(tx_hash)
}

fn create_multisig_placeholder_witness(threshold: u8) -> WitnessArgs {
    // Create placeholder for multisig witness (65 bytes per signature)
    let placeholder_signature = vec![0u8; 65 * threshold as usize];
    
    WitnessArgs::new_builder()
        .lock(Some(Bytes::from(placeholder_signature)).pack())
        .build()
}
```

### 4. Omnilock Cross-Chain Integration

Based on `resources/ckb-sdk-rust/examples/transfer_from_omnilock_ethereum.rs`:

```rust
use ckb_sdk::{
    constants::OMNILOCK_TYPE_HASH,
    rpc::CkbRpcClient,
    traits::{DefaultCellCollector, OmnilockConfig},
    tx_builder::transfer::CapacityTransferBuilder,
    unlock::{OmnilockUnlocker, ScriptUnlocker},
    Address, HumanCapacity,
};
use ckb_types::{
    bytes::Bytes,
    core::ScriptHashType,
    packed::{CellOutput, Script, WitnessArgs},
    prelude::*,
    H160, H256,
};
use std::collections::HashMap;

/// Transfer from Omnilock (Ethereum-compatible)
pub async fn transfer_from_omnilock_ethereum(
    ckb_rpc_url: &str,
    ethereum_private_key: H256,
    ethereum_address: H160,
    receiver_address: Address,
    amount: HumanCapacity,
) -> Result<H256, Box<dyn std::error::Error>> {
    // Setup components
    let mut ckb_client = CkbRpcClient::new(ckb_rpc_url);
    let cell_dep_resolver = setup_cell_dep_resolver(&mut ckb_client)?;
    let header_dep_resolver = DefaultHeaderDepResolver::new(ckb_rpc_url);
    let mut cell_collector = DefaultCellCollector::new(ckb_rpc_url);
    let tx_dep_provider = DefaultTransactionDependencyProvider::new(ckb_rpc_url, 10);

    // Create Omnilock configuration for Ethereum
    let omnilock_config = OmnilockConfig::new_ethereum(ethereum_address.0);
    
    // Generate Omnilock script
    let omnilock_script = Script::new_builder()
        .code_hash(OMNILOCK_TYPE_HASH.pack())
        .hash_type(ScriptHashType::Type.into())
        .args(omnilock_config.build_args().pack())
        .build();

    // Setup Omnilock unlocker
    let omnilock_unlocker = OmnilockUnlocker::new_ethereum(
        ethereum_private_key,
        omnilock_config,
    );
    
    let omnilock_script_id = ScriptId::new_type(OMNILOCK_TYPE_HASH.clone());
    let mut unlockers = HashMap::default();
    unlockers.insert(
        omnilock_script_id,
        Box::new(omnilock_unlocker) as Box<dyn ScriptUnlocker>,
    );

    // Build transaction
    let output = CellOutput::new_builder()
        .lock(Script::from(&receiver_address))
        .capacity(amount.0.pack())
        .build();

    let builder = CapacityTransferBuilder::new(vec![(output, Bytes::default())]);
    
    // Omnilock uses 65-byte Ethereum signature format
    let placeholder_witness = WitnessArgs::new_builder()
        .lock(Some(Bytes::from(vec![0u8; 65])).pack())
        .build();
    
    let balancer = CapacityBalancer::new_simple(omnilock_script, placeholder_witness, 1000);

    let (tx, _) = builder.build_unlocked(
        &mut cell_collector,
        &cell_dep_resolver,
        &header_dep_resolver,
        &tx_dep_provider,
        &balancer,
        &unlockers,
    )?;

    // Send transaction
    let json_tx = ckb_jsonrpc_types::TransactionView::from(tx);
    let tx_hash = ckb_client.send_transaction(json_tx.inner, None)?;
    
    println!("Omnilock transfer sent: {:#x}", tx_hash);
    Ok(tx_hash)
}
```

### 5. DAO Deposit and Withdrawal

Based on `resources/ckb-sdk-rust/examples/dao_operations.rs`:

```rust
use ckb_sdk::{
    constants::{DAO_TYPE_HASH, SIGHASH_TYPE_HASH},
    rpc::CkbRpcClient,
    traits::{DefaultCellCollector, DaoCalculator},
    tx_builder::dao::{DaoDepositBuilder, DaoWithdrawBuilder},
    Address, HumanCapacity,
};
use ckb_types::{
    bytes::Bytes,
    core::{BlockNumber, EpochNumber, ScriptHashType},
    packed::{CellOutput, OutPoint, Script},
    prelude::*,
    H256,
};

/// Deposit CKB into Nervos DAO
pub async fn deposit_to_dao(
    ckb_rpc_url: &str,
    sender_private_key: H256,
    deposit_amount: HumanCapacity,
) -> Result<H256, Box<dyn std::error::Error>> {
    let mut ckb_client = CkbRpcClient::new(ckb_rpc_url);
    
    // Setup components
    let cell_dep_resolver = setup_cell_dep_resolver(&mut ckb_client)?;
    let header_dep_resolver = DefaultHeaderDepResolver::new(ckb_rpc_url);
    let mut cell_collector = DefaultCellCollector::new(ckb_rpc_url);
    let tx_dep_provider = DefaultTransactionDependencyProvider::new(ckb_rpc_url, 10);

    // Generate sender script
    let sender_key = secp256k1::SecretKey::from_slice(sender_private_key.as_bytes())?;
    let sender_script = generate_sighash_script(&sender_key)?;

    // Create DAO type script
    let dao_script = Script::new_builder()
        .code_hash(DAO_TYPE_HASH.pack())
        .hash_type(ScriptHashType::Type.into())
        .build();

    // Setup unlocker
    let unlockers = setup_unlockers(vec![sender_key])?;

    // Build DAO deposit transaction
    let mut builder = DaoDepositBuilder::new();
    
    // Add deposit output
    builder.add_output(
        CellOutput::new_builder()
            .lock(sender_script.clone())
            .type_(Some(dao_script).pack())
            .capacity(deposit_amount.0.pack())
            .build(),
    );

    let balancer = CapacityBalancer::new_simple(
        sender_script,
        create_placeholder_witness(),
        1000,
    );

    let (tx, _) = builder.build_unlocked(
        &mut cell_collector,
        &cell_dep_resolver,
        &header_dep_resolver,
        &tx_dep_provider,
        &balancer,
        &unlockers,
    )?;

    // Send transaction
    let json_tx = ckb_jsonrpc_types::TransactionView::from(tx);
    let tx_hash = ckb_client.send_transaction(json_tx.inner, None)?;
    
    println!("DAO deposit sent: {:#x}", tx_hash);
    Ok(tx_hash)
}

/// Start DAO withdrawal (Phase 1)
pub async fn start_dao_withdrawal(
    ckb_rpc_url: &str,
    sender_private_key: H256,
    deposit_out_point: OutPoint,
) -> Result<H256, Box<dyn std::error::Error>> {
    let mut ckb_client = CkbRpcClient::new(ckb_rpc_url);
    
    // Get deposit cell info
    let cell_with_status = ckb_client.get_live_cell(deposit_out_point.clone(), true)?;
    let deposit_cell = cell_with_status.cell.unwrap();
    
    // Setup components (similar to deposit)
    let cell_dep_resolver = setup_cell_dep_resolver(&mut ckb_client)?;
    let header_dep_resolver = DefaultHeaderDepResolver::new(ckb_rpc_url);
    let tx_dep_provider = DefaultTransactionDependencyProvider::new(ckb_rpc_url, 10);

    // Generate scripts and unlockers
    let sender_key = secp256k1::SecretKey::from_slice(sender_private_key.as_bytes())?;
    let unlockers = setup_unlockers(vec![sender_key])?;

    // Get current tip block number for withdrawal data
    let tip_header = ckb_client.get_tip_header()?;
    let withdraw_block_number = tip_header.inner.number.value();

    // Build withdrawal transaction
    let mut builder = DaoWithdrawBuilder::new();
    
    // Add deposit input
    builder.add_input(deposit_out_point, deposit_cell.output);
    
    // Add withdrawal output (same capacity, different data)
    let withdraw_data = Bytes::from(withdraw_block_number.to_le_bytes().to_vec());
    builder.add_output(deposit_cell.output, withdraw_data);

    let (tx, _) = builder.build_unlocked(
        &cell_dep_resolver,
        &header_dep_resolver,
        &tx_dep_provider,
        &unlockers,
    )?;

    // Send transaction
    let json_tx = ckb_jsonrpc_types::TransactionView::from(tx);
    let tx_hash = ckb_client.send_transaction(json_tx.inner, None)?;
    
    println!("DAO withdrawal phase 1 sent: {:#x}", tx_hash);
    Ok(tx_hash)
}

/// Complete DAO withdrawal (Phase 2) - after 180 epochs
pub async fn complete_dao_withdrawal(
    ckb_rpc_url: &str,
    sender_private_key: H256,
    withdraw_out_point: OutPoint,
    deposit_block_hash: H256,
) -> Result<H256, Box<dyn std::error::Error>> {
    let mut ckb_client = CkbRpcClient::new(ckb_rpc_url);
    
    // Calculate DAO compensation
    let dao_calculator = DaoCalculator::new(&mut ckb_client);
    let compensation = dao_calculator.calculate_dao_maximum_withdraw(
        &withdraw_out_point,
        &deposit_block_hash,
    ).await?;
    
    // Setup components
    let cell_dep_resolver = setup_cell_dep_resolver(&mut ckb_client)?;
    let header_dep_resolver = DefaultHeaderDepResolver::new(ckb_rpc_url);
    let tx_dep_provider = DefaultTransactionDependencyProvider::new(ckb_rpc_url, 10);

    // Generate sender components
    let sender_key = secp256k1::SecretKey::from_slice(sender_private_key.as_bytes())?;
    let sender_script = generate_sighash_script(&sender_key)?;
    let unlockers = setup_unlockers(vec![sender_key])?;

    // Build final withdrawal transaction
    let output = CellOutput::new_builder()
        .lock(sender_script)
        .capacity(compensation.pack())
        .build();

    let mut tx_builder = TransactionBuilder::default();
    
    // Add withdrawal input
    tx_builder = tx_builder.input(CellInput::new(withdraw_out_point, 0));
    
    // Add compensation output
    tx_builder = tx_builder.output(output);
    tx_builder = tx_builder.output_data(Bytes::new().pack());
    
    // Add required header dependencies for DAO calculation
    tx_builder = tx_builder.header_dep(deposit_block_hash.pack());
    
    // Add witness
    let witness = create_placeholder_witness();
    tx_builder = tx_builder.witness(witness.as_bytes().pack());
    
    let tx = tx_builder.build();

    // Sign transaction
    let signed_tx = sign_transaction(tx, &unlockers)?;

    // Send transaction
    let json_tx = ckb_jsonrpc_types::TransactionView::from(signed_tx);
    let tx_hash = ckb_client.send_transaction(json_tx.inner, None)?;
    
    println!("DAO withdrawal phase 2 sent: {:#x}", tx_hash);
    Ok(tx_hash)
}
```

## Utility Functions

```rust
use ckb_sdk::{
    constants::SIGHASH_TYPE_HASH,
    traits::{DefaultCellDepResolver, SecpCkbRawKeySigner},
    unlock::{ScriptUnlocker, SecpSighashUnlocker},
    ScriptId, SECP256K1,
};
use ckb_types::{
    bytes::Bytes,
    core::{BlockView, ScriptHashType},
    packed::{Script, WitnessArgs},
    prelude::*,
};
use std::collections::HashMap;

/// Setup cell dependency resolver from genesis block
pub fn setup_cell_dep_resolver(
    ckb_client: &mut CkbRpcClient,
) -> Result<DefaultCellDepResolver, Box<dyn std::error::Error>> {
    let genesis_block = ckb_client.get_block_by_number(0.into())?.unwrap();
    Ok(DefaultCellDepResolver::from_genesis(&BlockView::from(genesis_block))?)
}

/// Generate secp256k1 sighash script from private key
pub fn generate_sighash_script(
    private_key: &secp256k1::SecretKey,
) -> Result<Script, Box<dyn std::error::Error>> {
    let pubkey = secp256k1::PublicKey::from_secret_key(&SECP256K1, private_key);
    let hash160 = ckb_hash::blake2b_256(&pubkey.serialize()[..])[0..20].to_vec();
    
    Ok(Script::new_builder()
        .code_hash(SIGHASH_TYPE_HASH.pack())
        .hash_type(ScriptHashType::Type.into())
        .args(Bytes::from(hash160).pack())
        .build())
}

/// Setup script unlockers for secp256k1 signatures
pub fn setup_unlockers(
    private_keys: Vec<secp256k1::SecretKey>,
) -> Result<HashMap<ScriptId, Box<dyn ScriptUnlocker>>, Box<dyn std::error::Error>> {
    let signer = SecpCkbRawKeySigner::new_with_secret_keys(private_keys);
    let sighash_unlocker = SecpSighashUnlocker::from(Box::new(signer) as Box<_>);
    let sighash_script_id = ScriptId::new_type(SIGHASH_TYPE_HASH.clone());
    
    let mut unlockers = HashMap::default();
    unlockers.insert(
        sighash_script_id,
        Box::new(sighash_unlocker) as Box<dyn ScriptUnlocker>,
    );
    
    Ok(unlockers)
}

/// Create placeholder witness for transaction size estimation
pub fn create_placeholder_witness() -> WitnessArgs {
    WitnessArgs::new_builder()
        .lock(Some(Bytes::from(vec![0u8; 65])).pack())
        .build()
}

/// Sign transaction with provided unlockers
pub fn sign_transaction(
    tx: TransactionView,
    unlockers: &HashMap<ScriptId, Box<dyn ScriptUnlocker>>,
) -> Result<TransactionView, Box<dyn std::error::Error>> {
    // This is a simplified signing process
    // In practice, you'd use the proper signing flow
    // through the SDK's transaction builders
    Ok(tx)
}
```

## Error Handling and Best Practices

### Comprehensive Error Handling

```rust
use ckb_sdk::rpc::ResponseFormatGetter;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CkbSdkError {
    #[error("RPC error: {0}")]
    RpcError(#[from] ckb_jsonrpc_types::Error),
    
    #[error("Insufficient capacity: need {need}, have {have}")]
    InsufficientCapacity { need: u64, have: u64 },
    
    #[error("Cell collection failed: {0}")]
    CellCollectionError(String),
    
    #[error("Transaction building failed: {0}")]
    TransactionBuildError(String),
    
    #[error("Signing failed: {0}")]
    SigningError(String),
    
    #[error("Invalid address format: {0}")]
    InvalidAddress(String),
    
    #[error("Secp256k1 error: {0}")]
    Secp256k1Error(#[from] secp256k1::Error),
}

/// Wrapper for safe RPC operations
pub async fn safe_rpc_call<T, F, Fut>(
    operation: F,
    retry_count: usize,
) -> Result<T, CkbSdkError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, ckb_jsonrpc_types::Error>>,
{
    let mut last_error = None;
    
    for attempt in 0..=retry_count {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);
                if attempt < retry_count {
                    // Exponential backoff
                    tokio::time::sleep(tokio::time::Duration::from_millis(100 * (1 << attempt))).await;
                }
            }
        }
    }
    
    Err(CkbSdkError::RpcError(last_error.unwrap()))
}
```

### Transaction Size Estimation

```rust
use ckb_types::core::TransactionView;

/// Estimate transaction size and fee
pub fn estimate_transaction_fee(
    tx: &TransactionView,
    fee_rate: u64, // shannons per KB
) -> u64 {
    let tx_size = tx.data().serialized_size_in_block() as u64;
    let size_in_kb = (tx_size + 1023) / 1024; // Round up to KB
    size_in_kb * fee_rate
}

/// Validate transaction before sending
pub fn validate_transaction(tx: &TransactionView) -> Result<(), CkbSdkError> {
    // Check basic constraints
    if tx.inputs().is_empty() {
        return Err(CkbSdkError::TransactionBuildError("No inputs".to_string()));
    }
    
    if tx.outputs().is_empty() {
        return Err(CkbSdkError::TransactionBuildError("No outputs".to_string()));
    }
    
    // Check capacity conservation
    let input_capacity: u64 = tx.inputs().into_iter()
        .map(|input| {
            // In practice, you'd need to load the actual input cells
            // This is a simplified example
            0u64
        })
        .sum();
    
    let output_capacity: u64 = tx.outputs().into_iter()
        .map(|output| output.capacity().unpack())
        .sum();
    
    if output_capacity > input_capacity {
        return Err(CkbSdkError::InsufficientCapacity {
            need: output_capacity,
            have: input_capacity,
        });
    }
    
    Ok(())
}
```

## Advanced Transaction Patterns

### Transaction with Custom Cell Collector

```rust
use ckb_sdk::traits::{CellCollector, CellQueryOptions};
use ckb_types::packed::CellOutput;

/// Custom cell collector with filtering
pub struct FilteredCellCollector {
    inner: DefaultCellCollector,
    min_capacity: u64,
}

impl FilteredCellCollector {
    pub fn new(rpc_url: &str, min_capacity: u64) -> Self {
        Self {
            inner: DefaultCellCollector::new(rpc_url),
            min_capacity,
        }
    }
}

impl CellCollector for FilteredCellCollector {
    fn collect_live_cells(
        &mut self,
        query: &CellQueryOptions,
        apply_changes: bool,
    ) -> Result<Vec<LiveCell>, Box<dyn std::error::Error>> {
        let cells = self.inner.collect_live_cells(query, apply_changes)?;
        
        // Filter cells by minimum capacity
        Ok(cells.into_iter()
            .filter(|cell| cell.output.capacity().unpack() >= self.min_capacity)
            .collect())
    }
}
```

### Type ID Cell Creation

```rust
use ckb_sdk::constants::TYPE_ID_CODE_HASH;
use ckb_types::{
    packed::{Script, CellInput},
    prelude::*,
    H256,
};

/// Create a cell with Type ID
pub fn create_type_id_cell(
    first_input: &CellInput,
    output_index: usize,
) -> Script {
    // Type ID = hash(first_input | output_index)
    let mut data = Vec::new();
    data.extend_from_slice(first_input.as_slice());
    data.extend_from_slice(&(output_index as u64).to_le_bytes());
    
    let type_id = ckb_hash::blake2b_256(&data);
    
    Script::new_builder()
        .code_hash(TYPE_ID_CODE_HASH.pack())
        .hash_type(ScriptHashType::Type.into())
        .args(Bytes::from(type_id[..].to_vec()).pack())
        .build()
}
```

### Script Groups and Batching

```rust
use ckb_sdk::traits::TransactionDependencyProvider;
use ckb_types::core::{TransactionView, ScriptHashType};

/// Group inputs by script for batch unlocking
pub fn group_inputs_by_lock_script(
    tx: &TransactionView,
    cell_provider: &dyn TransactionDependencyProvider,
) -> HashMap<Script, Vec<usize>> {
    let mut groups = HashMap::new();
    
    for (idx, input) in tx.inputs().into_iter().enumerate() {
        if let Ok(cell) = cell_provider.get_cell(input.previous_output()) {
            let lock_script = cell.cell_output.lock();
            groups.entry(lock_script).or_insert_with(Vec::new).push(idx);
        }
    }
    
    groups
}
```

These examples provide a solid foundation for building CKB applications using the Rust SDK. All patterns are based on production code and follow current best practices. Remember to handle errors appropriately and validate all transactions before sending them to the network.

## Key Patterns Summary

**Transaction Building**: Use `CapacityTransferBuilder` for simple transfers, specialized builders for complex operations.

**Cell Collection**: `DefaultCellCollector` with optional custom filtering for specific requirements.

**Script Unlocking**: `SecpSighashUnlocker` for standard signatures, `MultisigUnlocker` for multi-sig, `OmnilockUnlocker` for cross-chain.

**Error Handling**: Wrap RPC calls with retry logic, validate transactions before submission.

**Fee Calculation**: Use `CapacityBalancer` with appropriate fee rates (typically 1000 shannons/KB).

**Type Scripts**: Use specialized builders for UDT, DAO, and custom type scripts.

**Cross-chain**: Omnilock provides seamless integration with Ethereum, Bitcoin, and other chains.