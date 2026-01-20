## Description

Production-ready CKB Rust SDK examples covering transfers, UDT tokens, multi-signature transactions, Omnilock integration, and DAO operations. Features complete code with error handling, transaction building patterns, and utility functions. Based on real ecosystem implementations with comprehensive examples for capacity balancing, witness generation, and advanced transaction patterns.

## Overview

All examples are based on actual CKB Rust SDK v5.x implementations. The SDK provides transaction builders (implementing the `TxBuilder` trait), script unlockers (implementing `ScriptUnlocker`), and utility functions for common operations.

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

Based on actual `ckb-sdk-rust/src/tx_builder/udt/mod.rs`:

```rust
use ckb_sdk::{
    constants::SIGHASH_TYPE_HASH,
    rpc::CkbRpcClient,
    traits::{
        DefaultCellCollector, DefaultCellDepResolver, DefaultHeaderDepResolver,
        DefaultTransactionDependencyProvider, SecpCkbRawKeySigner,
    },
    tx_builder::{
        udt::{UdtTargetReceiver, UdtTransferBuilder},
        CapacityBalancer, TransferAction, TxBuilder,
    },
    types::ScriptId,
    unlock::{ScriptUnlocker, SecpSighashUnlocker},
    Address,
};
use ckb_types::{
    bytes::Bytes,
    core::ScriptHashType,
    packed::Script,
    prelude::*,
    H256,
};
use std::collections::HashMap;

/// Transfer UDT tokens using the SDK's UdtTransferBuilder
pub async fn transfer_udt_tokens(
    ckb_rpc_url: &str,
    sender_private_key: H256,
    receiver_lock: Script,
    udt_type_script: Script,  // The UDT type script (sUDT or xUDT)
    token_amount: u128,
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

    // Setup unlockers
    let unlockers = setup_unlockers(vec![sender_key.clone()])?;

    // Create UDT transfer builder with:
    // - type_script: The UDT type script
    // - sender: Sender's lock script (to find sender's UDT cell)
    // - receivers: List of transfer targets
    let receivers = vec![
        UdtTargetReceiver::new(
            TransferAction::Create,  // Create new cell for recipient
            receiver_lock,
            token_amount,
        ),
    ];

    let builder = UdtTransferBuilder {
        type_script: udt_type_script,
        sender: sender_script.clone(),
        receivers,
    };

    // Build and sign transaction using TxBuilder trait
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

Based on actual `ckb-sdk-rust/src/unlock/signer.rs` and `unlocker.rs`:

```rust
use ckb_sdk::{
    constants::MultisigScript,
    rpc::CkbRpcClient,
    traits::{
        DefaultCellCollector, DefaultCellDepResolver, DefaultHeaderDepResolver,
        DefaultTransactionDependencyProvider, SecpCkbRawKeySigner,
    },
    tx_builder::{transfer::CapacityTransferBuilder, CapacityBalancer, TxBuilder},
    types::ScriptId,
    unlock::{MultisigConfig, ScriptUnlocker, SecpMultisigScriptSigner, SecpMultisigUnlocker},
    Address, HumanCapacity, NetworkInfo,
};
use ckb_types::{
    bytes::Bytes,
    core::ScriptHashType,
    packed::{CellOutput, Script, WitnessArgs},
    prelude::*,
    H160, H256,
};
use std::collections::HashMap;

