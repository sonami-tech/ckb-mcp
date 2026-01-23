# CKB Rust Script Development Patterns

## Description

Rust patterns for CKB script development. Project structure, Cargo.toml configuration for RISC-V targets, entry point macros, error handling with granular error codes. UDT (User Defined Token) implementation with owner mode and conservation rules. State machine pattern for counter contracts. Lock script signature verification with secp256k1. JSON cell data validation. Multi-cell aggregation with nonce ordering. Advanced witness parsing, versioned script arguments, time-based logic with since fields, and Molecule serialization integration.

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
opt-level = "s"           # Optimize for size
lto = true               # Link-time optimization
codegen-units = 1        # Single codegen unit for size
panic = "abort"          # Smaller binary size
strip = true            # Strip symbols

# Target for CKB-VM (RISC-V)
[build]
target = "riscv64imac-unknown-none-elf"
```

### Main Entry Point Pattern

```rust
// src/main.rs
#![cfg_attr(not(any(feature = "library", test)), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(any(feature = "library", test))]
extern crate alloc;

// Import error types
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

### Error Handling Pattern

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

### Granular Error Code Pattern

Smart contracts often use generic error codes that mask the true cause of failures, making debugging extremely difficult. When multiple distinct failure conditions return the same error code, developers must manually trace through complex contract logic to determine which specific condition triggered the error.

Implementing granular error codes creates a one-to-one mapping between each error condition and its specific error code, dramatically improving debugging efficiency.

#### Before: Generic Error Codes

```rust
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Error {
    IndexOutOfBound = 1,
    ItemMissing = 2,
    InvalidData = 3,
    InvalidTransaction = 4,  // Too generic!
    InvalidArgs = 5,         // Too generic!
    Unauthorized = 6,        // Too generic!
}

pub fn validate() -> Result<(), Error> {
    // Multiple different conditions all return the same error
    if input_count() > 1 {
        return Err(Error::InvalidTransaction);  // Could be many things
    }
    if output_count() != 1 {
        return Err(Error::InvalidTransaction);  // Same error code!
    }
    if required_output_missing() {
        return Err(Error::InvalidTransaction);  // Same error code!
    }
    Ok(())
}
```

#### After: Granular Error Codes

```rust
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Error {
    // Syscall errors
    IndexOutOfBound = 1,
    ItemMissing = 2,
    LengthNotEnough = 3,

    // Specific transaction structure errors
    MultipleInputsNotAllowed = 10,
    RequiredOutputMissing = 11,
    IncorrectOutputCount = 12,
    UnexpectedExtraOutput = 13,

    // Specific argument errors
    InvalidOwnerLockHash = 20,
    ArgumentLengthIncorrect = 21,
    MissingRequiredArgument = 22,

    // Specific authorization errors
    OwnerSignatureMissing = 30,
    InvalidOwnerMode = 31,
    UnauthorizedMintingAttempt = 32,
}

pub fn validate() -> Result<(), Error> {
    // Each condition has a specific, descriptive error
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

#### Error Code Organization Strategy

```rust
// Group related errors by category using number ranges
pub enum Error {
    // Syscall errors (1-9)
    IndexOutOfBound = 1,
    ItemMissing = 2,
    LengthNotEnough = 3,

