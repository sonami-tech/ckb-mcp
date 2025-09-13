## Description

Complete Molecule serialization API reference with multi-language examples (Rust, C, JavaScript). Covers Entity, Reader, and Builder traits, zero-copy parsing, error handling, performance optimization, and CKB-specific schemas. Essential for type-safe CKB transaction construction, script development, and protocol integration with practical patterns for efficient data handling.

Molecule provides code-generated APIs in multiple languages for type-safe serialization.

## Rust API Reference

### Core Traits

Every Molecule type implements these fundamental traits:

```rust
use molecule::prelude::*;

// Entity trait - for owned data
pub trait Entity: Clone + Eq + Debug {
    type Builder;
    
    fn new_unchecked(data: Bytes) -> Self;
    fn new_builder() -> Self::Builder;
    fn as_bytes(&self) -> Bytes;
    fn as_slice(&self) -> &[u8];
}

// Reader trait - for zero-copy access
pub trait Reader<'r>: Clone + Debug {
    type Entity;
    
    fn from_slice(slice: &'r [u8]) -> Result<Self, VerificationError>;
    fn as_slice(&self) -> &'r [u8];
    fn to_entity(&self) -> Self::Entity;
}

// Builder trait - for construction
pub trait Builder: Default + Clone + Debug {
    type Entity;
    
    fn build(self) -> Self::Entity;
}
```

### Basic Type Operations

#### Primitive Arrays

```rust
use molecule::prelude::*;

// Create Byte32 from array
let hash_bytes = [0u8; 32];
let hash = Byte32::from_slice(&hash_bytes).unwrap();

// Access raw data
let raw_data: &[u8] = hash.raw_data();
let as_array: [u8; 32] = hash.raw_data().try_into().unwrap();

// Build from builder
let hash = Byte32::new_builder()
    .set([
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
        0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
        0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
        0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
    ])
    .build();
```

#### Vectors

```rust
// Create vector of bytes
let data = vec![0x01, 0x02, 0x03, 0x04];
let bytes = Bytes::new_builder()
    .extend(data.iter().map(|&b| b.into()))
    .build();

// Access vector elements
let length = bytes.len();
for i in 0..length {
    let byte_value = bytes.get(i).unwrap();
    println!("Byte {}: {:#x}", i, byte_value.into());
}

// Convert to Vec<u8>
let raw_vec: Vec<u8> = bytes.raw_data().to_vec();

// Create from slice
let bytes_from_slice = Bytes::from_slice(&data).unwrap();
```

#### Structs

```rust
// OutPoint struct creation
let tx_hash = [0u8; 32];
let out_point = OutPoint::new_builder()
    .tx_hash(Byte32::from_slice(&tx_hash).unwrap().pack())
    .index(0u32.pack())
    .build();

// Access struct fields
let tx_hash_field = out_point.tx_hash();
let index_field = out_point.index();

// Convert to raw values
let index_value = u32::from_le_bytes(
    index_field.raw_data().try_into().unwrap()
);
```

#### Tables

```rust
// Script table creation
let code_hash = [0u8; 32];
let args = vec![0x01, 0x02, 0x03];

let script = Script::new_builder()
    .code_hash(Byte32::from_slice(&code_hash).unwrap().pack())
    .hash_type(0u8.into())
    .args(Bytes::from_slice(&args).unwrap().pack())
    .build();

// Access table fields
let code_hash_field = script.code_hash();
let hash_type_field = script.hash_type();
let args_field = script.args();

// Check field presence (all fields in tables are always present)
println!("Code hash: {:?}", code_hash_field.raw_data());
println!("Args length: {}", args_field.len());
```

#### Options

```rust
// Create optional script
let script_opt = ScriptOpt::new_builder()
    .set(Some(script))
    .build();

// Check if option is some/none
if script_opt.is_some() {
    let inner_script = script_opt.to_opt().unwrap();
    println!("Script present: {:?}", inner_script.code_hash());
} else {
    println!("No script present");
}

// Create empty option
let empty_script_opt = ScriptOpt::new_builder().build();
assert!(empty_script_opt.is_none());
```

#### Unions

