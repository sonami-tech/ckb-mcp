# CKB SDK Examples and Patterns

## Description

Comprehensive CKB SDK patterns and examples covering Rust SDK, Lumos (legacy), and Sparse Merkle Tree implementations. Features advanced transaction building, cell collection, multi-signature operations, DAO interactions, error handling, and performance optimization. Includes practical patterns for script development, state management, and production deployment.

This guide provides comprehensive examples and patterns for using CKB SDKs, including ckb-sdk-rust, Lumos (maintenance mode), and sparse merkle tree implementations.

## Rust SDK Patterns

### 1. Basic SDK Setup and Configuration

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

### 2. Cell Collection and Management

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

### 3. Advanced Transaction Building

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

### 4. Multi-Signature and Omnilock Integration

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

### 5. DAO (Nervos DAO) Operations

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

## Lumos SDK Patterns (Legacy Reference)

### 1. Transaction Building with Lumos

```typescript
// Note: Lumos is in maintenance mode, use CCC SDK for new projects
import { TransactionSkeleton, sealTransaction } from "@ckb-lumos/helpers";
import { common } from "@ckb-lumos/common-scripts";
import { RPC } from "@ckb-lumos/rpc";
import { Indexer } from "@ckb-lumos/indexer";

export class LumosTransactionBuilder {
  private rpc: RPC;
  private indexer: Indexer;
  
  constructor(nodeUrl: string, indexerUrl: string) {
    this.rpc = new RPC(nodeUrl);
    this.indexer = new Indexer(indexerUrl);
  }
  
  // Build simple transfer (legacy pattern)
  async buildTransfer(
    fromAddress: string,
    toAddress: string,
    amount: bigint,
    feeRate: bigint = 1000n,
  ) {
    let txSkeleton = TransactionSkeleton({ cellProvider: this.indexer });
    
    // Add transfer
    txSkeleton = await common.transfer(
      txSkeleton,
      [fromAddress],
      toAddress,
      amount,
    );
    
    // Pay fee
    txSkeleton = await common.payFeeByFeeRate(
      txSkeleton,
      [fromAddress],
      feeRate,
    );
    
    return txSkeleton;
  }
  
  // Build UDT transfer (legacy pattern)
  async buildUdtTransfer(
    udtTypeScript: Script,
    fromAddress: string,
    toAddress: string,
    amount: bigint,
  ) {
    let txSkeleton = TransactionSkeleton({ cellProvider: this.indexer });
    
    // Add UDT transfer
    txSkeleton = await common.sudtTransfer(
      txSkeleton,
      [fromAddress],
      udtTypeScript,
      toAddress,
      amount,
    );
    
    // Balance capacity
    txSkeleton = await common.payFeeByFeeRate(
      txSkeleton,
      [fromAddress],
      1000n,
    );
    
    return txSkeleton;
  }
  
  // Multi-signature support (legacy pattern)
  async buildMultisigTransfer(
    multisigScript: Script,
    fromAddresses: string[],
    toAddress: string,
    amount: bigint,
  ) {
    let txSkeleton = TransactionSkeleton({ cellProvider: this.indexer });
    
    // Use multisig transfer
    txSkeleton = await common.transfer(
      txSkeleton,
      fromAddresses,
      toAddress,
      amount,
      undefined,
      undefined,
      { config: { PREFIX: "ckb", SCRIPTS: { SECP256K1_BLAKE160_MULTISIG: multisigScript } } }
    );
    
    return txSkeleton;
  }
}
```

**Reference:** `resources/lumos/packages/common-scripts/`

### 2. Cell Collection Patterns (Lumos)

```typescript
import { Cell, QueryOptions } from "@ckb-lumos/base";
import { Indexer } from "@ckb-lumos/indexer";

export class LumosCellCollector {
  constructor(private indexer: Indexer) {}
  
  // Collect cells by lock script
  async collectCellsByLock(
    lockScript: Script,
    options?: { capacityRequired?: bigint }
  ): Promise<Cell[]> {
    const query: QueryOptions = {
      lock: lockScript,
    };
    
    if (options?.capacityRequired) {
      query.data = "0x";
    }
    
    const cells: Cell[] = [];
    const collector = this.indexer.collector(query);
    
    for await (const cell of collector.collect()) {
      cells.push(cell);
      
      if (options?.capacityRequired) {
        const totalCapacity = cells.reduce(
          (sum, c) => sum + BigInt(c.cellOutput.capacity),
          0n
        );
        if (totalCapacity >= options.capacityRequired) {
          break;
        }
      }
    }
    
    return cells;
  }
  
  // Collect UDT cells
  async collectUdtCells(
    typeScript: Script,
    lockScript?: Script,
  ): Promise<Array<{ cell: Cell; amount: bigint }>> {
    const query: QueryOptions = {
      type: typeScript,
    };
    
    if (lockScript) {
      query.lock = lockScript;
    }
    
    const udtCells: Array<{ cell: Cell; amount: bigint }> = [];
    const collector = this.indexer.collector(query);
    
    for await (const cell of collector.collect()) {
      if (cell.data && cell.data.length >= 34) { // "0x" + 32 bytes
        const amountHex = cell.data.slice(2, 34);
        const amount = BigInt("0x" + amountHex);
        udtCells.push({ cell, amount });
      }
    }
    
    return udtCells;
  }
}
```