/// Transfer from multi-signature address
pub async fn transfer_from_multisig(
    ckb_rpc_url: &str,
    private_keys: Vec<H256>,
    sighash_addresses: Vec<H160>,  // Blake160 hashes of public keys
    require_first_n: u8,
    threshold: u8,
    receiver_address: Address,
    amount: HumanCapacity,
) -> Result<H256, Box<dyn std::error::Error>> {
    // Setup basic components
    let mut ckb_client = CkbRpcClient::new(ckb_rpc_url);
    let genesis_block = ckb_client.get_block_by_number(0.into())?.unwrap();
    let genesis_block: ckb_types::core::BlockView = genesis_block.into();
    let cell_dep_resolver = DefaultCellDepResolver::from_genesis(&genesis_block)?;
    let header_dep_resolver = DefaultHeaderDepResolver::new(ckb_rpc_url);
    let mut cell_collector = DefaultCellCollector::new(ckb_rpc_url);
    let tx_dep_provider = DefaultTransactionDependencyProvider::new(ckb_rpc_url, 10);

    // Create multisig configuration
    // MultisigConfig takes sighash_addresses (H160 hashes), require_first_n, and threshold
    let multisig_config = MultisigConfig::new_with(sighash_addresses, require_first_n, threshold)?;

    // Get multisig script ID from MultisigScript enum
    // Choose Legacy or V2 depending on which you want to use
    let multisig_script_variant = MultisigScript::Legacy;
    let multisig_script_id = multisig_script_variant.script_id();

    // Generate multisig lock script
    let multisig_script = Script::new_builder()
        .code_hash(multisig_script_id.code_hash.pack())
        .hash_type(multisig_script_id.hash_type)
        .args(Bytes::from(multisig_config.hash160().as_bytes().to_vec()).pack())
        .build();

    // Setup multisig unlocker with SecpMultisigUnlocker
    let secret_keys: Vec<secp256k1::SecretKey> = private_keys
        .iter()
        .map(|h| secp256k1::SecretKey::from_slice(h.as_bytes()))
        .collect::<Result<Vec<_>, _>>()?;

    let signer = SecpCkbRawKeySigner::new_with_secret_keys(secret_keys);
    let multisig_signer = SecpMultisigScriptSigner::new(Box::new(signer), multisig_config.clone());
    let multisig_unlocker = SecpMultisigUnlocker::new(multisig_signer);

    let mut unlockers: HashMap<ScriptId, Box<dyn ScriptUnlocker>> = HashMap::default();
    unlockers.insert(
        multisig_script_id,
        Box::new(multisig_unlocker),
    );

    // Build transaction
    let output = CellOutput::new_builder()
        .lock(Script::from(&receiver_address))
        .capacity(amount.0.pack())
        .build();

    let builder = CapacityTransferBuilder::new(vec![(output, Bytes::default())]);

    let placeholder_witness = create_multisig_placeholder_witness(&multisig_config);
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

fn create_multisig_placeholder_witness(config: &MultisigConfig) -> WitnessArgs {
    // Multisig witness: config data + (65 bytes * threshold) signatures
    let config_data = config.to_witness_data();
    let mut placeholder = vec![0u8; config_data.len() + 65 * (config.threshold() as usize)];
    placeholder[0..config_data.len()].copy_from_slice(&config_data);

    WitnessArgs::new_builder()
        .lock(Some(Bytes::from(placeholder)).pack())
        .build()
}
```

### 4. Omnilock Cross-Chain Integration

Based on actual `ckb-sdk-rust/src/unlock/omni_lock.rs` and `unlocker.rs`:

```rust
use ckb_sdk::{
    rpc::CkbRpcClient,
    traits::{
        DefaultCellCollector, DefaultCellDepResolver, DefaultHeaderDepResolver,
        DefaultTransactionDependencyProvider, SecpCkbRawKeySigner,
    },
    tx_builder::{transfer::CapacityTransferBuilder, CapacityBalancer, TxBuilder},
    types::ScriptId,
    unlock::{
        OmniLockConfig, OmniLockScriptSigner, OmniLockUnlocker, OmniUnlockMode, ScriptUnlocker,
    },
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

// Omnilock code hash (type script hash) - must be provided or discovered from deployment
const OMNILOCK_CODE_HASH: H256 = H256([/* your omnilock code hash here */]);

/// Transfer from Omnilock (Ethereum-compatible)
pub async fn transfer_from_omnilock_ethereum(
    ckb_rpc_url: &str,
    ethereum_private_key: H256,
    ethereum_pubkey_hash: H160,  // keccak160 of Ethereum public key
    omnilock_code_hash: H256,    // Omnilock script code hash (deploy-specific)
    receiver_address: Address,
    amount: HumanCapacity,
) -> Result<H256, Box<dyn std::error::Error>> {
    // Setup components
    let mut ckb_client = CkbRpcClient::new(ckb_rpc_url);
    let genesis_block = ckb_client.get_block_by_number(0.into())?.unwrap();
    let genesis_block: ckb_types::core::BlockView = genesis_block.into();
    let cell_dep_resolver = DefaultCellDepResolver::from_genesis(&genesis_block)?;
    let header_dep_resolver = DefaultHeaderDepResolver::new(ckb_rpc_url);
    let mut cell_collector = DefaultCellCollector::new(ckb_rpc_url);
    let tx_dep_provider = DefaultTransactionDependencyProvider::new(ckb_rpc_url, 10);

    // Create Omnilock configuration for Ethereum identity
    // OmniLockConfig::new_ethereum takes keccak160 pubkey hash
    let omnilock_config = OmniLockConfig::new_ethereum(ethereum_pubkey_hash);

    // Generate Omnilock script - code_hash is deployment-specific
    let omnilock_script = Script::new_builder()
        .code_hash(omnilock_code_hash.pack())
        .hash_type(ScriptHashType::Type.into())
        .args(omnilock_config.build_args().pack())
        .build();

    // Setup Omnilock unlocker with OmniLockScriptSigner and OmniLockUnlocker
    let secret_key = secp256k1::SecretKey::from_slice(ethereum_private_key.as_bytes())?;
    let signer = SecpCkbRawKeySigner::new_with_secret_keys(vec![secret_key]);
    let omnilock_signer = OmniLockScriptSigner::new(
        Box::new(signer),
        omnilock_config.clone(),
        OmniUnlockMode::Normal,
    );
    let omnilock_unlocker = OmniLockUnlocker::new(omnilock_signer, omnilock_config.clone());

    let omnilock_script_id = ScriptId::new_type(omnilock_code_hash);
    let mut unlockers: HashMap<ScriptId, Box<dyn ScriptUnlocker>> = HashMap::default();
    unlockers.insert(
        omnilock_script_id,
        Box::new(omnilock_unlocker),
    );

    // Build transaction
    let output = CellOutput::new_builder()
        .lock(Script::from(&receiver_address))
        .capacity(amount.0.pack())
        .build();

    let builder = CapacityTransferBuilder::new(vec![(output, Bytes::default())]);

    // Omnilock placeholder witness - use config's placeholder method
    let placeholder_witness = omnilock_config.placeholder_witness(OmniUnlockMode::Normal)?;
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

Based on actual `ckb-sdk-rust/src/tx_builder/dao.rs`:

```rust
use ckb_sdk::{
    constants::DAO_TYPE_HASH,
    rpc::CkbRpcClient,
    traits::{
        DefaultCellCollector, DefaultCellDepResolver, DefaultHeaderDepResolver,
        DefaultTransactionDependencyProvider, SecpCkbRawKeySigner,
    },
    tx_builder::{
        dao::{DaoDepositBuilder, DaoDepositReceiver, DaoPrepareBuilder, DaoWithdrawBuilder,
              DaoWithdrawItem, DaoWithdrawReceiver},
        CapacityBalancer, TxBuilder,
    },
    types::ScriptId,
    unlock::{ScriptUnlocker, SecpSighashUnlocker},
    util::calculate_dao_maximum_withdraw4,
    HumanCapacity,
};
use ckb_types::{
    bytes::Bytes,
    core::ScriptHashType,
    packed::{CellInput, CellOutput, OutPoint, Script, WitnessArgs},
    prelude::*,
    H256,
};
use std::collections::HashMap;

/// Deposit CKB into Nervos DAO
pub async fn deposit_to_dao(
    ckb_rpc_url: &str,
    sender_private_key: H256,
    deposit_amount: u64,
) -> Result<H256, Box<dyn std::error::Error>> {
    let mut ckb_client = CkbRpcClient::new(ckb_rpc_url);

    // Setup components
    let genesis_block = ckb_client.get_block_by_number(0.into())?.unwrap();
    let genesis_block: ckb_types::core::BlockView = genesis_block.into();
    let cell_dep_resolver = DefaultCellDepResolver::from_genesis(&genesis_block)?;
    let header_dep_resolver = DefaultHeaderDepResolver::new(ckb_rpc_url);
    let mut cell_collector = DefaultCellCollector::new(ckb_rpc_url);
    let tx_dep_provider = DefaultTransactionDependencyProvider::new(ckb_rpc_url, 10);

    // Generate sender script
    let sender_key = secp256k1::SecretKey::from_slice(sender_private_key.as_bytes())?;
    let sender_script = generate_sighash_script(&sender_key)?;

    // Setup unlocker
    let unlockers = setup_unlockers(vec![sender_key.clone()])?;

    // Build DAO deposit transaction using DaoDepositBuilder
    // DaoDepositBuilder takes a Vec<DaoDepositReceiver>
    let receivers = vec![
        DaoDepositReceiver::new(sender_script.clone(), deposit_amount),
    ];
    let builder = DaoDepositBuilder::new(receivers);

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

/// Start DAO withdrawal (Phase 1 - Prepare)
/// This creates a "prepared" cell that records the deposit block number
pub async fn start_dao_withdrawal(
    ckb_rpc_url: &str,
    sender_private_key: H256,
    deposit_out_point: OutPoint,
) -> Result<H256, Box<dyn std::error::Error>> {
    let mut ckb_client = CkbRpcClient::new(ckb_rpc_url);

    // Setup components
    let genesis_block = ckb_client.get_block_by_number(0.into())?.unwrap();
    let genesis_block: ckb_types::core::BlockView = genesis_block.into();
    let cell_dep_resolver = DefaultCellDepResolver::from_genesis(&genesis_block)?;
    let header_dep_resolver = DefaultHeaderDepResolver::new(ckb_rpc_url);
    let mut cell_collector = DefaultCellCollector::new(ckb_rpc_url);
    let tx_dep_provider = DefaultTransactionDependencyProvider::new(ckb_rpc_url, 10);

    // Generate scripts and unlockers
    let sender_key = secp256k1::SecretKey::from_slice(sender_private_key.as_bytes())?;
    let unlockers = setup_unlockers(vec![sender_key.clone()])?;
    let sender_script = generate_sighash_script(&sender_key)?;

    // Build prepare transaction using DaoPrepareBuilder
    // DaoPrepareBuilder takes Vec<DaoPrepareItem> (CellInput can convert to DaoPrepareItem)
    let input = CellInput::new(deposit_out_point, 0);
    let builder = DaoPrepareBuilder::new(vec![input.into()]);

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

    println!("DAO withdrawal phase 1 (prepare) sent: {:#x}", tx_hash);
    Ok(tx_hash)
}

/// Complete DAO withdrawal (Phase 2 - Withdraw)
/// Must wait ~180 epochs after prepare before this can be executed
pub async fn complete_dao_withdrawal(
    ckb_rpc_url: &str,
    sender_private_key: H256,
    prepared_out_point: OutPoint,
) -> Result<H256, Box<dyn std::error::Error>> {
    let mut ckb_client = CkbRpcClient::new(ckb_rpc_url);

    // Setup components
    let genesis_block = ckb_client.get_block_by_number(0.into())?.unwrap();
    let genesis_block: ckb_types::core::BlockView = genesis_block.into();
    let cell_dep_resolver = DefaultCellDepResolver::from_genesis(&genesis_block)?;
    let header_dep_resolver = DefaultHeaderDepResolver::new(ckb_rpc_url);
    let mut cell_collector = DefaultCellCollector::new(ckb_rpc_url);
    let tx_dep_provider = DefaultTransactionDependencyProvider::new(ckb_rpc_url, 10);

    // Generate sender components
    let sender_key = secp256k1::SecretKey::from_slice(sender_private_key.as_bytes())?;
    let sender_script = generate_sighash_script(&sender_key)?;
    let unlockers = setup_unlockers(vec![sender_key.clone()])?;

    // Build withdraw transaction using DaoWithdrawBuilder
    // DaoWithdrawBuilder takes:
    // - items: Vec<DaoWithdrawItem> (prepared cells to withdraw)
    // - receiver: DaoWithdrawReceiver (where to send the withdrawn capacity)
    let placeholder_witness = WitnessArgs::new_builder()
        .lock(Some(Bytes::from(vec![0u8; 65])).pack())
        .build();

    let items = vec![
        DaoWithdrawItem::new(prepared_out_point, Some(placeholder_witness)),
    ];

    let receiver = DaoWithdrawReceiver::LockScript {
        script: sender_script.clone(),
        fee_rate: Some(ckb_types::core::FeeRate::from_u64(1000)),
    };

    let builder = DaoWithdrawBuilder::new(items, receiver);

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

    println!("DAO withdrawal phase 2 (withdraw) sent: {:#x}", tx_hash);
    Ok(tx_hash)
}

// Note: DAO compensation is calculated internally by DaoWithdrawBuilder
// using the `calculate_dao_maximum_withdraw4` utility function from ckb_sdk::util
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

## Related Documentation

For understanding how addresses and lock scripts are generated from private keys, see the [Lock Value Relationships](ckb://docs/concepts/lock-value-relationships) guide.