```rust
// Create union with specific variant
let witness_args = WitnessArgs::new_builder()
    .lock(Some(Bytes::from_slice(&[0x01, 0x02]).unwrap()).pack())
    .input_type(Some(Bytes::from_slice(&[0x03, 0x04]).unwrap()).pack())
    .output_type(Some(Bytes::from_slice(&[0x05, 0x06]).unwrap()).pack())
    .build();

// Access union fields
let lock_field = witness_args.lock();
let input_type_field = witness_args.input_type();
let output_type_field = witness_args.output_type();

// Check optional fields
if let Some(lock_bytes) = lock_field.to_opt() {
    println!("Lock witness: {:?}", lock_bytes.raw_data());
}
```

### Reader API Patterns

#### Zero-Copy Access

```rust
// Efficient parsing without allocations
fn parse_transaction_efficiently(tx_bytes: &[u8]) -> Result<u64, VerificationError> {
    let tx_reader = TransactionReader::from_slice(tx_bytes)?;
    
    let mut total_capacity = 0u64;
    let outputs_reader = tx_reader.outputs();
    
    for i in 0..outputs_reader.len() {
        let output_reader = outputs_reader.get(i)?;
        let capacity_bytes = output_reader.capacity().raw_data();
        let capacity = u64::from_le_bytes(
            capacity_bytes.try_into().unwrap()
        );
        total_capacity += capacity;
    }
    
    Ok(total_capacity)
}

// Convert reader to entity when needed
fn reader_to_entity_example(tx_bytes: &[u8]) -> Result<Transaction, VerificationError> {
    let tx_reader = TransactionReader::from_slice(tx_bytes)?;
    Ok(tx_reader.to_entity())
}
```

#### Iterating Collections

```rust
// Iterate over vector readers
fn iterate_cell_deps(tx_bytes: &[u8]) -> Result<(), VerificationError> {
    let tx_reader = TransactionReader::from_slice(tx_bytes)?;
    let cell_deps_reader = tx_reader.cell_deps();
    
    for i in 0..cell_deps_reader.len() {
        let cell_dep_reader = cell_deps_reader.get(i)?;
        let out_point_reader = cell_dep_reader.out_point();
        
        println!("Cell dep {}: tx_hash={:?}, index={}", 
            i,
            out_point_reader.tx_hash().raw_data(),
            u32::from_le_bytes(
                out_point_reader.index().raw_data().try_into().unwrap()
            )
        );
    }
    
    Ok(())
}
```

### Builder Patterns

#### Fluent Construction

```rust
// Chain builder methods
let transaction = TransactionBuilder::default()
    .version(0u32.pack())
    .cell_deps({
        let mut cell_deps = CellDepVec::new_builder();
        cell_deps = cell_deps.push(cell_dep_1);
        cell_deps = cell_deps.push(cell_dep_2);  
        cell_deps.build()
    })
    .header_deps({
        Byte32Vec::new_builder()
            .push(header_hash_1)
            .push(header_hash_2)
            .build()
    })
    .inputs({
        CellInputVec::new_builder()
            .push(input_1)
            .push(input_2)
            .build()
    })
    .outputs({
        CellOutputVec::new_builder()
            .push(output_1)
            .push(output_2)
            .build()
    })
    .outputs_data({
        BytesVec::new_builder()
            .push(data_1)
            .push(data_2)
            .build()
    })
    .witnesses({
        BytesVec::new_builder()
            .push(witness_1)
            .push(witness_2)
            .build()
    })
    .build();
```

#### Collection Builders

```rust
// Build vector from iterator
let script_vec = scripts.into_iter()
    .fold(ScriptVec::new_builder(), |builder, script| {
        builder.push(script)
    })
    .build();

// Build with extend
let bytes_vec = BytesVec::new_builder()
    .extend(data_items.into_iter().map(|data| {
        Bytes::from_slice(&data).unwrap()
    }))
    .build();

// Conditional building
let mut witness_builder = WitnessArgs::new_builder();

if let Some(lock_witness) = lock_witness_data {
    witness_builder = witness_builder.lock(Some(
        Bytes::from_slice(&lock_witness).unwrap()
    ).pack());
}

if let Some(input_type_witness) = input_type_witness_data {
    witness_builder = witness_builder.input_type(Some(
        Bytes::from_slice(&input_type_witness).unwrap()
    ).pack());
}

let witness_args = witness_builder.build();
```

