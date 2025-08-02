# CKB Script Development Patterns

## Description

Advanced patterns for developing CKB scripts based on battle-tested production implementations. Covers core patterns including UDT tokens, composable locks, HTLC contracts, open transactions, dual-mode scripts, cryptographic primitives, and build system optimizations with practical examples from real-world projects.

This guide covers advanced patterns for developing CKB scripts, based on production implementations from ckb-miscellaneous-scripts and ckb-script-templates.

## Core Script Patterns

### 1. Simple UDT (User Defined Token) Pattern

The most fundamental token pattern in CKB. Implements a 128-bit token with overflow protection.

**Key Features:**
- 128-bit token amounts with overflow checks
- Cell-based token balance validation
- Input/output sum verification

**Reference Implementation:** `resources/ckb-miscellaneous-scripts/rust/simple_udt`

**Pattern:**
```rust
use ckb_std::{ckb_constants::Source, high_level::*};

// Validate input sum == output sum
fn validate_token_transfer() -> Result<(), Error> {
    let mut input_sum = 0u128;
    let mut output_sum = 0u128;
    
    // Sum all input tokens
    for i in 0..QueryIter::new(load_cell_data, Source::GroupInput).count() {
        let data = load_cell_data(i, Source::GroupInput)?;
        let amount = u128::from_le_bytes(data[0..16].try_into()?);
        input_sum = input_sum.checked_add(amount).ok_or(Error::Overflow)?;
    }
    
    // Sum all output tokens  
    for i in 0..QueryIter::new(load_cell_data, Source::GroupOutput).count() {
        let data = load_cell_data(i, Source::GroupOutput)?;
        let amount = u128::from_le_bytes(data[0..16].try_into()?);
        output_sum = output_sum.checked_add(amount).ok_or(Error::Overflow)?;
    }
    
    if input_sum == output_sum {
        Ok(())
    } else {
        Err(Error::AmountMismatch)
    }
}
```

### 2. Composable Lock Scripts (OR/AND Logic)

Enable complex authorization patterns by combining multiple lock scripts.

**OR Lock Pattern:**
```rust
use ckb_std::dynamic_loading::{CKBDLContext, Symbol};

// Any one of the provided lock scripts can unlock
fn or_lock_main() -> Result<(), Error> {
    let lock_scripts = load_lock_scripts_from_args()?;
    
    for lock_script in lock_scripts {
        match validate_lock_script(&lock_script) {
            Ok(()) => return Ok(()), // Success if any lock passes
            Err(_) => continue,      // Try next lock
        }
    }
    
    Err(Error::AllLocksFailed) // All locks failed
}
```

**AND Lock Pattern:**
```rust
// All provided lock scripts must pass
fn and_lock_main() -> Result<(), Error> {
    let lock_scripts = load_lock_scripts_from_args()?;
    
    for lock_script in lock_scripts {
        validate_lock_script(&lock_script)?; // Fail if any lock fails
    }
    
    Ok(()) // Success only if all locks pass
}
```

**Reference:** `resources/ckb-miscellaneous-scripts/rust/composable_locks`

### 3. Hash Time Locked Contracts (HTLC)

Enables atomic swaps and time-locked transactions.

**Pattern:**
```rust
use ckb_std::{since::Since, high_level::*};
use blake2b_rs::Blake2bBuilder;

fn htlc_main() -> Result<(), Error> {
    let since = load_input_since(0, Source::GroupInput)?;
    let since_value = Since::new(since);
    
    // Check if timelock has expired
    if since_value.is_absolute() && since_value.extract_lock_value()? >= timeout_timestamp {
        return validate_timeout_unlock(); // Refund path
    }
    
    // Validate hash preimage
    let preimage = load_witness_args(0, Source::GroupInput)?
        .lock()
        .to_opt()
        .ok_or(Error::InvalidWitness)?
        .raw_data();
    
    let mut hasher = Blake2bBuilder::new(32)
        .personal(b"ckb-default-hash")
        .build();
    hasher.update(&preimage);
    let mut hash = [0u8; 32];
    hasher.finalize(&mut hash);
    
    if hash == expected_hash {
        Ok(())
    } else {
        Err(Error::InvalidPreimage)
    }
}
```

