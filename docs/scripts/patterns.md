## Description

Rust patterns for CKB script development. Project structure, Cargo.toml configuration for RISC-V targets, entry point macros, error handling with granular error codes. UDT token implementation with owner mode and conservation rules. State machine pattern for counter contracts. Lock script signature verification. Composable locks (OR/AND logic), HTLC contracts, open transactions, dual-mode scripts. Cryptographic primitives: secp256k1, secp256r1, RSA, BLS signature aggregation. JSON cell data validation, multi-cell aggregation with nonce ordering, witness parsing, versioned script arguments, time-based logic with since fields, and Molecule serialization. Cross-compilation setup and Docker-based reproducible builds.

## Project Structure and Setup

### Standard Project Layout

```
my-ckb-script/
├── Cargo.toml              # Dependencies and build config
├── Makefile               # Build automation
├── build.rs               # Build script for custom compilation
├── src/
│   ├── main.rs           # Script entry point
│   ├── lib.rs            # Library interface (optional)
│   ├── error.rs          # Error definitions
│   └── entry.rs          # Main business logic
├── tests/
│   ├── Cargo.toml        # Test dependencies
│   └── src/
│       ├── lib.rs        # Test utilities
│       └── test_*.rs     # Test cases
└── schemas/
    └── types.mol         # Molecule schema definitions
```

### Cargo.toml Configuration

```toml
[package]
name = "my-ckb-script"
version = "0.1.0"
edition = "2021"

[dependencies]
ckb-std = "1.0"

# Optional: For Molecule serialization
molecule = "0.7.5"

# Optional: For JSON parsing
lite-json = { version = "0.2.0", default-features = false }

# Optional: For secp256k1 operations
secp256k1 = { version = "0.24.0", default-features = false, features = ["hashes"] }

[profile.release]
overflow-checks = true
opt-level = "s"
lto = true
codegen-units = 1
panic = "abort"
strip = true

# Target for CKB-VM (RISC-V)
[build]
target = "riscv64imac-unknown-none-elf"
```

### Main Entry Point

```rust
// src/main.rs
#![cfg_attr(not(any(feature = "library", test)), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(any(feature = "library", test))]
extern crate alloc;

mod error;
mod entry;

use error::Error;

#[cfg(not(any(feature = "library", test)))]
ckb_std::entry!(program_entry);

#[cfg(not(any(feature = "library", test)))]
ckb_std::default_alloc!(16384, 1258306, 64);

pub fn program_entry() -> i8 {
    match entry::main() {
        Ok(()) => 0,
        Err(err) => err as i8,
    }
}
```

## Error Handling

### Standard Error Enum

```rust
// src/error.rs
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Error {
    // CKB syscall errors
    IndexOutOfBound = 1,
    ItemMissing = 2,
    LengthNotEnough = 3,
    InvalidData = 4,

    // Script-specific errors
    InvalidArgs = 10,
    InvalidWitness = 11,
    InvalidTransaction = 12,

    // Business logic errors
    InsufficientBalance = 20,
    InvalidAmount = 21,
    Unauthorized = 22,
    InvalidState = 23,

    // Encoding errors
    Utf8Error = 30,
    JsonError = 31,
    MoleculeError = 32,
}

impl From<ckb_std::error::SysError> for Error {
    fn from(err: ckb_std::error::SysError) -> Self {
        use ckb_std::error::SysError::*;
        match err {
            IndexOutOfBound => Error::IndexOutOfBound,
            ItemMissing => Error::ItemMissing,
            LengthNotEnough => Error::LengthNotEnough,
            Encoding => Error::InvalidData,
            Unknown(_) => Error::InvalidData,
        }
    }
}
```

### Granular Error Codes

Generic error codes mask failure causes, making debugging difficult. Use granular codes for a one-to-one mapping between each failure condition and its error code.

Before (generic):
```rust
pub fn validate() -> Result<(), Error> {
    if input_count() > 1 {
        return Err(Error::InvalidTransaction);  // Ambiguous
    }
    if output_count() != 1 {
        return Err(Error::InvalidTransaction);  // Same code!
    }
    Ok(())
}
```

