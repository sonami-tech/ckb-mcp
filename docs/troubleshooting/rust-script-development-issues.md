## Description

Troubleshooting guide for CKB script development in Rust addressing compilation issues, target architecture problems, memory management challenges, and debugging techniques. RISC-V compilation setup, no_std environment configuration, syscall integration, dependency management, and performance optimization. Solutions for common build errors, memory allocation issues, and deployment problems specific to CKB-VM development environment.

## Overview

Common issues when developing CKB scripts in Rust, with solutions and best practices based on real-world development experience.

## Build and Compilation Issues

### 1. Target Architecture Problems

**Issue**: Cannot compile for RISC-V target
```
error[E0463]: can't find crate for `std`
```

**Solution**: Ensure proper target configuration
```toml
# Cargo.toml
[build]
target = "riscv64imac-unknown-none-elf"

# Install target if missing
rustup target add riscv64imac-unknown-none-elf
```

**Alternative using .cargo/config.toml**:
```toml
[build]
target = "riscv64imac-unknown-none-elf"

[target.riscv64imac-unknown-none-elf]
runner = "ckb-debugger"
```

### 2. No-Std Environment Issues

**Issue**: Standard library functions not available
```rust
error[E0433]: failed to resolve: use of undeclared crate or module `std`
```

**Solution**: Use `core` and `alloc` instead of `std`
```rust
// ❌ Wrong
use std::vec::Vec;
use std::collections::HashMap;
use std::string::String;

// ✅ Correct
use alloc::vec::Vec;
use alloc::collections::BTreeMap; // HashMap not available in no_std
use alloc::string::String;

// Add at top of main.rs
#![cfg_attr(not(any(feature = "library", test)), no_std)]
#[cfg(any(feature = "library", test))]
extern crate alloc;
```

### 3. Memory Allocation Issues

**Issue**: Heap allocation failures
```
error: no global memory allocator found but one is required
```

**Solution**: Configure heap allocator
```rust
// Add to main.rs
ckb_std::default_alloc!(16384, 1258306, 64);
//                      ^      ^       ^
//                      |      |       Minimal block size (64 bytes)
//                      |      Dynamic heap (1.2MB rounded up)
//                      Fixed heap (16KB)
```

**Custom allocator configuration**:
```rust
// For memory-intensive scripts
ckb_std::default_alloc!(32768, 2516612, 128);

// For simple scripts
ckb_std::default_alloc!(8192, 629153, 32);
```

### 4. Binary Size Issues

**Issue**: Script binary too large
```
Transaction verification failed: ExceededMaximumCycles
```

**Solution**: Optimize build configuration
```toml
[profile.release]
opt-level = "s"        # Optimize for size
lto = true            # Link-time optimization
codegen-units = 1     # Single codegen unit
panic = "abort"       # Smaller panic handler
strip = true          # Strip debug symbols
overflow-checks = true # Keep overflow checks for safety
```

**Advanced size optimization**:
```bash
# Check binary size breakdown
cargo bloat --release --crates

# Use wee_alloc for smaller allocator
# Add to Cargo.toml dependencies:
wee_alloc = { version = "0.4.5", default-features = false }

# Enable in main.rs:
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
```

## Runtime Errors

### 1. Syscall Errors

**Issue**: Index out of bounds errors
```rust
Error::IndexOutOfBound
```

**Common Causes and Solutions**:

```rust
// ❌ Wrong: Assuming cells exist
let data = load_cell_data(0, Source::GroupInput)?; // May not exist

// ✅ Correct: Check if cells exist
let cells: Vec<_> = QueryIter::new(load_cell_data, Source::GroupInput).collect();
if cells.is_empty() {
    return Ok(()); // No cells to process
}

// ✅ Alternative: Handle the error gracefully
match load_cell_data(0, Source::GroupInput) {
    Ok(data) => process_data(&data)?,
    Err(ckb_std::error::SysError::IndexOutOfBound) => return Ok(()),
    Err(e) => return Err(Error::Syscall(e)),
}
```

### 2. Memory Access Violations

**Issue**: Buffer overflow or underflow
```rust
// ❌ Dangerous: Unchecked slice access
let amount = u128::from_le_bytes(data[0..16]); // Panics if data.len() < 16
```

