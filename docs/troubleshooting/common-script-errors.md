## Description

Comprehensive troubleshooting guide for CKB script development covering common runtime errors, debugging techniques, and practical solutions. Addresses frequent issues like IndexOutOfBound, ItemMissing, and InvalidData errors with detailed code examples showing both problematic and corrected approaches. Includes error code explanations, syscall debugging patterns, capacity calculation issues, and transaction validation failures. Essential reference for developers debugging CKB smart contracts and understanding error handling best practices in the CKB-VM environment.

## Script Execution Errors

### Error Code 1: IndexOutOfBound
```rust
// Problem: Trying to access cell/witness that doesn't exist
let cell = load_cell(999, Source::Input)?; // ❌ Index too high

// Solution: Check bounds or use iteration
let mut index = 0;
loop {
    match load_cell(index, Source::Input) {
        Ok(cell) => {
            // Process cell
            index += 1;
        }
        Err(SysError::IndexOutOfBound) => break, // ✅ Normal termination
        Err(e) => return Err(e.into()),
    }
}
```

### Error Code 2: ItemMissing  
```rust
// Problem: Required witness/data is missing
let witness_args = load_witness_args(0, Source::GroupInput)?;
let lock = witness_args.lock().to_opt().unwrap(); // ❌ Panics if missing

// Solution: Handle missing data gracefully
let lock = witness_args
    .lock()
    .to_opt()
    .ok_or(Error::MissingLockWitness)?; // ✅ Proper error handling
```

### Error Code 3: LengthNotEnough
```rust
// Problem: Buffer too small for data
let mut buffer = [0u8; 10];
let data = load_cell_data(0, Source::Input)?; // ❌ Data might be > 10 bytes

// Solution: Use dynamic allocation or check size
let data = load_cell_data(0, Source::Input)?; // ✅ Returns Vec<u8>
```

## Transaction Validation Errors

### Invalid Cell Reference
```rust
// Problem: OutPoint references non-existent cell
let invalid_outpoint = OutPoint::new_builder()
    .tx_hash(H256::from([0u8; 32])) // ❌ Invalid hash
    .index(0u32.pack())
    .build();

// Solution: Use valid transaction hash from blockchain
let valid_outpoint = OutPoint::new_builder()
    .tx_hash(actual_tx_hash) // ✅ Real transaction hash
    .index(0u32.pack())
    .build();
```

### Insufficient Capacity
```rust
// Problem: Output capacity < minimum required
let output = CellOutput::new_builder()
    .capacity(Capacity::shannons(1000).pack()) // ❌ Too small
    .lock(lock_script)
    .build();

// Solution: Calculate minimum capacity
fn min_capacity(lock: &Script, type_opt: &Option<Script>, data_len: usize) -> u64 {
    61_00000000 + // Basic overhead
    lock.as_slice().len() as u64 +
    type_opt.as_ref().map_or(0, |t| t.as_slice().len() as u64) +
    data_len as u64
}

let required = min_capacity(&lock_script, &None, data.len());
let output = CellOutput::new_builder()
    .capacity(Capacity::shannons(required).pack()) // ✅ Sufficient capacity
    .lock(lock_script)
    .build();
```

## Type Script Conservation Errors
```rust
// Problem: Token conservation violation
fn validate_conservation() -> Result<(), Error> {
    let input_amount = 1000u128;
    let output_amount = 1001u128; // ❌ Creating tokens out of thin air
    
    if input_amount < output_amount {
        return Err(Error::ConservationViolation);
    }
    Ok(())
}

// Solution: Proper conservation check with owner mode
fn validate_conservation_with_owner_mode() -> Result<(), Error> {
    let script = load_script()?;
    let args = script.args().raw_data();
    
    // Check if any input uses owner lock (minting allowed)
    let owner_mode = QueryIter::new(load_cell_lock_hash, Source::Input)
        .any(|lock_hash| args[..] == lock_hash[..]);
    
    if owner_mode {
        return Ok(()); // ✅ Owner can mint/burn
    }
    
    // Otherwise enforce conservation
    let input_total = collect_input_amounts()?;
    let output_total = collect_output_amounts()?;
    
    if input_total < output_total {
        return Err(Error::ConservationViolation);
    }
    
    Ok(())
}
```

## Debugging Techniques

### Debug Logging
```rust
// Add debug statements to trace execution
ckb_std::debug!("Script started with args: {:?}", args);
ckb_std::debug!("Input amount: {}", input_amount);
ckb_std::debug!("Output amount: {}", output_amount);

// Use conditional debugging
#[cfg(debug_assertions)]
ckb_std::debug!("Debug: Processing cell {}", index);
```

### Error Context
```rust
// Provide context in error messages
#[repr(i8)]
pub enum Error {
    IndexOutOfBound = 1,
    InvalidDataLength = 5,
    UnexpectedInputCount = 6,
    ConservationViolation = 7,
}

// Add error context
fn validate_input_count() -> Result<(), Error> {
    let input_count = count_inputs(Source::GroupInput)?;
    if input_count != 1 {
        ckb_std::debug!("Expected 1 input, got {}", input_count);
        return Err(Error::UnexpectedInputCount);
    }
    Ok(())
}
```

### Testing Patterns
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_conditions() {
        // Test IndexOutOfBound
        let result = load_cell(999, Source::Input);
        assert!(matches!(result, Err(SysError::IndexOutOfBound)));
        
        // Test conservation violation
        let result = validate_with_insufficient_input();
        assert!(matches!(result, Err(Error::ConservationViolation)));
    }
}
```

## Best Practice: Using Granular Error Codes

For improved debugging efficiency and AI-assisted development, implement granular error codes that provide specific information about each failure condition. Instead of generic errors like "InvalidTransaction" that could indicate multiple different problems, use precise error codes like "MultipleInputsNotAllowed" or "RequiredOutputMissing" that immediately identify the exact issue.

**Reference**: See the [Granular Error Code Pattern](ckb://docs/scripts/rust-patterns#granular-error-code-pattern) for detailed implementation examples and best practices.

```rust
// ❌ Generic error codes mask the root cause
pub enum Error {
    InvalidTransaction = 4,  // Could be anything!
    InvalidArgs = 5,         // Too vague
}

// ✅ Granular error codes pinpoint exact issues
pub enum Error {
    MultipleInputsNotAllowed = 10,
    RequiredOutputMissing = 11,
    InvalidOwnerLockHash = 20,
    ArgumentLengthIncorrect = 21,
}
```

## Common Anti-Patterns

### ❌ Ignoring Errors
```rust
let data = load_cell_data(0, Source::Input).unwrap(); // Dangerous!
```

### ✅ Proper Error Handling
```rust
let data = load_cell_data(0, Source::Input)
    .map_err(|e| Error::from(e))?;
```

### ❌ Hardcoded Indices
```rust
let input_cell = load_cell(0, Source::Input)?; // Assumes exactly one input
```

### ✅ Dynamic Iteration
```rust
for cell in QueryIter::new(load_cell, Source::GroupInput) {
    // Process each cell
}
```