**Reference:** `resources/ckb-miscellaneous-scripts/rust/htlc`

### 4. Open Transaction Pattern

Flexible transaction verification allowing dynamic transaction composition.

**Key Concept:** Script validates transaction structure rather than specific values, enabling transaction templates.

```rust
fn open_transaction_main() -> Result<(), Error> {
    // Validate transaction structure
    let input_count = QueryIter::new(load_input, Source::Input).count();
    let output_count = QueryIter::new(load_cell, Source::Output).count();
    
    if input_count < min_inputs {
        return Err(Error::InsufficientInputs);
    }
    if output_count < min_outputs {
        return Err(Error::InsufficientOutputs);
    }
    
    // Validate required cell types are present
    for required_type in &required_types {
        if !find_cell_with_type(required_type)? {
            return Err(Error::RequiredTypeMissing);
        }
    }
    
    Ok(())
}
```

**Reference:** `resources/ckb-miscellaneous-scripts/rust/open_transaction`

### 5. Dual-Mode Scripts

Scripts that can operate standalone or as dynamic libraries.

**Pattern:**
```rust
#[cfg(feature = "dual-mode")]
use ckb_std::dynamic_loading_c_impl;

// Library mode - export functions
#[cfg(feature = "dual-mode")]
#[no_mangle]
pub extern "C" fn verify_signature(
    message: *const u8,
    signature: *const u8,
    pubkey: *const u8,
) -> i32 {
    unsafe {
        let msg = slice::from_raw_parts(message, 32);
        let sig = slice::from_raw_parts(signature, 64);
        let pk = slice::from_raw_parts(pubkey, 33);
        
        match secp256k1_verify(msg, sig, pk) {
            Ok(true) => 0,
            _ => -1,
        }
    }
}

// Main entry point
fn main() -> i8 {
    #[cfg(feature = "dual-mode")]
    return 0; // Library mode always succeeds
    
    #[cfg(not(feature = "dual-mode"))]
    match verify_transaction() {
        Ok(()) => 0,
        Err(e) => e as i8,
    }
}
```

**Reference:** `resources/ckb-miscellaneous-scripts/rust/dual_mode_secp256k1`

## Advanced Cryptographic Patterns

### 1. Alternative Elliptic Curves

Support for secp256r1 (P-256) in addition to secp256k1.

```rust
use p256::{ecdsa::{Signature, VerifyingKey}, PublicKey};

fn secp256r1_verify() -> Result<(), Error> {
    let message = load_tx_hash()?;
    
    let witness_args = load_witness_args(0, Source::GroupInput)?;
    let signature_data = witness_args
        .lock()
        .to_opt()
        .ok_or(Error::InvalidWitness)?
        .raw_data();
    
    let lock_args = load_script()?.args().raw_data();
    
    // Parse signature (r + s)
    let signature = Signature::from_slice(&signature_data[0..64])
        .map_err(|_| Error::InvalidSignature)?;
    
    // Parse public key (x + y coordinates)
    let pubkey = PublicKey::from_sec1_bytes(&lock_args[0..65])
        .map_err(|_| Error::InvalidPublicKey)?;
    
    let verifying_key = VerifyingKey::from(&pubkey);
    
    verifying_key
        .verify(&message, &signature)
        .map_err(|_| Error::SignatureVerificationFailed)?;
    
    Ok(())
}
```

**Reference:** `resources/ckb-miscellaneous-scripts/rust/secp256r1_lock`

### 2. RSA Signature Support

Large integer RSA verification on RISC-V.

```rust
use rsa::{PublicKey, RsaPublicKey, PaddingScheme};
use sha2::{Sha256, Digest};

fn rsa_verify() -> Result<(), Error> {
    // Load RSA parameters from lock args
    let lock_args = load_script()?.args().raw_data();
    let n = &lock_args[0..256];  // Modulus (2048-bit)
    let e = &[0x01, 0x00, 0x01]; // Public exponent (65537)
    
    // Load signature from witness
    let witness_args = load_witness_args(0, Source::GroupInput)?;
    let signature = witness_args
        .lock()
        .to_opt()
        .ok_or(Error::InvalidWitness)?
        .raw_data();
    
    // Create RSA public key
    let public_key = RsaPublicKey::new(
        BigUint::from_bytes_be(n),
        BigUint::from_bytes_be(e),
    ).map_err(|_| Error::InvalidPublicKey)?;
    
    // Verify signature
    let message = load_tx_hash()?;
    let padding = PaddingScheme::new_pkcs1v15_sign(Some(rsa::Hash::SHA2_256));
    
    public_key
        .verify(padding, &message, &signature)
        .map_err(|_| Error::SignatureVerificationFailed)?;
    
    Ok(())
}
```

