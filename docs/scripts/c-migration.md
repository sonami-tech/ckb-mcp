## Description

Migrate existing CKB smart contracts from C to Rust for improved safety, maintainability, and developer experience. Step-by-step migration patterns for UDT tokens, signature verification scripts, and advanced cryptographic operations. Error handling transformation, memory management improvements, iteration patterns, and testing strategies for contract modernization.

## Overview

Migrate existing CKB scripts from C to Rust. Rust is now the recommended language for CKB script development due to its memory safety, rich ecosystem, and excellent tooling support through `ckb-std`.

## Why Migrate from C to Rust?

### Advantages of Rust

1. **Memory Safety**: Eliminates buffer overflows, null pointer dereferences, and memory leaks
2. **Type Safety**: Strong type system prevents many runtime errors
3. **Rich Ecosystem**: Access to crates.io libraries (when compatible)
4. **Better Tooling**: Excellent IDE support, debugging, and testing
5. **Maintainability**: Cleaner, more readable code with better error handling
6. **Performance**: Zero-cost abstractions with optimizations

### Legacy C Limitations

1. **Manual Memory Management**: Prone to leaks and corruption
2. **Buffer Overflow Risks**: Common source of security vulnerabilities
3. **Limited Error Handling**: Difficult to propagate and handle errors cleanly
4. **Verbose Syscall Management**: Manual buffer size calculations and checks

## Migration Patterns

### 1. Simple UDT: C to Rust

**Original C Implementation** (from `ckb-miscellaneous-scripts/c/simple_udt.c`):

```c
#include "ckb_syscalls.h"
#include "blockchain.h"

#define BLAKE2B_BLOCK_SIZE 32
#define SCRIPT_SIZE 32768
#define ERROR_ARGUMENTS_LEN -1
#define ERROR_ENCODING -2
#define ERROR_AMOUNT -52

typedef unsigned __int128 uint128_t;

int main() {
    // Load script and extract args
    unsigned char script[SCRIPT_SIZE];
    uint64_t len = SCRIPT_SIZE;
    int ret = ckb_load_script(script, &len, 0);
    if (ret != CKB_SUCCESS) {
        return ERROR_SYSCALL;
    }
    
    // Parse molecule data
    mol_seg_t script_seg;
    script_seg.ptr = (uint8_t *)script;
    script_seg.size = len;
    
    if (MolReader_Script_verify(&script_seg, false) != MOL_OK) {
        return ERROR_ENCODING;
    }
    
    mol_seg_t args_seg = MolReader_Script_get_args(&script_seg);
    mol_seg_t args_bytes_seg = MolReader_Bytes_raw_bytes(&args_seg);
    if (args_bytes_seg.size != BLAKE2B_BLOCK_SIZE) {
        return ERROR_ARGUMENTS_LEN;
    }
    
    // Check owner mode
    int owner_mode = 0;
    size_t i = 0;
    while (1) {
        uint8_t buffer[BLAKE2B_BLOCK_SIZE];
        uint64_t len = BLAKE2B_BLOCK_SIZE;
        ret = ckb_checked_load_cell_by_field(buffer, &len, 0, i, 
                                           CKB_SOURCE_INPUT, CKB_CELL_FIELD_LOCK_HASH);
        if (ret == CKB_INDEX_OUT_OF_BOUND) {
            break;
        }
        if (ret != CKB_SUCCESS) {
            return ret;
        }
        
        if (memcmp(args_bytes_seg.ptr, buffer, BLAKE2B_BLOCK_SIZE) == 0) {
            owner_mode = 1;
            break;
        }
        i += 1;
    }
    
    if (owner_mode) {
        return 0;
    }
    
    // Token conservation logic...
    return 0;
}
```

**Modern Rust Implementation**:

```rust
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{bytes::Bytes, prelude::*},
    high_level::{load_script, load_cell_lock_hash, load_cell_data, QueryIter},
};
use core::result::Result;

// Modern error handling with enums
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Error {
    InvalidArgs = -1,
    Encoding = -2,
    Amount = -52,
}

const LOCK_HASH_LEN: usize = 32;
const UDT_DATA_LEN: usize = 16;

pub fn main() -> Result<(), Error> {
    // Load script with safe error handling
    let script = load_script().map_err(|_| Error::Encoding)?;
    let args: Bytes = script.args().unpack();
    
    // Type-safe argument validation
    if args.len() != LOCK_HASH_LEN {
        return Err(Error::InvalidArgs);
    }
    
    // Check owner mode with iterator (no manual index management)
    if check_owner_mode(&args)? {
        return Ok(()); // Owner can do anything
    }
    
    // Validate token conservation
    validate_token_conservation()
}

fn check_owner_mode(args: &Bytes) -> Result<bool, Error> {
    // Safe iteration with QueryIter - no buffer management needed
    let is_owner = QueryIter::new(load_cell_lock_hash, Source::Input)
        .any(|lock_hash| args[..] == lock_hash[..]);
    
    Ok(is_owner)
}

fn validate_token_conservation() -> Result<(), Error> {
    let input_amount = calculate_token_amount(Source::GroupInput)?;
    let output_amount = calculate_token_amount(Source::GroupOutput)?;
    
    // Prevent inflation (allow burning)
    if output_amount > input_amount {
        return Err(Error::Amount);
    }
    
    Ok(())
}

fn calculate_token_amount(source: Source) -> Result<u128, Error> {
    let mut total = 0u128;
    
    // Safe iteration without manual buffer management
    for data in QueryIter::new(load_cell_data, source) {
        if data.len() < UDT_DATA_LEN {
            return Err(Error::Encoding);
        }
        
        // Safe array access with bounds checking
        let mut buffer = [0u8; UDT_DATA_LEN];
        buffer.copy_from_slice(&data[0..UDT_DATA_LEN]);
        let amount = u128::from_le_bytes(buffer);
        
        // Safe arithmetic with overflow checking
        total = total.checked_add(amount).ok_or(Error::Amount)?;
    }
    
    Ok(total)
}
```

### 2. Secp256k1 Lock Script: C to Rust

**Original C Pattern**:

```c
#include "blake2b.h"
#include "ckb_syscalls.h"
#include "secp256k1_helper.h"

#define BLAKE160_SIZE 20
#define PUBKEY_SIZE 33
#define TEMP_SIZE 32768
#define RECID_INDEX 64
#define SIGNATURE_SIZE 65

int main() {
    // Manual buffer management
    uint8_t script[TEMP_SIZE];
    uint64_t len = TEMP_SIZE;
    int ret = ckb_load_script(script, &len, 0);
    if (ret != CKB_SUCCESS) {
        return ret;
    }
    
    // Manual witness loading
    uint8_t witness[TEMP_SIZE];
    len = TEMP_SIZE;
    ret = ckb_load_witness(witness, &len, 0, 0, CKB_SOURCE_GROUP_INPUT);
    if (ret != CKB_SUCCESS) {
        return ret;
    }
    
    // Manual signature extraction
    mol_seg_t witness_seg;
    witness_seg.ptr = witness;
    witness_seg.size = len;
    
    // Complex witness parsing...
    // Manual secp256k1 verification...
    // Manual hash comparison...
    
    return 0;
}
```

**Modern Rust Implementation**:

```rust
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{bytes::Bytes, prelude::*},
    high_level::{load_script, load_witness_args, load_tx_hash},
};
use ckb_hash::{Blake2bBuilder, Blake2b};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Error {
    InvalidArgs = -1,
    InvalidWitness = -2,
    InvalidSignature = -3,
}

const BLAKE160_SIZE: usize = 20;
const SIGNATURE_SIZE: usize = 65;

pub fn main() -> Result<(), Error> {
    // Type-safe script loading
    let script = load_script().map_err(|_| Error::InvalidArgs)?;
    let args = script.args().raw_data();
    
    if args.len() != BLAKE160_SIZE {
        return Err(Error::InvalidArgs);
    }
    
    // Safe witness loading with automatic parsing
    let witness_args = load_witness_args(0, Source::GroupInput)
        .map_err(|_| Error::InvalidWitness)?;
    
    let signature = witness_args.lock().to_opt()
        .ok_or(Error::InvalidWitness)?
        .raw_data();
    
    if signature.len() != SIGNATURE_SIZE {
        return Err(Error::InvalidWitness);
    }
    
    // Safe transaction hash loading
    let tx_hash = load_tx_hash().map_err(|_| Error::InvalidArgs)?;
    
    // Verify signature with proper error handling
    verify_signature(&signature, &tx_hash, &args)
}

fn verify_signature(signature: &[u8], message: &[u8], expected_hash: &[u8]) -> Result<(), Error> {
    // Recover public key from signature
    let pubkey = recover_secp256k1_pubkey(signature, message)?;
    
    // Calculate Blake160 hash
    let pubkey_hash = calculate_blake160(&pubkey);
    
    // Safe comparison
    if pubkey_hash[..] != expected_hash[..] {
        return Err(Error::InvalidSignature);
    }
    
    Ok(())
}

fn recover_secp256k1_pubkey(signature: &[u8], message: &[u8]) -> Result<[u8; 33], Error> {
    // Use secp256k1 crate for safe signature verification
    use secp256k1::{recovery::RecoveryId, recovery::RecoverableSignature, Message, Secp256k1};
    
    let secp = Secp256k1::verification_only();
    
    let recovery_id = RecoveryId::from_i32(signature[64] as i32)
        .map_err(|_| Error::InvalidSignature)?;
    
    let sig = RecoverableSignature::from_compact(&signature[0..64], recovery_id)
        .map_err(|_| Error::InvalidSignature)?;
    
    let msg = Message::from_slice(message)
        .map_err(|_| Error::InvalidSignature)?;
    
    let pubkey = secp.recover_ecdsa(&msg, &sig)
        .map_err(|_| Error::InvalidSignature)?;
    
    Ok(pubkey.serialize())
}

fn calculate_blake160(data: &[u8]) -> [u8; 20] {
    let mut hasher = Blake2bBuilder::new(32)
        .personal(b"ckb-default-hash")
        .build();
    hasher.update(data);
    
    let mut hash = [0u8; 32];
    hasher.finalize(&mut hash);
    
    let mut result = [0u8; 20];
    result.copy_from_slice(&hash[0..20]);
    result
}
```

### 3. Advanced Cryptographic Scripts

**BLS Signature Migration**:

```rust
// Modern Rust implementation for BLS signatures
use ckb_std::{
    ckb_constants::Source,
    high_level::{load_witness_args, load_tx_hash, QueryIter},
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Error {
    InvalidWitness = -1,
    InvalidSignature = -2,
    TooManySigners = -3,
}

const BLS_SIGNATURE_SIZE: usize = 96;
const BLS_PUBKEY_SIZE: usize = 48;
const MAX_SIGNERS: usize = 255;

pub fn main() -> Result<(), Error> {
    // Load aggregated signature from witness
    let witness_args = load_witness_args(0, Source::GroupInput)
        .map_err(|_| Error::InvalidWitness)?;
    
    let aggregated_signature = witness_args.lock().to_opt()
        .ok_or(Error::InvalidWitness)?
        .raw_data();
    
    if aggregated_signature.len() != BLS_SIGNATURE_SIZE {
        return Err(Error::InvalidWitness);
    }
    
    // Collect public keys from script args
    let public_keys = collect_public_keys()?;
    
    // Load message to sign
    let message = load_tx_hash().map_err(|_| Error::InvalidWitness)?;
    
    // Verify BLS aggregate signature
    verify_bls_aggregate_signature(&aggregated_signature, &public_keys, &message)
}

fn collect_public_keys() -> Result<Vec<[u8; BLS_PUBKEY_SIZE]>, Error> {
    let mut public_keys = Vec::new();
    
    // Use iterator for safe collection
    for args in QueryIter::new(load_script_args, Source::GroupInput) {
        if args.len() % BLS_PUBKEY_SIZE != 0 {
            return Err(Error::InvalidWitness);
        }
        
        let signer_count = args.len() / BLS_PUBKEY_SIZE;
        if signer_count > MAX_SIGNERS {
            return Err(Error::TooManySigners);
        }
        
        for i in 0..signer_count {
            let start = i * BLS_PUBKEY_SIZE;
            let end = start + BLS_PUBKEY_SIZE;
            
            let mut pubkey = [0u8; BLS_PUBKEY_SIZE];
            pubkey.copy_from_slice(&args[start..end]);
            public_keys.push(pubkey);
        }
    }
    
    Ok(public_keys)
}

fn verify_bls_aggregate_signature(
    signature: &[u8],
    public_keys: &[[u8; BLS_PUBKEY_SIZE]],
    message: &[u8],
) -> Result<(), Error> {
    // BLS signature verification logic
    // This would use a BLS library like blst or bls12_381
    
    #[cfg(feature = "bls")]
    {
        use blst::*;
        
        // Convert to BLS types
        let sig = Signature::from_bytes(signature)
            .map_err(|_| Error::InvalidSignature)?;
        
        let mut pubkeys = Vec::new();
        for pk_bytes in public_keys {
            let pk = PublicKey::from_bytes(pk_bytes)
                .map_err(|_| Error::InvalidSignature)?;
            pubkeys.push(pk);
        }
        
        // Verify aggregate signature
        let result = sig.verify(true, message, b"", &[], &pubkeys, true);
        if result != BLST_ERROR::BLST_SUCCESS {
            return Err(Error::InvalidSignature);
        }
    }
    
    #[cfg(not(feature = "bls"))]
    {
        // Fallback implementation or error
        return Err(Error::InvalidSignature);
    }
    
    Ok(())
}
```