## Sparse Merkle Tree Patterns

### 1. Basic SMT Operations

```rust
use sparse_merkle_tree::{
    SparseMerkleTree, BranchNode, LeafNode, 
    blake2b::Blake2bHasher, CompiledMerkleProof,
};
use std::collections::HashMap;

pub struct CkbSparseMerkleTree {
    tree: SparseMerkleTree<Blake2bHasher, [u8; 32], [u8; 32]>,
    store: HashMap<[u8; 32], Vec<u8>>,
}

impl CkbSparseMerkleTree {
    pub fn new() -> Self {
        let tree = SparseMerkleTree::new();
        let store = HashMap::new();
        
        Self { tree, store }
    }
    
    // Update tree with key-value pairs
    pub fn update(
        &mut self,
        updates: Vec<([u8; 32], [u8; 32])>,
    ) -> Result<[u8; 32], Box<dyn std::error::Error>> {
        let mut tree_updates = Vec::new();
        
        for (key, value) in updates {
            // Store in local storage
            self.store.insert(key, value.to_vec());
            
            // Prepare tree update
            tree_updates.push((key, value));
        }
        
        // Apply updates to tree
        let new_root = self.tree.update_all(tree_updates)?;
        
        Ok(new_root)
    }
    
    // Generate merkle proof for key
    pub fn generate_proof(
        &self,
        key: &[u8; 32],
    ) -> Result<CompiledMerkleProof, Box<dyn std::error::Error>> {
        let proof = self.tree.merkle_proof(vec![*key])?;
        let compiled_proof = proof.compile(vec![*key])?;
        
        Ok(compiled_proof)
    }
    
    // Verify merkle proof
    pub fn verify_proof(
        &self,
        root: &[u8; 32],
        proof: &CompiledMerkleProof,
        key: &[u8; 32],
        value: &[u8; 32],
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let is_valid = proof.verify(root, vec![(*key, *value)])?;
        Ok(is_valid)
    }
    
    // Get tree statistics
    pub fn get_stats(&self) -> TreeStats {
        TreeStats {
            root: self.tree.root(),
            leaf_count: self.store.len(),
            tree_size: self.calculate_tree_size(),
        }
    }
    
    fn calculate_tree_size(&self) -> usize {
        // Calculate approximate tree size in bytes
        self.store.len() * (32 + 32) // key + value size
    }
}

pub struct TreeStats {
    pub root: [u8; 32],
    pub leaf_count: usize,
    pub tree_size: usize,
}
```

**Reference:** `resources/sparse-merkle-tree/src/tree.rs`

### 2. SMT Integration with CKB Scripts

