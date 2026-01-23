# CKB Rust Script Testing and Security

## Description

Testing frameworks and security patterns for CKB script development. Unit test setup with ckb-tool, TestContext utilities, and integration test examples for UDT validation. Performance optimization: memory-efficient streaming, const generics for fixed buffers, syscall batching. Security best practices: input validation, bounds checking, checksum verification, safe arithmetic with checked_add. Authorization patterns with permission flags. Debug output with syscalls::debug and error context macros.

## Testing Patterns

### Unit Testing Setup

```rust
// tests/src/lib.rs
use ckb_tool::{
    ckb_error::assert_error_eq,
    ckb_script::{ScriptError, TransactionScriptsVerifier},
    ckb_types::{
        bytes::Bytes,
        core::{TransactionBuilder, TransactionView},
        packed::*,
        prelude::*,
    },
    context::Context,
};

const MAX_CYCLES: u64 = 10_000_000;

pub struct TestContext {
    pub context: Context,
    pub contract_out_point: OutPoint,
}

impl TestContext {
    pub fn new() -> Self {
        let mut context = Context::default();
        let contract_bin = include_bytes!("../../build/my-script");
        let contract_out_point = context.deploy_cell(contract_bin.to_vec().into());

        Self {
            context,
            contract_out_point,
        }
    }

    pub fn build_script(&self, args: &[u8]) -> Script {
        self.context.build_script(&self.contract_out_point, Bytes::from(args.to_vec()))
    }

    pub fn verify_tx(&mut self, tx: &TransactionView) -> Result<u64, ScriptError> {
        self.context.verify_tx(tx, MAX_CYCLES)
    }
}

pub fn build_cell_with_data(lock: Script, type_: Option<Script>, data: &[u8]) -> (CellOutput, Bytes) {
    let cell = CellOutput::new_builder()
        .capacity((data.len() as u64 + 1000).pack())
        .lock(lock)
        .type_(type_.pack())
        .build();

    (cell, Bytes::from(data.to_vec()))
}
```

### Integration Tests

```rust
// tests/src/test_udt.rs
use super::*;
use my_ckb_script::error::Error;

#[test]
fn test_token_conservation() {
    let mut ctx = TestContext::new();
    let owner_hash = [1u8; 32];
    let script = ctx.build_script(&owner_hash);

    // Build transaction: Transfer 100 tokens
    let (input_cell, input_data) = build_token_cell(&script, 200);
    let (output1_cell, output1_data) = build_token_cell(&script, 100);
    let (output2_cell, output2_data) = build_token_cell(&script, 100);

    let tx = TransactionBuilder::default()
        .input(CellInput::new(OutPoint::new(h256!("0x1234").pack(), 0), 0))
        .output(output1_cell)
        .output(output2_cell)
        .outputs_data(vec![output1_data, output2_data].pack())
        .build();

    let cycles = ctx.verify_tx(&tx).expect("Transaction should pass");
    println!("Consumed cycles: {}", cycles);
}

#[test]
fn test_token_inflation_prevention() {
    let mut ctx = TestContext::new();
    let owner_hash = [1u8; 32];
    let script = ctx.build_script(&owner_hash);

    // Try to create more tokens than input
    let (input_cell, input_data) = build_token_cell(&script, 100);
    let (output_cell, output_data) = build_token_cell(&script, 200);

    let tx = TransactionBuilder::default()
        .input(CellInput::new(OutPoint::new(h256!("0x1234").pack(), 0), 0))
        .output(output_cell)
        .outputs_data(vec![output_data].pack())
        .build();

    let err = ctx.verify_tx(&tx).expect_err("Should fail due to inflation");
    assert_error_eq!(err, ScriptError::ValidationFailure(Error::InvalidAmount as i8));
}

fn build_token_cell(script: &Script, amount: u128) -> (CellOutput, Bytes) {
    let data = amount.to_le_bytes().to_vec();
    build_cell_with_data(
        Script::default(), // dummy lock
        Some(script.clone()),
        &data,
    )
}
```

## Performance Optimization

### Memory Management

```rust
// Efficient memory usage patterns
pub fn process_large_dataset() -> Result<(), Error> {
    // Use streaming instead of loading everything
    let mut total = 0u128;

    for (index, data) in QueryIter::new(load_cell_data, Source::GroupInput).enumerate() {
        // Process incrementally to avoid large allocations
        let amount = parse_amount_from_slice(&data)?;
        total = total.checked_add(amount).ok_or(Error::InvalidAmount)?;

        // Early termination if possible
        if total > MAX_ALLOWED_TOTAL {
            return Err(Error::InvalidAmount);
        }
    }

    Ok(())
}

// Use const generics for fixed-size buffers
fn parse_fixed_data<const N: usize>(data: &[u8]) -> Result<[u8; N], Error> {
    if data.len() < N {
        return Err(Error::InvalidData);
    }

    let mut buffer = [0u8; N];
    buffer.copy_from_slice(&data[0..N]);
    Ok(buffer)
}
```

