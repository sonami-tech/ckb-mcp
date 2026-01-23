## Description

Concise reference guide optimized for AI assistants helping with CKB blockchain development. Provides ready-to-use code snippets and patterns for common CKB development tasks including transaction building in TypeScript/Rust, script templates for locks and types, token operations (SUDT/xUDT), cell queries and management, error handling patterns, and deployment workflows. Features side-by-side comparisons of CCC SDK and ckb-sdk-rust implementations, common pitfalls to avoid, and quick lookups for syscalls, capacity calculations, and data encoding. Designed to enable AI assistants to quickly provide accurate, working CKB code examples without extensive context switching.

## Transaction Building Patterns

### TypeScript (CCC)
```typescript
const tx = ccc.Transaction.from({
  outputs: [{ lock: recipientLock, capacity: ccc.fixedPointFrom("100") }],
  outputsData: [new Uint8Array()]
});
await tx.completeInputsByCapacity(signer);
await tx.completeFeeBy(signer, 1000);
const txHash = await signer.sendTransaction(tx);
```

### Rust (ckb-sdk-rust)
```rust
let builder = CapacityTransferBuilder::new(vec![(output, Bytes::default())]);
let (tx, _) = builder.build_unlocked(
    &mut cell_collector,
    &cell_dep_resolver,
    &header_dep_resolver,
    &tx_dep_provider,
    &balancer,
    &unlockers,
)?;
```

## Script Templates

### Lock Script (Rust no_std)
```rust
#![no_std]
#![no_main]

ckb_std::entry!(program_entry);
ckb_std::default_alloc!();

pub fn program_entry() -> i8 {
    match validate() {
        Ok(_) => 0,
        Err(_) => 1,
    }
}

fn validate() -> Result<(), Error> {
    // Verification logic
    Ok(())
}
```

### Type Script Pattern
```rust
use ckb_std::high_level::{load_script, QueryIter};
use ckb_std::ckb_constants::Source;

pub fn main() -> Result<(), Error> {
    let script = load_script()?;
    let args: Bytes = script.args().unpack();
    
    // Owner mode check
    if check_owner_mode(&args)? {
        return Ok(());
    }
    
    // Conservation check
    let input_amount = calculate_amount(Source::GroupInput)?;
    let output_amount = calculate_amount(Source::GroupOutput)?;
    
    if input_amount < output_amount {
        return Err(Error::Amount);
    }
    
    Ok(())
}
```

## Common Type Scripts

### sUDT/xUDT
- **Args**: owner_lock_hash (32 bytes) [+ flags (4 bytes) for xUDT].
- **Data**: amount (u128, 16 bytes, little-endian).
- **Validation**: Conservation unless owner mode.

### Omnilock
- **Args**: 21-byte auth + optional config.
- **Supports**: Bitcoin, Ethereum, Nostr signatures.
- **Witness**: Signature in lock field of WitnessArgs.

### Spore
- **Type ID**: Unique via hash(first_input | output_index).
- **Data**: SporeData with content_type, content, cluster_id.
- **Immutable**: Content cannot change after creation.

### Digital Objects (DOB)
- **Based on**: Spore protocol extension.
- **Content Type**: "dob/0" or "dob/1".
- **Data**: JSON with DNA field for trait generation.
- **Cluster**: Contains pattern and decoder configuration.

## Proxy Lock Patterns

### Delegate Lock
- **Args**: flags (1 byte) + delegate_hash (32 bytes) [+ data_hash (32 bytes)].
- **Purpose**: Ownership delegation with conditions.
- **Flags**: delegate_type | forbid_trade | self_destruction | restrict_data.

### Time Lock
- **Args**: required_lock_hash (32 bytes) + since_value (8 bytes).
- **Purpose**: Time-based + lock-based authorization.
- **Since**: Block number, epoch, or timestamp.

### Type Proxy Locks
- **Input Type**: Unlock when type script in inputs.
- **Output Type**: Unlock when type script in outputs.
- **Type Burn**: Unlock when type script burned.
- **Args**: target_type_hash (32 bytes).

### Single Use Lock
- **Args**: required_outpoint (36 bytes: 32-byte tx_hash + 4-byte index).
- **Purpose**: One-time unlock by consuming specific outpoint.