After (granular):
```rust
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Error {
    // Syscall errors (1-9)
    IndexOutOfBound = 1,
    ItemMissing = 2,
    LengthNotEnough = 3,

    // Transaction validation errors (10-19)
    MultipleInputsNotAllowed = 10,
    RequiredOutputMissing = 11,
    IncorrectOutputCount = 12,
    UnexpectedExtraOutput = 13,

    // Token amount errors (20-29)
    TokenAmountOverflow = 20,
    TokenAmountUnderflow = 21,
    ConservationViolation = 22,

    // Authorization errors (30-39)
    OwnerSignatureMissing = 30,
    InvalidOwnerMode = 31,
    UnauthorizedOperation = 32,

    // Data format errors (40-49)
    InvalidTokenData = 40,
    MalformedWitness = 41,
    CorruptedCellData = 42,
}

pub fn validate() -> Result<(), Error> {
    if input_count() > 1 {
        return Err(Error::MultipleInputsNotAllowed);
    }
    if output_count() != 1 {
        return Err(Error::IncorrectOutputCount);
    }
    if required_output_missing() {
        return Err(Error::RequiredOutputMissing);
    }
    Ok(())
}
```

## Core Patterns

### 1. Simple UDT (User Defined Token)

Token implementation with owner privileges and conservation rules:

```rust
// src/entry.rs
use core::result::Result;
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{bytes::Bytes, prelude::*},
    high_level::{load_script, load_cell_lock_hash, load_cell_data, QueryIter},
};
use crate::error::Error;

const LOCK_HASH_LEN: usize = 32;
const UDT_DATA_LEN: usize = 16;

pub fn main() -> Result<(), Error> {
    let script = load_script()?;
    let args: Bytes = script.args().unpack();

    if check_owner_mode(&args)? {
        return Ok(());
    }

    validate_token_conservation()?;
    Ok(())
}

fn check_owner_mode(args: &Bytes) -> Result<bool, Error> {
    if args.len() != LOCK_HASH_LEN {
        return Err(Error::InvalidArgs);
    }

    let is_owner = QueryIter::new(load_cell_lock_hash, Source::Input)
        .any(|lock_hash| args[..] == lock_hash[..]);

    Ok(is_owner)
}

fn validate_token_conservation() -> Result<(), Error> {
    let input_amount = calculate_token_amount(Source::GroupInput)?;
    let output_amount = calculate_token_amount(Source::GroupOutput)?;

    if output_amount > input_amount {
        return Err(Error::InvalidAmount);
    }

    Ok(())
}

fn calculate_token_amount(source: Source) -> Result<u128, Error> {
    let mut total = 0u128;

    for data in QueryIter::new(load_cell_data, source) {
        if data.len() < UDT_DATA_LEN {
            return Err(Error::InvalidData);
        }

        let mut buffer = [0u8; UDT_DATA_LEN];
        buffer.copy_from_slice(&data[0..UDT_DATA_LEN]);
        let amount = u128::from_le_bytes(buffer);

        total = total.checked_add(amount)
            .ok_or(Error::InvalidAmount)?;
    }

    Ok(total)
}
```

### 2. State Machine (Counter)

