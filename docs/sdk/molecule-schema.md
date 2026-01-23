## Description

Master Molecule schema development for type-safe CKB data structures. Learn schema language syntax, design patterns, hierarchical organization, extensible schema evolution, modular development, and automated code generation workflows. Covers production patterns like CKBFS, testing strategies, performance optimization, and best practices for robust blockchain application development.

## Overview

Molecule schema development is fundamental to CKB blockchain programming. Schemas define data structures for transactions, cells, and script communication using Molecule's type-safe serialization format. This guide covers development patterns, best practices, and workflows for building robust Molecule schemas.

## Schema Language Fundamentals

### Basic Syntax

```molecule
// Comments use double slashes
/* Multi-line comments
   are also supported */

// Primitive types (built-in)
array Byte32 [byte; 32];
array Hash [byte; 32];
array Uint64 [byte; 8];

// Vector types (dynamic length)
vector Bytes <byte>;
vector BytesVec <Bytes>;

// Struct types (fixed layout)
struct OutPoint {
    tx_hash: Byte32,
    index: Uint32,
}

// Table types (flexible layout)
table Script {
    code_hash: Byte32,
    hash_type: byte,
    args: Bytes,
}

// Union types (tagged variants)
union ScriptOpt {
    Script,
}

// Option types (nullable)
option BytesOpt (Bytes);
```

### Naming Conventions

```molecule
// Types: PascalCase
table TransactionView { ... }
struct CellInput { ... }
union WitnessArgs { ... }

// Fields: snake_case
table Cell {
    cell_output: CellOutput,
    output_data: Bytes,
    out_point: OutPoint,
}

// Arrays/Vectors: Descriptive + Type suffix
vector ScriptVec <Script>;
array Pubkey [byte; 33];
vector TransactionVec <Transaction>;
```

## Schema Design Patterns

### Core Data Structure Pattern

Design schemas that mirror CKB's fundamental concepts:

```molecule
// CKB Transaction Schema
array ProposalShortId [byte; 10];

struct OutPoint {
    tx_hash: Byte32,
    index: Uint32,
}

struct CellInput {
    since: Uint64,
    previous_output: OutPoint,
}

struct CellOutput {
    capacity: Uint64,
    lock: Script,
    type_: ScriptOpt,
}

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

### Hierarchical Schema Pattern

Build complex schemas from simple components:

```molecule
// Base components
struct Point {
    x: Uint32,
    y: Uint32,
}

struct Color {
    r: byte,
    g: byte,
    b: byte,
    a: byte,
}

// Composite types
table Shape {
    position: Point,
    color: Color,
    metadata: Bytes,
}

// Collections
vector ShapeVec <Shape>;

// Top-level container
table Canvas {
    width: Uint32,
    height: Uint32,
    shapes: ShapeVec,
    background: Color,
}
```

### Extensible Schema Pattern

Design for future compatibility:

```molecule
// Version 1: Basic structure
table TokenInfoV1 {
    name: Bytes,
    symbol: Bytes,
    decimals: byte,
}

// Version 2: Extended structure (backwards compatible)
table TokenInfoV2 {
    name: Bytes,
    symbol: Bytes,
    decimals: byte,
    // New fields at end maintain compatibility
    description: BytesOpt,
    icon_url: BytesOpt,
    website: BytesOpt,
}

// Versioned union for evolution
union TokenInfo {
    TokenInfoV1,
    TokenInfoV2,
}
```

### Script Communication Pattern

Define interfaces between scripts:

```molecule
// Script arguments structure
table LockArgs {
    pubkey_hash: Byte20,
    signature_threshold: byte,
    timeout: Uint64,
}

// Script witness data
table LockWitness {
    signature: Bytes,
    proof: BytesOpt,
}

// Script communication via cell data
table ScriptMessage {
    operation: byte,
    payload: Bytes,
    nonce: Uint64,
}
```

## Modular Schema Development

### Import System

```molecule
// Base schema file: types.mol
array Byte32 [byte; 32];
array Uint64 [byte; 8];

struct OutPoint {
    tx_hash: Byte32,
    index: Uint32,
}

// Extended schema file: transaction.mol
import types;

table Transaction {
    inputs: CellInputVec,
    outputs: CellOutputVec,
    // ... other fields
}

// Application schema: token.mol  
import types;
import transaction;

table TokenTransfer {
    base_tx: Transaction,
    token_type: Byte32,
    amounts: Uint64Vec,
}
```

### Schema Organization

```
schemas/
├── base/
│   ├── primitives.mol      # Basic types and arrays
│   ├── blockchain.mol      # Core CKB structures
│   └── scripts.mol         # Common script types
├── protocols/
│   ├── sudt.mol           # Simple UDT schemas
│   ├── xudt.mol           # Extensible UDT schemas
│   └── cota.mol           # CoTA NFT schemas
├── applications/
│   ├── dex.mol            # DEX-specific schemas
│   ├── lending.mol        # Lending protocol schemas
│   └── governance.mol     # DAO governance schemas
└── tests/
    ├── fixtures.mol       # Test data structures
    └── mock.mol          # Mock objects for testing