### Error Handling

#### Verification Errors

```rust
use molecule::error::VerificationError;

fn handle_molecule_errors(data: &[u8]) -> Result<Transaction, String> {
    match Transaction::from_slice(data) {
        Ok(tx) => Ok(tx),
        Err(VerificationError::TotalSizeNotMatch(expected, actual)) => {
            Err(format!("Size mismatch: expected {}, got {}", expected, actual))
        }
        Err(VerificationError::HeaderIsBroken) => {
            Err("Invalid molecule header".to_string())
        }
        Err(VerificationError::UnknownItem { item_id }) => {
            Err(format!("Unknown union variant: {}", item_id))
        }
        Err(VerificationError::FieldCountNotMatch(expected, actual)) => {
            Err(format!("Field count mismatch: expected {}, got {}", expected, actual))
        }
        Err(err) => Err(format!("Verification error: {:?}", err))
    }
}
```

## C API Reference

### Basic Usage

```c
#include "molecule.h"
#include "blockchain.h"

// Parse transaction from bytes
int parse_transaction(const uint8_t* data, size_t len) {
    mol_seg_t tx_seg;
    if (MolReader_Transaction_verify(&tx_seg, data, len, false) != MOL_OK) {
        return -1; // Invalid data
    }
    
    // Access transaction fields
    mol_seg_t version_seg = MolReader_Transaction_get_version(&tx_seg);
    uint32_t version = mol_unpack_number(&version_seg);
    
    mol_seg_t inputs_seg = MolReader_Transaction_get_inputs(&tx_seg);
    uint32_t input_count = MolReader_CellInputVec_length(&inputs_seg);
    
    printf("Transaction version: %u, inputs: %u\n", version, input_count);
    return 0; 
}

// Build transaction
mol_builder_t build_simple_transaction() {
    mol_builder_t tx_builder = MolBuilder_Transaction_init();
    
    // Set version
    uint32_t version = 0;
    mol_seg_t version_seg = mol_pack_number(&version);
    MolBuilder_Transaction_set_version(&tx_builder, &version_seg);
    
    // Build empty vectors
    mol_builder_t cell_deps_builder = MolBuilder_CellDepVec_init();
    mol_seg_t cell_deps = MolBuilder_CellDepVec_build(&cell_deps_builder);
    MolBuilder_Transaction_set_cell_deps(&tx_builder, &cell_deps);
    
    // ... set other fields
    
    return tx_builder;
}
```

### Memory Management

```c
// Safe memory handling
typedef struct {
    uint8_t* data;
    size_t len;
    size_t capacity;
} buffer_t;

buffer_t* buffer_new(size_t capacity) {
    buffer_t* buf = malloc(sizeof(buffer_t));
    buf->data = malloc(capacity);
    buf->len = 0;
    buf->capacity = capacity;
    return buf;
}

void buffer_free(buffer_t* buf) {
    if (buf) {
        free(buf->data);
        free(buf);
    }
}

// Serialize with proper cleanup
int serialize_transaction(mol_builder_t* tx_builder, buffer_t* output) {
    mol_seg_t tx_seg = MolBuilder_Transaction_build(tx_builder);
    
    if (tx_seg.size > output->capacity) {
        return -1; // Buffer too small
    }
    
    memcpy(output->data, tx_seg.ptr, tx_seg.size);
    output->len = tx_seg.size;
    
    // Clean up builder
    MolBuilder_Transaction_clear(tx_builder);
    
    return 0;
}
```

## JavaScript API Reference

### Module Import and Basic Usage

```javascript
import { blockchain } from '@ckb-lumos/molecule';

// Parse transaction from Uint8Array
function parseTransaction(data) {
    try {
        const tx = blockchain.Transaction.unpack(data);
        return {
            version: tx.version,
            inputCount: tx.inputs.length,
            outputCount: tx.outputs.length,
            cellDeps: tx.cellDeps,
            headerDeps: tx.headerDeps,
            witnesses: tx.witnesses
        };
    } catch (error) {
        console.error('Failed to parse transaction:', error);
        return null;
    }
}

// Build transaction
function buildTransaction(config) {
    const tx = {
        version: config.version || 0,
        cellDeps: config.cellDeps || [],
        headerDeps: config.headerDeps || [],
        inputs: config.inputs || [],
        outputs: config.outputs || [],
        outputsData: config.outputsData || [],
        witnesses: config.witnesses || []
    };
    
    return blockchain.Transaction.pack(tx);
}
```

