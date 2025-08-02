# Minimal Lock Script Pattern

## Description

Learn the foundational lock script pattern for CKB smart contract development. Covers script entry point setup, argument loading, witness data access, hash verification, and return code handling. Perfect starting point for understanding CKB script structure, error handling patterns, and basic authentication mechanisms using preimage-hash validation.

## Purpose
This is the most basic lock script pattern that demonstrates:
- Script entry point setup
- Argument loading from script
- Witness data access
- Hash verification
- Return code handling

## Complete Working Example

```rust
#![no_std]
#![cfg_attr(not(test), no_main)]

use ckb_hash::blake2b_256;
use ckb_std::ckb_constants::Source;
use ckb_std::error::SysError;
use ckb_std::{entry, default_alloc, high_level::{load_script, load_witness_args}};

entry!(program_entry);
default_alloc!();

#[repr(i8)]
pub enum Error {
    IndexOutOfBound = 1,
    ItemMissing,
    LengthNotEnough,
    Encoding,
    CheckError,
    UnMatch,
}

impl From<SysError> for Error {
    fn from(err: SysError) -> Self {
        match err {
            SysError::IndexOutOfBound => Self::IndexOutOfBound,
            SysError::ItemMissing => Self::ItemMissing,
            SysError::LengthNotEnough(_) => Self::LengthNotEnough,
            SysError::Encoding => Self::Encoding,
            _ => Self::CheckError,
        }
    }
}

pub fn program_entry() -> i8 {
    match main() {
        Ok(_) => 0,
        Err(err) => err as i8,
    }
}

fn main() -> Result<(), Error> {
    // Load script to get expected hash from args
    let script = load_script()?;
    let expect_hash = script.args().raw_data().to_vec();

    // Load witness to get preimage
    let witness_args = load_witness_args(0, Source::GroupInput)?;
    let preimage = witness_args
        .lock()
        .to_opt()
        .ok_or(Error::CheckError)?
        .raw_data();

    // Verify hash matches
    let hash = blake2b_256(preimage.as_ref());
    if hash.eq(&expect_hash.as_ref()) {
        Ok(())
    } else {
        Err(Error::UnMatch)
    }
}
```

## Key Patterns
1. **Script Setup**: `entry!()` and `default_alloc!()` macros
2. **Error Handling**: Custom error enum with SysError conversion
3. **Script Args Access**: `load_script()?.args().raw_data()`
4. **Witness Access**: `load_witness_args(0, Source::GroupInput)?`
5. **Return Codes**: 0 for success, non-zero for failure

## When to Use
- Simple ownership verification
- Hash-based locks
- Password-protected cells
- Educational examples

## Memory Configuration

Standard CKB script memory allocation:
```rust
ckb_std::default_alloc!(16384, 1258306, 64);
// 16KB fixed heap + 1.2MB dynamic heap + 64-byte min blocks
```

## Script Template Variants

### Basic Template Structure
```rust
#![cfg_attr(not(any(feature = "library", test)), no_std)]
#![cfg_attr(not(test), no_main)]

ckb_std::entry!(program_entry);
ckb_std::default_alloc!(16384, 1258306, 64);

pub fn program_entry() -> i8 {
    match main() {
        Ok(_) => 0,
        Err(err) => err as i8,
    }
}
```

### Testing Integration
For contracts with test support:
```rust
#[cfg(test)]
extern crate alloc;

#[cfg(test)]
mod tests;
```

## Common Variations

### Simple Always-Success Lock
```rust
pub fn program_entry() -> i8 {
    0  // Always allow unlock
}
```

### Debug-Enabled Version
```rust
pub fn program_entry() -> i8 {
    ckb_std::debug!("Lock script executing");
    // validation logic here
    0
}
```

## Build Configuration

Standard Cargo.toml setup:
```toml
[package]
name = "minimal-lock"
version = "0.1.0"
edition = "2021"

[dependencies]
ckb-std = "0.15.1"
ckb-hash = "0.114.0"

[features]
library = []
```

## Deployment Notes

- Compile to RISC-V target: `riscv64imac-unknown-none-elf`
- Binary size typically 2-4KB for minimal implementations
- Gas costs scale with script complexity and data access patterns