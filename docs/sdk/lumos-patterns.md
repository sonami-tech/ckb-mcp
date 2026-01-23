# Lumos SDK Patterns

## Description

Lumos JavaScript/TypeScript SDK patterns for CKB development (legacy reference). Transaction building with TransactionSkeleton, cell collection with async iterators, UDT transfers, and multi-signature support. Lumos is in maintenance mode; use CCC SDK for new projects. Sparse Merkle Tree patterns for state management included: basic SMT operations, CKB script integration for state validation, and performance optimization with batching and parallel proof generation.

## Transaction Building with Lumos

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

## Cell Collection Patterns

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

### Basic SMT Operations

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

### SMT Integration with CKB Scripts

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

### SMT Performance Optimization

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