```rust
use ckb_std::{
    ckb_constants::Source,
    high_level::{load_cell_data, QueryIter},
};
use crate::error::Error;

const STATE_DATA_LEN: usize = 8;

pub fn main() -> Result<(), Error> {
    validate_state_structure()?;

    let current_state = load_state(Source::GroupInput)?;
    let next_state = load_state(Source::GroupOutput)?;

    validate_state_transition(current_state, next_state)?;
    Ok(())
}

fn validate_state_structure() -> Result<(), Error> {
    let input_count = QueryIter::new(load_cell_data, Source::GroupInput).count();
    let output_count = QueryIter::new(load_cell_data, Source::GroupOutput).count();

    if input_count == 0 {
        return Ok(());
    }

    if input_count != 1 || output_count != 1 {
        return Err(Error::InvalidTransaction);
    }

    Ok(())
}

fn load_state(source: Source) -> Result<u64, Error> {
    let data = load_cell_data(0, source)?;

    if data.len() < STATE_DATA_LEN {
        return Err(Error::InvalidData);
    }

    let mut buffer = [0u8; STATE_DATA_LEN];
    buffer.copy_from_slice(&data[0..STATE_DATA_LEN]);
    Ok(u64::from_le_bytes(buffer))
}

fn validate_state_transition(current: u64, next: u64) -> Result<(), Error> {
    if next != current + 1 {
        return Err(Error::InvalidState);
    }

    Ok(())
}
```

### 3. Lock Script (Signature Verification)

```rust
use ckb_std::{
    ckb_constants::Source,
    high_level::{load_witness_args, load_tx_hash, load_script},
    ckb_types::{bytes::Bytes, prelude::*},
};
use crate::error::Error;

const SECP256K1_PUBKEY_SIZE: usize = 33;
const SECP256K1_SIGNATURE_SIZE: usize = 65;

pub fn main() -> Result<(), Error> {
    let script = load_script()?;
    let args = script.args().raw_data();

    if args.len() != 20 {
        return Err(Error::InvalidArgs);
    }

    let witness_args = load_witness_args(0, Source::GroupInput)?;
    let signature = witness_args.lock().to_opt()
        .ok_or(Error::InvalidWitness)?
        .raw_data();

    if signature.len() != SECP256K1_SIGNATURE_SIZE {
        return Err(Error::InvalidWitness);
    }

    let tx_hash = load_tx_hash()?;
    verify_secp256k1_signature(&signature, &tx_hash, &args)?;
    Ok(())
}

fn verify_secp256k1_signature(
    signature: &[u8],
    message: &[u8],
    pubkey_hash: &[u8]
) -> Result<(), Error> {
    let pubkey = recover_secp256k1_pubkey(signature, message)?;
    let recovered_hash = blake2b_hash(&pubkey);

    if recovered_hash[0..20] != pubkey_hash[..] {
        return Err(Error::Unauthorized);
    }

    Ok(())
}

fn recover_secp256k1_pubkey(signature: &[u8], message: &[u8]) -> Result<[u8; SECP256K1_PUBKEY_SIZE], Error> {
    use secp256k1::{recovery::RecoveryId, recovery::RecoverableSignature, Message, Secp256k1};

    let secp = Secp256k1::verification_only();
    let recovery_id = RecoveryId::from_i32(signature[64] as i32)
        .map_err(|_| Error::InvalidWitness)?;

    let signature = RecoverableSignature::from_compact(&signature[0..64], recovery_id)
        .map_err(|_| Error::InvalidWitness)?;

    let message = Message::from_slice(message)
        .map_err(|_| Error::InvalidData)?;

    let pubkey = secp.recover_ecdsa(&message, &signature)
        .map_err(|_| Error::Unauthorized)?;

    Ok(pubkey.serialize())
}

fn blake2b_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = ckb_hash::Blake2bBuilder::new(32).build();
    hasher.update(data);

    let mut result = [0u8; 32];
    hasher.finalize(&mut result);
    result
}
```

## Advanced Patterns

### 1. Composable Lock Scripts (OR/AND Logic)

**OR Lock** -- any one of the provided lock scripts can unlock:
```rust
use ckb_std::dynamic_loading::{CKBDLContext, Symbol};

fn or_lock_main() -> Result<(), Error> {
    let lock_scripts = load_lock_scripts_from_args()?;

    for lock_script in lock_scripts {
        match validate_lock_script(&lock_script) {
            Ok(()) => return Ok(()),
            Err(_) => continue,
        }
    }

    Err(Error::AllLocksFailed)
}
```

