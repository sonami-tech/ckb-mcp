## Description

Comprehensive guide to modern CKB smart contract development using Rust-based workspace templates. Covers project setup, build automation, testing frameworks, and deployment patterns using the latest ckb-script-templates. Includes memory optimization, cross-platform development, and reproducible builds. Essential for professional CKB contract development workflows.

## Overview

The CKB script templates provide a mature workspace-based development environment for building, testing, and deploying smart contracts at scale. This approach enables multi-contract projects with shared dependencies and standardized build processes.

## Workspace Setup

### Creating a New Workspace

```bash
# Install cargo-generate if not already installed
cargo install cargo-generate

# Create workspace from template
cargo generate gh:cryptape/ckb-script-templates workspace

# Follow interactive prompts:
# Project name: my-ckb-workspace
# Author: Your Name
# Description: My CKB contracts workspace
```

### Workspace Structure

```
my-ckb-workspace/
├── Cargo.toml              # Workspace configuration
├── Makefile               # Build automation
├── contracts/             # Contract implementations
├── crates/               # Shared libraries
├── tests/                # Integration tests
├── scripts/              # Build and deployment scripts
└── target/               # Build artifacts
```

## Contract Generation

### Adding New Contracts

```bash
# Generate a new contract in the workspace
make generate CRATE=my-token

# Available contract types:
make generate CRATE=my-token TEMPLATE=contract            # Standard contract
make generate CRATE=my-lock TEMPLATE=stack-reorder-contract  # Memory optimized
make generate CRATE=my-type TEMPLATE=atomics-contract       # Atomic operations
```

### Contract Template Options

- **Standard Contract**: Basic structure with 16KB + 1.2MB heap
- **Stack Reorder Contract**: Custom memory layout for optimization
- **Atomics Contract**: Support for atomic operations (deprecated but maintained)

## Build System

### Automated Builds

```bash
# Build all contracts
make build

# Build specific contract
make build-contract CRATE=my-token

# Release build with optimizations
make build-release

# Debug build with symbols
make build-debug
```

### Build Configuration

The workspace uses optimized RISC-V compilation:

```toml
# .cargo/config.toml
[build]
target = "riscv64imac-unknown-none-elf"

[target.riscv64imac-unknown-none-elf]
rustflags = [
  "-C", "target-feature=+zba,+zbb,+zbc,+zbs,-a",
  "-C", "force-frame-pointers=no",
  "-C", "relocation-model=static"
]
```

## Testing Framework

### Unit Tests

```rust
// In contract source
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_contract_logic() {
        // Test contract functions directly
        let result = main_logic(&input_data);
        assert_eq!(result, expected_output);
    }
}
```

### Integration Tests

```bash
# Run all tests
make test

# Run tests for specific contract
make test-contract CRATE=my-token

# Run with debug logging
RUST_LOG=debug make test
```

### Native Simulator Testing

```bash
# Generate native simulator for existing contract
make generate-native-simulator CRATE=my-token

# Run simulator tests
make test-native-simulator CRATE=my-token
```

## Memory Management Patterns

### Standard Heap Configuration

```rust
// Cargo.toml
[dependencies]
ckb-std = { version = "0.17.0", default-features = false }

// In main.rs
#![no_std]
#![no_main]

use ckb_std::default_alloc;

// 16KB fixed heap + 1.2MB dynamic heap
default_alloc!(16 * 1024, 1200 * 1024, 64);
```

### Custom Memory Layout

For memory-critical contracts using stack-reorder template:

```rust
// Custom bootloader assembly
extern "C" {
    fn _start();
}

#[no_mangle]
pub unsafe extern "C" fn ckb_entry() {
    // Custom stack initialization
    _start();
}
```

## Advanced Development Patterns

### Multi-Contract Dependencies

```toml
# contracts/my-token/Cargo.toml
[dependencies]
shared-utils = { path = "../../crates/shared-utils" }
my-lock = { path = "../my-lock" }
```

### Cross-Platform Development

```bash
# Conditional compilation for different environments
make build-native     # Native development build
make build-simulator  # x64 simulator build
make build-contract   # RISC-V contract build
```

### Reproducible Builds

```bash
# Docker-based reproducible builds
make docker-build

# Verify build reproducibility
make verify-reproducible
```

## Testing Strategies

### Property-Based Testing

```rust
// Using quickcheck for property testing
#[cfg(test)]
mod property_tests {
    use quickcheck::quickcheck;
    
    quickcheck! {
        fn contract_invariant(input: Vec<u8>) -> bool {
            let result = contract_function(&input);
            // Test invariants always hold
            result.len() <= input.len() * 2
        }
    }
}
```

### Integration Test Patterns

