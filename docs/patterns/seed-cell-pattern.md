# Seed Cell Pattern

## Description

A fundamental CKB pattern for generating guaranteed unique identifiers using cell outpoints and Blake2b hashing. This pattern leverages the inherent uniqueness of cell outpoints to create deterministic, collision-resistant IDs essential for NFTs, DIDs, certificates, and any application requiring guaranteed uniqueness.

## Overview

The seed cell pattern is a fundamental CKB development pattern for generating guaranteed unique identifiers. It leverages the inherent uniqueness of cell outpoints combined with output positioning to create deterministic, collision-resistant hashes for any application requiring unique IDs.

## Key Concept

The pattern uses a **seed cell** (typically the first input cell) and combines its outpoint with the current output index to generate unique instance IDs through Blake2b hashing.

## Algorithm

```
instance_id = Blake2b(seed_outpoint.tx_hash + seed_outpoint.index + output_index)
```

**Components:**
- **`seed_outpoint.tx_hash`** (32 bytes): Transaction hash where the seed cell was created
- **`seed_outpoint.index`** (4 bytes, little-endian): Output index where the seed cell was created
- **`output_index`** (4 bytes, little-endian): Position of the new cell being created in current transaction

## Implementation Example

```rust
use blake2b_ref::Blake2bBuilder;

fn calculate_instance_id(seed_outpoint: &OutPoint, output_index: usize) -> [u8; 32] {
    let mut blake2b = Blake2bBuilder::new(32)
        .personal(b"ckb-default-hash")
        .build();
    
    blake2b.update(&seed_outpoint.tx_hash().raw_data());
    blake2b.update(&seed_outpoint.index().raw_data());
    blake2b.update(&(output_index as u32).to_le_bytes());
    
    let mut hash = [0u8; 32];
    blake2b.finalize(&mut hash);
    hash
}
```

## Transaction Pattern

```
Transaction:
  Inputs:
    [0] Seed Cell (outpoint: 0xabc123...def, index: 2)
    [1] Additional inputs...
  
  Outputs:
    [0] New Cell (ID: Blake2b(0xabc123...def + 2 + 0))
    [1] New Cell (ID: Blake2b(0xabc123...def + 2 + 1))
    [2] Change Cell, etc.
```

## Use Cases

- **NFTs**: Unique token instance identifiers
- **DIDs**: Decentralized identity generation
- **Certificates**: Unique credential IDs
- **Smart Contracts**: Any application requiring guaranteed unique identifiers

## Security Considerations

**Uniqueness vs Randomness:**
- ✅ **Guarantees uniqueness**: Impossible to generate duplicate IDs
- ⚠️ **Predictable**: IDs can be calculated if seed outpoint is known
- ❌ **Not random**: Should not be used where unpredictability is required

## Best Practices

1. **Seed Cell Selection**: Use the first input cell as it's guaranteed to exist (transaction fees requirement)
2. **Validation**: Smart contracts should validate that provided IDs match the calculated values
3. **Batch Operations**: Generate multiple unique IDs efficiently in single transactions
4. **Off-chain Generation**: Calculate IDs during transaction construction, validate on-chain

## Transaction Flow

1. **Off-chain**: Developer constructs transaction with seed-derived IDs
2. **On-chain**: Smart contract validates IDs were calculated correctly
3. **Execution**: Transaction processes with verified unique identifiers

## Why It Works

- **Global Uniqueness**: Outpoints are unique across all CKB history
- **Transaction Uniqueness**: Output indices prevent collisions within transactions  
- **Cryptographic Security**: Blake2b provides collision resistance
- **Deterministic**: Same inputs always produce same output
- **Efficient**: Lightweight calculation suitable for batch operations

This pattern exemplifies CKB's design philosophy of moving computation off-chain while maintaining on-chain verification and integrity.