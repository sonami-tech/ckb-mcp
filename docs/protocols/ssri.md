## Description

Extension protocol enabling CKB scripts to provide rich information through off-chain execution with enhanced syscalls. Covers method path systems, execution levels, SSRI/UDT trait implementations, enhanced syscall capabilities, server infrastructure, and integration patterns for complex query logic while maintaining on-chain verification.

## Related Resources

- [ckb://docs/patterns/ssri-implementation-guide](ckb://docs/patterns/ssri-implementation-guide) - Implementation guide for Script-Sourced Rich Information in CKB smart contracts
- [ckb://docs/tools/ssri-server](ckb://docs/tools/ssri-server) - Comprehensive integration guide for SSRI server enabling off-chain CKB script execution
- [ckb://docs/api-reference/ccc-sdk-ssri](ckb://docs/api-reference/ccc-sdk-ssri) - Guide to Script-Sourced Rich Information framework in the CCC SDK

SSRI extends CKB script capabilities by enabling off-chain execution with enhanced syscalls, allowing scripts to provide rich information and implement complex query logic while maintaining on-chain verification.

## Overview

SSRI addresses key interoperability challenges:
- **Off-chain Execution**: Scripts can run outside blockchain context
- **Rich Information**: Scripts expose metadata and behavior descriptions
- **Method-based Interface**: Standardized 64-bit method paths for interaction
- **Enhanced Syscalls**: Access to live cell data during off-chain execution

## Core Concepts

### Method Path System

SSRI uses 64-bit method paths for invoking script methods:

```rust
// Method path is first 8 bytes of CKB hash of method signature
// Format: [<trait>.]<method>
// Examples: "UDT.balance", "calc_unlock_time"

fn calculate_method_path(signature: &str) -> [u8; 8] {
    let hash = ckb_hash(signature.as_bytes());
    hash[0..8].try_into().unwrap()
}
```

### Execution Levels

SSRI defines four execution contexts with increasing capabilities:

1. **Code**: Only program code available
2. **Script**: Script structure accessible
3. **Cell**: Complete cell data available
4. **Transaction**: Full transaction context

### Method Distribution

Scripts read method path from argv[0]:

```rust
fn main() -> i8 {
    let args = ckb_std::env::argv();
    
    if args.is_empty() {
        // No method specified - run as verifier
        return verify_transaction();
    }
    
    let method_path = &args[0];
    if method_path.len() != 8 {
        return -1; // Invalid method path
    }
    
    // Route to appropriate method
    match method_path {
        b"UDT.name" => handle_udt_name(),
        b"UDT.bala" => handle_udt_balance(), // First 8 bytes of "UDT.balance"
        _ => -1, // Unknown method
    }
}
```

## Enhanced Syscalls

### Standard Syscalls (Modified Behavior)

```rust
// VM Version - Returns -1 for off-chain execution
fn ckb_vm_version() -> i32;

// Set Content - Max output 256KB minimum
fn ckb_set_content(content: &[u8]) -> i32;

// Load operations respect execution level
fn ckb_load_cell(buf: &mut [u8], offset: usize, index: usize, source: Source) -> i32;
```

### SSRI-Specific Syscalls

```rust
// Find live cell by type script
fn ckb_find_out_point_by_type(
    type_script: &[u8]
) -> Result<OutPoint, Error>;

// Load cell by outpoint
fn ckb_find_cell_by_out_point(
    out_point: &OutPoint
) -> Result<CellOutput, Error>;

// Load cell data by outpoint
fn ckb_find_cell_data_by_out_point(
    out_point: &OutPoint
) -> Result<Vec<u8>, Error>;
```

## SSRI Trait Implementation

All SSRI-compliant scripts must implement:

```rust
trait SSRI {
    // Protocol version (currently 0)
    fn version() -> u8;
    
    // List all available methods
    fn get_methods(offset: u64, limit: u64) -> Vec<[u8; 8]>;
    
    // Check method existence
    fn has_methods(methods: &[[u8; 8]]) -> Vec<bool>;
    
    // List required cell dependencies
    fn get_cell_deps(offset: u64, limit: u64) -> Vec<CellDep>;
}
```

## UDT Trait

Standard interface for User Defined Tokens:

```rust
trait UDT {
    // Get balance from cell (Cell level)
    fn balance() -> u128;
    
    // Get token name (Script level)
    fn name() -> String;
    
    // Get token symbol (Script level)
    fn symbol() -> String;
    
    // Get decimal places (Script level)
    fn decimals() -> u8;
}
```

### Implementation Example

```rust
// In script main function
match method_path {
    UDT_BALANCE => {
        // Cell level - read from cell data
        let data = load_cell_data(0, Source::GroupInput)?;
        let balance = u128::from_le_bytes(&data[0..16]);
        set_content(&balance.to_le_bytes())?;
    }
    UDT_NAME => {
        // Script level - return constant
        set_content(b"My Token")?;
    }
    UDT_SYMBOL => {
        set_content(b"MTK")?;
    }
    UDT_DECIMALS => {
        set_content(&[8u8])?; // 8 decimal places
    }
}
```

## SSRI Server

Reference implementation for off-chain execution:

```rust
// Execute at different levels
async fn run_script_level_code(code: &[u8], method: &str) -> Result<Vec<u8>>;
async fn run_script_level_script(script: &Script, method: &str) -> Result<Vec<u8>>;
async fn run_script_level_cell(cell: &Cell, method: &str) -> Result<Vec<u8>>;
async fn run_script_level_tx(tx: &Transaction, method: &str) -> Result<Vec<u8>>;
```

## Benefits

- **Rich Metadata**: Scripts provide structured information
- **Query Capabilities**: Complex off-chain data access
- **Standardization**: Common interfaces for token/NFT information
- **Flexibility**: Scripts define custom methods as needed
- **Performance**: Off-chain execution for expensive operations

## Security Considerations

- Off-chain results are informational only
- On-chain verification remains source of truth
- Method execution should be deterministic
- Resource limits prevent DoS attacks

## Best Practices

1. **Method Naming**: Use trait prefixes to avoid conflicts
2. **Error Handling**: Return appropriate error codes
3. **Backwards Compatibility**: Support verifier mode when no method specified
4. **Documentation**: Clearly document all exposed methods
5. **Testing**: Verify both on-chain and off-chain behavior

## Integration Example

```rust
// Query token information off-chain
let ssri_server = SsriServer::new("http://localhost:9090");

// Get token metadata
let name = ssri_server.execute_method(&script, "UDT.name").await?;
let symbol = ssri_server.execute_method(&script, "UDT.symbol").await?;
let decimals = ssri_server.execute_method(&script, "UDT.decimals").await?;

// Get balance from specific cell
let balance = ssri_server.execute_method_cell(&cell, "UDT.balance").await?;
```

## Future Extensions

SSRI enables future enhancements:
- NFT metadata and rendering
- Complex authorization logic
- Cross-chain bridge information
- Automated market maker calculations
- Governance participation rules