    // Transaction validation errors (10-19)
    MultipleInputsNotAllowed = 10,
    RequiredOutputMissing = 11,
    IncorrectOutputCount = 12,

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
```

## Core Development Patterns

### 1. Simple UDT (User Defined Token) Pattern

A production-quality token implementation with owner privileges and conservation rules:

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

    // Check owner mode (allows minting/burning)
    if check_owner_mode(&args)? {
        return Ok(());
    }

    // Validate token conservation
    validate_token_conservation()?;

    Ok(())
}

fn check_owner_mode(args: &Bytes) -> Result<bool, Error> {
    if args.len() != LOCK_HASH_LEN {
        return Err(Error::InvalidArgs);
    }

    // Check if any input cell has lock hash matching owner
    let is_owner = QueryIter::new(load_cell_lock_hash, Source::Input)
        .any(|lock_hash| args[..] == lock_hash[..]);

    Ok(is_owner)
}

fn validate_token_conservation() -> Result<(), Error> {
    let input_amount = calculate_token_amount(Source::GroupInput)?;
    let output_amount = calculate_token_amount(Source::GroupOutput)?;

    // Prevent token inflation (allow burning)
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

### 2. State Machine Pattern

Implementing a simple counter with state validation:

```rust
use ckb_std::{
    ckb_constants::Source,
    high_level::{load_cell_data, QueryIter},
};
use crate::error::Error;

const STATE_DATA_LEN: usize = 8;

pub fn main() -> Result<(), Error> {
    // Validate 1:1 state transition
    validate_state_structure()?;

    // Load current and next state
    let current_state = load_state(Source::GroupInput)?;
    let next_state = load_state(Source::GroupOutput)?;

    // Validate state transition
    validate_state_transition(current_state, next_state)?;

    Ok(())
}

fn validate_state_structure() -> Result<(), Error> {
    let input_count = QueryIter::new(load_cell_data, Source::GroupInput).count();
    let output_count = QueryIter::new(load_cell_data, Source::GroupOutput).count();

    // Skip validation if no inputs (creation case)
    if input_count == 0 {
        return Ok(());
    }

    // Require exactly 1:1 state transition
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
    // Counter can only increment by 1
    if next != current + 1 {
        return Err(Error::InvalidState);
    }

    Ok(())
}
```

### 3. Lock Script Pattern (Signature Verification)

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
    // Load script arguments (public key hash)
    let script = load_script()?;
    let args = script.args().raw_data();

    if args.len() != 20 {
        return Err(Error::InvalidArgs);
    }

    // Load signature from witness
    let witness_args = load_witness_args(0, Source::GroupInput)?;
    let signature = witness_args.lock().to_opt()
        .ok_or(Error::InvalidWitness)?
        .raw_data();

    if signature.len() != SECP256K1_SIGNATURE_SIZE {
        return Err(Error::InvalidWitness);
    }

    // Load transaction hash
    let tx_hash = load_tx_hash()?;

    // Verify signature
    verify_secp256k1_signature(&signature, &tx_hash, &args)?;

    Ok(())
}

fn verify_secp256k1_signature(
    signature: &[u8],
    message: &[u8],
    pubkey_hash: &[u8]
) -> Result<(), Error> {
    // Recover public key from signature
    let pubkey = recover_secp256k1_pubkey(signature, message)?;

    // Hash recovered public key
    let recovered_hash = blake2b_hash(&pubkey);

    // Compare with expected hash
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
    use ckb_std::ckb_types::packed::Byte32;
    use ckb_std::ckb_types::prelude::*;

    let mut hasher = ckb_hash::Blake2bBuilder::new(32).build();
    hasher.update(data);

    let mut result = [0u8; 32];
    hasher.finalize(&mut result);
    result
}
```

### 4. Data Validation Pattern (JSON Cell)

Validating structured data in cells:

```rust
use ckb_std::{
    ckb_constants::Source,
    high_level::{load_cell_data, QueryIter},
};
use lite_json::json_parser::parse_json;
use core::str;
use crate::error::Error;

pub fn main() -> Result<(), Error> {
    // Validate all output cells contain valid JSON
    for (index, data) in QueryIter::new(load_cell_data, Source::GroupOutput).enumerate() {
        validate_json_data(&data)
            .map_err(|_| Error::InvalidData)?;
    }

    Ok(())
}

fn validate_json_data(data: &[u8]) -> Result<(), Error> {
    // Parse as UTF-8 string
    let json_str = str::from_utf8(data)
        .map_err(|_| Error::Utf8Error)?;

    // Validate JSON syntax
    parse_json(json_str)
        .map_err(|_| Error::JsonError)?;

    Ok(())
}
```

### 5. Multi-Cell Aggregation Pattern

Handling multiple inputs or outputs efficiently:

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

fn collect_aggregation_data(source: Source) -> Result<Vec<AggregationData>, Error> {
    let mut data_vec = Vec::new();

    for (index, data) in QueryIter::new(load_cell_data, source).enumerate() {
        let parsed_data = parse_aggregation_data(&data)
            .map_err(|_| Error::InvalidData)?;
        data_vec.push(parsed_data);
    }

    Ok(data_vec)
}

#[derive(Debug, Clone)]
struct AggregationData {
    amount: u128,
    nonce: u64,
    metadata: Vec<u8>,
}

fn parse_aggregation_data(data: &[u8]) -> Result<AggregationData, Error> {
    if data.len() < 24 {
        return Err(Error::InvalidData);
    }

    let amount = u128::from_le_bytes(data[0..16].try_into().unwrap());
    let nonce = u64::from_le_bytes(data[16..24].try_into().unwrap());
    let metadata = data[24..].to_vec();

    Ok(AggregationData {
        amount,
        nonce,
        metadata,
    })
}

fn validate_aggregation(inputs: &[AggregationData], outputs: &[AggregationData]) -> Result<(), Error> {
    // Calculate totals
    let input_total: u128 = inputs.iter()
        .map(|d| d.amount)
        .try_fold(0u128, |acc, x| acc.checked_add(x))
        .ok_or(Error::InvalidAmount)?;

    let output_total: u128 = outputs.iter()
        .map(|d| d.amount)
        .try_fold(0u128, |acc, x| acc.checked_add(x))
        .ok_or(Error::InvalidAmount)?;

    // Validate conservation
    if output_total > input_total {
        return Err(Error::InvalidAmount);
    }

    // Validate nonce ordering (prevent replay)
    validate_nonce_ordering(inputs, outputs)?;

    Ok(())
}

fn validate_nonce_ordering(inputs: &[AggregationData], outputs: &[AggregationData]) -> Result<(), Error> {
    let max_input_nonce = inputs.iter()
        .map(|d| d.nonce)
        .max()
        .unwrap_or(0);

    let min_output_nonce = outputs.iter()
        .map(|d| d.nonce)
        .min()
        .unwrap_or(0);

    if min_output_nonce <= max_input_nonce {
        return Err(Error::InvalidState);
    }

    Ok(())
}
```

## Advanced Patterns

### 1. Witness Data Parsing

Handling complex witness structures:

```rust
use ckb_std::{
    ckb_constants::Source,
    high_level::{load_witness_args},
    ckb_types::{bytes::Bytes, prelude::*},
};
use crate::error::Error;

pub fn parse_complex_witness() -> Result<ComplexWitness, Error> {
    let witness_args = load_witness_args(0, Source::GroupInput)?;

    // Parse lock witness (signature)
    let lock_witness = witness_args.lock().to_opt()
        .ok_or(Error::InvalidWitness)?
        .raw_data();

    // Parse input type witness (proof data)
    let input_type_witness = witness_args.input_type().to_opt()
        .map(|w| w.raw_data());

    // Parse output type witness (new state)
    let output_type_witness = witness_args.output_type().to_opt()
        .map(|w| w.raw_data());

    Ok(ComplexWitness {
        signature: lock_witness.to_vec(),
        proof: input_type_witness.map(|w| w.to_vec()),
        state: output_type_witness.map(|w| w.to_vec()),
    })
}

#[derive(Debug)]
struct ComplexWitness {
    signature: Vec<u8>,
    proof: Option<Vec<u8>>,
    state: Option<Vec<u8>>,
}
```

### 2. Script Arguments Parsing

Flexible argument parsing with versioning:

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

### 3. Time-Based Logic

Implementing time locks using since field:

```rust
use ckb_std::{ckb_constants::Source, high_level::load_input_since};
use crate::error::Error;

const SINCE_TYPE_ABSOLUTE_TIMESTAMP: u64 = 0x4000000000000000;

pub fn validate_timelock(unlock_timestamp: u64) -> Result<(), Error> {
    let since = load_input_since(0, Source::GroupInput)?;

    // Check if since uses absolute timestamp
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

### 4. Molecule Serialization Integration

Using Molecule for type-safe serialization:

```rust
// First, define schema in schemas/types.mol
/*
struct TokenInfo {
    name: Bytes,
    symbol: Bytes,
    decimals: byte,
    total_supply: Uint128,
}
*/

use molecule::prelude::*;
// Include generated code
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
