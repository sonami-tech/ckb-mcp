# Molecule Serialization in CKB

## Description

Complete guide to Molecule serialization in CKB, covering type system, memory layout, zero-copy access, and schema design. Features practical examples of transaction structures, UDT serialization, compatibility patterns, and performance optimization. Essential for CKB development with comprehensive coverage of arrays, vectors, tables, options, unions, and real-world usage patterns.

## Overview

Molecule is CKB's canonical serialization format, designed as a minimalist and canonicalization system for blockchain data structures. All CKB transaction data, cell structures, and script communication rely on Molecule serialization, making it essential for CKB development.

## Key Features

- **Minimalist Design**: Simple, efficient binary format with minimal overhead
- **Canonicalization**: Deterministic encoding ensures identical data produces identical bytes
- **Zero-Copy**: Read data without deserializing, crucial for blockchain performance
- **Schema-Based**: Strongly-typed with compile-time guarantees
- **Cross-Language**: Official support for Rust and C, with community implementations

## Type System

### Primitive Type

#### `byte`
The fundamental building block - a single byte (8 bits).

```rust
// Examples
let data: u8 = 0x42;
let serialized = data.to_le_bytes(); // [0x42]
```

### Composite Types

#### `array` - Fixed Size
Arrays have a fixed inner type and fixed length. No serialization overhead.

```rust
// Schema definition
array Byte3 [byte; 3];
array Uint32 [byte; 4];

// Serialization examples
// Byte3 with values [0x01, 0x02, 0x03]
// Serialized: 01 02 03

// Uint32 with value 0x01020304 (little-endian)
// Serialized: 04 03 02 01
```

#### `struct` - Fixed Size
Structs contain fixed-size fields stored consecutively.

```rust
// Schema
struct CellInput {
    since: [byte; 8],
    previous_output: OutPoint,
}

// Serialized as: <8 bytes since><32 bytes tx_hash><4 bytes index>
```

#### `vector` - Dynamic Size
Two types based on inner item size:

##### Fixed Vector (`fixvec`)
For fixed-size inner items.

```rust
// Schema
vector Bytes <byte>;

// Examples:
// Empty: 00 00 00 00 (length = 0)
// [0x12]: 01 00 00 00 12 (length = 1, data = 0x12)
// [0x12, 0x34]: 02 00 00 00 12 34
```

##### Dynamic Vector (`dynvec`) 
For dynamic-size inner items.

```rust
// Schema
vector BytesVec <Bytes>;

// Example: [[0x12, 0x34], [0x56]]
// Serialized:
// 16 00 00 00        // Full size (22 bytes)
// 0c 00 00 00        // Offset to first item  
// 12 00 00 00        // Offset to second item
// 02 00 00 00 12 34  // First item: Bytes([0x12, 0x34])
// 01 00 00 00 56     // Second item: Bytes([0x56])
```

#### `table` - Dynamic Size
Like `dynvec` but with fixed field count.

```rust
// Schema
table CellOutput {
    capacity: [byte; 8],
    lock: Script,
    type_: ScriptOpt,
}

// Serialization: <full_size><field_offsets...><field_data...>
```

#### `option` - Dynamic Size
Optional data - empty or contains inner item.

```rust
// Schema  
option ScriptOpt (Script);

// Examples:
// None: (empty - 0 bytes)
// Some(script): <script_data> (same size as script)
```

#### `union` - Dynamic Size
Tagged union with type ID.

```rust
// Schema
union ScriptHashType {
    Data,     // ID = 0
    Type,     // ID = 1  
    Data1,    // ID = 2
}

// Examples:
// Data: 00 00 00 00
// Type: 01 00 00 00
// Data1: 02 00 00 00
```

## Memory Layout Summary

| Type | Header | Body |
|------|--------|------|
| `array` | None | `item-0 \| item-1 \| ... \| item-N` |
| `struct` | None | `field-0 \| field-1 \| ... \| field-N` |
| `fixvec` | `items-count` | `item-0 \| item-1 \| ... \| item-N` |
| `dynvec` | `full-size \| offset-0 \| ... \| offset-N` | `item-0 \| item-1 \| ... \| item-N` |
| `table` | `full-size \| offset-0 \| ... \| offset-N` | `field-0 \| field-1 \| ... \| field-N` |
| `option` | None | `item or empty` |
| `union` | `item-type-id` | `item` |

- All header values are 32-bit unsigned integers in little-endian format

## CKB Transaction Structure Example

```rust
// Core CKB transaction structure
table Transaction {
    version: [byte; 4],
    cell_deps: CellDepVec,
    header_deps: Byte32Vec,
    inputs: CellInputVec,
    outputs: CellOutputVec,
    outputs_data: BytesVec,
    witnesses: BytesVec,
}

table CellInput {
    since: [byte; 8],
    previous_output: OutPoint,
}

table OutPoint {
    tx_hash: [byte; 32],
    index: [byte; 4],
}

table CellOutput {
    capacity: [byte; 8],
    lock: Script,
    type_: ScriptOpt,
}

table Script {
    code_hash: [byte; 32],
    hash_type: [byte; 1],
    args: Bytes,
}
```

## Real-World Serialization

### Simple Transfer Transaction

```rust
// Create a basic CKB transfer
let input = CellInput::new_builder()
    .since(0u64.pack())
    .previous_output(OutPoint::new_builder()
        .tx_hash(prev_tx_hash.pack())
        .index(0u32.pack())
        .build())
    .build();

let output = CellOutput::new_builder()
    .capacity((100_000_000u64).pack()) // 100 CKB
    .lock(recipient_lock_script)
    .build();

let transaction = Transaction::new_builder()
    .version(0u32.pack())
    .inputs(vec![input].pack())
    .outputs(vec![output].pack())
    .outputs_data(vec![Bytes::new()].pack())
    .build();

// Get serialized bytes
let serialized = transaction.as_bytes();
```