**AND Lock** -- all provided lock scripts must pass:
```rust
fn and_lock_main() -> Result<(), Error> {
    let lock_scripts = load_lock_scripts_from_args()?;

    for lock_script in lock_scripts {
        validate_lock_script(&lock_script)?;
    }

    Ok(())
}
```

**Reference:** `resources/ckb-miscellaneous-scripts/rust/composable_locks`

### 2. Hash Time Locked Contracts (HTLC)

```rust
use ckb_std::{since::Since, high_level::*};
use blake2b_rs::Blake2bBuilder;

fn htlc_main() -> Result<(), Error> {
    let since = load_input_since(0, Source::GroupInput)?;
    let since_value = Since::new(since);

    // Timelock expired: refund path
    if since_value.is_absolute() && since_value.extract_lock_value()? >= timeout_timestamp {
        return validate_timeout_unlock();
    }

    // Hash preimage path
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

### 3. Open Transaction Pattern

Validates transaction structure rather than specific values, enabling transaction templates:

```rust
fn open_transaction_main() -> Result<(), Error> {
    let input_count = QueryIter::new(load_input, Source::Input).count();
    let output_count = QueryIter::new(load_cell, Source::Output).count();

    if input_count < min_inputs {
        return Err(Error::InsufficientInputs);
    }
    if output_count < min_outputs {
        return Err(Error::InsufficientOutputs);
    }

    for required_type in &required_types {
        if !find_cell_with_type(required_type)? {
            return Err(Error::RequiredTypeMissing);
        }
    }

    Ok(())
}
```

**Reference:** `resources/ckb-miscellaneous-scripts/rust/open_transaction`

### 4. Dual-Mode Scripts

Scripts that work both as standalone validators and as dynamically-loaded libraries:

```rust
#[cfg(feature = "dual-mode")]
use ckb_std::dynamic_loading_c_impl;

// Library mode: export functions for other scripts
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

## Cryptographic Patterns

### 1. Alternative Elliptic Curves (secp256r1)

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

    let signature = Signature::from_slice(&signature_data[0..64])
        .map_err(|_| Error::InvalidSignature)?;

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

```rust
use rsa::{PublicKey, RsaPublicKey, PaddingScheme};
use sha2::{Sha256, Digest};

fn rsa_verify() -> Result<(), Error> {
    let lock_args = load_script()?.args().raw_data();
    let n = &lock_args[0..256];  // Modulus (2048-bit)
    let e = &[0x01, 0x00, 0x01]; // Public exponent (65537)

    let witness_args = load_witness_args(0, Source::GroupInput)?;
    let signature = witness_args
        .lock()
        .to_opt()
        .ok_or(Error::InvalidWitness)?
        .raw_data();

    let public_key = RsaPublicKey::new(
        BigUint::from_bytes_be(n),
        BigUint::from_bytes_be(e),
    ).map_err(|_| Error::InvalidPublicKey)?;

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

```rust
use bls12_381::{G1Affine, G2Affine, Scalar};
use group::Curve;

fn bls_verify() -> Result<(), Error> {
    let witness_args = load_witness_args(0, Source::GroupInput)?;
    let sig_data = witness_args
        .lock()
        .to_opt()
        .ok_or(Error::InvalidWitness)?
        .raw_data();

    let aggregated_signature = G2Affine::from_compressed(&sig_data[0..96].try_into()?)
        .ok_or(Error::InvalidSignature)?;

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

    verify_aggregate_signature(
        &aggregated_signature,
        &public_keys,
        &messages,
    ).map_err(|_| Error::SignatureVerificationFailed)?;

    Ok(())
}
```

**Reference:** `resources/ckb-miscellaneous-scripts/rust/bls_lock`

## Data Patterns

### 1. JSON Cell Data Validation

```rust
use ckb_std::{
    ckb_constants::Source,
    high_level::{load_cell_data, QueryIter},
};
use lite_json::json_parser::parse_json;
use core::str;
use crate::error::Error;

