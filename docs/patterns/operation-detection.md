# Operation Detection Pattern

## Description

An advanced pattern for automatically detecting transaction operations (CREATE/TRANSFER/UPDATE/BURN) by analyzing input and output cell counts. This pattern demonstrates sophisticated state validation, overflow protection, and structured error handling for complex CKB applications requiring different behavior based on transaction intent.

## Purpose
Advanced pattern for detecting transaction intent (CREATE/TRANSFER/UPDATE/BURN operations) by analyzing cell counts. This pattern demonstrates:
- Operation mode detection from input/output cell counts
- State validation for different operations
- Overflow protection and value constraints
- Structured error handling for complex validation

## Complete Working Implementation

### Operation Modes
```rust
#![no_std]
#![no_main]

use core::result::Result;
use ckb_std::ckb_constants::Source;
use ckb_std::high_level::{load_cell, load_cell_data, QueryIter};
use ckb_std::{entry, default_alloc};

entry!(program_entry);
default_alloc!();

// The modes of operation for the script
#[derive(Debug, PartialEq)]
enum Mode {
    Burn,     // Consume an existing cell (1 input, 0 outputs)
    Create,   // Create a new cell (0 inputs, 1 output) 
    Transfer, // Transfer/update a cell (1 input, 1 output)
}

#[repr(i8)]
pub enum Error {
    IndexOutOfBound = 1,
    ItemMissing,
    LengthNotEnough,
    Encoding,
    CounterValueOverflow,
    InvalidTransactionStructure,
    InvalidInputCellData,
    InvalidOutputCellData, 
    InvalidCounterValue,
}

impl From<ckb_std::error::SysError> for Error {
    fn from(err: ckb_std::error::SysError) -> Self {
        use ckb_std::error::SysError::*;
        match err {
            IndexOutOfBound => Self::IndexOutOfBound,
            ItemMissing => Self::ItemMissing,
            LengthNotEnough(_) => Self::LengthNotEnough,
            Encoding => Self::Encoding,
            Unknown(err_code) => panic!("unexpected sys error {}", err_code),
        }
    }
}

fn program_entry() -> i8 {
    match main() {
        Ok(_) => 0,
        Err(err) => err as i8,
    }
}
```

### Operation Detection Logic
```rust
/// Determines the mode of operation by analyzing cell counts
fn determine_mode() -> Result<Mode, Error> {
    // Count group input and output cells (cells with same script)
    let group_input_count = QueryIter::new(load_cell, Source::GroupInput).count();
    let group_output_count = QueryIter::new(load_cell, Source::GroupOutput).count();

    // Detect operation based on cell count patterns
    match (group_input_count, group_output_count) {
        (1, 0) => Ok(Mode::Burn),     // Destroying a cell
        (0, 1) => Ok(Mode::Create),   // Creating a new cell
        (1, 1) => Ok(Mode::Transfer), // Updating/transferring a cell
        _ => Err(Error::InvalidTransactionStructure),
    }
}
```

### Validation Functions
```rust
/// Validate cell creation - ensure initial state is correct
fn validate_create() -> Result<(), Error> {
    // Load output cell data and verify initial value is 0
    let cell_data = load_cell_data(0, Source::GroupOutput)?;
    
    // Counter must start at 0
    if cell_data != 0u64.to_le_bytes().to_vec() {
        return Err(Error::InvalidOutputCellData);
    }

    Ok(())
}

/// Validate cell transfer/update - ensure state transition is valid
fn validate_transfer() -> Result<(), Error> {
    // Load and validate input cell data
    let input_data = load_cell_data(0, Source::GroupInput)?;
    if input_data.len() != 8 {
        return Err(Error::InvalidInputCellData);
    }

    // Load and validate output cell data
    let output_data = load_cell_data(0, Source::GroupOutput)?;
    if output_data.len() != 8 {
        return Err(Error::InvalidOutputCellData);
    }

    // Parse data as u64 values
    let mut buffer = [0u8; 8];
    
    buffer.copy_from_slice(&input_data[0..8]);
    let input_value = u64::from_le_bytes(buffer);
    
    buffer.copy_from_slice(&output_data[0..8]);
    let output_value = u64::from_le_bytes(buffer);

    // Check for overflow scenario
    if input_value == u64::MAX {
        return Err(Error::CounterValueOverflow);
    }

    // Ensure output value is exactly input + 1 (increment rule)
    if input_value + 1 != output_value {
        return Err(Error::InvalidCounterValue);
    }

    Ok(())
}
```

