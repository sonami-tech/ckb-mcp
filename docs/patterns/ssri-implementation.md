## Description

Implementation guide for Script-Sourced Rich Information (SSRI) in CKB smart contracts. Covers method routing, SSRI trait implementation, domain-specific traits (UDT), execution context handling, dynamic token information, enhanced syscalls, and testing strategies for enabling off-chain queries and rich metadata.

## Related Resources

- [ckb-dev-context://protocols/ssri](ckb-dev-context://protocols/ssri) - Extension protocol enabling CKB scripts to provide rich information through off-chain execution
- [ckb-dev-context://tools/ssri-server](ckb-dev-context://tools/ssri-server) - Comprehensive integration guide for SSRI server enabling off-chain CKB script execution
- [ckb-dev-context://api-reference/ccc-sdk-ssri](ckb-dev-context://api-reference/ccc-sdk-ssri) - Guide to Script-Sourced Rich Information framework in the CCC SDK

Implement Script-Sourced Rich Information (SSRI) in CKB smart contracts to enable off-chain queries and rich metadata.

## Overview

SSRI implementation involves:
1. Method routing in script entry point
2. Implementing SSRI trait methods
3. Creating domain-specific traits (e.g., UDT)
4. Handling different execution contexts

## Basic Script Structure

### Entry Point with Method Routing

```rust
use ckb_std::{ckb_constants::Source, ckb_types::prelude::*, env};

// Method path constants (first 8 bytes of hash)
const SSRI_VERSION: [u8; 8] = [0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0];
const SSRI_GET_METHODS: [u8; 8] = [0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01];
const UDT_BALANCE: [u8; 8] = [0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12];
const UDT_NAME: [u8; 8] = [0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23];

fn main() -> i8 {
    let argv = env::argv();
    
    // No method specified - run as validator
    if argv.is_empty() {
        return match validate_transaction() {
            Ok(_) => 0,
            Err(e) => e as i8,
        };
    }
    
    // Route to method handler
    let method = match argv[0].as_bytes().try_into() {
        Ok(method) => method,
        Err(_) => return -1, // Invalid method path length
    };
    
    match route_method(method, &argv[1..]) {
        Ok(_) => 0,
        Err(e) => e as i8,
    }
}

fn route_method(method: [u8; 8], args: &[String]) -> Result<(), i8> {
    match method {
        SSRI_VERSION => handle_ssri_version(),
        SSRI_GET_METHODS => handle_ssri_get_methods(args),
        UDT_BALANCE => handle_udt_balance(),
        UDT_NAME => handle_udt_name(),
        _ => Err(-2), // Unknown method
    }
}
```

### Output Helper Functions

```rust
use ckb_std::syscalls;

fn set_output(data: &[u8]) -> Result<(), i8> {
    match syscalls::set_content(data) {
        Ok(_) => Ok(()),
        Err(_) => Err(-3),
    }
}

fn output_u8(value: u8) -> Result<(), i8> {
    set_output(&[value])
}

fn output_u128(value: u128) -> Result<(), i8> {
    set_output(&value.to_le_bytes())
}

fn output_string(value: &str) -> Result<(), i8> {
    // Molecule vector<byte> encoding
    let bytes = value.as_bytes();
    let len = (bytes.len() as u32).to_le_bytes();
    
    let mut output = Vec::with_capacity(4 + bytes.len());
    output.extend_from_slice(&len);
    output.extend_from_slice(bytes);
    
    set_output(&output)
}
```

## Implementing SSRI Trait

### Version Method

```rust
fn handle_ssri_version() -> Result<(), i8> {
    output_u8(0) // Current version is 0
}
```

### Get Methods

```rust
fn handle_ssri_get_methods(args: &[String]) -> Result<(), i8> {
    // Parse pagination parameters
    let offset = parse_u64(&args.get(0).cloned().unwrap_or_default())
        .unwrap_or(0);
    let limit = parse_u64(&args.get(1).cloned().unwrap_or_default())
        .unwrap_or(0);
    
    // All available methods
    let all_methods = vec![
        SSRI_VERSION,
        SSRI_GET_METHODS,
        SSRI_HAS_METHODS,
        SSRI_GET_CELL_DEPS,
        UDT_BALANCE,
        UDT_NAME,
        UDT_SYMBOL,
        UDT_DECIMALS,
    ];
    
    // Apply pagination
    let start = offset as usize;
    let end = if limit == 0 {
        all_methods.len()
    } else {
        (start + limit as usize).min(all_methods.len())
    };
    
    let methods = &all_methods[start..end];
    
    // Encode as Molecule vector
    output_method_array(methods)
}

fn output_method_array(methods: &[[u8; 8]]) -> Result<(), i8> {
    let mut output = Vec::new();
    
    // Vector length
    output.extend_from_slice(&(methods.len() as u32).to_le_bytes());
    
    // Method data
    for method in methods {
        output.extend_from_slice(method);
    }
    
    set_output(&output)
}
```

### Has Methods

```rust
fn handle_ssri_has_methods(args: &[String]) -> Result<(), i8> {
    // Parse method array from args
    let methods = parse_method_array(&args[0])?;
    
    let available_methods = get_available_methods();
    let mut results = Vec::new();
    
    for method in methods {
        let exists = available_methods.contains(&method);
        results.push(if exists { 1u8 } else { 0u8 });
    }
    
    // Output as Molecule vector<bool>
    output_bool_array(&results)
}
```

### Get Cell Dependencies

```rust
fn handle_ssri_get_cell_deps(args: &[String]) -> Result<(), i8> {
    let offset = parse_u64(&args.get(0).cloned().unwrap_or_default())
        .unwrap_or(0);
    let limit = parse_u64(&args.get(1).cloned().unwrap_or_default())
        .unwrap_or(0);
    
    // Define required dependencies
    let deps = vec![
        CellDep::new_builder()
            .out_point(
                OutPoint::new_builder()
                    .tx_hash(SECP256K1_BLAKE160_SIGHASH_ALL_TYPE_HASH.pack())
                    .index(0u32.pack())
                    .build()
            )
            .dep_type(DepType::DepGroup.into())
            .build(),
    ];
    
    // Apply pagination and output
    let paginated = paginate_slice(&deps, offset, limit);
    output_cell_dep_array(paginated)
}
```

## Implementing UDT Trait

### Balance Method (Cell Context)

```rust
fn handle_udt_balance() -> Result<(), i8> {
    // Verify execution context
    if !is_cell_context() {
        return Err(-4); // Wrong context
    }
    
    // Load cell data
    let data = load_cell_data(0, Source::GroupInput)
        .map_err(|_| -5)?;
    
    // Parse balance (first 16 bytes for UDT)
    if data.len() < 16 {
        return Err(-6); // Invalid data
    }
    
    let mut balance_bytes = [0u8; 16];
    balance_bytes.copy_from_slice(&data[0..16]);
    let balance = u128::from_le_bytes(balance_bytes);
    
    output_u128(balance)
}
```

### Token Metadata Methods (Script Context)

```rust
fn handle_udt_name() -> Result<(), i8> {
    // These can be constants or loaded from cell data
    output_string("My Token")
}

fn handle_udt_symbol() -> Result<(), i8> {
    output_string("MTK")
}

fn handle_udt_decimals() -> Result<(), i8> {
    output_u8(8) // 8 decimal places
}
```

## Advanced Patterns

### Dynamic Token Information

```rust
fn handle_udt_name() -> Result<(), i8> {
    // Load from xUDT extension data
    let script = load_script()?;
    let args = script.args().raw_data();
    
    if args.len() >= 32 {
        // Has extension data
        match find_extension_data() {
            Ok(data) => {
                let name = parse_name_from_extension(&data)?;
                output_string(&name)
            }
            Err(_) => output_string("Unknown Token")
        }
    } else {
        output_string("Legacy UDT")
    }
}
```

### Using Enhanced Syscalls

```rust
fn handle_get_total_supply() -> Result<(), i8> {
    let script = load_script()?;
    let type_script = script.as_slice();
    
    let mut total_supply = 0u128;
    let mut offset = 0;
    
    // Find all cells with this type script
    loop {
        let mut out_point = [0u8; 36];
        match find_out_point_by_type(&mut out_point, type_script) {
            Ok(_) => {
                // Load cell and add balance
                if let Ok(cell_data) = find_cell_data_by_out_point(&out_point) {
                    if cell_data.len() >= 16 {
                        let balance = u128::from_le_bytes(
                            cell_data[0..16].try_into().unwrap()
                        );
                        total_supply += balance;
                    }
                }
                offset += 1;
            }
            Err(_) => break, // No more cells
        }
    }
    
    output_u128(total_supply)
}
```

### Context Detection

```rust
fn is_cell_context() -> bool {
    // Try to load cell - will fail if not in cell context
    match load_cell(0, Source::GroupInput) {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn get_execution_level() -> ExecutionLevel {
    if is_transaction_context() {
        ExecutionLevel::Transaction
    } else if is_cell_context() {
        ExecutionLevel::Cell
    } else if is_script_context() {
        ExecutionLevel::Script
    } else {
        ExecutionLevel::Code
    }
}
```

## Testing SSRI Scripts

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ckb_std::simulator;
    
    #[test]
    fn test_ssri_version() {
        simulator::init_simulator();
        
        let output = simulator::run_method(SSRI_VERSION, vec![]);
        assert_eq!(output, vec![0u8]);
    }
    
    #[test]
    fn test_udt_balance() {
        simulator::init_simulator();
        
        // Setup cell with balance
        let cell_data = 1000u128.to_le_bytes();
        simulator::set_cell_data(0, Source::GroupInput, cell_data);
        
        let output = simulator::run_method(UDT_BALANCE, vec![]);
        assert_eq!(output, 1000u128.to_le_bytes());
    }
}
```

### Integration Testing

```rust
use ssri_server::SsriServer;

#[tokio::test]
async fn test_ssri_integration() {
    let server = SsriServer::new("http://localhost:9090");
    
    // Test script level execution
    let name = server.execute_script_method(
        &udt_script,
        "UDT.name"
    ).await.unwrap();
    
    assert_eq!(
        String::from_utf8(parse_molecule_string(&name)).unwrap(),
        "My Token"
    );
    
    // Test cell level execution
    let balance = server.execute_cell_method(
        &udt_cell,
        "UDT.balance"
    ).await.unwrap();
    
    assert_eq!(
        u128::from_le_bytes(balance.try_into().unwrap()),
        1000
    );
}
```

## Best Practices

### Method Path Generation

```rust
// Generate method paths at build time
const fn generate_method_path(signature: &str) -> [u8; 8] {
    let hash = ckb_hash(signature.as_bytes());
    [
        hash[0], hash[1], hash[2], hash[3],
        hash[4], hash[5], hash[6], hash[7]
    ]
}

// Use in constants
const MY_METHOD: [u8; 8] = generate_method_path("MyTrait.myMethod");
```

### Error Handling

```rust
#[repr(i8)]
enum SsriError {
    InvalidMethod = -1,
    UnknownMethod = -2,
    OutputFailed = -3,
    WrongContext = -4,
    LoadFailed = -5,
    InvalidData = -6,
}
```

### Optimization

1. **Minimize Output Size**: Use efficient encoding
2. **Cache Computations**: Avoid redundant calculations
3. **Early Returns**: Fail fast on errors
4. **Memory Management**: Use stack allocation when possible

## Common Patterns

### Paginated Results

```rust
fn paginate_slice<T>(items: &[T], offset: u64, limit: u64) -> &[T] {
    let start = offset as usize;
    if start >= items.len() {
        return &[];
    }
    
    let end = if limit == 0 {
        items.len()
    } else {
        (start + limit as usize).min(items.len())
    };
    
    &items[start..end]
}
```

### Molecule Encoding Helpers

```rust
fn encode_vector<T: AsRef<[u8]>>(items: &[T]) -> Vec<u8> {
    let mut output = Vec::new();
    
    // Vector length
    output.extend_from_slice(&(items.len() as u32).to_le_bytes());
    
    // Items
    for item in items {
        output.extend_from_slice(item.as_ref());
    }
    
    output
}
```

### Method Registration Macro

```rust
macro_rules! register_methods {
    ($($method:ident => $handler:ident),*) => {
        fn route_method(method: [u8; 8], args: &[String]) -> Result<(), i8> {
            match method {
                $($method => $handler(args),)*
                _ => Err(SsriError::UnknownMethod as i8),
            }
        }
    };
}

register_methods! {
    SSRI_VERSION => handle_version,
    UDT_BALANCE => handle_balance,
    UDT_NAME => handle_name
}
```