pub fn main() -> Result<(), Error> {
    for (index, data) in QueryIter::new(load_cell_data, Source::GroupOutput).enumerate() {
        validate_json_data(&data)
            .map_err(|_| Error::InvalidData)?;
    }

    Ok(())
}

fn validate_json_data(data: &[u8]) -> Result<(), Error> {
    let json_str = str::from_utf8(data)
        .map_err(|_| Error::Utf8Error)?;

    parse_json(json_str)
        .map_err(|_| Error::JsonError)?;

    Ok(())
}
```

### 2. Multi-Cell Aggregation with Nonce Ordering

```rust
use ckb_std::{
    ckb_constants::Source,
    high_level::{load_cell_data, QueryIter},
};
use crate::error::Error;

pub fn main() -> Result<(), Error> {
    let inputs = collect_aggregation_data(Source::GroupInput)?;
    let outputs = collect_aggregation_data(Source::GroupOutput)?;

    validate_aggregation(&inputs, &outputs)?;
    Ok(())
}

#[derive(Debug, Clone)]
struct AggregationData {
    amount: u128,
    nonce: u64,
    metadata: Vec<u8>,
}

fn collect_aggregation_data(source: Source) -> Result<Vec<AggregationData>, Error> {
    let mut data_vec = Vec::new();

    for (index, data) in QueryIter::new(load_cell_data, source).enumerate() {
        let parsed_data = parse_aggregation_data(&data)?;
        data_vec.push(parsed_data);
    }

    Ok(data_vec)
}

fn parse_aggregation_data(data: &[u8]) -> Result<AggregationData, Error> {
    if data.len() < 24 {
        return Err(Error::InvalidData);
    }

    let amount = u128::from_le_bytes(data[0..16].try_into().unwrap());
    let nonce = u64::from_le_bytes(data[16..24].try_into().unwrap());
    let metadata = data[24..].to_vec();

    Ok(AggregationData { amount, nonce, metadata })
}

fn validate_aggregation(inputs: &[AggregationData], outputs: &[AggregationData]) -> Result<(), Error> {
    let input_total: u128 = inputs.iter()
        .map(|d| d.amount)
        .try_fold(0u128, |acc, x| acc.checked_add(x))
        .ok_or(Error::InvalidAmount)?;

    let output_total: u128 = outputs.iter()
        .map(|d| d.amount)
        .try_fold(0u128, |acc, x| acc.checked_add(x))
        .ok_or(Error::InvalidAmount)?;

    if output_total > input_total {
        return Err(Error::InvalidAmount);
    }

    validate_nonce_ordering(inputs, outputs)?;
    Ok(())
}

fn validate_nonce_ordering(inputs: &[AggregationData], outputs: &[AggregationData]) -> Result<(), Error> {
    let max_input_nonce = inputs.iter().map(|d| d.nonce).max().unwrap_or(0);
    let min_output_nonce = outputs.iter().map(|d| d.nonce).min().unwrap_or(0);

    if min_output_nonce <= max_input_nonce {
        return Err(Error::InvalidState);
    }

    Ok(())
}
```

### 3. Witness Data Parsing

```rust
use ckb_std::{
    ckb_constants::Source,
    high_level::{load_witness_args},
    ckb_types::{bytes::Bytes, prelude::*},
};
use crate::error::Error;

#[derive(Debug)]
struct ComplexWitness {
    signature: Vec<u8>,
    proof: Option<Vec<u8>>,
    state: Option<Vec<u8>>,
}

pub fn parse_complex_witness() -> Result<ComplexWitness, Error> {
    let witness_args = load_witness_args(0, Source::GroupInput)?;

    let lock_witness = witness_args.lock().to_opt()
        .ok_or(Error::InvalidWitness)?
        .raw_data();

    let input_type_witness = witness_args.input_type().to_opt()
        .map(|w| w.raw_data());

    let output_type_witness = witness_args.output_type().to_opt()
        .map(|w| w.raw_data());

    Ok(ComplexWitness {
        signature: lock_witness.to_vec(),
        proof: input_type_witness.map(|w| w.to_vec()),
        state: output_type_witness.map(|w| w.to_vec()),
    })
}
```

### 4. Molecule Serialization Integration

```rust
// Define schema in schemas/types.mol:
// struct TokenInfo {
//     name: Bytes,
//     symbol: Bytes,
//     decimals: byte,
//     total_supply: Uint128,
// }

