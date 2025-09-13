## Description

CKB-VM syscalls for script development, covering transaction data access, source enumeration, and programming patterns. Complete syscall reference matrix, high-level vs low-level API comparisons, security patterns, error handling, and performance optimization for CKB smart contract development with practical examples in Rust and C.

## Overview

CKB scripts access transaction information through CKB-VM syscalls. Scripts written in Rust use the CKB-STD library which provides both high-level functions and direct syscall access.

## Programming Approach

### Rust (Recommended for Modern Development)
**Rust is the preferred language for CKB smart contract development** due to its safety guarantees, excellent tooling, and strong ecosystem support. Use CKB-STD high-level functions when possible:

```rust
use ckb_std::{
    ckb_constants::Source,
    high_level::{load_cell_data, load_script, QueryIter},
    syscalls,
};

// High-level approach (preferred for most cases)
let script = load_script()?;
let cell_data = load_cell_data(0, Source::GroupOutput)?;

// Direct syscall approach (when performance-critical or specific requirements)
let mut buf = [0u8; 1024];
let mut len = buf.len();
syscalls::load_cell_data(&mut buf, &mut len, 0, 0, Source::GroupOutput as u64)?;
```

**Benefits of Rust for CKB Development:**
- **Memory Safety**: Prevents common vulnerabilities like buffer overflows
- **Modern Tooling**: Cargo for dependency management and building
- **Type Safety**: Compile-time error checking reduces runtime bugs
- **Growing Ecosystem**: Most new CKB tools and libraries target Rust
- **Better Testing**: Integration with ckb-testtool and testing frameworks

### C (Legacy Support)
While C is still supported and some system scripts use it, **Rust is recommended for new development**:

```c
// C is supported but consider migrating to Rust for new projects
#include "ckb_syscalls.h"

int ret = ckb_load_cell_data(buf, &len, 0, 0, CKB_SOURCE_GROUP_OUTPUT);
if (ret != CKB_SUCCESS) {
    return ret;
}
```

**Note**: Existing C scripts can be gradually migrated to Rust using interop patterns or complete rewrites for better maintainability.

## Syscall Reference Matrix

| **Function** | **CKB-STD High-Level** | **CKB-STD Syscall** | **C Syscall** |
|-------------|------------------------|---------------------|---------------|
| **Script Operations** | | | |
| Load current script | `load_script()` | `load_script()` | `ckb_load_script()` |
| Load script hash | `load_script_hash()` | `load_script_hash()` | `ckb_load_script_hash()` |
| **Cell Operations** | | | |
| Load complete cell | `load_cell()` | `load_cell()` | `ckb_load_cell()` |
| Load cell field | — | `load_cell_by_field()` | `ckb_load_cell_by_field()` |
| Load cell data | `load_cell_data()` | `load_cell_data()` | `ckb_load_cell_data()` |
| Load cell capacity | `load_cell_capacity()` | — | — |
| Load cell lock | `load_cell_lock()` | — | — |
| Load cell type | `load_cell_type()` | — | — |
| Load cell as code | — | `load_cell_code()` | `ckb_load_cell_data_as_code()` |
| **Transaction Operations** | | | |
| Load transaction | `load_transaction()` | `load_transaction()` | `ckb_load_transaction()` |
| Load tx hash | `load_tx_hash()` | `load_tx_hash()` | `ckb_load_tx_hash()` |
| **Input Operations** | | | |
| Load input | `load_input()` | `load_input()` | `ckb_load_input()` |
| Load input field | — | `load_input_by_field()` | `ckb_load_input_by_field()` |
| Load input OutPoint | `load_input_out_point()` | — | — |
| Load input since | `load_input_since()` | — | — |
| **Header Operations** | | | |
| Load header | `load_header()` | `load_header()` | `ckb_load_header()` |
| Load header field | — | `load_header_by_field()` | `ckb_load_header_by_field()` |
| Load header epoch | `load_header_epoch_*()` | — | — |
| **Witness Operations** | | | |
| Load witness | — | `load_witness()` | `ckb_load_witness()` |
| Load witness args | `load_witness_args()` | — | — |
| **Utility Operations** | | | |
| Debug output | — | `debug()` | `ckb_debug()` |
| Exit program | — | `exit()` | `exit()` |

