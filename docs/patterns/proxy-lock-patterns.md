# Proxy Lock Patterns

## Description

Advanced lock script patterns for CKB providing delegation, time-based constraints, and conditional unlocking mechanisms. Covers delegate locks, proxy locks, time locks, single-use locks, type-based locks, and ownership delegation patterns. Essential for building sophisticated authorization systems, vesting schedules, token-gated access, and hierarchical permission structures.

## Introduction

Proxy lock patterns enable sophisticated authorization mechanisms beyond simple signature verification. These patterns support delegation, time constraints, conditional unlocking, and complex permission hierarchies.

## Core Concepts

### Lock Delegation
Transfer unlocking authority to another lock script without changing ownership permanently.

### Conditional Unlocking  
Unlock cells based on transaction context (inputs, outputs, time, etc.).

### Proxy Authorization
Enable one lock script to authorize actions for another.

### Temporal Constraints
Time, block, or epoch-based unlocking restrictions.

## Delegate Lock Pattern

### Purpose
Generic ownership delegation allowing temporary or conditional control transfer.

### Structure
```rust
// Delegate Lock Args
struct DelegateLockArgs {
    mode_flags: u8,           // Feature flags
    delegate_script_hash: [u8; 32], // Target script hash
    delegate_data_hash: Option<[u8; 32]>, // Optional data validation
}
```

### Mode Flags
```rust
const DELEGATE_TYPE_SCRIPT: u8 = 0b00000001; // Delegate to type script (vs lock)
const FORBID_TRADE: u8 = 0b00000010;         // Prevent re-trading  
const SELF_DESTRUCTION: u8 = 0b00000100;     // Must destroy on unlock
const RESTRICT_DATA: u8 = 0b00001000;        // Require data hash match
```

### Implementation Example
```rust
use ckb_std::{
    ckb_constants::Source,
    high_level::{load_cell_type_hash, load_cell_lock_hash, QueryIter},
};

pub fn main() -> Result<(), Error> {
    let script = load_script()?;
    let args = parse_delegate_args(script.args())?;
    
    // Check self-destruction requirement
    if args.flags.self_destruction {
        verify_cell_destruction()?;
    }
    
    // Check trade restrictions  
    if args.flags.forbid_trade {
        verify_no_reuse_in_outputs()?;
    }
    
    // Verify delegate authorization
    let target = if args.flags.delegate_type_script {
        LoadHashTarget::Type
    } else {
        LoadHashTarget::Lock  
    };
    
    verify_delegate_presence(args.ref_hash, target)?;
    
    // Optional data validation
    if let Some(data_hash) = args.data_hash {
        verify_data_hash_match(data_hash)?;
    }
    
    Ok(())
}

fn verify_delegate_presence(hash: [u8; 32], target: LoadHashTarget) -> Result<(), Error> {
    let found = match target {
        LoadHashTarget::Type => {
            QueryIter::new(load_cell_type_hash, Source::Input)
                .any(|type_hash| type_hash.unwrap_or_default() == hash)
        }
        LoadHashTarget::Lock => {
            QueryIter::new(load_cell_lock_hash, Source::Input)
                .any(|lock_hash| lock_hash == hash)
        }
    };
    
    if !found {
        return Err(Error::DelegateNotFound);
    }
    
    Ok(())
}
```

### Use Cases
- **Temporary Delegation**: Grant temporary control without permanent transfer
- **Conditional Ownership**: Ownership contingent on other cell presence
- **Hierarchical Permissions**: Multi-level authorization systems
- **Trustless Escrow**: Lock funds until conditions are met

## Time Lock Pattern

### Purpose
Combine temporal constraints with lock script verification for time-based unlocking.

### Structure
```rust
// Time Lock Args: lock_hash (32 bytes) + since_value (8 bytes)
struct TimeLockArgs {
    required_lock_hash: [u8; 32], // Must be present in inputs
    locked_until: u64,            // Since value (time/block/epoch)
}
```

### Implementation Example
```rust
use ckb_std::{
    ckb_constants::Source,
    high_level::{load_cell_lock_hash, load_input_since, QueryIter},
    since::Since,
};

pub fn main() -> Result<(), Error> {
    let script = load_script()?;
    let args = parse_time_lock_args(script.args())?;
    
    // Verify required lock script exists in inputs
    if !QueryIter::new(load_cell_lock_hash, Source::Input)
        .any(|lock_hash| lock_hash == args.required_lock_hash) {
        return Err(Error::RequiredLockNotFound);
    }
    
    // Verify time constraint is satisfied
    let locked_until = Since::new(args.locked_until);
    for since_value in QueryIter::new(load_input_since, Source::GroupInput) {
        let since = Since::new(since_value);
        if since.lt(&locked_until) {
            return Err(Error::LockTimeNotPassed);
        }
    }
    
    Ok(())
}
```