### Working with Complex Types

```javascript
// Script construction
function createScript(codeHash, hashType, args) {
    return {
        codeHash: new Uint8Array(codeHash),
        hashType: hashType,
        args: new Uint8Array(args)
    };
}

// Cell output with optional type script
function createCellOutput(capacity, lock, type = null) {
    return {
        capacity: capacity,
        lock: lock,
        type: type
    };
}

// Complete example: Simple transfer
function createTransferTransaction(inputs, outputs, witnesses) {
    const transaction = {
        version: 0,
        cellDeps: [
            {
                outPoint: {
                    txHash: new Uint8Array(32), // secp256k1 dep
                    index: 0
                },
                depType: 0 // DepGroup
            }
        ],
        headerDeps: [],
        inputs: inputs.map(input => ({
            since: 0,
            previousOutput: {
                txHash: new Uint8Array(input.txHash),
                index: input.index
            }
        })),
        outputs: outputs,
        outputsData: outputs.map(() => new Uint8Array(0)),
        witnesses: witnesses
    };
    
    return blockchain.Transaction.pack(transaction);
}
```

## Advanced Patterns

### Schema Versioning and Migration

```rust
// Version-aware parsing
#[derive(Debug)]
enum TokenInfo {
    V1(TokenInfoV1),
    V2(TokenInfoV2),
}

impl TokenInfo {
    fn from_slice(data: &[u8]) -> Result<Self, VerificationError> {
        // Try V2 first (latest version)
        if let Ok(v2) = TokenInfoV2::from_slice(data) {
            return Ok(TokenInfo::V2(v2));
        }
        
        // Fall back to V1
        if let Ok(v1) = TokenInfoV1::from_slice(data) {
            return Ok(TokenInfo::V1(v1));
        }
        
        Err(VerificationError::HeaderIsBroken)
    }
    
    fn migrate_to_v2(self) -> TokenInfoV2 {
        match self {
            TokenInfo::V1(v1) => {
                TokenInfoV2::new_builder()
                    .name(v1.name())
                    .symbol(v1.symbol()) 
                    .decimals(v1.decimals())
                    // New fields get default values
                    .description(None::<Bytes>.pack())
                    .icon_url(None::<Bytes>.pack())
                    .website(None::<Bytes>.pack())
                    .build()
            }
            TokenInfo::V2(v2) => v2,
        }
    }
}
```

### Custom Serialization Helpers

```rust
// Helper traits for common patterns
pub trait PackHelper<T> {
    fn pack(self) -> T;
}

impl PackHelper<Uint64> for u64 {
    fn pack(self) -> Uint64 {
        Uint64::from_slice(&self.to_le_bytes()).unwrap()
    }
}

impl PackHelper<Byte32> for [u8; 32] {
    fn pack(self) -> Byte32 {
        Byte32::from_slice(&self).unwrap()
    }
}

impl PackHelper<Bytes> for Vec<u8> {
    fn pack(self) -> Bytes {
        Bytes::from_slice(&self).unwrap()
    }
}

// Usage with helpers
let capacity = 1000u64.pack();
let hash = [0u8; 32].pack();
let args = vec![1, 2, 3, 4].pack();
```

### Performance Optimization

