# CKB Rust SDK Advanced Operations

## Description

Advanced CKB Rust SDK patterns for multi-signature transactions, Omnilock cross-chain integration, and Nervos DAO operations. MultisigConfig and SecpMultisigUnlocker for m-of-n signatures. OmniLockConfig for Ethereum-compatible addresses. DaoDepositBuilder, DaoPrepareBuilder, and DaoWithdrawBuilder for DAO lifecycle. Error handling and transaction validation patterns.

## Multi-Signature Transaction

Based on `ckb-sdk-rust/src/unlock/signer.rs` and `unlocker.rs`:

```rust
use ckb_sdk::{
    constants::MultisigScript,
    rpc::CkbRpcClient,
    traits::{DefaultCellCollector, DefaultCellDepResolver, DefaultHeaderDepResolver,
             DefaultTransactionDependencyProvider, SecpCkbRawKeySigner},
    tx_builder::{transfer::CapacityTransferBuilder, CapacityBalancer, TxBuilder},
    types::ScriptId,
    unlock::{MultisigConfig, ScriptUnlocker, SecpMultisigScriptSigner, SecpMultisigUnlocker},
    Address, HumanCapacity,
};
use ckb_types::{bytes::Bytes, core::ScriptHashType, packed::{CellOutput, Script, WitnessArgs},
                prelude::*, H160, H256};
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
    let mut ckb_client = CkbRpcClient::new(ckb_rpc_url);
    let genesis_block: ckb_types::core::BlockView =
        ckb_client.get_block_by_number(0.into())?.unwrap().into();
    let cell_dep_resolver = DefaultCellDepResolver::from_genesis(&genesis_block)?;
    let header_dep_resolver = DefaultHeaderDepResolver::new(ckb_rpc_url);
    let mut cell_collector = DefaultCellCollector::new(ckb_rpc_url);
    let tx_dep_provider = DefaultTransactionDependencyProvider::new(ckb_rpc_url, 10);

    // Create multisig configuration
    let multisig_config = MultisigConfig::new_with(sighash_addresses, require_first_n, threshold)?;
    let multisig_script_variant = MultisigScript::Legacy;
    let multisig_script_id = multisig_script_variant.script_id();

    // Generate multisig lock script
    let multisig_script = Script::new_builder()
        .code_hash(multisig_script_id.code_hash.pack())
        .hash_type(multisig_script_id.hash_type)
        .args(Bytes::from(multisig_config.hash160().as_bytes().to_vec()).pack())
        .build();

    // Setup multisig unlocker
    let secret_keys: Vec<secp256k1::SecretKey> = private_keys.iter()
        .map(|h| secp256k1::SecretKey::from_slice(h.as_bytes()))
        .collect::<Result<Vec<_>, _>>()?;
    let signer = SecpCkbRawKeySigner::new_with_secret_keys(secret_keys);
    let multisig_signer = SecpMultisigScriptSigner::new(Box::new(signer), multisig_config.clone());
    let multisig_unlocker = SecpMultisigUnlocker::new(multisig_signer);

    let mut unlockers: HashMap<ScriptId, Box<dyn ScriptUnlocker>> = HashMap::default();
    unlockers.insert(multisig_script_id, Box::new(multisig_unlocker));

    // Build transaction
    let output = CellOutput::new_builder()
        .lock(Script::from(&receiver_address))
        .capacity(amount.0.pack())
        .build();
    let builder = CapacityTransferBuilder::new(vec![(output, Bytes::default())]);

    // Multisig witness: config data + (65 bytes * threshold) signatures
    let config_data = multisig_config.to_witness_data();
    let mut placeholder = vec![0u8; config_data.len() + 65 * (multisig_config.threshold() as usize)];
    placeholder[0..config_data.len()].copy_from_slice(&config_data);
    let placeholder_witness = WitnessArgs::new_builder()
        .lock(Some(Bytes::from(placeholder)).pack())
        .build();

    let balancer = CapacityBalancer::new_simple(multisig_script, placeholder_witness, 1000);
    let (tx, _) = builder.build_unlocked(&mut cell_collector, &cell_dep_resolver,
        &header_dep_resolver, &tx_dep_provider, &balancer, &unlockers)?;

    let json_tx = ckb_jsonrpc_types::TransactionView::from(tx);
    let tx_hash = ckb_client.send_transaction(json_tx.inner, None)?;
    Ok(tx_hash)
}
```

