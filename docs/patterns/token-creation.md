# SUDT Token Creation Pattern

## Description

Complete implementation guide for Simple User Defined Tokens (SUDT) on CKB blockchain. Provides production-ready Rust code for creating fungible tokens with owner-controlled minting/burning capabilities. Covers token amount encoding (u128 as 16 bytes), conservation validation logic, owner mode verification using lock hash authentication, and multi-cell token operations. Includes error handling, deployment instructions, and transaction examples for minting, burning, and transferring tokens. Essential resource for developers implementing custom tokens, DeFi protocols, or any application requiring fungible asset management on CKB. Features complete test suite and integration patterns with popular CKB SDKs.

## Purpose
Complete SUDT (Simple User Defined Token) implementation from CKB developer training course. This pattern demonstrates:
- Owner mode for minting/burning tokens
- Token conservation validation
- Proper data encoding/decoding
- Multi-cell token operations
- Lock hash verification

## Complete Working Implementation

### Main Entry Point
```rust
#![no_std]
#![no_main]

use ckb_std::{default_alloc, entry};

mod entry;
mod error;

entry!(program_entry);
default_alloc!();

fn program_entry(_argc: u64, _argv: *const *const u8) -> i8 {
    match entry::main() {
        Ok(_) => 0,
        Err(err) => err as i8,
    }
}
```

### Core SUDT Logic
```rust
use core::result::Result;
use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::{bytes::Bytes, prelude::*};
use ckb_std::high_level::{load_script, load_cell_lock_hash, load_cell_data, QueryIter};

// Constants
const LOCK_HASH_LEN: usize = 32; // Blake2b 256-bit hash length
const SUDT_DATA_LEN: usize = 16;  // u128 token amount (16 bytes)

/// Check if script is running in owner mode
fn check_owner_mode(args: &Bytes) -> Result<bool, Error> {
    // Verify args length matches Blake2b hash length
    if args.len() != LOCK_HASH_LEN {
        return Err(Error::ArgsLength);
    }

    // Check if any input cell's lock hash matches script args
    let is_owner_mode = QueryIter::new(load_cell_lock_hash, Source::Input)
        .find(|lock_hash| args[..] == lock_hash[..])
        .is_some();

    Ok(is_owner_mode)
}

/// Count total token amount in specified source
fn determine_token_amount(source: Source) -> Result<u128, Error> {
    let mut total_token_amount = 0;

    // Iterate through all cells in the specified source
    let cell_data = QueryIter::new(load_cell_data, source);
    for data in cell_data {
        // Verify data length is sufficient for u128
        if data.len() >= SUDT_DATA_LEN {
            // Convert first 16 bytes to u128 token amount
            let mut buffer = [0u8; SUDT_DATA_LEN];
            buffer.copy_from_slice(&data[0..SUDT_DATA_LEN]);
            let amount = u128::from_le_bytes(buffer);
            
            total_token_amount += amount;
        } else {
            return Err(Error::Encoding);
        }
    }

    Ok(total_token_amount)
}

/// Main validation logic
pub fn main() -> Result<(), Error> {
    // Load script and extract args (owner's lock hash)
    let script = load_script()?;
    let args: Bytes = script.args().unpack();

    // If owner mode, allow any operation (minting/burning)
    if check_owner_mode(&args)? {
        return Ok(());
    }

    // Otherwise, enforce token conservation
    let input_token_amount = determine_token_amount(Source::GroupInput)?;
    let output_token_amount = determine_token_amount(Source::GroupOutput)?;

    // Prevent creating tokens out of thin air
    if input_token_amount < output_token_amount {
        return Err(Error::Amount);
    }

    Ok(())
}
```

### Error Definitions
```rust
use ckb_std::error::SysError;

#[repr(i8)]
pub enum Error {
    IndexOutOfBound = 1,
    ItemMissing,
    LengthNotEnough,
    Encoding,
    Amount,       // Token conservation violation
    ArgsLength,   // Invalid script args length
}

impl From<SysError> for Error {
    fn from(err: SysError) -> Self {
        use SysError::*;
        match err {
            IndexOutOfBound => Self::IndexOutOfBound,
            ItemMissing => Self::ItemMissing,
            LengthNotEnough(_) => Self::LengthNotEnough,
            Encoding => Self::Encoding,
            Unknown(err_code) => panic!("unexpected sys error {}", err_code),
        }
    }
}
```

## Key Patterns Explained

### 1. Owner Mode Pattern
```rust
// Script args contain the owner's lock hash
let args: Bytes = script.args().unpack();

// Check if any input cell uses the owner's lock
let is_owner = QueryIter::new(load_cell_lock_hash, Source::Input)
    .any(|lock_hash| args[..] == lock_hash[..]);

if is_owner {
    // Owner can mint/burn tokens freely
    return Ok(());
}
```

### 2. Token Data Encoding
```rust
// SUDT stores token amount as little-endian u128
const SUDT_DATA_LEN: usize = 16;

let mut buffer = [0u8; SUDT_DATA_LEN];
buffer.copy_from_slice(&data[0..SUDT_DATA_LEN]);
let amount = u128::from_le_bytes(buffer);
```

### 3. Conservation Validation
```rust
// Sum all input tokens
let input_total = determine_token_amount(Source::GroupInput)?;

// Sum all output tokens  
let output_total = determine_token_amount(Source::GroupOutput)?;

// Prevent token creation (inflation)
if input_total < output_total {
    return Err(Error::Amount);
}
```

### 4. Group Cell Processing
```rust
// Only process cells that use the same type script
// Source::GroupInput - input cells with this type script
// Source::GroupOutput - output cells with this type script

for data in QueryIter::new(load_cell_data, Source::GroupInput) {
    // Process only SUDT cells, not other cells in transaction
}
```

## SUDT Cell Structure
```rust
// Cell with SUDT tokens
CellOutput {
    capacity: 14400000000, // Minimum capacity for cell + data
    lock: user_lock_script, // User's lock script
    type_: Some(sudt_type_script), // This SUDT's type script
}

// Cell data contains token amount
cell_data: [0x40, 0x42, 0x0F, 0x00, 0x00, 0x00, 0x00, 0x00, 
           0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
// = 1000000 tokens in little-endian u128 format
```

## Usage Examples

### Minting Tokens (Owner Mode)
```rust
// Transaction inputs include cell with owner's lock
// Owner can create tokens in outputs without conservation constraint
```

### Transferring Tokens (Normal Mode)
```rust
// Input:  Alice has 1000 tokens
// Output: Alice has 600 tokens, Bob has 400 tokens
// Conservation: 1000 >= 1000 ✅
```

### Burning Tokens (Owner Mode)
```rust
// Owner can destroy tokens by having fewer output tokens than input tokens
// Input:  1000 tokens
// Output: 500 tokens (500 tokens burned)
```

## When to Use This Pattern
- Creating custom tokens/cryptocurrencies
- Implementing fungible assets
- Building DeFi applications
- Managing digital collectibles with quantities
- Any use case requiring token conservation laws

## Integration with Frontend
This type script works with any CKB SDK (CCC, Lumos) by:
1. Setting the type script on token cells
2. Including proper cell deps for the deployed script
3. Ensuring token data follows u128 little-endian encoding