### UDT Token Transfer

```rust
// UDT cell data: 16-byte amount in little-endian
let amount: u128 = 1000_000_000; // 1000 tokens
let udt_data = amount.to_le_bytes();

let udt_output = CellOutput::new_builder()
    .capacity((14200000000u64).pack()) // CKB capacity
    .lock(recipient_lock)
    .type_(Some(udt_type_script).pack())
    .build();

// UDT transaction
let udt_transaction = Transaction::new_builder()
    .inputs(udt_inputs.pack())
    .outputs(vec![udt_output].pack())
    .outputs_data(vec![Bytes::from(udt_data.to_vec())].pack())
    .build();
```

## Schema Compatibility

Molecule supports forward compatibility for `table` types:

```rust
// Original schema
table OriginalCell {
    capacity: [byte; 8],
    lock: Script,
}

// Extended schema
table ExtendedCell {
    capacity: [byte; 8],
    lock: Script,
    type_: ScriptOpt,     // New field
    extra_data: Bytes,    // Another new field
}

// Old code can read new data using compatible APIs
let old_cell = OriginalCell::from_compatible_slice(&extended_data)?;
```

## Performance Characteristics

### Zero-Copy Access

```rust
// No deserialization needed for field access
let transaction = Transaction::from_slice(&raw_data)?;
let input_count = transaction.inputs().len(); // O(1)
let first_input = transaction.inputs().get(0)?; // O(1)
let capacity = first_input.previous_output().tx_hash(); // O(1)
```

### Memory Efficiency

- **Fixed types**: No overhead beyond data itself
- **Dynamic types**: Minimal header overhead (4-8 bytes typically)
- **Nested structures**: Efficient offset-based access
- **Large data**: True zero-copy for read operations

## Common Patterns in CKB

### Cell Data Parsing

```rust
// Parse UDT amount from cell data
fn parse_udt_amount(data: &[u8]) -> Result<u128, Error> {
    if data.len() < 16 {
        return Err(Error::InvalidDataLength);
    }
    
    let amount_bytes: [u8; 16] = data[0..16].try_into()?;
    Ok(u128::from_le_bytes(amount_bytes))
}

// Parse remaining data after UDT amount
fn parse_extension_data(data: &[u8]) -> &[u8] {
    if data.len() > 16 {
        &data[16..]
    } else {
        &[]
    }
}
```

### Script Argument Parsing

```rust
// Parse multi-component script args
fn parse_script_args(args: &Bytes) -> Result<(Hash, Vec<u8>), Error> {
    let raw_args = args.raw_data();
    if raw_args.len() < 32 {
        return Err(Error::InvalidArgs);
    }
    
    let hash: [u8; 32] = raw_args[0..32].try_into()?;
    let extra_data = raw_args[32..].to_vec();
    
    Ok((hash.into(), extra_data))
}
```

### Witness Structure

```rust
// Standard witness format
table WitnessArgs {
    lock: BytesOpt,       // Lock script witness
    input_type: BytesOpt, // Input type script witness  
    output_type: BytesOpt, // Output type script witness
}

// Parse witness in script
let witness_args = WitnessArgs::from_slice(&witness_data)?;
if let Some(lock_witness) = witness_args.lock().to_opt() {
    // Process signature data
    let signature = lock_witness.raw_data();
}
```

## Development Tools

### Schema Compiler

```bash
# Install moleculec
cargo install moleculec --locked

# Generate Rust code
moleculec --language rust --schema-file transaction.mol

# Generate C code  
moleculec --language c --schema-file transaction.mol
```

### Testing Serialization

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_roundtrip_serialization() {
        let original = build_test_transaction();
        let serialized = original.as_bytes();
        let deserialized = Transaction::from_slice(&serialized).unwrap();
        
        assert_eq!(original.as_bytes(), deserialized.as_bytes());
    }
    
    #[test]
    fn test_field_access() {
        let tx = build_test_transaction();
        assert_eq!(tx.version().unpack(), 0u32);
        assert_eq!(tx.inputs().len(), 1);
        assert_eq!(tx.outputs().len(), 1);
    }
}
```

## Best Practices

### Schema Design

1. **Use appropriate types**: Choose between fixed arrays and dynamic vectors based on use case
2. **Field ordering**: Put fixed-size fields first in tables for better performance
3. **Avoid deep nesting**: Prefer flat structures when possible
4. **Version compatibility**: Design tables to allow future field additions

### Performance

1. **Minimize allocations**: Use readers for temporary access
2. **Batch operations**: Build complex structures once rather than incrementally
3. **Cache serialized data**: Store `Bytes` rather than rebuilding repeatedly
4. **Profile memory usage**: Monitor allocation patterns in performance-critical code

### Error Handling

```rust
// Robust parsing with proper error handling
fn parse_transaction_safely(data: &[u8]) -> Result<Transaction, Error> {
    // Verify data integrity first
    Transaction::from_slice(data)
        .map_err(|e| Error::InvalidTransaction(e.to_string()))
}

// Validate before using
fn validate_cell_output(output: &CellOutput) -> Result<(), Error> {
    if output.capacity().unpack() == 0 {
        return Err(Error::ZeroCapacity);
    }
    
    // Additional validation...
    Ok(())
}
```

Molecule serialization is the foundation of all CKB data structures. Understanding its type system, memory layout, and performance characteristics is essential for efficient CKB development. The combination of strong typing, zero-copy access, and minimal overhead makes it ideal for blockchain applications where performance and determinism are critical.