### Main Entry Point
```rust
/// Main validation logic with operation detection
pub fn main() -> Result<(), Error> {
    // Determine operation mode and validate accordingly
    match determine_mode()? {
        Mode::Burn => {
            // Burn operations require no additional validation
            // Cell destruction is inherently valid if properly referenced
            Ok(())
        }
        Mode::Create => {
            // Validate cell creation rules
            validate_create()
        }
        Mode::Transfer => {
            // Validate state transition rules
            validate_transfer()
        }
    }
}
```

## Key Patterns Explained

### 1. Cell Count Analysis
```rust  
// Pattern: Analyze input/output counts to determine intent
let inputs = QueryIter::new(load_cell, Source::GroupInput).count();
let outputs = QueryIter::new(load_cell, Source::GroupOutput).count();

match (inputs, outputs) {
    (1, 0) => Mode::Burn,     // Destroying asset
    (0, 1) => Mode::Create,   // Minting asset  
    (1, 1) => Mode::Transfer, // Moving/updating asset
    (n, m) => {
        // Custom logic for multi-cell operations
        // e.g., (2, 1) could be "merge", (1, 2) could be "split"
    }
}
```

### 2. State Validation by Mode
```rust
// Pattern: Different validation rules per operation
match mode {
    Mode::Create => validate_initial_state(),
    Mode::Transfer => validate_state_transition(), 
    Mode::Burn => validate_destruction_rules(),
}
```

### 3. Overflow Protection
```rust
// Pattern: Check for edge cases before computation
if current_value == MAX_VALUE {
    return Err(Error::Overflow);
}

// Safe to increment
let new_value = current_value + 1;
```

### 4. Structured Data Parsing
```rust
// Pattern: Safely parse cell data to typed values
fn parse_u64_from_cell_data(data: &[u8]) -> Result<u64, Error> {
    if data.len() != 8 {
        return Err(Error::InvalidDataLength);
    }
    
    let mut buffer = [0u8; 8];
    buffer.copy_from_slice(&data[0..8]);
    Ok(u64::from_le_bytes(buffer))
}
```

## Advanced Operation Examples

### Multi-Cell Operations
```rust
// Pattern: Handle complex operations
fn determine_advanced_mode() -> Result<Mode, Error> {
    let inputs = QueryIter::new(load_cell, Source::GroupInput).count();
    let outputs = QueryIter::new(load_cell, Source::GroupOutput).count();
    
    match (inputs, outputs) {
        (1, 0) => Ok(Mode::Burn),
        (0, 1) => Ok(Mode::Create),
        (1, 1) => Ok(Mode::Transfer),
        (2, 1) => Ok(Mode::Merge),     // Combine two cells
        (1, 2) => Ok(Mode::Split),     // Split one cell into two
        (n, m) if n > 0 && m > 0 => Ok(Mode::Batch), // Batch operation
        _ => Err(Error::InvalidTransactionStructure),
    }
}
```

### State Machine Validation
```rust
// Pattern: Complex state transitions
fn validate_state_machine() -> Result<(), Error> {
    let input_state = parse_state_from_input()?;
    let output_state = parse_state_from_output()?;
    
    // Define valid state transitions
    let valid_transition = match (input_state, output_state) {
        (State::Pending, State::Active) => true,
        (State::Active, State::Completed) => true,
        (State::Active, State::Cancelled) => true,
        _ => false,
    };
    
    if !valid_transition {
        return Err(Error::InvalidStateTransition);
    }
    
    Ok(())
}
```

## When to Use This Pattern
- **Smart contracts with multiple operations** (tokens, NFTs, games)
- **State machines** requiring different validation per state
- **Complex DApps** with CREATE/UPDATE/DELETE semantics
- **Financial applications** with strict state transition rules
- **Multi-cell operations** requiring coordination

## Integration with Frontend
```typescript
// Frontend can hint at operation intent
const tx = ccc.Transaction.from({
    outputs: mode === 'create' ? [newCell] : [updatedCell],
    inputs: mode === 'burn' ? [existingCell] : [],
    // Script will detect actual operation from cell counts
});
```

This pattern provides robust operation detection and validation, essential for complex CKB applications requiring different behavior based on transaction intent.