## Omnilock Cross-Chain Integration

Based on `ckb-sdk-rust/src/unlock/omni_lock.rs`:

```rust
use ckb_sdk::{
    rpc::CkbRpcClient,
    traits::{DefaultCellCollector, DefaultCellDepResolver, DefaultHeaderDepResolver,
             DefaultTransactionDependencyProvider, SecpCkbRawKeySigner},
    tx_builder::{transfer::CapacityTransferBuilder, CapacityBalancer, TxBuilder},
    types::ScriptId,
    unlock::{OmniLockConfig, OmniLockScriptSigner, OmniLockUnlocker, OmniUnlockMode, ScriptUnlocker},
    Address, HumanCapacity,
};
use ckb_types::{bytes::Bytes, core::ScriptHashType, packed::{CellOutput, Script}, prelude::*, H160, H256};
use std::collections::HashMap;

/// Transfer from Omnilock (Ethereum-compatible)
pub async fn transfer_from_omnilock_ethereum(
    ckb_rpc_url: &str,
    ethereum_private_key: H256,
    ethereum_pubkey_hash: H160,  // keccak160 of Ethereum public key
    omnilock_code_hash: H256,    // Omnilock script code hash (deploy-specific)
    receiver_address: Address,
    amount: HumanCapacity,
) -> Result<H256, Box<dyn std::error::Error>> {
    let mut ckb_client = CkbRpcClient::new(ckb_rpc_url);
    let genesis_block: ckb_types::core::BlockView =
        ckb_client.get_block_by_number(0.into())?.unwrap().into();
    let cell_dep_resolver = DefaultCellDepResolver::from_genesis(&genesis_block)?;
    let header_dep_resolver = DefaultHeaderDepResolver::new(ckb_rpc_url);
    let mut cell_collector = DefaultCellCollector::new(ckb_rpc_url);
    let tx_dep_provider = DefaultTransactionDependencyProvider::new(ckb_rpc_url, 10);

    // Create Omnilock configuration for Ethereum identity
    let omnilock_config = OmniLockConfig::new_ethereum(ethereum_pubkey_hash);
    let omnilock_script = Script::new_builder()
        .code_hash(omnilock_code_hash.pack())
        .hash_type(ScriptHashType::Type.into())
        .args(omnilock_config.build_args().pack())
        .build();

    // Setup Omnilock unlocker
    let secret_key = secp256k1::SecretKey::from_slice(ethereum_private_key.as_bytes())?;
    let signer = SecpCkbRawKeySigner::new_with_secret_keys(vec![secret_key]);
    let omnilock_signer = OmniLockScriptSigner::new(
        Box::new(signer), omnilock_config.clone(), OmniUnlockMode::Normal);
    let omnilock_unlocker = OmniLockUnlocker::new(omnilock_signer, omnilock_config.clone());

    let omnilock_script_id = ScriptId::new_type(omnilock_code_hash);
    let mut unlockers: HashMap<ScriptId, Box<dyn ScriptUnlocker>> = HashMap::default();
    unlockers.insert(omnilock_script_id, Box::new(omnilock_unlocker));

    // Build transaction
    let output = CellOutput::new_builder()
        .lock(Script::from(&receiver_address))
        .capacity(amount.0.pack())
        .build();
    let builder = CapacityTransferBuilder::new(vec![(output, Bytes::default())]);

    let placeholder_witness = omnilock_config.placeholder_witness(OmniUnlockMode::Normal)?;
    let balancer = CapacityBalancer::new_simple(omnilock_script, placeholder_witness, 1000);
    let (tx, _) = builder.build_unlocked(&mut cell_collector, &cell_dep_resolver,
        &header_dep_resolver, &tx_dep_provider, &balancer, &unlockers)?;

    let json_tx = ckb_jsonrpc_types::TransactionView::from(tx);
    let tx_hash = ckb_client.send_transaction(json_tx.inner, None)?;
    Ok(tx_hash)
}
```