**Solution**: Always validate before accessing
```rust
// ✅ Safe: Check bounds first
if data.len() < 16 {
    return Err(Error::InvalidData);
}
let mut buffer = [0u8; 16];
buffer.copy_from_slice(&data[0..16]);
let amount = u128::from_le_bytes(buffer);

// ✅ Alternative: Use safe conversion
let amount = data.get(0..16)
    .and_then(|slice| slice.try_into().ok())
    .map(u128::from_le_bytes)
    .ok_or(Error::InvalidData)?;
```

### 3. Arithmetic Overflow

**Issue**: Integer overflow in calculations
```rust
// ❌ Can overflow silently in release mode
let total = input_amount + output_amount;
```

**Solution**: Use checked arithmetic
```rust
// ✅ Safe: Checked arithmetic
let total = input_amount.checked_add(output_amount)
    .ok_or(Error::Overflow)?;

// ✅ Alternative: Saturating arithmetic (if overflow should be capped)
let total = input_amount.saturating_add(output_amount);

// ✅ For debugging: Enable overflow checks in release
// Add to Cargo.toml
[profile.release]
overflow-checks = true
```

### 4. Type Script vs Lock Script Context

**Issue**: Using wrong Source for script type
```rust
// ❌ Wrong: Lock script trying to use GroupOutput
// Lock scripts don't execute on outputs
for cell in QueryIter::new(load_cell, Source::GroupOutput) {
    // This loop never executes for lock scripts!
}
```

**Solution**: Use appropriate sources
```rust
// ✅ Lock script: Only use GroupInput
pub fn lock_script_main() -> Result<(), Error> {
    for cell in QueryIter::new(load_cell, Source::GroupInput) {
        validate_spending_authorization(&cell)?;
    }
    Ok(())
}

// ✅ Type script: Can use both GroupInput and GroupOutput
pub fn type_script_main() -> Result<(), Error> {
    let input_amount = sum_amounts(Source::GroupInput)?;
    let output_amount = sum_amounts(Source::GroupOutput)?;
    
    if output_amount > input_amount {
        return Err(Error::TokenInflation);
    }
    Ok(())
}
```

## Testing Issues

### 1. Test Setup Problems

**Issue**: Tests fail to compile or run
```
error: could not find `ckb_tool` in registry
```

**Solution**: Proper test configuration
```toml
# tests/Cargo.toml
[dev-dependencies]
ckb-tool = "0.107.0"
ckb-hash = "0.107.0"
ckb-chain-spec = "0.107.0"

[features]
default = []
```

**Test structure**:
```rust
// tests/src/lib.rs
use ckb_tool::{
    ckb_types::{
        core::{TransactionBuilder, TransactionView},
        packed::*,
        prelude::*,
    },
    context::Context,
};

const MAX_CYCLES: u64 = 10_000_000;

pub struct TestContext {
    context: Context,
    contract_out_point: OutPoint,
}

impl TestContext {
    pub fn new() -> Self {
        let mut context = Context::default();
        let contract_bin = include_bytes!("../../build/my-script");
        let contract_out_point = context.deploy_cell(contract_bin.to_vec().into());
        
        Self { context, contract_out_point }
    }
}
```

### 2. Transaction Building in Tests

**Issue**: Complex transaction setup
```rust
// ❌ Repetitive transaction building
let tx = TransactionBuilder::default()
    .input(/* complex setup */)
    .output(/* complex setup */)
    .witness(/* complex setup */)
    .build();
```

**Solution**: Create helper functions
```rust
pub fn build_simple_transfer(
    input_capacity: u64,
    output_capacity: u64,
    script: &Script,
) -> TransactionView {
    let input = CellInput::new(OutPoint::null(), 0);
    let input_cell = CellOutput::new_builder()
        .capacity(input_capacity.pack())
        .lock(Script::default())
        .type_(Some(script.clone()).pack())
        .build();
    
    let output = CellOutput::new_builder()
        .capacity(output_capacity.pack())
        .lock(Script::default())
        .type_(Some(script.clone()).pack())
        .build();
    
    TransactionBuilder::default()
        .input(input)
        .output(output)
        .outputs_data(vec![Bytes::new()].pack())
        .build()
}

// Usage in tests
#[test]
fn test_token_conservation() {
    let mut ctx = TestContext::new();
    let script = ctx.build_script(&[]);
    
    let tx = build_simple_transfer(200, 150, &script);
    let cycles = ctx.verify_tx(&tx).expect("should pass");
    println!("Consumed {} cycles", cycles);
}
```

