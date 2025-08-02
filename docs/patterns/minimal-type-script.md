# Minimal Type Script Pattern

## Description

Master the fundamental type script pattern for CKB state validation and business logic enforcement. Learn input/output cell iteration, data validation, conservation checks, group cell processing, owner mode implementation, and UDT token amount verification. Essential foundation for building robust token contracts and state transition validation systems.

## Purpose
Type scripts validate state transitions and enforce rules on how cells can be used. This pattern shows:
- Input/output cell iteration
- Data validation
- Conservation checks
- Group cell processing

## Complete Working Example

```rust
#![no_std]
#![cfg_attr(not(test), no_main)]

use ckb_std::{
    ckb_constants::Source,
    ckb_types::{bytes::Bytes, prelude::*},
    error::SysError,
    high_level::{load_cell_data, load_cell_lock_hash, load_script, QueryIter},
    entry, default_alloc,
};

entry!(program_entry);
default_alloc!();

const UDT_AMOUNT_LEN: usize = 16;

#[repr(i8)]
pub enum Error {
    IndexOutOfBound = 1,
    ItemMissing,
    LengthNotEnough,
    Encoding,
    AmountEncoding = 12,
    InvalidAmount,
}

impl From<SysError> for Error {
    fn from(err: SysError) -> Self {
        match err {
            SysError::IndexOutOfBound => Self::IndexOutOfBound,
            SysError::ItemMissing => Self::ItemMissing,
            SysError::LengthNotEnough(_) => Self::LengthNotEnough,
            SysError::Encoding => Self::Encoding,
            _ => Self::Encoding,
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
    let script = load_script()?;
    let args: Bytes = script.args().unpack();

    // Check owner mode - if any input uses owner lock, allow anything
    if check_owner_mode(&args) {
        return Ok(());
    }

    // Otherwise, verify conservation of amounts
    let inputs_amount = collect_inputs_amount()?;
    let outputs_amount = collect_outputs_amount()?;

    if inputs_amount < outputs_amount {
        return Err(Error::InvalidAmount);
    }

    Ok(())
}

fn check_owner_mode(args: &Bytes) -> bool {
    QueryIter::new(load_cell_lock_hash, Source::Input)
        .any(|lock_hash| args[..] == lock_hash[..])
}

fn collect_inputs_amount() -> Result<u128, Error> {
    let mut buf = [0u8; UDT_AMOUNT_LEN];
    let amounts: Result<Vec<u128>, Error> = QueryIter::new(load_cell_data, Source::GroupInput)
        .map(|data| {
            if data.len() >= UDT_AMOUNT_LEN {
                buf.copy_from_slice(&data[0..UDT_AMOUNT_LEN]);
                Ok(u128::from_le_bytes(buf))
            } else {
                Err(Error::AmountEncoding)
            }
        })
        .collect();
    
    Ok(amounts?.into_iter().sum())
}

fn collect_outputs_amount() -> Result<u128, Error> {
    let mut buf = [0u8; UDT_AMOUNT_LEN];
    let amounts: Result<Vec<u128>, Error> = QueryIter::new(load_cell_data, Source::GroupOutput)
        .map(|data| {
            if data.len() >= UDT_AMOUNT_LEN {
                buf.copy_from_slice(&data[0..UDT_AMOUNT_LEN]);
                Ok(u128::from_le_bytes(buf))
            } else {
                Err(Error::AmountEncoding)
            }
        })
        .collect();
    
    Ok(amounts?.into_iter().sum())
}
```

## Key Patterns
1. **Group Processing**: `Source::GroupInput` and `Source::GroupOutput`
2. **Data Iteration**: `QueryIter::new(load_cell_data, source)`
3. **Owner Mode**: Check if any input uses the owner's lock
4. **Conservation Check**: Verify inputs >= outputs
5. **Data Parsing**: Convert bytes to structured data

## When to Use
- Token/UDT implementations
- State transition validation
- Conservation law enforcement
- Multi-cell operations

## Type Script vs Lock Script

**Type Script Characteristics:**
- Validates state transitions
- Enforces business logic rules
- Processes grouped cells
- Can read all transaction cells
- Runs when cells with matching type exist

**Lock Script Characteristics:**
- Validates ownership and permissions
- Processes individual cells
- Limited to current cell context
- Runs when cells with matching lock exist

## Source Iterations

```rust
use ckb_std::ckb_constants::Source;

// Process only cells with same type script
Source::GroupInput    // Input cells with same type
Source::GroupOutput   // Output cells with same type

// Process all transaction cells
Source::Input         // All input cells
Source::Output        // All output cells
```

## Advanced Patterns

### Batch Token Transfer Validation
```rust
fn validate_batch_transfer() -> Result<(), Error> {
    let script = load_script()?;
    let owner_lock_hash = script.args().raw_data();
    
    // Check if any input has owner lock (owner mode)
    let is_owner_mode = QueryIter::new(load_cell_lock_hash, Source::Input)
        .any(|lock_hash| lock_hash[..] == owner_lock_hash[..]);
    
    if is_owner_mode {
        return Ok(()); // Owner can do anything
    }
    
    // Validate conservation for normal transfers
    validate_token_conservation()
}
```

### Multi-Asset Support
```rust
fn load_typed_data<T: Default + FromBytes>(
    index: usize, 
    source: Source
) -> Result<T, Error> {
    let data = load_cell_data(index, source)?;
    T::from_bytes(&data).map_err(|_| Error::Encoding)
}
```

### Gas Optimization Patterns
```rust
// Early exit for owner mode
if check_owner_mode(&args) {
    return Ok(());
}

// Lazy evaluation for expensive checks
let input_sum = QueryIter::new(load_cell_data, Source::GroupInput)
    .try_fold(0u128, |acc, data| {
        parse_amount(&data).map(|amount| acc + amount)
    })?;
```

## Error Handling Standards

```rust
#[repr(i8)]
pub enum Error {
    IndexOutOfBound = 1,
    ItemMissing = 2,
    LengthNotEnough = 3,
    Encoding = 4,
    // Custom errors start from 10
    AmountEncoding = 10,
    InvalidAmount = 11,
    InsufficientBalance = 12,
    UnauthorizedOperation = 13,
}
```

## Testing Integration

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ckb_testtool::{builtin::ALWAYS_SUCCESS, context::Context};
    
    #[test]
    fn test_basic_transfer() {
        let mut context = Context::default();
        // Test setup and validation
    }
}
```

## Deployment Configuration

Standard type script deployment pattern:
- Deploy as data in system transaction
- Reference by data hash in script
- Use Type ID pattern for upgradeable contracts
- Consider capacity requirements for data storage