## Description

Fundamental CKB pattern for creating singleton cells and upgradeable scripts. Demonstrates Type ID calculation, validation rules, transaction patterns for creation/transfer/burning, upgradeable contract implementation, unique asset registries, security considerations, and testing strategies for maintaining uniqueness and enabling script upgrades.

## Overview

The Type ID pattern is a fundamental CKB design pattern that creates **singleton cells** - ensuring only one live cell of a specific type exists on the blockchain at any time. This pattern enables **upgradeable scripts** and **unique asset identification** while maintaining security and preventing duplication attacks.

## Key Concept

Type ID leverages CKB's `hash_type` field in script structures to create scripts that can reference other scripts by their **Type Script hash** rather than **data hash**, enabling upgradeability while maintaining uniqueness guarantees.

## Problem Solved

### Traditional Script Reference Issue
```rust
// Traditional approach - breaks on upgrades
let script = Script::new_builder()
    .code_hash(data_hash)      // Changes when code is updated
    .hash_type(ScriptHashType::Data.into())
    .build();
```

### Type ID Solution
```rust  
// Type ID approach - survives upgrades
let script = Script::new_builder()
    .code_hash(type_script_hash)  // Stays constant across upgrades
    .hash_type(ScriptHashType::Type.into())
    .build();
```

## Type ID Algorithm

### Core Validation Rules

1. **Singleton Enforcement**: At most one input cell and one output cell with the same Type ID
2. **Creation Rule**: For new Type ID cells, the ID must be calculated correctly
3. **Transfer Rule**: Existing Type ID cells can be transferred without recalculation

### Type ID Calculation

```rust
// Type ID = Blake2b(first_input_outpoint + output_index)
fn calculate_type_id(first_input: &OutPoint, output_index: usize) -> [u8; 32] {
    let mut blake2b = Blake2bBuilder::new(32)
        .personal(b"ckb-default-hash")
        .build();
    
    blake2b.update(first_input.as_slice());
    blake2b.update(&(output_index as u64).to_le_bytes());
    
    let mut type_id = [0u8; 32];
    blake2b.finalize(&mut type_id);
    type_id
}
```

## Implementation Patterns

### 1. Basic Type ID Validation

```rust
use ckb_std::{
    ckb_constants::Source,
    high_level::{load_input, load_script_hash, load_cell_type_hash},
    syscalls::load_cell,
};

pub fn validate_type_id(expected_type_id: [u8; 32]) -> Result<(), Error> {
    // Rule 1: Check for singleton property
    if has_multiple_type_id_cells()? {
        return Err(Error::TooManyTypeIdCells);
    }
    
    // Rule 2: If creating new Type ID, validate calculation
    if !has_type_id_input()? {
        let calculated_id = calculate_new_type_id()?;
        if calculated_id != expected_type_id {
            return Err(Error::InvalidTypeId);
        }
    }
    
    // Rule 3: Transfer case - validation passes automatically
    Ok(())
}

fn has_multiple_type_id_cells() -> Result<bool, Error> {
    // Check if there are more than 1 input or output cells with current type
    let input_count = count_group_cells(Source::GroupInput)?;
    let output_count = count_group_cells(Source::GroupOutput)?;
    
    Ok(input_count > 1 || output_count > 1)
}

fn calculate_new_type_id() -> Result<[u8; 32], Error> {
    // Get first input outpoint
    let first_input = load_input(0, Source::Input)?;
    
    // Find current cell's output index
    let output_index = find_current_output_index()?;
    
    // Calculate Type ID
    calculate_type_id(&first_input.previous_output(), output_index)
}
```

### 2. Using ckb-std Type ID Utility

```rust
use ckb_std::type_id::check_type_id;

// Simple Type ID validation using built-in function
fn main() -> Result<(), Error> {
    // Assumes Type ID starts at byte 32 in script args
    check_type_id(32).map_err(|_| Error::InvalidTypeId)?;
    
    // Your contract logic here
    Ok(())
}
```

## Transaction Patterns

### 1. Type ID Creation (Minting)

```rust
// Transaction creating a new Type ID cell
let transaction = TransactionBuilder::default()
    .input(CellInput::new(seed_outpoint, 0))  // Seed input for Type ID calculation
    .output(CellOutput::new_builder()
        .capacity(capacity.pack())
        .lock(owner_lock)
        .type_(Some(type_id_script))  // New Type ID script
        .build())
    .output_data(initial_data.pack())
    .build();

// Type ID will be: Blake2b(seed_outpoint + 0)
```

### 2. Type ID Transfer