### 3. Mock Data Generation

**Issue**: Need realistic test data
```rust
// Helper functions for test data
pub fn build_udt_cell(amount: u128, script: &Script) -> (CellOutput, Bytes) {
    let data = Bytes::from(amount.to_le_bytes().to_vec());
    let cell = CellOutput::new_builder()
        .capacity(calculate_capacity(&data, script))
        .lock(Script::default())
        .type_(Some(script.clone()).pack())
        .build();
    
    (cell, data)
}

pub fn calculate_capacity(data: &Bytes, script: &Script) -> Capacity {
    let occupied = data.len() + script.as_slice().len() + 8 + 32; // rough estimate
    Capacity::shannons((occupied as u64).max(61_00000000))
}
```

## Debugging Techniques

### 1. Debug Output

**Issue**: No visibility into script execution
```rust
// ✅ Add debug output (only works in ckb-debugger)
use ckb_std::syscalls;

pub fn debug_cell_info(index: usize, source: Source) {
    #[cfg(debug_assertions)]
    {
        match load_cell(index, source) {
            Ok(cell) => {
                let capacity = cell.capacity().unpack();
                let lock_hash = cell.lock().calc_script_hash();
                
                syscalls::debug(
                    format!("Cell {}: capacity={}, lock={:?}", 
                        index, capacity, lock_hash).as_bytes()
                );
            },
            Err(e) => {
                syscalls::debug(
                    format!("Failed to load cell {}: {:?}", index, e).as_bytes()
                );
            }
        }
    }
}

// Use in main function
pub fn main() -> Result<(), Error> {
    debug_cell_info(0, Source::GroupInput);
    debug_cell_info(0, Source::GroupOutput);
    
    // Your logic here
    Ok(())
}
```

### 2. Error Context

**Issue**: Generic error messages
```rust
// ❌ Unhelpful error
return Err(Error::InvalidData);

// ✅ Helpful error with context
#[derive(Debug)]
pub struct ErrorContext {
    pub error: Error,
    pub location: &'static str,
    pub details: String,
}

macro_rules! error_with_context {
    ($err:expr, $location:expr, $($args:tt)*) => {
        ErrorContext {
            error: $err,
            location: $location,
            details: format!($($args)*),
        }
    };
}

// Usage
if data.len() < 16 {
    return Err(error_with_context!(
        Error::InvalidData,
        "parse_token_amount",
        "Expected 16 bytes, got {}",
        data.len()
    ));
}
```

### 3. Transaction Analysis

**Issue**: Understanding transaction structure
```rust
// Debug transaction structure
pub fn debug_transaction_structure() -> Result<(), Error> {
    #[cfg(debug_assertions)]
    {
        // Count inputs and outputs
        let input_count = QueryIter::new(load_cell, Source::Input).count();
        let output_count = QueryIter::new(load_cell, Source::Output).count();
        let group_input_count = QueryIter::new(load_cell, Source::GroupInput).count();
        let group_output_count = QueryIter::new(load_cell, Source::GroupOutput).count();
        
        syscalls::debug(
            format!("Transaction: {} inputs, {} outputs", input_count, output_count).as_bytes()
        );
        syscalls::debug(
            format!("Script group: {} inputs, {} outputs", group_input_count, group_output_count).as_bytes()
        );
        
        // Debug each group cell
        for (i, cell) in QueryIter::new(load_cell, Source::GroupInput).enumerate() {
            syscalls::debug(
                format!("Group input {}: capacity={}", i, cell.capacity().unpack()).as_bytes()
            );
        }
    }
    Ok(())
}
```

## Performance Issues

### 1. Excessive Syscalls

**Issue**: Too many individual syscalls
```rust
// ❌ Inefficient: Multiple syscalls
for i in 0..10 {
    let data = load_cell_data(i, Source::Input)?;
    let capacity = load_cell_capacity(i, Source::Input)?;
    process_cell(&data, capacity)?;
}
```

**Solution**: Batch operations
```rust
// ✅ Efficient: Batch load then process
let cells: Vec<_> = QueryIter::new(load_cell, Source::Input).collect();
let data: Vec<_> = QueryIter::new(load_cell_data, Source::Input).collect();

for (cell, data) in cells.iter().zip(data.iter()) {
    process_cell(data, cell.capacity().unpack())?;
}
```

### 2. Memory Inefficiency