```rust
// Contract integration example
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{bytes::Bytes, prelude::*},
    high_level::{load_cell_data, load_script_hash},
    syscalls,
};

// SMT-based state validation
pub fn validate_state_transition() -> Result<(), Error> {
    // Load previous state root from input cell
    let input_data = load_cell_data(0, Source::GroupInput)?;
    if input_data.len() < 32 {
        return Err(Error::InvalidStateRoot);
    }
    let prev_root: [u8; 32] = input_data[0..32].try_into()
        .map_err(|_| Error::InvalidStateRoot)?;
    
    // Load new state root from output cell
    let output_data = load_cell_data(0, Source::GroupOutput)?;
    if output_data.len() < 32 {
        return Err(Error::InvalidStateRoot);
    }
    let new_root: [u8; 32] = output_data[0..32].try_into()
        .map_err(|_| Error::InvalidStateRoot)?;
    
    // Load state transition proof from witness
    let witness = load_witness_args(0, Source::GroupInput)?
        .input_type()
        .to_opt()
        .ok_or(Error::InvalidProof)?;
    
    let proof_data = witness.unpack();
    let proof = parse_merkle_proof(&proof_data)?;
    
    // Verify state transition
    if !verify_state_transition(&prev_root, &new_root, &proof)? {
        return Err(Error::InvalidStateTransition);
    }
    
    Ok(())
}

fn verify_state_transition(
    prev_root: &[u8; 32],
    new_root: &[u8; 32],
    proof: &StateTransitionProof,
) -> Result<bool, Error> {
    // Verify each state update in the proof
    for update in &proof.updates {
        // Verify proof for previous state
        if !verify_inclusion_proof(prev_root, &update.key, &update.old_value, &update.old_proof)? {
            return Ok(false);
        }
        
        // Verify proof for new state
        if !verify_inclusion_proof(new_root, &update.key, &update.new_value, &update.new_proof)? {
            return Ok(false);
        }
    }
    
    Ok(true)
}

fn verify_inclusion_proof(
    root: &[u8; 32],
    key: &[u8; 32],
    value: &[u8; 32],
    proof: &[u8],
) -> Result<bool, Error> {
    // Use CKB's built-in Blake2b for hashing
    let mut hasher = Blake2bBuilder::new(32).build();
    
    // Implement SMT proof verification
    let mut current_hash = blake2b_hash_leaf(key, value);
    let mut current_key = *key;
    
    for proof_byte in proof {
        for bit in 0..8 {
            let direction = (proof_byte >> bit) & 1;
            
            if direction == 0 {
                // Left branch
                current_hash = blake2b_hash_branch(&current_hash, &[0u8; 32]);
            } else {
                // Right branch  
                current_hash = blake2b_hash_branch(&[0u8; 32], &current_hash);
            }
            
            current_key[31] >>= 1;
            if current_key == [0u8; 32] {
                break;
            }
        }
    }
    
    Ok(current_hash == *root)
}

fn blake2b_hash_leaf(key: &[u8; 32], value: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Blake2bBuilder::new(32).build();
    hasher.update(&[0u8]); // Leaf prefix
    hasher.update(key);
    hasher.update(value);
    
    let mut result = [0u8; 32];
    hasher.finalize(&mut result);
    result
}

fn blake2b_hash_branch(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Blake2bBuilder::new(32).build();
    hasher.update(&[1u8]); // Branch prefix
    hasher.update(left);
    hasher.update(right);
    
    let mut result = [0u8; 32];
    hasher.finalize(&mut result);
    result
}

struct StateTransitionProof {
    updates: Vec<StateUpdate>,
}

struct StateUpdate {
    key: [u8; 32],
    old_value: [u8; 32],
    new_value: [u8; 32],
    old_proof: Vec<u8>,
    new_proof: Vec<u8>,
}
```

### 3. SMT Performance Optimization

```rust
use sparse_merkle_tree::tree::BranchNode;
use std::sync::Arc;

pub struct OptimizedSMT {
    tree: SparseMerkleTree<Blake2bHasher, [u8; 32], [u8; 32]>,
    cache: LruCache<[u8; 32], Arc<BranchNode<[u8; 32]>>>,
    batch_size: usize,
}

impl OptimizedSMT {
    pub fn new(cache_size: usize, batch_size: usize) -> Self {
        Self {
            tree: SparseMerkleTree::new(),
            cache: LruCache::new(cache_size),
            batch_size,
        }
    }
    
    // Batched updates for better performance
    pub fn batch_update(
        &mut self,
        updates: Vec<([u8; 32], [u8; 32])>,
    ) -> Result<[u8; 32], Box<dyn std::error::Error>> {
        // Process updates in batches
        let mut current_root = self.tree.root();
        
        for chunk in updates.chunks(self.batch_size) {
            let batch_updates: Vec<_> = chunk.iter().cloned().collect();
            current_root = self.tree.update_all(batch_updates)?;
        }
        
        Ok(current_root)
    }
    
    // Parallel proof generation
    pub fn generate_parallel_proofs(
        &self,
        keys: Vec<[u8; 32]>,
    ) -> Result<Vec<CompiledMerkleProof>, Box<dyn std::error::Error>> {
        use rayon::prelude::*;
        
        let proofs: Result<Vec<_>, _> = keys
            .par_iter()
            .map(|key| self.tree.merkle_proof(vec![*key]))
            .collect();
        
        let merkle_proofs = proofs?;
        
        let compiled_proofs: Result<Vec<_>, _> = merkle_proofs
            .into_par_iter()
            .zip(keys.par_iter())
            .map(|(proof, key)| proof.compile(vec![*key]))
            .collect();
        
        compiled_proofs
    }
}
```

**Reference:** `resources/sparse-merkle-tree/benches/`

## Best Practices and Error Handling

### 1. Comprehensive Error Handling

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

### 2. Network Abstraction

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

### 3. Configuration Management

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

These SDK examples and patterns provide comprehensive guidance for building robust CKB applications using various SDKs and tools, with proper error handling, optimization, and security considerations.