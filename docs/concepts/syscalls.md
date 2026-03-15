## Description

CKB-VM syscall reference for script development. Complete syscall matrix mapping CKB-STD high-level functions, low-level syscalls, and C equivalents. Source enumeration (Input, Output, CellDep, HeaderDep, GroupInput, GroupOutput), group source behavior for lock vs type scripts, script argument parsing, cell data validation, error handling patterns, and performance optimization. Rust and C API comparisons.

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

Token conservation uses `QueryIter` with `Source::GroupInput` and `Source::GroupOutput` to sum token amounts across cells and verify no tokens are created without authorization. For complete UDT token conservation patterns, see [Script Patterns](ckb://docs/scripts/patterns) and [Token Creation](ckb://docs/tokens/token-creation).

### 4. Signature Verification Pattern

Signature verification combines `load_witness_args`, `load_tx_hash`, and `load_script` to extract signature data, the signing message, and the expected public key hash. For complete secp256k1 and other signature verification implementations, see [Script Patterns](ckb://docs/scripts/patterns).

### 5. Header-Based Time Validation

Accessing header dependencies via `load_header` enables time-based logic:

```rust
// Core syscall usage: accessing header deps for timestamp
let header = load_header(0, Source::HeaderDep)?;
let current_time = header.timestamp();

// Compare against unlock time from script args
let script = load_script()?;
let unlock_time = u64::from_le_bytes(script.args().raw_data()[0..8].try_into()?);
```

For complete time-based validation patterns including since-field logic, see [Script Patterns](ckb://docs/scripts/patterns).

### 6. Multi-Cell State Validation

State machine patterns use `QueryIter` to collect cells from `Source::GroupInput` and `Source::GroupOutput`, then validate that state transitions follow allowed rules. For complete state machine implementations, see [Script Patterns](ckb://docs/scripts/patterns).

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

## Debugging and Development

### Debug Output

```rust
use ckb_std::syscalls;

// Debug output (only works in ckb-debugger)
syscalls::debug(format!("Cell {}: capacity={}", index, cell.capacity().unpack()).as_bytes());
```

For testing patterns and test transaction construction, see [Rust Testing](ckb://docs/scripts/rust-testing).

## Security Considerations

### Input Validation

Always validate syscall results before using them:

```rust
let script = load_script()?;
let args = script.args().raw_data();

if args.is_empty() || args.len() > 1024 {
    return Err(Error::InvalidArgs);
}
```

### Bounds Checking

Use safe slicing to prevent out-of-bounds access:

```rust
pub fn safe_slice(data: &[u8], start: usize, len: usize) -> Result<&[u8], Error> {
    if start.saturating_add(len) > data.len() {
        return Err(Error::OutOfBounds);
    }
    Ok(&data[start..start + len])
}
```

For overflow-safe arithmetic and complete security patterns, see [Script Patterns](ckb://docs/scripts/patterns).