## Sources Enumeration

```rust
pub enum Source {
    Input,       // Transaction inputs
    Output,      // Transaction outputs  
    CellDep,     // Cell dependencies
    HeaderDep,   // Header dependencies
    GroupInput,  // Filtered inputs (same script)
    GroupOutput, // Filtered outputs (same script)
}
```

### Source Usage Patterns

#### Standard Sources
```rust
// Access all transaction inputs
for i in 0.. {
    match load_cell_data(i, Source::Input) {
        Ok(data) => { /* process input cell data */ },
        Err(_) => break, // No more inputs
    }
}

// Access all transaction outputs
for i in 0.. {
    match load_cell_data(i, Source::Output) {
        Ok(data) => { /* process output cell data */ },
        Err(_) => break, // No more outputs
    }
}
```

#### Script Groups
Script groups provide filtered views of inputs/outputs for efficiency:

```rust
// Only cells with the same lock/type script as currently executing
use ckb_std::high_level::QueryIter;

// Type script: check all outputs in same group
for data in QueryIter::new(load_cell_data, Source::GroupOutput) {
    validate_cell_data(&data)?;
}

// Lock script: check all inputs in same group  
for input in QueryIter::new(load_input, Source::GroupInput) {
    validate_authorization(&input)?;
}
```

### Group Source Behavior