```

## Code Generation Workflow

### Development Setup

```bash
# Install moleculec compiler
cargo install moleculec

# Project structure
project/
├── schemas/
│   ├── token.mol
│   └── transaction.mol
├── bindings/
│   ├── rust/
│   ├── c/
│   └── javascript/
├── Makefile
└── moleculec.toml
```

### Configuration File

```toml
# moleculec.toml
[package]
name = "my-schemas"
version = "0.1.0"

[bindings.rust]
output_dir = "bindings/rust"
derive_traits = ["Debug", "Clone", "PartialEq"]

[bindings.c]
output_dir = "bindings/c"
include_guards = true

[bindings.javascript]
output_dir = "bindings/javascript"
module_type = "es6"
```

### Build Automation

```makefile
# Makefile
SCHEMA_DIR := schemas
RUST_DIR := bindings/rust
C_DIR := bindings/c
JS_DIR := bindings/javascript

.PHONY: all rust c javascript clean

all: rust c javascript

rust:
	moleculec --language rust \
		--schema-file $(SCHEMA_DIR)/*.mol \
		--output-dir $(RUST_DIR)

c:
	moleculec --language c \
		--schema-file $(SCHEMA_DIR)/*.mol \
		--output-dir $(C_DIR) \
		--include-guards

javascript:
	moleculec --language javascript \
		--schema-file $(SCHEMA_DIR)/*.mol \
		--output-dir $(JS_DIR) \
		--module-type es6

clean:
	rm -rf $(RUST_DIR)/* $(C_DIR)/* $(JS_DIR)/*

# Development workflow
dev: clean all test

test:
	cd $(RUST_DIR) && cargo test
	cd tests && npm test
```

### Integration with Build Systems

```rust
// build.rs for Rust projects
use std::process::Command;

fn main() {
    // Regenerate bindings when schemas change
    println!("cargo:rerun-if-changed=schemas/");
    
    let output = Command::new("moleculec")
        .args(&[
            "--language", "rust",
            "--schema-file", "schemas/*.mol", 
            "--output-dir", "src/generated"
        ])
        .output()
        .expect("Failed to run moleculec");
        
    if !output.status.success() {
        panic!("moleculec failed: {}", String::from_utf8_lossy(&output.stderr));
    }
}
```

### 7. Production Schema Patterns (CKBFS Example)

Real-world schema design from the ckbfs-types repository demonstrates best practices:

```molecule
// File system metadata schema
array Uint32 [byte; 4];
array Byte32 [byte; 32];
vector Bytes <byte>;
vector Indexes <Uint32>;
option Uint32Opt (Uint32);

// File metadata with relationships
table BackLink {
    indexes: Indexes,        // File location indexes
    checksum: Uint32,        // File integrity checksum
    tx_hash: Byte32,         // Transaction reference
}

vector BackLinkVec <BackLink>;

// Main file data structure
table CKBFSData {
    indexes: Indexes,        // File chunk locations
    checksum: Uint32,        // Overall file checksum
    content_type: Bytes,     // MIME type
    filename: Bytes,         // Original filename
    backlinks: BackLinkVec,  // File relationships
}
```

**Rust Implementation with Type Safety:**

```rust
#![no_std]
extern crate alloc;

use alloc::{string::String, vec::Vec};
use molecule::prelude::*;

// Native Rust types for ergonomic usage
#[derive(Debug, Clone)]
pub struct CKBFSDataNative {
    pub indexes: Vec<u32>,
    pub checksum: u32,
    pub content_type: String,
    pub filename: String,
    pub backlinks: Vec<BackLinkNative>,
}

#[derive(Debug, Clone)]
pub struct BackLinkNative {
    pub indexes: Vec<u32>,
    pub checksum: u32,
    pub tx_hash: [u8; 32],
}

// Bidirectional conversion patterns
impl From<CKBFSDataNative> for CKBFSData {
    fn from(native: CKBFSDataNative) -> Self {
        // Convert Vec<u32> to Molecule Indexes
        let indexes = Indexes::new_builder()
            .extend(native.indexes.into_iter().map(|i| i.pack()))
            .build();
            
        // Convert String to Molecule Bytes
        let content_type = Bytes::new_builder()
            .set(native.content_type.into_bytes().into_iter().map(Into::into).collect())
            .build();
            
        let filename = Bytes::new_builder()
            .set(native.filename.into_bytes().into_iter().map(Into::into).collect())
            .build();
            
        // Convert backlinks
        let backlinks = BackLinkVec::new_builder()
            .extend(native.backlinks.into_iter().map(BackLink::from))
            .build();
            
        CKBFSData::new_builder()
            .indexes(indexes)
            .checksum(native.checksum.pack())
            .content_type(content_type)
            .filename(filename)
            .backlinks(backlinks)
            .build()
    }
}

impl From<CKBFSData> for CKBFSDataNative {
    fn from(molecule: CKBFSData) -> Self {
        // Extract and convert back to native types
        let indexes: Vec<u32> = molecule.indexes().into_iter()
            .map(|i| i.unpack())
            .collect();
            
        let content_type = String::from_utf8(
            molecule.content_type().raw_data().to_vec()
        ).expect("Invalid UTF-8 in content_type");
        
        let filename = String::from_utf8(
            molecule.filename().raw_data().to_vec()
        ).expect("Invalid UTF-8 in filename");
        
        let backlinks: Vec<BackLinkNative> = molecule.backlinks().into_iter()
            .map(BackLinkNative::from)
            .collect();
            
        CKBFSDataNative {
            indexes,
            checksum: molecule.checksum().unpack(),
            content_type,
            filename,
            backlinks,
        }
    }
}

// Usage pattern for blockchain applications
impl CKBFSDataNative {
    pub fn new_file(filename: String, content_type: String, checksum: u32) -> Self {
        Self {
            indexes: vec![1], // First chunk
            checksum,
            content_type,
            filename,
            backlinks: vec![], // No backlinks for new files
        }
    }
    
    pub fn with_backlink(mut self, previous_file: BackLinkNative) -> Self {
        self.backlinks.push(previous_file);
        self
    }
    
    pub fn serialize(&self) -> Vec<u8> {
        let molecule_data: CKBFSData = self.clone().into();
        molecule_data.as_bytes().to_vec()
    }
    
    pub fn deserialize(data: &[u8]) -> Result<Self, molecule::error::VerificationError> {
        let molecule_data = CKBFSData::from_slice(data)?;
        Ok(molecule_data.into())
    }
}
```

## Testing and Validation Strategies

### Schema Validation Tests

```rust
#[cfg(test)]
mod schema_tests {
    use super::*;
    use molecule::prelude::*;
    
    #[test]
    fn test_transaction_roundtrip() {
        let original = TransactionBuilder::default()
            .version(0u32.pack())
            .cell_deps(CellDepVec::new_builder().build())
            .header_deps(Byte32Vec::new_builder().build())
            .inputs(CellInputVec::new_builder().build())
            .outputs(CellOutputVec::new_builder().build())
            .outputs_data(BytesVec::new_builder().build())
            .witnesses(BytesVec::new_builder().build())
            .build();
            
        let bytes = original.as_bytes();
        let decoded = Transaction::from_slice(&bytes).unwrap();
        
        assert_eq!(original.as_bytes(), decoded.as_bytes());
    }
    
    #[test]
    fn test_schema_compatibility() {
        // Test that new schema versions can read old data
        let v1_data = create_v1_token_info();
        let v2_reader = TokenInfoV2::from_slice(&v1_data.as_bytes());
        
        assert!(v2_reader.is_ok());
        // V2 should have default values for new fields
    }
    
    #[test]
    fn test_invalid_data_handling() {
        let invalid_bytes = vec![0x00, 0x01, 0x02]; // Truncated data
        let result = Transaction::from_slice(&invalid_bytes);
        
        assert!(result.is_err());
        // Should fail gracefully with verification error
    }
}
```

### Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_amount_serialization(amount: u128) {
        let bytes = amount.to_le_bytes();
        let molecule_amount = Uint128::from_slice(&bytes).unwrap();
        let reconstructed = u128::from_le_bytes(molecule_amount.raw_data().try_into().unwrap());
        
        prop_assert_eq!(amount, reconstructed);
    }
    
    #[test]
    fn test_script_args_length(args in prop::collection::vec(any::<u8>(), 0..1000)) {
        let bytes = Bytes::from(args.clone());
        let script = ScriptBuilder::default()
            .code_hash([0u8; 32].pack())
            .hash_type(0u8.into())
            .args(bytes.pack())
            .build();
            
        prop_assert_eq!(script.args().raw_data(), args.as_slice());
    }
}
```

### Integration Testing

```rust
// Integration test with actual CKB node
#[tokio::test]
async fn test_transaction_submission() {
    let ckb_client = CkbRpcClient::new("http://localhost:8114");
    
    // Build transaction using generated schema
    let tx = TransactionBuilder::default()
        .version(0u32.pack())
        .cell_deps(build_cell_deps())
        .inputs(build_inputs())
        .outputs(build_outputs())
        .outputs_data(build_outputs_data())
        .witnesses(build_witnesses())
        .build();
    
    // Verify schema produces valid transaction
    let tx_view = TransactionView::from(tx);
    let result = ckb_client.send_transaction(tx_view.data()).await;
    
    assert!(result.is_ok());
}
```

## Performance Optimization

### Zero-Copy Patterns

```rust
use molecule::prelude::*;

// Efficient: Direct slice access without copying
fn process_transaction_efficiently(tx_bytes: &[u8]) -> Result<u64, Error> {
    let tx = Transaction::from_slice(tx_bytes)?;
    
    // Zero-copy access to outputs
    let mut total_capacity = 0u64;
    for i in 0..tx.outputs().len() {
        let output = tx.outputs().get(i).unwrap();
        let capacity = u64::from_le_bytes(
            output.capacity().raw_data().try_into().unwrap()
        );
        total_capacity += capacity;
    }
    
    Ok(total_capacity)
}

// Inefficient: Unnecessary conversion to owned types
fn process_transaction_inefficiently(tx_bytes: &[u8]) -> Result<u64, Error> {
    let tx = Transaction::from_slice(tx_bytes)?;
    
    // This creates unnecessary allocations
    let outputs: Vec<CellOutput> = tx.outputs()
        .into_iter()
        .collect();
        
    let mut total_capacity = 0u64;
    for output in outputs {
        let capacity = u64::from_le_bytes(
            output.capacity().raw_data().try_into().unwrap()
        );
        total_capacity += capacity;
    }
    
    Ok(total_capacity)
}
```

### Memory Layout Optimization

```molecule
// Optimized: Frequent fields first, aligned access
table OptimizedCell {
    capacity: Uint64,        // 8 bytes, frequently accessed
    lock_hash: Byte32,       // 32 bytes, used for indexing  
    type_hash: Byte32Opt,    // 32 bytes optional, less frequent
    data: Bytes,             // Variable length, least frequent
}

// Less optimal: Variable fields interspersed
table UnoptimizedCell {
    data: Bytes,             // Variable length disrupts cache efficiency
    capacity: Uint64,        
    metadata: BytesOpt,      // Another variable field
    lock_hash: Byte32,
}
```

## Error Handling Patterns

### Comprehensive Error Types

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum SchemaError {
    InvalidLength { expected: usize, actual: usize },
    MissingRequiredField(String),
    InvalidEnumVariant(u8),
    VersionMismatch { expected: u32, actual: u32 },
    ValidationFailed(String),
}

impl From<molecule::error::VerificationError> for SchemaError {
    fn from(err: molecule::error::VerificationError) -> Self {
        match err {
            molecule::error::VerificationError::TotalSizeNotMatch(expected, actual) => {
                SchemaError::InvalidLength { expected, actual }
            }
            _ => SchemaError::ValidationFailed(format!("Molecule error: {:?}", err)),
        }
    }
}
```

### Defensive Validation

```rust
pub fn validate_transaction_schema(tx_bytes: &[u8]) -> Result<(), SchemaError> {
    // Basic molecule structure validation
    let tx = Transaction::from_slice(tx_bytes)
        .map_err(SchemaError::from)?;
    
    // Business logic validation
    if tx.inputs().is_empty() {
        return Err(SchemaError::ValidationFailed(
            "Transaction must have at least one input".to_string()
        ));
    }
    
    if tx.outputs().is_empty() {
        return Err(SchemaError::ValidationFailed(
            "Transaction must have at least one output".to_string()  
        ));
    }
    
    // Validate outputs_data length matches outputs length
    if tx.outputs().len() != tx.outputs_data().len() {
        return Err(SchemaError::ValidationFailed(
            "Outputs and outputs_data length mismatch".to_string()
        ));
    }
    
    Ok(())
}
```

## Best Practices Summary

### Schema Design

1. **Start Simple**: Begin with minimal schemas, extend gradually
2. **Use Hierarchical Design**: Build complex types from simple components  
3. **Plan for Evolution**: Add optional fields at the end for compatibility
4. **Consistent Naming**: Follow established conventions across all schemas
5. **Document Schemas**: Include comments explaining field purposes

### Development Workflow

1. **Version Control Schemas**: Track schema changes carefully
2. **Automate Code Generation**: Use build scripts for consistency
3. **Test Thoroughly**: Validate both happy path and error cases
4. **Performance Testing**: Profile serialization/deserialization costs
5. **Cross-Language Testing**: Ensure bindings work correctly across languages

### Production Deployment

1. **Schema Versioning**: Plan migration strategies for schema changes
2. **Backwards Compatibility**: Maintain compatibility with existing data
3. **Monitoring**: Track schema validation errors in production
4. **Documentation**: Keep schema documentation up-to-date
5. **Security Review**: Validate schemas can't cause security issues

Molecule schema development is essential for building robust CKB applications. Following these patterns ensures type-safe, efficient, and maintainable data structures that integrate seamlessly with CKB's blockchain infrastructure.