```rust
// Batch processing with readers
fn process_transactions_batch(tx_data_list: &[&[u8]]) -> Result<Vec<u64>, VerificationError> {
    tx_data_list
        .iter()
        .map(|&tx_bytes| {
            let tx_reader = TransactionReader::from_slice(tx_bytes)?;
            let outputs_reader = tx_reader.outputs();
            
            let mut total_capacity = 0u64;
            for i in 0..outputs_reader.len() {
                let output_reader = outputs_reader.get(i)?;
                let capacity_bytes = output_reader.capacity().raw_data();
                total_capacity += u64::from_le_bytes(
                    capacity_bytes.try_into().unwrap()
                );
            }
            
            Ok(total_capacity)
        })
        .collect()
}

// Streaming parser for large data
struct TransactionStream<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> TransactionStream<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }
    
    fn next_transaction(&mut self) -> Option<Result<TransactionReader<'a>, VerificationError>> {
        if self.offset >= self.data.len() {
            return None;
        }
        
        // Read transaction length (first 4 bytes)
        if self.offset + 4 > self.data.len() {
            return Some(Err(VerificationError::HeaderIsBroken));
        }
        
        let len_bytes = &self.data[self.offset..self.offset + 4];
        let tx_len = u32::from_le_bytes(len_bytes.try_into().unwrap()) as usize;
        
        if self.offset + tx_len > self.data.len() {
            return Some(Err(VerificationError::TotalSizeNotMatch(tx_len, self.data.len() - self.offset)));
        }
        
        let tx_data = &self.data[self.offset..self.offset + tx_len];
        let result = TransactionReader::from_slice(tx_data);
        
        self.offset += tx_len;
        Some(result)
    }
}
```

## Testing and Validation

### Comprehensive Test Suite

```rust
#[cfg(test)]
mod molecule_api_tests {
    use super::*;
    use proptest::prelude::*;
    
    #[test]
    fn test_basic_roundtrip() {
        let original_data = vec![1, 2, 3, 4, 5];
        let bytes = Bytes::from_slice(&original_data).unwrap();
        let roundtrip = bytes.raw_data().to_vec();
        assert_eq!(original_data, roundtrip);
    }
    
    #[test]
    fn test_reader_entity_equivalence() {
        let tx = create_sample_transaction();
        let tx_bytes = tx.as_bytes();
        
        let tx_reader = TransactionReader::from_slice(&tx_bytes).unwrap();
        let tx_from_reader = tx_reader.to_entity();
        
        assert_eq!(tx.as_bytes(), tx_from_reader.as_bytes());
    }
    
    proptest! {
        #[test]
        fn test_capacity_serialization(capacity: u64) {
            let capacity_packed = capacity.to_le_bytes();
            let uint64_mol = Uint64::from_slice(&capacity_packed).unwrap();
            let reconstructed = u64::from_le_bytes(
                uint64_mol.raw_data().try_into().unwrap()
            );
            prop_assert_eq!(capacity, reconstructed);
        }
    }
}
```

## CKB-Specific Molecule Schemas

### Common CKB Types

```molecule
// blockchain.mol - Core CKB types
array Uint32 [byte; 4];
array Uint64 [byte; 8];
array Uint128 [byte; 16];
array Byte32 [byte; 32];
array Uint256 [byte; 32];

vector Bytes <byte>;
option BytesOpt (Bytes);

vector BytesVec <Bytes>;
vector Byte32Vec <Byte32>;

table Script {
    code_hash:      Byte32,
    hash_type:      byte,
    args:           Bytes,
}

option ScriptOpt (Script);

struct OutPoint {
    tx_hash:        Byte32,
    index:          Uint32,
}

struct CellInput {
    since:          Uint64,
    previous_output: OutPoint,
}

table CellOutput {
    capacity:       Uint64,
    lock:           Script,
    type_:          ScriptOpt,
}

struct CellDep {
    out_point:      OutPoint,
    dep_type:       byte,
}

table Transaction {
    version:        Uint32,
    cell_deps:      CellDepVec,
    header_deps:    Byte32Vec,
    inputs:         CellInputVec,
    outputs:        CellOutputVec,
    outputs_data:   BytesVec,
    witnesses:      BytesVec,
}

table WitnessArgs {
    lock:           BytesOpt,
    input_type:     BytesOpt, 
    output_type:    BytesOpt,
}
```

### Token Schemas

```molecule
// Simple UDT amount (16 bytes)
array Uint128 [byte; 16];

// xUDT Extension
table ExtensionScript {
    code_hash:      Byte32,
    hash_type:      byte,
    args:           Bytes,
}

// CoTA NFT schemas
table CotaNFTInfo {
    configure:      byte,
    state:          byte,
    characteristic: Bytes,
}

table CotaNFTId {
    smt_root:       Byte32,
    token_index:    Uint16,
}
```