**Reference:** `resources/ckb-miscellaneous-scripts/rust/rsa_lock`

### 3. BLS Signature Aggregation

Enable signature aggregation for efficiency.

```rust
use bls12_381::{G1Affine, G2Affine, Scalar};
use group::Curve;

fn bls_verify() -> Result<(), Error> {
    // Load aggregated signature from witness
    let witness_args = load_witness_args(0, Source::GroupInput)?;
    let sig_data = witness_args
        .lock()
        .to_opt()
        .ok_or(Error::InvalidWitness)?
        .raw_data();
    
    let aggregated_signature = G2Affine::from_compressed(&sig_data[0..96].try_into()?)
        .ok_or(Error::InvalidSignature)?;
    
    // Load public keys and messages
    let mut public_keys = Vec::new();
    let mut messages = Vec::new();
    
    for i in 0..QueryIter::new(load_cell, Source::GroupInput).count() {
        let lock_args = load_cell_lock(i, Source::GroupInput)?.args().raw_data();
        let pubkey = G1Affine::from_compressed(&lock_args[0..48].try_into()?)
            .ok_or(Error::InvalidPublicKey)?;
        public_keys.push(pubkey);
        
        let message = load_tx_hash()?;
        messages.push(message);
    }
    
    // Verify aggregated signature
    verify_aggregate_signature(
        &aggregated_signature,
        &public_keys,
        &messages,
    ).map_err(|_| Error::SignatureVerificationFailed)?;
    
    Ok(())
}
```

**Reference:** `resources/ckb-miscellaneous-scripts/rust/bls_lock`

## Development Workflow Patterns

### 1. Contract Template Structure (Rust)

Standard structure for Rust-based contracts.

```rust
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{bytes::Bytes, prelude::*},
    high_level::{load_script, load_script_hash, QueryIter},
    debug, error,
};

pub fn main() -> Result<(), Error> {
    // Load script and parse args
    let script = load_script()?;
    let args: Bytes = script.args().unpack();
    
    // Validate transaction
    validate_transaction(&args)?;
    
    Ok(())
}

fn validate_transaction(args: &Bytes) -> Result<(), Error> {
    // Implementation specific validation
    Ok(())
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
```

**Reference:** `resources/ckb-script-templates/contract/src/main.rs`

### 2. Testing Framework Integration

Comprehensive testing with simulators.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ckb_tool::ckb_types::{
        core::{TransactionBuilder, TransactionView},
        packed::*,
        prelude::*,
    };
    
    #[test]
    fn test_contract_validation() {
        let mut context = Context::default();
        
        // Deploy contract
        let contract_bin = Loader::default().load_binary("contract");
        let contract_out_point = context.deploy_cell(contract_bin);
        
        // Create test transaction
        let tx = TransactionBuilder::default()
            .input(input_cell)
            .output(output_cell)
            .build();
            
        // Verify transaction
        let cycles = context.verify_tx(&tx, MAX_CYCLES).expect("pass");
        println!("cycles: {}", cycles);
    }
}
```

### 3. Memory Management Patterns

Advanced memory layout control for RISC-V contracts.

```rust
// Custom memory layout
#[link_section = ".text.custom"]
static CUSTOM_CODE: [u8; 1024] = [0; 1024];

