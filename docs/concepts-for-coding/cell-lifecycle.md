## Description

Practical cell lifecycle management for CKB developers, covering live and dead cell states, creation patterns, consumption mechanics, data updates, and capacity requirements. Features complete code examples for state machines, validation logic, and cell dependency patterns. Essential for understanding CKB's UTXO-based cell model and building robust applications.

## Cell States
```rust
// Live Cell - can be consumed as input
struct LiveCell {
    capacity: u64,           // CKB amount in Shannon
    lock: Script,            // Who can unlock this cell
    type_: Option<Script>,   // Optional validation rules
    data: Bytes,            // Arbitrary data
}

// Dead Cell - already consumed (cannot be used again)
// Dead cells exist for historical reference only
```

## Cell Creation Pattern
```rust
// In a transaction, you create output cells
let output_cell = CellOutput::new_builder()
    .capacity(Capacity::shannons(10000000000u64).pack()) // 100 CKB
    .lock(lock_script.clone())
    .type_(Some(type_script).pack())
    .build();

// Cell data is separate
let cell_data = Bytes::from(b"Hello, CKB!".to_vec());
```

## Cell Consumption (Spending)
```rust
// To update a cell, you must:
// 1. Consume the old cell as input
// 2. Create new cell(s) as output
// 3. The old cell becomes "dead"

// Input: reference to existing live cell
let input = CellInput::new_builder()
    .previous_output(
        OutPoint::new_builder()
            .tx_hash(previous_tx_hash)
            .index(0u32.pack())
            .build()
    )
    .build();
```

## Cell Data Updates
```rust
// To update cell data:
// Load current data
let current_data = load_cell_data(0, Source::GroupInput)?;

// Modify data
let new_data = modify_data(current_data);

// Verify in script that output has new data
let output_data = load_cell_data(0, Source::GroupOutput)?;
if output_data != new_data {
    return Err(Error::InvalidData);
}
```

## Cell Capacity Calculation (Authoritative Reference)

### Understanding Occupied Capacity

Every cell must have sufficient capacity to store all its components. The capacity serves dual purposes:
1. **Token value**: Amount of CKB tokens in the cell
2. **Storage limit**: Maximum bytes the cell can occupy

### Complete Formula

```rust
occupied_capacity = capacity_field + lock_script_size + type_script_size + data_size
```

**Component Breakdown**:
- **Capacity field**: 8 bytes (stores the capacity value itself)
- **Lock script**: `code_hash (32) + hash_type (1) + args (variable)`
- **Type script**: `code_hash (32) + hash_type (1) + args (variable)` (if present)
- **Data**: Actual cell data bytes

### Common Lock Script Sizes

| Lock Type | Size | Components |
|-----------|------|------------|
| Secp256k1 (standard) | 53 bytes | code_hash (32) + hash_type (1) + pubkey_hash (20) |
| Empty/Default | 33 bytes | code_hash (32) + hash_type (1) + empty args (0) |
| Multisig | 53+ bytes | code_hash (32) + hash_type (1) + multisig_config (20+) |

### Common Type Script Sizes

| Type | Size | Components |
|------|------|------------|
| UDT/sUDT | 65 bytes | code_hash (32) + hash_type (1) + owner_lock_hash (32) |
| xUDT | 65 bytes | code_hash (32) + hash_type (1) + unique_id (32) |
| No type script | 0 bytes | N/A |

### Minimum Cell Capacity

The absolute minimum for an empty cell is **61 CKBytes**:
- 8 (capacity) + 33 (minimal lock) + 20 (typical args) = 61 bytes

```rust
const BASIC_CAPACITY: u64 = 61_00000000; // 61 CKB minimum in shannons
```

### Manual Calculation Example

```rust
fn calculate_occupied_capacity(lock: &Script, type_opt: &Option<Script>, data: &[u8]) -> u64 {
    // Capacity field: 8 bytes
    // Lock script: serialized size (code_hash + hash_type + args)
    // Type script: serialized size (if present)
    // Data: actual data bytes

    let capacity_field = 8u64;
    let lock_size = lock.as_slice().len() as u64;
    let type_size = type_opt.as_ref().map_or(0, |t| t.as_slice().len() as u64);
    let data_size = data.len() as u64;

    capacity_field + lock_size + type_size + data_size
}
```

### Best Practice: Use SDK Method

**Always prefer the SDK's `occupied_capacity()` method for accurate calculation**:

```rust
// Build the output cell
let output = CellOutput::new_builder()
    .lock(lock_script)
    .type_(type_script.pack())
    .build();

// Let the SDK calculate exact capacity needed
let required_capacity = output.occupied_capacity(Capacity::bytes(data.len())?)?;

// Verify sufficient capacity
if cell.capacity().unpack() < required_capacity.as_u64() {
    return Err(Error::InsufficientCapacity);
}
```

