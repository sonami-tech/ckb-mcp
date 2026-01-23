# Script Source Patterns: Group Sources vs Manual Matching

## Description

CKB script source patterns explaining the critical difference between group sources (GroupInput/GroupOutput) and manual script matching. Group sources reflect script execution context not script presence. Lock scripts only execute on inputs, making Source::GroupOutput return zero even when outputs have the same lock. Complete Rust patterns for correctly counting and validating cells with matching scripts using manual hash comparison.

## The Core Problem

When developing CKB lock scripts, developers often incorrectly use `Source::GroupInput` and `Source::GroupOutput` to count cells with matching scripts in transaction validation logic. This fundamental misunderstanding causes validation failures because group sources reflect script execution context, not script presence.

## Understanding Script Execution Context

### Lock Scripts vs Type Scripts

**Lock Scripts**:
- Only execute on inputs (to validate spending authorization)
- Never execute on outputs (outputs are just being created)
- `Source::GroupInput` finds inputs where the lock script is executing
- `Source::GroupOutput` always returns zero results for lock scripts

**Type Scripts**:
- Execute on both inputs (validate destruction) and outputs (validate creation)
- `Source::GroupInput` finds inputs with the type script
- `Source::GroupOutput` finds outputs with the type script

### The Fundamental Mismatch

Group sources answer: "Where is my script executing?"
Manual matching answers: "Where is my script present?"

For lock scripts, these are different questions with critically different answers.

## Common Failure Pattern

### Incorrect Implementation

```rust
use ckb_std::{
    ckb_constants::Source,
    high_level::{load_cell, load_input, QueryIter},
};

pub fn validate_transaction_structure() -> Result<(), Error> {
    // INCORRECT - Will always fail for lock scripts
    let input_count = QueryIter::new(load_input, Source::GroupInput).count();   // Returns 1
    let output_count = QueryIter::new(load_cell, Source::GroupOutput).count();  // Always returns 0!

    // This validation always fails because output_count is always 0
    if input_count != 1 || output_count != 1 {
        return Err(Error::InvalidTransactionStructure);  // Always fails!
    }

    Ok(())
}
```

### Why This Fails

1. `Source::GroupInput` correctly returns 1 (the input where lock script is executing)
2. `Source::GroupOutput` returns 0 because lock scripts don't execute on outputs
3. Even if outputs have the exact same lock script, they won't be included
4. The validation always fails regardless of actual transaction structure

## Correct Solution: Manual Script Matching

### Implementation Pattern

```rust
use ckb_std::{
    ckb_constants::Source,
    high_level::{load_cell, load_script},
    ckb_types::prelude::*,
};

pub fn validate_transaction_structure() -> Result<(), Error> {
    // Load current script and calculate its hash
    let current_script = load_script()?;
    let current_script_hash = current_script.calc_script_hash();

    // Manually count cells with matching lock script
    let input_count = count_matching_lock_scripts(Source::Input, &current_script_hash)?;
    let output_count = count_matching_lock_scripts(Source::Output, &current_script_hash)?;

    // Now validation works correctly
    if input_count != 1 || output_count != 1 {
        return Err(Error::InvalidTransactionStructure);
    }

    Ok(())
}

fn count_matching_lock_scripts(source: Source, target_hash: &[u8; 32]) -> Result<usize, Error> {
    let mut count = 0;
    let mut index = 0;

    loop {
        match load_cell(index, source) {
            Ok(cell) => {
                if cell.lock().calc_script_hash().as_slice() == target_hash {
                    count += 1;
                }
                index += 1;
            }
            Err(_) => break,  // No more cells
        }
    }

    Ok(count)
}
```

### Optimized Version with Iterator

```rust
use ckb_std::{
    ckb_constants::Source,
    high_level::{load_cell, load_script, QueryIter},
    ckb_types::prelude::*,
};

pub fn validate_transaction_structure_optimized() -> Result<(), Error> {
    let current_script = load_script()?;
    let current_script_hash = current_script.calc_script_hash();

    // Use QueryIter but with Source::Input/Output, not GroupInput/GroupOutput
    let input_count = QueryIter::new(load_cell, Source::Input)
        .filter(|cell| cell.lock().calc_script_hash() == current_script_hash)
        .count();

    let output_count = QueryIter::new(load_cell, Source::Output)
        .filter(|cell| cell.lock().calc_script_hash() == current_script_hash)
        .count();

    if input_count != 1 || output_count != 1 {
        return Err(Error::InvalidTransactionStructure);
    }

    Ok(())
}
```