## Migration Checklist

### 1. Setup and Configuration

- [ ] **Create Rust project structure** with proper `Cargo.toml`
- [ ] **Add ckb-std dependency** version 1.0 or later
- [ ] **Configure build target** for `riscv64imac-unknown-none-elf`
- [ ] **Setup no_std environment** with proper heap allocation

```toml
[dependencies]
ckb-std = "1.0"

[profile.release]
overflow-checks = true
opt-level = "s"
lto = true
codegen-units = 1
panic = "abort"
```

### 2. Error Handling Migration

**C Pattern**:
```c
#define ERROR_INVALID_ARGS -1
#define ERROR_ENCODING -2

int ret = some_operation();
if (ret != CKB_SUCCESS) {
    return ERROR_ENCODING;
}
```

**Rust Pattern**:
```rust
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Error {
    InvalidArgs = -1,
    Encoding = -2,
}

impl From<ckb_std::error::SysError> for Error {
    fn from(err: ckb_std::error::SysError) -> Self {
        match err {
            ckb_std::error::SysError::IndexOutOfBound => Error::InvalidArgs,
            _ => Error::Encoding,
        }
    }
}

let result = some_operation()?; // Automatic error propagation
```

### 3. Memory Management Migration

**C Pattern**:
```c
uint8_t buffer[32768];
uint64_t len = 32768;
int ret = ckb_load_cell_data(buffer, &len, 0, 0, CKB_SOURCE_INPUT);
if (ret != CKB_SUCCESS) {
    return ret;
}
```

**Rust Pattern**:
```rust
// Automatic buffer management
let data = load_cell_data(0, Source::Input)?;
// `data` is automatically managed, no buffer overflow risk
```

### 4. Iteration Patterns

**C Pattern**:
```c
size_t i = 0;
while (1) {
    uint8_t buffer[1024];
    uint64_t len = 1024;
    int ret = ckb_load_cell_data(buffer, &len, 0, i, CKB_SOURCE_INPUT);
    if (ret == CKB_INDEX_OUT_OF_BOUND) {
        break;
    }
    if (ret != CKB_SUCCESS) {
        return ret;
    }
    // Process buffer
    i += 1;
}
```

**Rust Pattern**:
```rust
// Safe iteration with automatic bounds checking
for (index, data) in QueryIter::new(load_cell_data, Source::Input).enumerate() {
    // Process data - no manual index management or buffer handling
}
```

### 5. Data Parsing Migration

**C Pattern**:
```c
mol_seg_t script_seg;
script_seg.ptr = (uint8_t *)script;
script_seg.size = len;

if (MolReader_Script_verify(&script_seg, false) != MOL_OK) {
    return ERROR_ENCODING;
}

mol_seg_t args_seg = MolReader_Script_get_args(&script_seg);
```

**Rust Pattern**:
```rust
// Type-safe parsing with automatic validation
let script = load_script()?;
let args = script.args().raw_data();

// Or using Molecule with generated Rust code
use molecule::prelude::*;
let parsed_data = MyDataReader::from_slice(&raw_data)
    .map_err(|_| Error::Encoding)?;
```