### Since Value Formats
```rust
// Block number based (most common)
let block_lock = 1000u64; // Unlock after block 1000

// Epoch based  
let epoch_lock = (1u64 << 56) | 100u64; // Unlock after epoch 100

// Timestamp based (seconds since Unix epoch)
let time_lock = (2u64 << 56) | 1640995200u64; // Unlock after timestamp
```

### Use Cases
- **Vesting Schedules**: Gradual token release over time
- **Time-Delayed Payments**: Prevent immediate spending
- **Escrow with Timeout**: Auto-release after time period
- **Subscription Locks**: Time-based access control

## Type-Based Lock Patterns

### Input Type Proxy Lock
Unlocks when specific type script appears in transaction inputs.

```rust
pub fn main() -> Result<(), Error> {
    let script = load_script()?;
    let required_type_hash = parse_type_hash(script.args())?;
    
    // Check if required type script exists in inputs
    let found = QueryIter::new(load_cell_type_hash, Source::Input)
        .any(|type_hash| type_hash.unwrap_or_default() == required_type_hash);
        
    if !found {
        return Err(Error::RequiredTypeNotFound);
    }
    
    Ok(())
}
```

### Output Type Proxy Lock  
Unlocks when specific type script appears in transaction outputs.

```rust
pub fn main() -> Result<(), Error> {
    let script = load_script()?;
    let required_type_hash = parse_type_hash(script.args())?;
    
    // Check if required type script exists in outputs
    let found = QueryIter::new(load_cell_type_hash, Source::Output)
        .any(|type_hash| type_hash.unwrap_or_default() == required_type_hash);
        
    if !found {
        return Err(Error::RequiredTypeNotFound);
    }
    
    Ok(())
}
```

### Type Burn Lock
Unlocks when specific type script is consumed (appears in inputs but not outputs).

```rust
pub fn main() -> Result<(), Error> {
    let script = load_script()?;
    let burn_type_hash = parse_type_hash(script.args())?;
    
    // Verify type exists in inputs
    let in_inputs = QueryIter::new(load_cell_type_hash, Source::Input)
        .any(|type_hash| type_hash.unwrap_or_default() == burn_type_hash);
        
    if !in_inputs {
        return Err(Error::BurnTypeNotInInputs);
    }
    
    // Verify type does NOT exist in outputs (burned)
    let in_outputs = QueryIter::new(load_cell_type_hash, Source::Output)
        .any(|type_hash| type_hash.unwrap_or_default() == burn_type_hash);
        
    if in_outputs {
        return Err(Error::BurnTypeStillExists);
    }
    
    Ok(())
}
```

### Use Cases
- **Token-Gated Access**: Require specific token ownership
- **NFT-Based Permissions**: Access based on NFT possession  
- **Proof of Burn**: Verify asset destruction
- **Conditional Payments**: Payment upon asset creation/destruction

## Single-Use Lock Pattern

### Purpose
One-time unlock based on consuming a specific outpoint.

### Implementation
```rust
use ckb_std::{
    ckb_constants::Source,
    ckb_types::prelude::Unpack,
    high_level::{load_input, QueryIter},
};

pub fn main() -> Result<(), Error> {
    let script = load_script()?;
    let required_outpoint = parse_outpoint(script.args())?;
    
    // Check if required outpoint is consumed in this transaction
    let found = QueryIter::new(load_input, Source::Input)
        .any(|input| {
            let previous_output = input.previous_output();
            previous_output.tx_hash().unpack() == required_outpoint.tx_hash
                && previous_output.index().unpack() == required_outpoint.index
        });
        
    if !found {
        return Err(Error::RequiredOutpointNotConsumed);
    }
    
    Ok(())
}

struct OutPoint {
    tx_hash: [u8; 32],
    index: u32,
}

fn parse_outpoint(args: &[u8]) -> Result<OutPoint, Error> {
    if args.len() != 36 {
        return Err(Error::InvalidArgs);
    }
    
    let mut tx_hash = [0u8; 32];
    tx_hash.copy_from_slice(&args[0..32]);
    let index = u32::from_le_bytes(args[32..36].try_into().unwrap());
    
    Ok(OutPoint { tx_hash, index })
}
```

### Use Cases
- **Voucher Systems**: One-time redemption tokens
- **Access Tokens**: Single-use authorization
- **Proof of Payment**: Verify specific payment was made
- **Contest Entries**: One entry per specific outpoint

## Lock Proxy Pattern

### Purpose
Delegate unlocking authority to any cell using a specific lock script.