## Advanced Patterns

### Pattern 1: Validating Lock Script Conservation

```rust
/// Ensure the same number of cells with our lock script exist before and after
pub fn validate_lock_conservation() -> Result<(), Error> {
    let current_script = load_script()?;
    let current_script_hash = current_script.calc_script_hash();

    let inputs_with_lock = QueryIter::new(load_cell, Source::Input)
        .filter(|cell| cell.lock().calc_script_hash() == current_script_hash)
        .count();

    let outputs_with_lock = QueryIter::new(load_cell, Source::Output)
        .filter(|cell| cell.lock().calc_script_hash() == current_script_hash)
        .count();

    // Lock script cells must be conserved
    if inputs_with_lock != outputs_with_lock {
        return Err(Error::LockConservationViolated);
    }

    Ok(())
}
```

### Pattern 2: Multi-Signature Validation

```rust
/// Validate that multiple specific lock scripts are present in inputs
pub fn validate_multisig_authorization(required_lock_hashes: &[[u8; 32]]) -> Result<(), Error> {
    let mut found_locks = vec![false; required_lock_hashes.len()];

    // Check all inputs for required lock scripts
    for i in 0.. {
        match load_cell(i, Source::Input) {
            Ok(cell) => {
                let lock_hash = cell.lock().calc_script_hash();

                // Check if this lock hash is one of the required ones
                for (idx, required_hash) in required_lock_hashes.iter().enumerate() {
                    if lock_hash.as_slice() == required_hash {
                        found_locks[idx] = true;
                    }
                }
            }
            Err(_) => break,
        }
    }

    // Ensure all required locks were found
    if !found_locks.iter().all(|found| *found) {
        return Err(Error::MissingRequiredSignature);
    }

    Ok(())
}
```

### Pattern 3: Type Script Validation (Contrast)

```rust
/// Type scripts CAN use group sources effectively
pub fn validate_type_script_conservation() -> Result<(), Error> {
    // For type scripts, group sources work as expected
    let input_amount = calculate_udt_amount(Source::GroupInput)?;
    let output_amount = calculate_udt_amount(Source::GroupOutput)?;

    // This works correctly for type scripts
    if output_amount > input_amount {
        return Err(Error::TokenInflation);
    }

    Ok(())
}

fn calculate_udt_amount(source: Source) -> Result<u128, Error> {
    let mut total = 0u128;

    // Group sources work correctly for type scripts
    for cell_data in QueryIter::new(load_cell_data, source) {
        if cell_data.len() >= 16 {
            let mut amount_bytes = [0u8; 16];
            amount_bytes.copy_from_slice(&cell_data[0..16]);
            let amount = u128::from_le_bytes(amount_bytes);
            total = total.checked_add(amount).ok_or(Error::Overflow)?;
        }
    }

    Ok(total)
}
```

## Decision Matrix

### When to Use Group Sources

| Script Type | Scenario | Use Group Sources? | Reason |
|------------|----------|-------------------|---------|
| Lock Script | Count inputs with script | ✅ Yes | Lock executes on inputs |
| Lock Script | Count outputs with script | ❌ No | Lock doesn't execute on outputs |
| Lock Script | Validate transaction structure | ❌ No | Need manual matching for outputs |
| Type Script | Count inputs with script | ✅ Yes | Type executes on inputs |
| Type Script | Count outputs with script | ✅ Yes | Type executes on outputs |
| Type Script | Token conservation check | ✅ Yes | Type executes on both sides |

### Quick Reference

```rust
// Lock Script - Finding cells with your script
let my_inputs = count_with_manual_matching(Source::Input);    // ✅ Correct
let my_outputs = count_with_manual_matching(Source::Output);  // ✅ Correct

let my_inputs = count_with_group_source(Source::GroupInput);   // ✅ Works
let my_outputs = count_with_group_source(Source::GroupOutput); // ❌ Always 0

// Type Script - Finding cells with your script
let my_inputs = count_with_group_source(Source::GroupInput);   // ✅ Works
let my_outputs = count_with_group_source(Source::GroupOutput); // ✅ Works
```

## Complete Working Example

### Lock Script with Proper Validation