## DAO Deposit and Withdrawal

Based on `ckb-sdk-rust/src/tx_builder/dao.rs`:

```rust
use ckb_sdk::{
    rpc::CkbRpcClient,
    traits::{DefaultCellCollector, DefaultCellDepResolver, DefaultHeaderDepResolver,
             DefaultTransactionDependencyProvider, SecpCkbRawKeySigner},
    tx_builder::{dao::{DaoDepositBuilder, DaoDepositReceiver, DaoPrepareBuilder,
                       DaoWithdrawBuilder, DaoWithdrawItem, DaoWithdrawReceiver},
                 CapacityBalancer, TxBuilder},
    unlock::{ScriptUnlocker, SecpSighashUnlocker},
};
use ckb_types::{bytes::Bytes, packed::{CellInput, OutPoint, Script, WitnessArgs}, prelude::*, H256};

/// Deposit CKB into Nervos DAO
pub async fn deposit_to_dao(
    ckb_rpc_url: &str,
    sender_private_key: H256,
    deposit_amount: u64,
) -> Result<H256, Box<dyn std::error::Error>> {
    let mut ckb_client = CkbRpcClient::new(ckb_rpc_url);
    let genesis_block: ckb_types::core::BlockView =
        ckb_client.get_block_by_number(0.into())?.unwrap().into();
    let cell_dep_resolver = DefaultCellDepResolver::from_genesis(&genesis_block)?;
    let header_dep_resolver = DefaultHeaderDepResolver::new(ckb_rpc_url);
    let mut cell_collector = DefaultCellCollector::new(ckb_rpc_url);
    let tx_dep_provider = DefaultTransactionDependencyProvider::new(ckb_rpc_url, 10);

    let sender_key = secp256k1::SecretKey::from_slice(sender_private_key.as_bytes())?;
    let sender_script = generate_sighash_script(&sender_key)?;
    let unlockers = setup_unlockers(vec![sender_key.clone()])?;

    // DaoDepositBuilder takes a Vec<DaoDepositReceiver>
    let receivers = vec![DaoDepositReceiver::new(sender_script.clone(), deposit_amount)];
    let builder = DaoDepositBuilder::new(receivers);
    let balancer = CapacityBalancer::new_simple(sender_script, create_placeholder_witness(), 1000);

    let (tx, _) = builder.build_unlocked(&mut cell_collector, &cell_dep_resolver,
        &header_dep_resolver, &tx_dep_provider, &balancer, &unlockers)?;

    let json_tx = ckb_jsonrpc_types::TransactionView::from(tx);
    Ok(ckb_client.send_transaction(json_tx.inner, None)?)
}

/// Start DAO withdrawal (Phase 1 - Prepare)
pub async fn start_dao_withdrawal(
    ckb_rpc_url: &str,
    sender_private_key: H256,
    deposit_out_point: OutPoint,
) -> Result<H256, Box<dyn std::error::Error>> {
    let mut ckb_client = CkbRpcClient::new(ckb_rpc_url);
    let genesis_block: ckb_types::core::BlockView =
        ckb_client.get_block_by_number(0.into())?.unwrap().into();
    let cell_dep_resolver = DefaultCellDepResolver::from_genesis(&genesis_block)?;
    let header_dep_resolver = DefaultHeaderDepResolver::new(ckb_rpc_url);
    let mut cell_collector = DefaultCellCollector::new(ckb_rpc_url);
    let tx_dep_provider = DefaultTransactionDependencyProvider::new(ckb_rpc_url, 10);

    let sender_key = secp256k1::SecretKey::from_slice(sender_private_key.as_bytes())?;
    let sender_script = generate_sighash_script(&sender_key)?;
    let unlockers = setup_unlockers(vec![sender_key.clone()])?;

    // DaoPrepareBuilder takes Vec<DaoPrepareItem>
    let input = CellInput::new(deposit_out_point, 0);
    let builder = DaoPrepareBuilder::new(vec![input.into()]);
    let balancer = CapacityBalancer::new_simple(sender_script, create_placeholder_witness(), 1000);

    let (tx, _) = builder.build_unlocked(&mut cell_collector, &cell_dep_resolver,
        &header_dep_resolver, &tx_dep_provider, &balancer, &unlockers)?;

    let json_tx = ckb_jsonrpc_types::TransactionView::from(tx);
    Ok(ckb_client.send_transaction(json_tx.inner, None)?)
}

/// Complete DAO withdrawal (Phase 2 - Withdraw)
/// Must wait ~180 epochs after prepare before execution
pub async fn complete_dao_withdrawal(
    ckb_rpc_url: &str,
    sender_private_key: H256,
    prepared_out_point: OutPoint,
) -> Result<H256, Box<dyn std::error::Error>> {
    let mut ckb_client = CkbRpcClient::new(ckb_rpc_url);
    let genesis_block: ckb_types::core::BlockView =
        ckb_client.get_block_by_number(0.into())?.unwrap().into();
    let cell_dep_resolver = DefaultCellDepResolver::from_genesis(&genesis_block)?;
    let header_dep_resolver = DefaultHeaderDepResolver::new(ckb_rpc_url);
    let mut cell_collector = DefaultCellCollector::new(ckb_rpc_url);
    let tx_dep_provider = DefaultTransactionDependencyProvider::new(ckb_rpc_url, 10);

    let sender_key = secp256k1::SecretKey::from_slice(sender_private_key.as_bytes())?;
    let sender_script = generate_sighash_script(&sender_key)?;
    let unlockers = setup_unlockers(vec![sender_key.clone()])?;

    let placeholder_witness = WitnessArgs::new_builder()
        .lock(Some(Bytes::from(vec![0u8; 65])).pack())
        .build();
    let items = vec![DaoWithdrawItem::new(prepared_out_point, Some(placeholder_witness))];
    let receiver = DaoWithdrawReceiver::LockScript {
        script: sender_script.clone(),
        fee_rate: Some(ckb_types::core::FeeRate::from_u64(1000)),
    };

    let builder = DaoWithdrawBuilder::new(items, receiver);
    let balancer = CapacityBalancer::new_simple(sender_script, create_placeholder_witness(), 1000);

    let (tx, _) = builder.build_unlocked(&mut cell_collector, &cell_dep_resolver,
        &header_dep_resolver, &tx_dep_provider, &balancer, &unlockers)?;

    let json_tx = ckb_jsonrpc_types::TransactionView::from(tx);
    Ok(ckb_client.send_transaction(json_tx.inner, None)?)
}
```