```rust
// Transaction transferring a Type ID cell
let transaction = TransactionBuilder::default()
    .input(CellInput::new(type_id_outpoint, 0))  // Existing Type ID cell
    .output(CellOutput::new_builder()
        .capacity(capacity.pack())
        .lock(new_owner_lock)           // New owner
        .type_(Some(type_id_script))    // Same Type ID script
        .build())
    .output_data(updated_data.pack())
    .build();
```

### 3. Type ID Burning

```rust
// Transaction destroying a Type ID cell
let transaction = TransactionBuilder::default()
    .input(CellInput::new(type_id_outpoint, 0))  // Type ID cell to burn
    .output(CellOutput::new_builder()
        .capacity(capacity.pack())
        .lock(recipient_lock)
        .build())  // No type script = burned
    .output_data(Bytes::new().pack())
    .build();
```

## Upgradeable Scripts Pattern

### 1. Deploy Upgradeable Contract

```rust
// Step 1: Deploy contract with Type ID
let contract_data = compile_contract("my_contract.rs");
let type_id = calculate_type_id(&seed_outpoint, 0);

let deploy_tx = TransactionBuilder::default()
    .input(CellInput::new(seed_outpoint, 0))
    .output(CellOutput::new_builder()
        .capacity(contract_data.len().pack())
        .lock(always_success_lock())
        .type_(Some(type_id_script(type_id)))
        .build())
    .output_data(contract_data.pack())
    .build();
```

### 2. Reference Upgradeable Contract

```rust
// Step 2: Create script that references by Type ID
let user_script = Script::new_builder()
    .code_hash(type_id.pack())  // Reference by Type ID, not data hash
    .hash_type(ScriptHashType::Type.into())  // Use Type hash type
    .args(user_args.pack())
    .build();

// This script survives contract upgrades!
```

### 3. Upgrade Contract

```rust
// Step 3: Upgrade the contract code
let new_contract_data = compile_contract("my_contract_v2.rs");

let upgrade_tx = TransactionBuilder::default()
    .input(CellInput::new(old_contract_outpoint, 0))  // Old contract cell
    .output(CellOutput::new_builder()
        .capacity(new_contract_data.len().pack())
        .lock(always_success_lock())
        .type_(Some(type_id_script(type_id)))  // Same Type ID
        .build())
    .output_data(new_contract_data.pack())  // New code
    .build();

// All references continue working with new code!
```

## Advanced Use Cases

### 1. Unique Asset Registry

```rust
// Create unique asset with guaranteed uniqueness
struct AssetRegistry {
    type_id: [u8; 32],
    name: String,
    metadata: Vec<u8>,
}

impl AssetRegistry {
    fn create(name: String, metadata: Vec<u8>) -> Result<Self, Error> {
        let type_id = generate_unique_type_id()?;
        
        Ok(AssetRegistry {
            type_id,
            name,
            metadata,
        })
    }
    
    // Only one asset with this Type ID can exist
    fn validate_uniqueness(&self) -> Result<(), Error> {
        validate_type_id(self.type_id)
    }
}
```

### 2. Singleton State Machine

```rust
// Global state that can only have one instance
struct GlobalState {
    type_id: [u8; 32],
    counter: u64,
    last_update: u64,
}

impl GlobalState {
    fn update(&mut self, new_counter: u64) -> Result<(), Error> {
        // Ensure singleton property
        validate_type_id(self.type_id)?;
        
        // Update state
        self.counter = new_counter;
        self.last_update = current_timestamp();
        
        Ok(())
    }
}
```

### 3. Versioned Protocol

```rust
// Protocol that can be upgraded while maintaining compatibility
struct Protocol {
    type_id: [u8; 32],
    version: u32,
    config: ProtocolConfig,
}

impl Protocol {
    fn upgrade(&self, new_version: u32, new_config: ProtocolConfig) -> Result<Protocol, Error> {
        // Same Type ID ensures continuity
        Ok(Protocol {
            type_id: self.type_id,  // Preserved across upgrades
            version: new_version,
            config: new_config,
        })
    }
}
```

## Genesis Type ID

CKB includes Type ID as a **genesis script** with a special hash:

```rust
// Type ID genesis script hash
const TYPE_ID_CODE_HASH: [u8; 32] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x54, 0x59, 0x50, 0x45, 0x5f, 0x49, 0x44
];
// Last 8 bytes spell "TYPE_ID" in ASCII
```

## Security Considerations

### 1. Uniqueness Guarantees

```rust
// ✅ Correct: Validates singleton property
fn secure_type_id_script() -> Result<(), Error> {
    if count_group_cells(Source::GroupInput)? > 1 ||
       count_group_cells(Source::GroupOutput)? > 1 {
        return Err(Error::NonSingleton);
    }
    Ok(())
}

// ❌ Incorrect: Doesn't check singleton
fn insecure_script() -> Result<(), Error> {
    // Missing singleton validation allows duplication attacks
    Ok(())
}
```