| **Script Type** | **GroupInput** | **GroupOutput** |
|----------------|----------------|-----------------|
| **Lock Script** | ✅ Inputs with same lock | ❌ Empty (lock scripts don't execute on outputs) |
| **Type Script** | ✅ Inputs with same type | ✅ Outputs with same type |

## Common Usage Patterns

### 1. Script Argument Parsing

```rust
pub fn parse_script_args() -> Result<Args, Error> {
    let script = load_script()?;
    let args = script.args().raw_data();
    
    if args.len() < 32 {
        return Err(Error::InvalidArgs);
    }
    
    let owner_lock_hash: [u8; 32] = args[0..32].try_into()?;
    let config_data = &args[32..];
    
    Ok(Args {
        owner_lock_hash,
        config: parse_config(config_data)?,
    })
}
```

### 2. Cell Data Validation

```rust
pub fn validate_all_outputs() -> Result<(), Error> {
    // Use QueryIter for concise iteration
    for (i, data) in QueryIter::new(load_cell_data, Source::GroupOutput).enumerate() {
        validate_cell_data(&data)
            .map_err(|e| Error::InvalidOutput(i, e))?;
    }
    Ok(())
}

fn validate_cell_data(data: &[u8]) -> Result<(), Error> {
    if data.len() < 16 {
        return Err(Error::InsufficientData);
    }
    
    let amount = u128::from_le_bytes(data[0..16].try_into()?);
    if amount == 0 {
        return Err(Error::ZeroAmount);
    }
    
    Ok(())
}
```

### 3. Token Conservation Checking

```rust
pub fn verify_token_conservation() -> Result<(), Error> {
    let mut input_sum = 0u128;
    let mut output_sum = 0u128;
    
    // Sum input tokens
    for data in QueryIter::new(load_cell_data, Source::GroupInput) {
        let amount = parse_token_amount(&data)?;
        input_sum = input_sum.checked_add(amount)
            .ok_or(Error::Overflow)?;
    }
    
    // Sum output tokens
    for data in QueryIter::new(load_cell_data, Source::GroupOutput) {
        let amount = parse_token_amount(&data)?;
        output_sum = output_sum.checked_add(amount)
            .ok_or(Error::Overflow)?;
    }
    
    // Validate conservation (allow burning, prevent minting)
    if output_sum > input_sum {
        return Err(Error::TokenMinted);
    }
    
    Ok(())
}

fn parse_token_amount(data: &[u8]) -> Result<u128, Error> {
    if data.len() < 16 {
        return Err(Error::InvalidTokenData);
    }
    Ok(u128::from_le_bytes(data[0..16].try_into()?))
}
```

### 4. Signature Verification Pattern

```rust
pub fn verify_signature() -> Result<(), Error> {
    // Load witness containing signature
    let witness_args = load_witness_args(0, Source::GroupInput)?;
    let signature = witness_args.lock().to_opt()
        .ok_or(Error::MissingSignature)?
        .raw_data();
    
    if signature.len() != 65 {
        return Err(Error::InvalidSignatureLength);
    }
    
    // Load transaction hash
    let tx_hash = load_tx_hash()?;
    
    // Load public key from script args
    let script = load_script()?;
    let args = script.args().raw_data();
    if args.len() != 20 {
        return Err(Error::InvalidPubkeyHash);
    }
    
    // Verify signature (implementation depends on signature scheme)
    verify_secp256k1_signature(&signature, &tx_hash, &args)?;
    
    Ok(())
}
```

### 5. Header-Based Time Validation

```rust
pub fn validate_timelock() -> Result<(), Error> {
    let script = load_script()?;
    let args = script.args().raw_data();
    
    if args.len() < 8 {
        return Err(Error::InvalidTimelockArgs);
    }
    
    let unlock_time = u64::from_le_bytes(args[0..8].try_into()?);
    
    // Load current block header
    let header = load_header(0, Source::HeaderDep)?;
    let current_time = header.timestamp();
    
    if current_time < unlock_time {
        return Err(Error::TimelockNotExpired);
    }
    
    Ok(())
}
```

### 6. Multi-Cell State Validation

```rust
pub fn validate_state_machine() -> Result<(), Error> {
    let inputs = load_state_cells(Source::GroupInput)?;
    let outputs = load_state_cells(Source::GroupOutput)?;
    
    // State machine must have exactly one input and one output
    if inputs.len() != 1 || outputs.len() != 1 {
        return Err(Error::InvalidStateTransition);
    }
    
    let old_state = parse_state(&inputs[0])?;
    let new_state = parse_state(&outputs[0])?;
    
    // Validate state transition
    validate_transition(&old_state, &new_state)?;
    
    Ok(())
}

fn load_state_cells(source: Source) -> Result<Vec<Vec<u8>>, Error> {
    let mut cells = Vec::new();
    for data in QueryIter::new(load_cell_data, source) {
        cells.push(data.to_vec());
    }
    Ok(cells)
}
```

## Error Handling Best Practices

### Structured Error Types

```rust
#[derive(Debug, Clone, Copy)]
pub enum Error {
    // Syscall errors
    InvalidArgs = 1,
    InvalidTransaction = 2,
    
    // Business logic errors
    InsufficientFunds = 10,
    InvalidSignature = 11,
    UnauthorizedOperation = 12,
    
    // Data format errors
    InvalidCellData = 20,
    MalformedWitness = 21,
}

impl From<ckb_std::error::SysError> for Error {
    fn from(err: ckb_std::error::SysError) -> Self {
        match err {
            ckb_std::error::SysError::IndexOutOfBound => Error::InvalidArgs,
            _ => Error::InvalidTransaction,
        }
    }
}
```

### Safe Loading Patterns

```rust
// Safe cell data loading with bounds checking
pub fn safe_load_cell_data(index: usize, source: Source) -> Result<Vec<u8>, Error> {
    const MAX_DATA_SIZE: usize = 1024 * 1024; // 1MB limit
    
    let data = load_cell_data(index, source)?;
    
    if data.len() > MAX_DATA_SIZE {
        return Err(Error::DataTooLarge);
    }
    
    Ok(data.to_vec())
}

// Safe parsing with error propagation
pub fn safe_parse_u64(data: &[u8], offset: usize) -> Result<u64, Error> {
    if data.len() < offset + 8 {
        return Err(Error::InsufficientData);
    }
    
    let bytes: [u8; 8] = data[offset..offset + 8].try_into()
        .map_err(|_| Error::InvalidData)?;
    
    Ok(u64::from_le_bytes(bytes))
}
```

## Performance Optimization

### Minimize Syscall Overhead

```rust
// Efficient: Load all data once
let all_inputs: Vec<_> = QueryIter::new(load_cell_data, Source::GroupInput)
    .collect();

// Process in memory
for (i, data) in all_inputs.iter().enumerate() {
    process_data(i, data)?;
}

// Inefficient: Multiple syscall rounds
for i in 0.. {
    match load_cell_data(i, Source::GroupInput) {
        Ok(data) => {
            process_data(i, &data)?;
            // Additional processing that triggers more syscalls
            let capacity = load_cell_capacity(i, Source::GroupInput)?;
        },
        Err(_) => break,
    }
}
```

### Batch Operations

```rust
// Load multiple fields efficiently using lower-level syscalls when needed
pub fn load_cell_summary(index: usize, source: Source) -> Result<CellSummary, Error> {
    let cell = load_cell(index, source)?;
    let data = load_cell_data(index, source)?;
    
    Ok(CellSummary {
        capacity: cell.capacity().unpack(),
        lock_hash: cell.lock().calc_script_hash(),
        type_hash: cell.type_().to_opt().map(|s| s.calc_script_hash()),
        data_len: data.len(),
    })
}
```

## Debugging and Development

### Debug Output

```rust
use ckb_std::syscalls;

// Debug output (only works in ckb-debugger)
pub fn debug_cell_info(index: usize, source: Source) {
    match load_cell(index, source) {
        Ok(cell) => {
            syscalls::debug(format!("Cell {}: capacity={}, lock_hash={:?}", 
                index, 
                cell.capacity().unpack(),
                cell.lock().calc_script_hash()
            ).as_bytes());
        },
        Err(e) => {
            syscalls::debug(format!("Failed to load cell {}: {:?}", index, e).as_bytes());
        }
    }
}
```

### Testing Patterns

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ckb_tool::ckb_types::core::TransactionBuilder;
    
    #[test]
    fn test_token_conservation() {
        let mut context = Context::default();
        
        // Build test transaction
        let tx = TransactionBuilder::default()
            .input(build_token_input(100))
            .output(build_token_output(80))
            .output(build_token_output(20))
            .build();
        
        // Verify script passes
        let cycles = context.verify_tx(&tx, MAX_CYCLES).expect("pass");
        println!("Consumed cycles: {}", cycles);
    }
}
```

## Security Considerations

### Input Validation

```rust
// Always validate syscall results
pub fn secure_load_script_args() -> Result<Vec<u8>, Error> {
    let script = load_script()?;
    let args = script.args().raw_data();
    
    // Validate args length
    if args.len() > 1024 {
        return Err(Error::ArgsTooLong);
    }
    
    if args.is_empty() {
        return Err(Error::MissingArgs);
    }
    
    Ok(args.to_vec())
}

// Prevent integer overflow
pub fn safe_add_amounts(a: u128, b: u128) -> Result<u128, Error> {
    a.checked_add(b).ok_or(Error::Overflow)
}
```

### Bounds Checking

```rust
// Safe array access
pub fn safe_slice(data: &[u8], start: usize, len: usize) -> Result<&[u8], Error> {
    if start.saturating_add(len) > data.len() {
        return Err(Error::OutOfBounds);
    }
    Ok(&data[start..start + len])
}
```

Understanding CKB syscalls and sources is fundamental for effective script development. Use high-level functions when possible, implement proper error handling, and always validate external data to ensure secure and efficient scripts.