## Error Handling

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CkbSdkError {
    #[error("RPC error: {0}")]
    RpcError(#[from] ckb_jsonrpc_types::Error),
    #[error("Insufficient capacity: need {need}, have {have}")]
    InsufficientCapacity { need: u64, have: u64 },
    #[error("Transaction building failed: {0}")]
    TransactionBuildError(String),
    #[error("Secp256k1 error: {0}")]
    Secp256k1Error(#[from] secp256k1::Error),
}

/// Wrapper for safe RPC operations with retry
pub async fn safe_rpc_call<T, F, Fut>(operation: F, retry_count: usize) -> Result<T, CkbSdkError>
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
                    tokio::time::sleep(tokio::time::Duration::from_millis(100 * (1 << attempt))).await;
                }
            }
        }
    }
    Err(CkbSdkError::RpcError(last_error.unwrap()))
}
```

## Transaction Validation

```rust
use ckb_types::core::TransactionView;

pub fn estimate_transaction_fee(tx: &TransactionView, fee_rate: u64) -> u64 {
    let tx_size = tx.data().serialized_size_in_block() as u64;
    ((tx_size + 1023) / 1024) * fee_rate
}

pub fn validate_transaction(tx: &TransactionView) -> Result<(), CkbSdkError> {
    if tx.inputs().is_empty() {
        return Err(CkbSdkError::TransactionBuildError("No inputs".to_string()));
    }
    if tx.outputs().is_empty() {
        return Err(CkbSdkError::TransactionBuildError("No outputs".to_string()));
    }
    Ok(())
}
```

## Type ID Cell Creation

```rust
use ckb_sdk::constants::TYPE_ID_CODE_HASH;
use ckb_types::{packed::{Script, CellInput}, core::ScriptHashType, bytes::Bytes, prelude::*};

