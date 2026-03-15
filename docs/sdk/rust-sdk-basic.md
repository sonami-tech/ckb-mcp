## Description

CKB Rust SDK transaction building fundamentals. Simple CKB transfers using CapacityTransferBuilder and SecpSighashUnlocker. UDT token transfers with UdtTransferBuilder. Complete code with RPC client setup, cell dependency resolution, capacity balancing, and witness generation. Based on ckb-sdk-rust v5.x implementations.

## Overview

All examples are based on actual CKB Rust SDK v5.x implementations. The SDK provides transaction builders (implementing the `TxBuilder` trait), script unlockers (implementing `ScriptUnlocker`), and utility functions for common operations.

## Simple CKB Transfer

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

## UDT Token Transfer

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
```

## Key Patterns

**Transaction Building**: Use `CapacityTransferBuilder` for simple CKB transfers, `UdtTransferBuilder` for token transfers.

**Cell Collection**: `DefaultCellCollector` handles finding and selecting cells automatically.

**Script Unlocking**: `SecpSighashUnlocker` for standard secp256k1 signatures.

**Fee Calculation**: Use `CapacityBalancer` with appropriate fee rates (typically 1000 shannons/KB).

## Related Documentation

- [Rust SDK Advanced Operations](ckb://docs/sdk/rust-sdk-advanced) - Multi-signature, Omnilock, and DAO operations
- [Lock Value Relationships](ckb://docs/concepts/lock-values) - Address and lock script generation from private keys