use molecule::prelude::*;
include!(concat!(env!("OUT_DIR"), "/types.rs"));

use crate::error::Error;

pub fn parse_token_info(data: &[u8]) -> Result<TokenInfo, Error> {
    TokenInfoReader::from_slice(data)
        .map_err(|_| Error::MoleculeError)?
        .to_entity()
}

pub fn create_token_info(
    name: &str,
    symbol: &str,
    decimals: u8,
    total_supply: u128,
) -> TokenInfo {
    TokenInfo::new_builder()
        .name(Bytes::from(name.as_bytes().to_vec()).pack())
        .symbol(Bytes::from(symbol.as_bytes().to_vec()).pack())
        .decimals(decimals.into())
        .total_supply(Uint128::from_slice(&total_supply.to_le_bytes()).unwrap())
        .build()
}
```

## Time and Arguments

### 1. Script Arguments with Versioning

```rust
use ckb_std::{high_level::load_script, ckb_types::prelude::*};
use crate::error::Error;

#[derive(Debug)]
pub struct ScriptConfig {
    pub version: u8,
    pub owner_lock_hash: [u8; 32],
    pub flags: u32,
}

pub fn parse_script_config() -> Result<ScriptConfig, Error> {
    let script = load_script()?;
    let args = script.args().raw_data();

    if args.len() < 37 {
        return Err(Error::InvalidArgs);
    }

    let version = args[0];
    let mut owner_lock_hash = [0u8; 32];
    owner_lock_hash.copy_from_slice(&args[1..33]);
    let flags = u32::from_le_bytes(args[33..37].try_into().unwrap());

    Ok(ScriptConfig { version, owner_lock_hash, flags })
}
```

### 2. Time-Based Logic (Since Field)

```rust
use ckb_std::{ckb_constants::Source, high_level::load_input_since};
use crate::error::Error;

const SINCE_TYPE_ABSOLUTE_TIMESTAMP: u64 = 0x4000000000000000;

pub fn validate_timelock(unlock_timestamp: u64) -> Result<(), Error> {
    let since = load_input_since(0, Source::GroupInput)?;

    if since & SINCE_TYPE_ABSOLUTE_TIMESTAMP == 0 {
        return Err(Error::InvalidTransaction);
    }

    let since_timestamp = since & 0x00ffffffffffffff;
    if since_timestamp < unlock_timestamp {
        return Err(Error::InvalidState);
    }

    Ok(())
}
```

## Build System

### Cross-Compilation

```makefile
# Build for CKB-VM RISC-V target
build:
	cargo build --release --target riscv64imac-unknown-none-elf
	cp target/riscv64imac-unknown-none-elf/release/contract contract
	riscv64-unknown-elf-strip contract
```

### Reproducible Builds (Docker)

```bash
FROM nervos/ckb-riscv-gnu-toolchain:jammy-20230214

# Deterministic build environment
ENV SOURCE_DATE_EPOCH=1234567890
ENV BUILD_DATE=2024-01-01
ENV TZ=UTC

RUN make clean && make all \
    CC_VERSION_CHECK=false \
    SECP256K1_CUSTOM_FUNCS=1
```

### Testing Framework Integration

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

        let contract_bin = Loader::default().load_binary("contract");
        let contract_out_point = context.deploy_cell(contract_bin);

        let tx = TransactionBuilder::default()
            .input(input_cell)
            .output(output_cell)
            .build();

        let cycles = context.verify_tx(&tx, MAX_CYCLES).expect("pass");
        println!("cycles: {}", cycles);
    }
}
```

**Reference:** `resources/ckb-script-templates/contract/src/main.rs`