### 6. Cryptographic Operations

**C Pattern**:
```c
secp256k1_context *ctx = secp256k1_context_create(SECP256K1_CONTEXT_VERIFY);
secp256k1_ecdsa_recoverable_signature sig;
secp256k1_pubkey pubkey;

int ret = secp256k1_ecdsa_recoverable_signature_parse_compact(ctx, &sig, signature, recid);
if (ret == 0) {
    return ERROR_INVALID_SIGNATURE;
}
```

**Rust Pattern**:
```rust
use secp256k1::{recovery::RecoverableSignature, Secp256k1, Message};

let secp = Secp256k1::verification_only();
let sig = RecoverableSignature::from_compact(&signature[0..64], recovery_id)?;
let msg = Message::from_slice(message)?;
let pubkey = secp.recover_ecdsa(&msg, &sig)?;
```

## Testing Migration

### C Testing (Limited)
```c
// Manual testing with fixed scenarios
int test_basic_functionality() {
    // Manual setup
    // Limited assertions
    return 0;
}
```

### Rust Testing (Comprehensive)
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ckb_tool::{
        ckb_types::{core::TransactionBuilder, packed::*},
        context::Context,
    };

    #[test]
    fn test_token_conservation() {
        let mut context = Context::default();
        let contract_bin = include_bytes!("../build/my-script");
        let out_point = context.deploy_cell(contract_bin.to_vec().into());
        
        let tx = TransactionBuilder::default()
            .input(build_input())
            .output(build_output())
            .build();
        
        let cycles = context.verify_tx(&tx, MAX_CYCLES).expect("pass");
        println!("Consumed {} cycles", cycles);
    }
    
    #[test]
    fn test_error_conditions() {
        // Test various error scenarios
    }
}
```

## Performance Considerations

### Memory Usage
- **Rust**: Stack-allocated by default, explicit heap when needed
- **C**: Manual management, prone to leaks
- **Migration**: Use `Vec` and `String` sparingly, prefer arrays and slices

### Binary Size
- **Rust**: Use `opt-level = "s"`, `lto = true`, `panic = "abort"`
- **C**: Already minimal
- **Migration**: Profile and optimize with `cargo bloat`

### Execution Speed
- **Rust**: Zero-cost abstractions, comparable to C
- **C**: Direct system calls
- **Migration**: Profile with actual transaction data

## Common Pitfalls and Solutions

### 1. Integer Overflow

**C Problem**:
```c
uint128_t total = input_amount + output_amount; // Can overflow silently
```

**Rust Solution**:
```rust
let total = input_amount.checked_add(output_amount)
    .ok_or(Error::Overflow)?; // Explicit overflow handling
```

### 2. Buffer Management

**C Problem**:
```c
uint8_t buffer[1024]; // Fixed size, can overflow
uint64_t len = 1024;
ckb_load_cell_data(buffer, &len, 0, 0, CKB_SOURCE_INPUT);
```

**Rust Solution**:
```rust
let data = load_cell_data(0, Source::Input)?; // Automatic sizing
```

### 3. Error Propagation

**C Problem**:
```c
int ret = operation1();
if (ret != 0) return ret;
ret = operation2();
if (ret != 0) return ret;
// Repetitive error checking
```

**Rust Solution**:
```rust
operation1()?;
operation2()?;
// Automatic error propagation
```

## Migration Timeline

### Phase 1: Direct Translation (1-2 weeks)
1. Convert C logic to Rust 1:1
2. Replace syscalls with ckb-std equivalents
3. Add basic error handling
4. Ensure functionality parity

### Phase 2: Rust Optimization (1 week)
1. Use QueryIter for iterations
2. Implement proper error types
3. Add comprehensive testing
4. Optimize for size and performance

### Phase 3: Advanced Features (Optional)
1. Add support for new features
2. Integrate with Rust ecosystem crates
3. Implement advanced error recovery
4. Add property-based testing

Migrating from C to Rust for CKB scripts provides significant benefits in safety, maintainability, and developer experience. The patterns shown here provide a solid foundation for successful migrations while maintaining or improving performance.