pub fn create_type_id_cell(first_input: &CellInput, output_index: usize) -> Script {
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

## Utility Functions

```rust
use ckb_sdk::{constants::SIGHASH_TYPE_HASH, traits::SecpCkbRawKeySigner,
              unlock::{ScriptUnlocker, SecpSighashUnlocker}, ScriptId, SECP256K1};
use ckb_types::{bytes::Bytes, core::ScriptHashType, packed::{Script, WitnessArgs}, prelude::*};
use std::collections::HashMap;

pub fn generate_sighash_script(private_key: &secp256k1::SecretKey) -> Result<Script, Box<dyn std::error::Error>> {
    let pubkey = secp256k1::PublicKey::from_secret_key(&SECP256K1, private_key);
    let hash160 = ckb_hash::blake2b_256(&pubkey.serialize()[..])[0..20].to_vec();
    Ok(Script::new_builder()
        .code_hash(SIGHASH_TYPE_HASH.pack())
        .hash_type(ScriptHashType::Type.into())
        .args(Bytes::from(hash160).pack())
        .build())
}

pub fn setup_unlockers(private_keys: Vec<secp256k1::SecretKey>)
    -> Result<HashMap<ScriptId, Box<dyn ScriptUnlocker>>, Box<dyn std::error::Error>> {
    let signer = SecpCkbRawKeySigner::new_with_secret_keys(private_keys);
    let sighash_unlocker = SecpSighashUnlocker::from(Box::new(signer) as Box<_>);
    let mut unlockers = HashMap::default();
    unlockers.insert(ScriptId::new_type(SIGHASH_TYPE_HASH.clone()),
                     Box::new(sighash_unlocker) as Box<dyn ScriptUnlocker>);
    Ok(unlockers)
}

pub fn create_placeholder_witness() -> WitnessArgs {
    WitnessArgs::new_builder().lock(Some(Bytes::from(vec![0u8; 65])).pack()).build()
}
```

## Key Patterns Summary

**Multi-signature**: `MultisigConfig` + `SecpMultisigUnlocker` for m-of-n signing requirements.

**Omnilock**: `OmniLockConfig` + `OmniLockUnlocker` for cross-chain address compatibility.

**DAO Operations**: Three-phase lifecycle with `DaoDepositBuilder`, `DaoPrepareBuilder`, and `DaoWithdrawBuilder`.

**Error Handling**: Wrap RPC calls with retry logic, validate transactions before submission.

## Related Documentation

- [Rust SDK Basic Operations](ckb://docs/sdk/rust-sdk-basic) - Simple transfers and UDT operations
- [Lock Value Relationships](ckb://docs/concepts/lock-value-relationships) - Address and lock script generation