## Cell Capacity Requirements

```
Minimum cell: 61 CKBytes
With data: 61 + data.length
With type script: 61 + 33 + data.length
UDT cell: ~142 CKBytes (with 16-byte amount)
```

## Script Deployment Info

### Mainnet
```javascript
SECP256K1_BLAKE160: {
  codeHash: "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
  hashType: "type",
  depType: "depGroup"
}

XUDT: {
  codeHash: "0x50bd8d6680b8b9cf98b73f3c08faf8b2a21914311954118ad6609be6e78a1b95",
  hashType: "data1"
}

OMNILOCK: {
  codeHash: "0x00000000000000000000000000000000000000000000000000545950455f4944",
  hashType: "type"
}
```

## Error Codes

```rust
0: Success
1: IndexOutOfBound
2: ItemMissing
3: LengthNotEnough
4: Encoding
5: InsufficientCapacity
6: InvalidArgs
7: InvalidWitness
8: InvalidSignature
```

## Molecule Schemas

### Transaction
```molecule
table Transaction {
    version: Uint32,
    cell_deps: CellDepVec,
    header_deps: Byte32Vec,
    inputs: CellInputVec,
    outputs: CellOutputVec,
    outputs_data: BytesVec,
    witnesses: BytesVec,
}
```

### Script
```molecule
table Script {
    code_hash: Byte32,
    hash_type: byte,
    args: Bytes,
}
```

## Key Functions

### CKB Syscalls
- `load_script()`: Current script.
- `load_cell_data(index, source)`: Cell data.
- `load_cell_lock_hash(index, source)`: Lock script hash.
- `load_witness(index, source)`: Witness data.
- `verify_signature(sig, msg, pubkey)`: Secp256k1 verification.

### Sources
- `Source::Input`: Input cells.
- `Source::Output`: Output cells.
- `Source::GroupInput`: Input cells with same type.
- `Source::GroupOutput`: Output cells with same type.

## Transaction Lifecycle

1. **Build**: Construct outputs, select inputs.
2. **Balance**: Add change output if needed.
3. **Sign**: Generate witnesses for each input.
4. **Send**: Submit to mempool.
5. **Confirm**: Wait for block inclusion.

## Common Pitfalls

1. **Capacity underflow**: Always check inputs >= outputs.
2. **Data encoding**: Use consistent endianness (little-endian).
3. **Script args length**: Validate before parsing.
4. **Witness format**: Use proper WitnessArgs structure.
5. **Group vs All sources**: Type scripts use Group, locks use specific cell.
6. **Memory allocation**: Use default_alloc! for heap.

## Quick Data Patterns

### U128 Amount Encoding
```rust
// Encode
let amount_bytes = amount.to_le_bytes();

// Decode 
let amount = u128::from_le_bytes(data[0..16].try_into()?);
```

### Address to Script
```typescript
// CCC
const { script } = await ccc.Address.fromString(address, client);

// Legacy
const script = helpers.parseAddress(address);
```

### Cell Collection
```rust
// Iterate inputs
QueryIter::new(load_cell_data, Source::GroupInput)
    .collect::<Vec<_>>()

// Sum capacities
QueryIter::new(load_cell, Source::Input)
    .map(|cell| cell.capacity().unpack())
    .sum::<u64>()
```

## Debug and Testing

### Contract Debugging
```rust
ckb_std::debug!("Debug message: {}", value);
```

### Test Framework
```rust
use ckb_testtool::{builtin::ALWAYS_SUCCESS, context::Context};

#[test]
fn test_script() {
    let mut context = Context::default();
    let script = context.build_script(&ALWAYS_SUCCESS, Bytes::default());
    // Test logic
}
```

## Best Practices

1. **Always validate capacity**: outputs <= inputs.
2. **Use group sources**: For type script validation.
3. **Check data length**: Before parsing.
4. **Owner mode**: Verify lock hash matches.
5. **Use granular error codes**: Return specific error codes like `MultipleInputsNotAllowed` instead of generic `InvalidTransaction`.
6. **Test thoroughly**: Use OffCKB for local testing.