### When to Use Each Method

- **SDK Method (`occupied_capacity()`)**: Production code, transaction building, accurate calculations
- **Manual Calculation**: Understanding the internals, educational purposes, debugging
- **Simplified Estimates**: Quick cost estimates, test data generation

### Why Capacity Matters

1. **Transaction Validation**: Cells with insufficient capacity are rejected
2. **Storage Economics**: 1 CKB = 1 byte on-chain storage
3. **Capacity Conservation**: Total output capacity cannot exceed input capacity (minus fees)
```

## Cell Dependency Pattern
```rust
// Cells can depend on other cells for script code
let cell_dep = CellDep::new_builder()
    .out_point(
        OutPoint::new_builder()
            .tx_hash(script_tx_hash)
            .index(0u32.pack())
            .build()
    )
    .dep_type(DepType::Code.into())
    .build();
```

## State Machine with Cells
```rust
// Example: Counter state machine
struct CounterState {
    value: u64,
    owner: [u8; 20],
}

impl CounterState {
    fn from_cell_data(data: &[u8]) -> Result<Self, Error> {
        if data.len() != 28 {
            return Err(Error::InvalidData);
        }
        Ok(CounterState {
            value: u64::from_le_bytes(data[0..8].try_into().unwrap()),
            owner: data[8..28].try_into().unwrap(),
        })
    }
    
    fn to_cell_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.value.to_le_bytes());
        data.extend_from_slice(&self.owner);
        data
    }
    
    fn increment(&mut self) {
        self.value += 1;
    }
}

// In type script validation
fn main() -> Result<(), Error> {
    let input_data = load_cell_data(0, Source::GroupInput)?;
    let output_data = load_cell_data(0, Source::GroupOutput)?;
    
    let input_state = CounterState::from_cell_data(&input_data)?;
    let output_state = CounterState::from_cell_data(&output_data)?;
    
    // Verify valid state transition
    if output_state.value != input_state.value + 1 {
        return Err(Error::InvalidTransition);
    }
    
    Ok(())
}
```

## Cell Collection and Iteration
```rust
// Iterate through multiple cells
use ckb_std::high_level::QueryIter;

// Collect all input cells with same type
let input_cells: Vec<CellOutput> = QueryIter::new(load_cell, Source::GroupInput)
    .collect();

// Process cells with data
let total_amount: u128 = QueryIter::new(load_cell_data, Source::GroupInput)
    .map(|data| u128::from_le_bytes(data[0..16].try_into().unwrap_or([0u8; 16])))
    .sum();
```

## Cell Validation Patterns
```rust
// Common validation patterns
fn validate_capacity_conservation() -> Result<(), Error> {
    let input_capacity = QueryIter::new(load_cell, Source::Input)
        .map(|cell| cell.capacity().unpack())
        .sum::<u64>();
    
    let output_capacity = QueryIter::new(load_cell, Source::Output)
        .map(|cell| cell.capacity().unpack())
        .sum::<u64>();
    
    if input_capacity < output_capacity {
        return Err(Error::CapacityNotConserved);
    }
    Ok(())
}

fn validate_single_cell_update() -> Result<(), Error> {
    // Ensure exactly one input and one output
    if QueryIter::new(load_cell, Source::GroupInput).count() != 1 {
        return Err(Error::InvalidCellCount);
    }
    if QueryIter::new(load_cell, Source::GroupOutput).count() != 1 {
        return Err(Error::InvalidCellCount);
    }
    Ok(())
}
```

## Practical Cell Management
```typescript
// Frontend: Track cell lifecycle
class CellTracker {
    async waitForCellConfirmation(outpoint: OutPoint): Promise<LiveCell> {
        while (true) {
            try {
                const cell = await this.client.getCell(outpoint);
                if (cell.status === "live") {
                    return cell;
                }
            } catch (e) {
                // Cell not found yet, continue waiting
            }
            await sleep(1000);
        }
    }
    
    async getCellHistory(outpoint: OutPoint): Promise<Transaction[]> {
        const transactions = [];
        let currentOutpoint = outpoint;
        
        while (currentOutpoint) {
            const tx = await this.client.getTransaction(currentOutpoint.txHash);
            transactions.push(tx);
            
            // Find previous outpoint in inputs
            const input = tx.inputs.find(i => 
                i.previousOutput.index === currentOutpoint.index
            );
            currentOutpoint = input?.previousOutput;
        }
        
        return transactions.reverse();
    }
}
```

## Performance Considerations
- **Batch Operations**: Process multiple cells in single transaction
- **Cell Indexing**: Use proper indexing for cell queries
- **Data Size**: Keep cell data minimal for lower fees
- **Script Complexity**: Simpler validation = lower gas costs
- **Dependency Management**: Minimize cell dependencies
```