**Issue**: Excessive memory allocation
```rust
// ❌ Inefficient: Unnecessary allocations
let mut results = Vec::new();
for data in QueryIter::new(load_cell_data, Source::GroupInput) {
    let processed = data.to_vec(); // Unnecessary copy
    results.push(processed);
}
```

**Solution**: Process in-place
```rust
// ✅ Efficient: Process without extra allocations
let mut total = 0u128;
for data in QueryIter::new(load_cell_data, Source::GroupInput) {
    let amount = parse_amount_from_slice(&data)?; // No allocation
    total = total.checked_add(amount).ok_or(Error::Overflow)?;
}
```

## Common Pitfalls

### 1. Script Arguments Parsing

**Issue**: Incorrect argument parsing
```rust
// ❌ Wrong: Unsafe parsing
let script = load_script()?;
let args = script.args().raw_data();
let owner_hash: [u8; 32] = args[0..32].try_into().unwrap(); // Panics!
```

**Solution**: Safe parsing with validation
```rust
// ✅ Safe: Validate before parsing
pub fn parse_owner_hash() -> Result<[u8; 32], Error> {
    let script = load_script().map_err(|_| Error::InvalidScript)?;
    let args = script.args().raw_data();
    
    if args.len() != 32 {
        return Err(Error::InvalidArgs);
    }
    
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&args[0..32]);
    Ok(hash)
}
```

### 2. Witness Parsing

**Issue**: Complex witness structure handling
```rust
// ❌ Fragile: Assumes witness structure
let witness = load_witness_args(0, Source::GroupInput)?;
let signature = witness.lock().to_opt().unwrap().raw_data(); // Can panic!
```

**Solution**: Robust witness parsing
```rust
// ✅ Robust: Handle missing or malformed witnesses
pub fn parse_signature() -> Result<Vec<u8>, Error> {
    let witness_args = load_witness_args(0, Source::GroupInput)
        .map_err(|_| Error::InvalidWitness)?;
    
    let signature = witness_args.lock().to_opt()
        .ok_or(Error::MissingSignature)?
        .raw_data();
    
    if signature.len() != 65 {
        return Err(Error::InvalidSignatureLength);
    }
    
    Ok(signature.to_vec())
}
```

### 3. Group Source Confusion

**Issue**: Misunderstanding group behavior
```rust
// ❌ Wrong assumption: Lock script using GroupOutput
pub fn wrong_lock_script() -> Result<(), Error> {
    // This will always be empty for lock scripts!
    for cell in QueryIter::new(load_cell, Source::GroupOutput) {
        // This code never executes
        validate_cell(&cell)?;
    }
    Ok(())
}
```

**Solution**: Use correct sources for script type
```rust
// ✅ Correct: Lock script using GroupInput only
pub fn correct_lock_script() -> Result<(), Error> {
    // Validate all inputs controlled by this lock
    for cell in QueryIter::new(load_cell, Source::GroupInput) {
        validate_spending_authorization(&cell)?;
    }
    Ok(())
}

// ✅ Correct: Type script using both sources
pub fn correct_type_script() -> Result<(), Error> {
    let input_sum = calculate_sum(Source::GroupInput)?;
    let output_sum = calculate_sum(Source::GroupOutput)?;
    
    // Validate state transition
    validate_conservation(input_sum, output_sum)?;
    Ok(())
}
```

## Quick Reference Checklist

### Before Debugging
- [ ] Check script type (lock vs type) and use appropriate sources
- [ ] Verify argument parsing with proper bounds checking
- [ ] Ensure proper error handling for all syscalls
- [ ] Validate all external data before processing
- [ ] Use checked arithmetic operations

### Common Error Patterns
- [ ] `IndexOutOfBound`: Check if cells/witnesses exist before accessing
- [ ] `LengthNotEnough`: Validate buffer sizes before copying
- [ ] `Encoding`: Check Molecule deserialization and UTF-8 conversion
- [ ] Panics: Replace `unwrap()` with proper error handling
- [ ] Overflow: Use `checked_*` arithmetic operations

### Performance Optimization
- [ ] Batch syscall operations when possible
- [ ] Minimize memory allocations in no_std environment
- [ ] Use `QueryIter` instead of manual index loops
- [ ] Configure appropriate heap size for script complexity
- [ ] Profile binary size and optimize build configuration

Following these patterns and solutions will help you develop more robust and efficient CKB scripts in Rust.