```rust
// tests/integration_test.rs
use ckb_testtool::{builtin::ALWAYS_SUCCESS, context::Context};

#[test]
fn test_token_transfer() {
    let mut context = Context::default();
    
    // Load contract
    let contract_bin = include_bytes!("../target/riscv64imac-unknown-none-elf/release/my-token");
    let contract_out_point = context.deploy_cell(contract_bin.to_vec().into());
    
    // Build transaction
    let tx = TransactionBuilder::default()
        .input(CellInput::new(prev_out_point, 0))
        .output(CellOutput::new_builder()
            .capacity(Capacity::shannons(100_000_000_000).pack())
            .lock(lock_script)
            .type_(Some(type_script).pack())
            .build())
        .build();
    
    // Verify transaction
    let result = context.verify_tx(&tx, MAX_CYCLES);
    assert!(result.is_ok());
}
```

## Build Optimization

### Compiler Optimizations

```toml
# Cargo.toml profile for contracts
[profile.release]
overflow-checks = true
opt-level = "s"
lto = true
codegen-units = 1
panic = "abort"
```

### Size Optimization

```bash
# Strip debug symbols
make strip-contracts

# Analyze binary size
make size-analysis CRATE=my-token

# Optimize for minimum size
make build-min-size
```

## Development Workflow

### Standard Development Cycle

```bash
# 1. Generate new contract
make generate CRATE=new-feature

# 2. Implement contract logic
# Edit contracts/new-feature/src/main.rs

# 3. Add unit tests
# Edit contracts/new-feature/src/main.rs

# 4. Build contract
make build-contract CRATE=new-feature

# 5. Run tests
make test-contract CRATE=new-feature

# 6. Integration testing
# Add test to tests/ directory
make test

# 7. Optimize and verify
make build-release
make verify-reproducible
```

### Debugging Workflow

```bash
# Enable debug symbols
make build-debug CRATE=my-contract

# Run with detailed logging
RUST_LOG=debug make test-contract CRATE=my-contract

# Use native simulator for debugging
make generate-native-simulator CRATE=my-contract
make test-native-simulator CRATE=my-contract
```

## Best Practices

### Code Organization

```rust
// contracts/my-token/src/main.rs
mod error;
mod types;
mod validation;
mod operations;

use error::Error;
use types::*;

pub fn main() -> Result<(), Error> {
    let action = validation::parse_args()?;
    
    match action {
        Action::Transfer(transfer) => operations::transfer(transfer),
        Action::Mint(mint) => operations::mint(mint),
        Action::Burn(burn) => operations::burn(burn),
    }
}
```

### Error Handling

```rust
// error.rs
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    IndexOutOfBound = 1,
    ItemMissing,
    LengthNotEnough,
    Encoding,
    InvalidWitness,
    InvalidArgs,
}

impl From<SysError> for Error {
    fn from(err: SysError) -> Self {
        match err {
            SysError::IndexOutOfBound => Error::IndexOutOfBound,
            SysError::ItemMissing => Error::ItemMissing,
            SysError::LengthNotEnough(_, _) => Error::LengthNotEnough,
            SysError::Encoding => Error::Encoding,
        }
    }
}
```

### Documentation

```rust
/// Transfer tokens between addresses
/// 
/// # Arguments
/// * `from` - Source address lock script
/// * `to` - Destination address lock script  
/// * `amount` - Transfer amount in token units
/// 
/// # Returns
/// * `Ok(())` - Transfer successful
/// * `Err(Error)` - Transfer failed with specific error
pub fn transfer(from: Script, to: Script, amount: u128) -> Result<(), Error> {
    // Implementation
}
```

## Deployment

### Local Testing

```bash
# Start local CKB dev chain
ckb run

# Deploy contracts to local chain
make deploy-local
```

### Testnet Deployment

```bash
# Configure testnet endpoint
export CKB_RPC_URL="https://testnet.ckb.dev"

# Deploy to testnet
make deploy-testnet
```

### Mainnet Deployment

```bash
# Verify all tests pass
make test-all

# Verify reproducible build
make verify-reproducible

# Deploy to mainnet (requires manual verification)
make deploy-mainnet
```

## Troubleshooting

### Common Build Issues

1. **RISC-V Target Missing**: `rustup target add riscv64imac-unknown-none-elf`
2. **Clang Not Found**: Install LLVM/Clang 16+
3. **Memory Overflow**: Adjust heap sizes or use stack-reorder template
4. **Cycle Limits**: Optimize contract logic or increase test cycle limits

### Performance Issues

```bash
# Profile contract cycles
make profile-cycles CRATE=my-contract

# Analyze memory usage
make analyze-memory CRATE=my-contract

# Benchmark performance
make benchmark CRATE=my-contract
```

This workspace approach provides a professional foundation for CKB smart contract development with comprehensive tooling, testing, and optimization capabilities.