```rust
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{bytes::Bytes, core::ScriptHashType, prelude::*},
    high_level::{load_cell, load_script, load_witness_args, QueryIter},
};

#[repr(i8)]
pub enum Error {
    IndexOutOfBound = 1,
    InvalidTransactionStructure = 2,
    UnauthorizedModification = 3,
}

pub fn main() -> Result<(), Error> {
    // Load and verify signature (standard lock script validation)
    verify_signature()?;

    // Additional validation: Ensure exactly one output with same lock
    validate_lock_preservation()?;

    Ok(())
}

fn validate_lock_preservation() -> Result<(), Error> {
    let current_script = load_script()?;
    let current_script_hash = current_script.calc_script_hash();

    // Manual counting for both inputs and outputs
    let mut input_count = 0;
    let mut output_count = 0;

    // Count inputs with our lock script
    for i in 0.. {
        match load_cell(i, Source::Input) {
            Ok(cell) => {
                if cell.lock().calc_script_hash() == current_script_hash {
                    input_count += 1;
                }
            }
            Err(_) => break,
        }
    }

    // Count outputs with our lock script
    for i in 0.. {
        match load_cell(i, Source::Output) {
            Ok(cell) => {
                if cell.lock().calc_script_hash() == current_script_hash {
                    output_count += 1;
                }
            }
            Err(_) => break,
        }
    }

    // Validate structure
    match (input_count, output_count) {
        (1, 1) => Ok(()),  // Standard transfer
        (1, 0) => Ok(()),  // Consuming the cell (allowed)
        _ => Err(Error::InvalidTransactionStructure),
    }
}

fn verify_signature() -> Result<(), Error> {
    // Standard signature verification logic
    // (Implementation details omitted for brevity)
    Ok(())
}
```

## Testing Patterns

### Unit Test for Script Source Behavior

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ckb_testtool::{builtin::ALWAYS_SUCCESS, context::Context};

    #[test]
    fn test_group_source_vs_manual_matching() {
        let mut context = Context::default();

        // Create a lock script
        let lock_script = context.build_script(&ALWAYS_SUCCESS, Bytes::from(vec![1]));
        let lock_hash = lock_script.calc_script_hash();

        // Build transaction with 2 inputs and 2 outputs using same lock
        let tx = build_test_transaction(&mut context, &lock_script, 2, 2);

        // In lock script context:
        // GroupInput would return 2 (executing on 2 inputs)
        // GroupOutput would return 0 (not executing on outputs)
        // Manual matching would correctly find 2 inputs and 2 outputs

        // Verify the transaction
        let result = context.verify_tx(&tx, 70_000_000);
        assert!(result.is_ok());
    }

    fn build_test_transaction(
        context: &mut Context,
        lock_script: &Script,
        input_count: usize,
        output_count: usize,
    ) -> TransactionView {
        let mut tx_builder = TransactionBuilder::default();

        // Add inputs
        for _ in 0..input_count {
            let input_out_point = context.create_cell(
                CellOutput::new_builder()
                    .capacity(1000_00000000u64.pack())
                    .lock(lock_script.clone())
                    .build(),
                Bytes::new(),
            );
            tx_builder = tx_builder.input(
                CellInput::new_builder()
                    .previous_output(input_out_point)
                    .build()
            );
        }

        // Add outputs
        for _ in 0..output_count {
            tx_builder = tx_builder
                .output(
                    CellOutput::new_builder()
                        .capacity(900_00000000u64.pack())
                        .lock(lock_script.clone())
                        .build()
                )
                .output_data(Bytes::new().pack());
        }

        tx_builder.build()
    }
}
```

## Key Takeaways

1. **Group sources reflect execution context**, not script presence
2. **Lock scripts need manual matching** for output validation
3. **Type scripts can use group sources** for both inputs and outputs
4. **Always test** script source behavior in your validation logic
5. **Document** whether your script uses group sources or manual matching

## Common Mistakes to Avoid

- ❌ Using `Source::GroupOutput` in lock scripts to find outputs
- ❌ Assuming group sources find all cells with your script
- ❌ Mixing group sources and manual matching incorrectly
- ❌ Not testing validation logic with multiple inputs/outputs

## Best Practices

1. **Be explicit** about which counting method you're using
2. **Comment** why you chose group sources vs manual matching
3. **Test** with various transaction structures
4. **Validate** both positive and negative cases
5. **Consider performance** - manual matching requires more iterations

This pattern is fundamental to correct CKB script development. Understanding the distinction between script execution context and script presence will prevent common validation failures and ensure robust smart contract behavior.