### Omnilock Schemas

```molecule
// omni_lock.mol
array Identity [byte; 21];

struct Auth {
    flag:           byte,
    sign_data:      Identity,
}

vector AuthVec <Auth>;

table OmniLockWitnessLock {
    signature:      BytesOpt,
    omni_identity:  IdentityOpt,
    preimage:       BytesOpt,
}
```

### Spore Protocol Schemas

```molecule
// spore.mol
table SporeData {
    content_type:   Bytes,
    content:        Bytes,
    cluster_id:     BytesOpt,
}

table ClusterData {
    name:           Bytes,
    description:    Bytes,
}
```

## Schema Design Patterns

### Version Evolution

```molecule
// v1 schema
table TokenInfoV1 {
    name:           Bytes,
    symbol:         Bytes,
    decimals:       byte,
}

// v2 schema with new fields
table TokenInfoV2 {
    name:           Bytes,
    symbol:         Bytes,
    decimals:       byte,
    description:    BytesOpt,  // New optional field
    icon_url:       BytesOpt,  // New optional field
    website:        BytesOpt,  // New optional field
}
```

### Compact Storage

```molecule
// Bit-packed flags
struct ConfigFlags {
    data:           byte,  // 8 boolean flags in 1 byte
}

// Fixed-size arrays for efficiency
array Address [byte; 20];
array TokenId [byte; 16];

// Compact timestamp (4 bytes instead of 8)
array Timestamp [byte; 4];
```

### Union Types for Flexibility

```molecule
union Message {
    Request,
    Response,
    Notification,
}

table Request {
    id:             Uint32,
    method:         Bytes,
    params:         Bytes,
}

table Response {
    id:             Uint32,
    result:         BytesOpt,
    error:          BytesOpt,
}
```

## Integration with CKB Scripts

### In Smart Contracts (C)

```c
#include "blockchain.h"

int validate_script_args(const uint8_t* args, size_t args_len) {
    // Validate minimum length for lock hash
    if (args_len < 32) {
        return ERROR_INVALID_ARGS;
    }
    
    // Parse structured args if present
    if (args_len > 32) {
        mol_seg_t config_seg;
        config_seg.ptr = args + 32;
        config_seg.size = args_len - 32;
        
        if (MolReader_ConfigData_verify(&config_seg, false) != MOL_OK) {
            return ERROR_ENCODING;
        }
    }
    
    return CKB_SUCCESS;
}
```

### In Transaction Building (Rust)

```rust
use ckb_types::{molecule::prelude::*, packed};

fn build_dao_deposit_data() -> packed::Bytes {
    // DAO deposit data is 8 bytes of zeros
    let deposit_data = vec![0u8; 8];
    packed::Bytes::new_builder()
        .extend(deposit_data.into_iter().map(Into::into))
        .build()
}

fn parse_dao_data(data: &[u8]) -> Result<u64, Error> {
    if data.len() != 8 {
        return Err(Error::InvalidData);
    }
    Ok(u64::from_le_bytes(data.try_into().unwrap()))
}
```

## Performance Best Practices

1. **Use Readers for Read-Only Access**: Avoid converting to Entity unless necessary.
2. **Batch Operations**: Process multiple items in single iteration.
3. **Pre-allocate Builders**: Reuse builders when constructing multiple similar objects.
4. **Validate Once**: Perform validation at boundaries, trust validated data internally.
5. **Schema Optimization**: Use fixed arrays instead of vectors for known-size data.

## Common Pitfalls

1. **Endianness**: All numeric types use little-endian encoding.
2. **Option Handling**: Check `is_some()` before calling `to_opt()`.
3. **Vector Bounds**: Always check length before indexing.
4. **Memory Allocation**: Be mindful in no_std environments.
5. **Schema Evolution**: Add new fields as optional for backward compatibility.

The Molecule API provides powerful, type-safe serialization across multiple languages. Use the Reader API for performance-critical parsing, Entity API for owned data manipulation, and Builder API for constructing complex structures. Always handle verification errors appropriately and consider using zero-copy patterns when processing large amounts of data.