### Syscall Optimization

```rust
// Batch operations when possible
pub fn efficient_cell_processing() -> Result<Vec<ProcessedCell>, Error> {
    // Load all data in one pass
    let all_cells: Vec<_> = QueryIter::new(load_cell, Source::GroupInput).collect();
    let all_data: Vec<_> = QueryIter::new(load_cell_data, Source::GroupInput).collect();

    // Process in memory
    let mut results = Vec::with_capacity(all_cells.len());
    for (cell, data) in all_cells.iter().zip(all_data.iter()) {
        results.push(ProcessedCell {
            capacity: cell.capacity().unpack(),
            data_hash: blake2b_hash(data),
            processed_data: process_data(data)?,
        });
    }

    Ok(results)
}

#[derive(Debug)]
struct ProcessedCell {
    capacity: u64,
    data_hash: [u8; 32],
    processed_data: ProcessedData,
}
```

## Security Best Practices

### Input Validation

```rust
pub fn secure_data_parsing(data: &[u8]) -> Result<ParsedData, Error> {
    // Length validation
    if data.len() > MAX_ALLOWED_SIZE {
        return Err(Error::InvalidData);
    }

    if data.len() < MIN_REQUIRED_SIZE {
        return Err(Error::InvalidData);
    }

    // Content validation
    validate_data_format(data)?;

    // Parse with bounds checking
    safe_parse_data(data)
}

fn validate_data_format(data: &[u8]) -> Result<(), Error> {
    // Check magic bytes
    if data.len() >= 4 && &data[0..4] != b"CKBT" {
        return Err(Error::InvalidData);
    }

    // Validate checksum
    let checksum = calculate_checksum(&data[4..data.len()-4]);
    let expected = u32::from_le_bytes(data[data.len()-4..].try_into().unwrap());

    if checksum != expected {
        return Err(Error::InvalidData);
    }

    Ok(())
}

// Prevent integer overflow
pub fn safe_arithmetic_operations(a: u128, b: u128) -> Result<u128, Error> {
    a.checked_add(b).ok_or(Error::InvalidAmount)
}
```

### Authorization Patterns

```rust
pub fn verify_authorization(required_permissions: &[Permission]) -> Result<(), Error> {
    let script = load_script()?;
    let permissions = extract_permissions_from_args(&script.args().raw_data())?;

    for required in required_permissions {
        if !permissions.contains(required) {
            return Err(Error::Unauthorized);
        }
    }

    Ok(())
}

#[derive(Debug, PartialEq)]
enum Permission {
    Read,
    Write,
    Admin,
    Transfer,
}

fn extract_permissions_from_args(args: &[u8]) -> Result<Vec<Permission>, Error> {
    if args.len() < 1 {
        return Ok(Vec::new());
    }

    let flags = args[0];
    let mut permissions = Vec::new();

    if flags & 0x01 != 0 { permissions.push(Permission::Read); }
    if flags & 0x02 != 0 { permissions.push(Permission::Write); }
    if flags & 0x04 != 0 { permissions.push(Permission::Admin); }
    if flags & 0x08 != 0 { permissions.push(Permission::Transfer); }

    Ok(permissions)
}
```

## Debugging and Development Tools

### Debug Output

```rust
#[cfg(debug_assertions)]
use ckb_std::syscalls;

pub fn debug_transaction_info() {
    #[cfg(debug_assertions)]
    {
        let input_count = QueryIter::new(load_cell, Source::Input).count();
        let output_count = QueryIter::new(load_cell, Source::Output).count();

        syscalls::debug(
            format!("Transaction: {} inputs, {} outputs", input_count, output_count).as_bytes()
        );

        for (i, cell) in QueryIter::new(load_cell, Source::GroupInput).enumerate() {
            syscalls::debug(
                format!("Group Input {}: capacity={}", i, cell.capacity().unpack()).as_bytes()
            );
        }
    }
}
```

### Error Context

```rust
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

pub fn detailed_validation() -> Result<(), ErrorContext> {
    let script = load_script()
        .map_err(|e| error_with_context!(
            Error::from(e),
            "load_script",
            "Failed to load current script"
        ))?;

    let args = script.args().raw_data();
    if args.len() != 32 {
        return Err(error_with_context!(
            Error::InvalidArgs,
            "args_validation",
            "Expected 32 bytes, got {}",
            args.len()
        ));
    }

    Ok(())
}
```

These patterns provide a foundation for testing, optimizing, and securing CKB scripts. Focus on thorough testing with realistic transaction scenarios, efficient memory usage, and comprehensive input validation to ensure script reliability and security.