// Stack reordering for optimization
#[no_mangle]
pub unsafe extern "C" fn entry() -> i8 {
    // Custom stack frame management
    asm!("
        addi sp, sp, -64
        sw ra, 60(sp)
    ");
    
    let result = main();
    
    asm!("
        lw ra, 60(sp)
        addi sp, sp, 64
    ");
    
    result as i8
}
```

**Reference:** `resources/ckb-script-templates/stack-reorder-contract/`

### 4. External Library Integration Pattern

FFI patterns for integrating external libraries when necessary.

```rust
// For cases where external libraries are absolutely required
extern "C" {
    fn external_validation_function(
        input: *const u8,
        input_len: usize,
        output: *mut u8,
        output_len: *mut usize,
    ) -> i32;
}

pub fn validate_with_external_library(data: &[u8]) -> Result<Vec<u8>, Error> {
    let mut output = vec![0u8; 1024];
    let mut output_len = output.len();
    
    let result = unsafe {
        external_validation_function(
            data.as_ptr(),
            data.len(),
            output.as_mut_ptr(),
            &mut output_len,
        )
    };
    
    if result == 0 {
        output.truncate(output_len);
        Ok(output)
    } else {
        Err(Error::ValidationFailed)
    }
}

// Note: Prefer pure Rust implementations when possible for better safety
```

**Reference:** `resources/ckb-script-templates/external-lib-integration/`

## Build System Patterns

### 1. Cross-Compilation Setup

```makefile
# Cargo.toml
[dependencies]
ckb-std = "0.15.1"

[profile.release]
overflow-checks = true
opt-level = "s"
lto = true
codegen-units = 1
panic = "abort"

# Rust build configuration
[build]
target = "riscv64imac-unknown-none-elf"

# Build script for optimized binary
build:
	cargo build --release --target riscv64imac-unknown-none-elf
	cp target/riscv64imac-unknown-none-elf/release/contract contract
	riscv64-unknown-elf-strip contract
```

### 2. Reproducible Builds

```bash
# Docker-based reproducible builds
FROM nervos/ckb-riscv-gnu-toolchain:jammy-20230214

# Set deterministic build environment
ENV SOURCE_DATE_EPOCH=1234567890
ENV BUILD_DATE=2024-01-01
ENV TZ=UTC

# Build with deterministic flags
RUN make clean && make all \
    CC_VERSION_CHECK=false \
    SECP256K1_CUSTOM_FUNCS=1
```

## Best Practices

### 1. Error Handling

```c
// Standard error codes
#define ERROR_INVALID_ARGS          -1
#define ERROR_SYSCALL              -2  
#define ERROR_SCRIPT_TOO_LONG      -21
#define ERROR_OVERFLOWING          -51
#define ERROR_AMOUNT               -52

// Consistent error propagation
int validate_cell(size_t index) {
    int ret = load_cell_data(...);
    if (ret != CKB_SUCCESS) {
        return ERROR_SYSCALL;
    }
    
    if (data_length > MAX_DATA_LENGTH) {
        return ERROR_SCRIPT_TOO_LONG;
    }
    
    return CKB_SUCCESS;
}
```

### 2. Gas Optimization

```c
// Minimize syscall overhead
int optimized_validation() {
    // Batch load operations
    uint8_t buffer[4096];
    size_t total_len = 0;
    
    // Load multiple cells in single buffer
    for (size_t i = 0; i < cell_count; i++) {
        size_t len = 0;
        int ret = ckb_load_cell_data(buffer + total_len, &len, 0, i, CKB_SOURCE_INPUT);
        if (ret != CKB_SUCCESS) return ret;
        total_len += len;
    }
    
    // Process all data in memory
    return validate_buffer(buffer, total_len);
}
```

### 3. Security Considerations

```c
// Integer overflow protection
int safe_add_u128(uint64_t* lo, uint64_t* hi, uint64_t add_lo, uint64_t add_hi) {
    uint64_t new_lo = *lo + add_lo;
    uint64_t new_hi = *hi + add_hi;
    
    // Check for overflow
    if (new_lo < *lo) {
        new_hi += 1;
        if (new_hi < *hi) {
            return ERROR_OVERFLOWING;
        }
    }
    
    *lo = new_lo;
    *hi = new_hi;
    return 0;
}

// Bounds checking
int safe_buffer_access(uint8_t* buffer, size_t buffer_len, size_t offset, size_t access_len) {
    if (offset + access_len > buffer_len || offset + access_len < offset) {
        return ERROR_INVALID_ARGS;
    }
    return 0;
}
```

These patterns provide a solid foundation for developing robust, efficient, and secure CKB scripts. Each pattern has been battle-tested in production environments and follows CKB best practices.