### 2. Type ID Calculation Validation

```rust
// ✅ Correct: Validates Type ID calculation
fn validate_creation(type_id: [u8; 32]) -> Result<(), Error> {
    if !has_input_with_type_id()? {
        let calculated = calculate_type_id_from_transaction()?;
        if calculated != type_id {
            return Err(Error::InvalidTypeId);
        }
    }
    Ok(())
}
```

### 3. Reference Integrity

```rust
// ✅ Correct: Use Type hash for upgradeability
let upgradeable_reference = Script::new_builder()
    .code_hash(type_id_hash)
    .hash_type(ScriptHashType::Type.into())
    .build();

// ❌ Incorrect: Data hash breaks on upgrades
let fragile_reference = Script::new_builder() 
    .code_hash(data_hash)
    .hash_type(ScriptHashType::Data.into())
    .build();
```

## Common Pitfalls

### 1. Multiple Type ID Cells

```rust
// ❌ Wrong: Creating multiple cells with same Type ID
let transaction = TransactionBuilder::default()
    .input(seed_input)
    .output(type_id_cell_1)  // Type ID: abc123...
    .output(type_id_cell_2)  // Type ID: abc123... - INVALID!
    .build();

// ✅ Correct: Only one cell per Type ID
let transaction = TransactionBuilder::default()
    .input(seed_input)
    .output(type_id_cell)    // Type ID: abc123...
    .build();
```

### 2. Incorrect Type ID Calculation

```rust
// ❌ Wrong: Using wrong inputs for calculation
let wrong_type_id = Blake2b(second_input + output_index);

// ✅ Correct: Always use first input
let correct_type_id = Blake2b(first_input + output_index);
```

### 3. Missing Singleton Validation

```rust
// ❌ Wrong: Not checking for multiple cells
fn main() -> Result<(), Error> {
    // Process without validation - allows duplication
    process_type_id_cell()?;
    Ok(())
}

// ✅ Correct: Always validate singleton property
fn main() -> Result<(), Error> {
    validate_singleton()?;
    process_type_id_cell()?;
    Ok(())
}
```

## Testing Patterns

### 1. Type ID Creation Test

```rust
#[test]
fn test_type_id_creation() {
    let mut context = Context::default();
    
    // Deploy Type ID contract
    let contract_bin = load_type_id_contract();
    let contract_outpoint = context.deploy_cell(contract_bin);
    
    // Create seed input
    let seed_input = create_seed_cell(&mut context);
    
    // Calculate expected Type ID
    let expected_type_id = calculate_type_id(&seed_input.previous_output(), 0);
    
    // Build Type ID script with calculated ID
    let type_script = context.build_script(
        &contract_outpoint,
        Bytes::from([vec![0u8; 32], expected_type_id.to_vec()].concat())
    ).unwrap();
    
    // Create transaction
    let tx = TransactionBuilder::default()
        .input(seed_input)
        .output(CellOutput::new_builder()
            .type_(Some(type_script))
            .build())
        .build();
    
    // Verify transaction succeeds
    assert!(context.verify_tx(&tx, MAX_CYCLES).is_ok());
}
```

### 2. Type ID Transfer Test

```rust
#[test]
fn test_type_id_transfer() {
    let mut context = Context::default();
    
    // Setup existing Type ID cell
    let type_id_cell = create_type_id_cell(&mut context);
    
    // Create transfer transaction
    let tx = TransactionBuilder::default()
        .input(CellInput::new(type_id_cell.outpoint, 0))
        .output(CellOutput::new_builder()
            .type_(type_id_cell.type_script)  // Same Type ID
            .lock(new_owner_lock)             // New owner
            .build())
        .build();
    
    // Verify transfer succeeds
    assert!(context.verify_tx(&tx, MAX_CYCLES).is_ok());
}
```

## Best Practices

### 1. **Always Validate Singleton Property**
- Check for multiple cells with the same Type ID
- Reject transactions that violate uniqueness

### 2. **Use Built-in Validation When Possible**
- Leverage `ckb-std::type_id::check_type_id()` for standard cases
- Implement custom validation only when needed

### 3. **Plan for Upgradeability**
- Use Type ID for contracts that may need upgrades
- Design upgrade mechanisms from the beginning

### 4. **Test All Scenarios**
- Test creation, transfer, and burning
- Verify singleton enforcement
- Test upgrade scenarios

### 5. **Document Type ID Usage**
- Clear documentation of Type ID purpose
- Specify args layout and Type ID location
- Provide upgrade procedures

The Type ID pattern is essential for creating unique, upgradeable assets and contracts on CKB. By ensuring singleton properties and enabling script upgradeability, it provides a powerful foundation for sophisticated blockchain applications while maintaining security and preventing duplication attacks.