### Implementation
```rust
pub fn main() -> Result<(), Error> {
    let script = load_script()?;
    let owner_lock_hash = parse_lock_hash(script.args())?;
    
    // Check if owner lock script exists in transaction inputs
    let authorized = QueryIter::new(load_cell_lock_hash, Source::Input)
        .any(|lock_hash| lock_hash == owner_lock_hash);
        
    if !authorized {
        return Err(Error::OwnerLockNotFound);
    }
    
    Ok(())
}
```

### Use Cases
- **Multi-sig Integration**: Delegate to multi-sig lock
- **Hierarchical Wallets**: Parent-child key relationships
- **Service Authentication**: Delegate to service-specific locks
- **Emergency Recovery**: Backup authorization mechanisms

## Advanced Patterns

### Composite Conditions
```rust
// Multiple conditions must be satisfied
pub fn main() -> Result<(), Error> {
    let script = load_script()?;
    let args = parse_composite_args(script.args())?;
    
    // Condition 1: Time constraint
    verify_time_constraint(args.time_lock)?;
    
    // Condition 2: Required lock present  
    verify_lock_presence(args.required_lock)?;
    
    // Condition 3: Token balance sufficient
    verify_token_balance(args.min_balance)?;
    
    Ok(())
}
```

### Upgradeable Proxy
```rust
// Proxy that can change its delegate target
pub fn main() -> Result<(), Error> {
    let script = load_script()?;
    let args = parse_upgradeable_args(script.args())?;
    
    if is_upgrade_transaction()? {
        verify_upgrade_authorization(args.admin_lock)?;
        update_delegate_target()?;
    } else {
        verify_current_delegate(args.current_delegate)?;
    }
    
    Ok(())
}
```

### State-Based Locks
```rust
// Lock behavior changes based on cell data
pub fn main() -> Result<(), Error> {
    let data = load_current_cell_data()?;
    let state = parse_cell_state(&data)?;
    
    match state {
        State::Locked => verify_unlock_conditions()?,
        State::Vesting => verify_vesting_schedule()?,
        State::Mature => allow_unrestricted_transfer()?,
    }
    
    Ok(())
}
```

## Security Considerations

### Delegation Risks
- **Permanent Delegation**: Ensure revocability when needed
- **Circular Dependencies**: Avoid A delegates to B delegates to A
- **Key Compromise**: Consider time limits on delegations

### Time Lock Vulnerabilities  
- **Clock Manipulation**: Use block-based rather than timestamp when possible
- **Since Field Validation**: Ensure proper since value format
- **Timezone Issues**: Use UTC for timestamp-based locks

### Type Script Dependencies
- **Type Script Changes**: Consider type script upgradeability
- **Hash Collisions**: Extremely unlikely but theoretically possible
- **Missing Dependencies**: Verify type scripts exist before relying on them

## Best Practices

### Error Handling
```rust
#[repr(i8)]
pub enum Error {
    IndexOutOfBound = 1,
    ItemMissing = 2,
    LengthNotEnough = 3,
    Encoding = 4,
    // Proxy-specific errors
    DelegateNotFound = 10,
    TimeConstraintNotMet = 11,
    RequiredTypeNotFound = 12,
    UnauthorizedOperation = 13,
}
```

### Argument Validation
```rust
fn validate_args(args: &[u8], expected_len: usize) -> Result<(), Error> {
    if args.len() != expected_len {
        return Err(Error::InvalidArgs);
    }
    
    // Additional validation logic
    Ok(())
}
```

### Gas Optimization
```rust
// Early exit patterns
if !basic_condition_met() {
    return Err(Error::BasicConditionFailed);
}

// Avoid expensive operations when possible
if let Some(cached_result) = check_cache() {
    return Ok(cached_result);
}
```

## Testing Patterns

### Unit Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ckb_testtool::{builtin::ALWAYS_SUCCESS, context::Context};
    
    #[test]
    fn test_time_lock_basic() {
        let mut context = Context::default();
        
        // Setup test scenario
        let time_lock = build_time_lock_script(owner_lock_hash, future_time);
        
        // Test before time passes
        let result = context.verify_script(&time_lock);
        assert!(result.is_err());
        
        // Test after time passes  
        context.set_current_time(future_time + 1);
        let result = context.verify_script(&time_lock);
        assert!(result.is_ok());
    }
}
```

## Deployment Information

### Contract Addresses
Refer to the latest deployment information in the ckb-proxy-locks repository for current mainnet and testnet addresses.

### Build Configuration
```toml
[package]
name = "proxy-lock"
version = "0.1.0"
edition = "2021"

[dependencies]
ckb-std = "0.15.1"
ckb-hash = "0.114.0"

[profile.release]
overflow-checks = true
opt-level = "s"
lto = true
codegen-units = 1
panic = "abort"
```