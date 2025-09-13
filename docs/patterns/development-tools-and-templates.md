## Description

CKB development tools and templates. OffCKB local development environment, project templates for Next.js and React, smart contract integration patterns, Cargo Generate templates, and automated testing pipelines. Build systems, deployment workflows, debugging techniques, and best practices for efficient CKB application development.

Development tools, project templates, and workflow patterns for CKB development, based on OffCKB and ckb-script-templates.

## OffCKB Development Environment

### 1. Quick Start Setup

OffCKB provides a complete development environment with pre-configured tools and test accounts.

```bash
# Install OffCKB
npm install -g @offckb/cli

# Initialize new project
offckb init my-ckb-project
cd my-ckb-project

# Start development environment
offckb node
```

**Key Features:**
- **20 Pre-funded Accounts**: Each with 42M CKB (42,000,000 CKB)
- **Automatic Restart**: Node restarts when blockchain state becomes inconsistent
- **Built-in Indexer**: Fast cell queries and transaction tracking
- **Debug Tools**: Transaction analysis and script debugging

**Reference:** `resources/offckb/README.md`

### 2. CLI Commands and Usage

```bash
# Node management
offckb node                    # Start local development node
offckb node --reset           # Reset blockchain state
offckb node --port 8115       # Custom RPC port

# Account management
offckb account list           # List all pre-funded accounts
offckb account balance <address>  # Check account balance
offckb account private-key <address>  # Get private key

# Transaction operations
offckb transfer <from> <to> <amount>  # Simple CKB transfer
offckb deploy <script.bin>    # Deploy script to blockchain
offckb call <script-hash> <args>      # Call deployed script

# Development utilities
offckb compile <contract.c>   # Compile C contract
offckb test <test-file>       # Run contract tests
offckb molecule <schema.mol>  # Generate molecule bindings
```

**Reference:** `resources/offckb/src/cli.ts`

### 3. Project Templates

OffCKB provides full-stack templates for rapid development.

#### Next.js + CKB Template

```typescript
// pages/api/transfer.ts
import { OffCKBProvider } from '@offckb/next';
import { generateSecp256k1Account } from '@ckb-ccc/core';

export default async function handler(req, res) {
  const provider = new OffCKBProvider();
  
  const { from, to, amount } = req.body;
  
  try {
    // Get account private key
    const privateKey = await provider.getAccountPrivateKey(from);
    const account = generateSecp256k1Account(privateKey);
    
    // Build and send transaction
    const tx = await provider.buildTransfer({
      from: account.address,
      to,
      amount: BigInt(amount),
    });
    
    const txHash = await provider.sendTransaction(tx);
    
    res.status(200).json({ success: true, txHash });
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
}
```

#### React Component Pattern

```typescript
// components/CKBTransfer.tsx
import React, { useState } from 'react';
import { useCKBProvider } from '@offckb/react';

export function CKBTransfer() {
  const { provider, accounts } = useCKBProvider();
  const [loading, setLoading] = useState(false);
  
  const handleTransfer = async (formData) => {
    setLoading(true);
    
    try {
      const tx = await provider.buildTransfer({
        from: formData.from,
        to: formData.to,
        amount: BigInt(formData.amount),
      });
      
      const txHash = await provider.sendTransaction(tx);
      console.log('Transfer successful:', txHash);
      
    } catch (error) {
      console.error('Transfer failed:', error);
    } finally {
      setLoading(false);
    }
  };
  
  return (
    <div className="transfer-form">
      <h2>CKB Transfer</h2>
      {/* Form implementation */}
    </div>
  );
}
```

**Reference:** `resources/offckb/templates/v3/nextjs-template/`

### 4. Smart Contract Integration

OffCKB includes pre-deployed contracts for common patterns.

```typescript
// Contract interaction patterns
import { 
  XUDTContract, 
  OmnilockContract, 
  AnyoneCanPayContract,
  SporeContract 
} from '@offckb/contracts';

export class ContractManager {
  constructor(private provider: OffCKBProvider) {}
  
  // xUDT token operations
  async createToken(name: string, symbol: string, decimals: number) {
    const xudt = new XUDTContract(this.provider);
    
    const tokenInfo = {
      name,
      symbol,
      decimals,
      totalSupply: BigInt(1000000) * BigInt(10 ** decimals),
    };
    
    return await xudt.deployToken(tokenInfo);
  }
  
  async transferToken(tokenTypeScript: Script, from: string, to: string, amount: bigint) {
    const xudt = new XUDTContract(this.provider);
    
    return await xudt.transfer({
      typeScript: tokenTypeScript,
      from,
      to,
      amount,
    });
  }
  
  // Omnilock operations
  async createOmnilockAddress(publicKey: string, lockType: 'secp256k1' | 'ethereum') {
    const omnilock = new OmnilockContract(this.provider);
    
    return await omnilock.generateAddress({
      publicKey,
      lockType,
      enableAnyoneCanPay: false,
    });
  }
  
  // Anyone-Can-Pay operations
  async createFundingPool(targetAmount: bigint, deadline: number) {
    const acp = new AnyoneCanPayContract(this.provider);
    
    return await acp.createPool({
      targetAmount,
      deadline,
      creator: this.provider.getCurrentAccount(),
    });
  }
  
  // Spore NFT operations
  async createNFT(content: Uint8Array, contentType: string) {
    const spore = new SporeContract(this.provider);
    
    return await spore.createSpore({
      content,
      contentType,
      owner: this.provider.getCurrentAccount(),
    });
  }
}
```

**Reference:** `resources/offckb/ckb/` (pre-deployed contracts)

## Script Templates and Patterns

### 1. Cargo Generate Templates

Create new contract projects with standardized structure.

```bash
# Generate new contract project
cargo generate --git https://github.com/cryptape/ckb-script-templates.git

# Available templates:
# - workspace: Multi-contract workspace
# - contract: Single contract
# - atomics-contract: Atomic operations support
# - c-wrapper-crate: C integration
# - stack-reorder-contract: Memory optimization
```

### 2. Standard Contract Structure

```rust
// Cargo.toml
[package]
name = "my-contract"
version = "0.1.0"
edition = "2021"

[dependencies]
ckb-std = "0.15.1"

[profile.release]
overflow-checks = true
opt-level = "s"
lto = true
codegen-units = 1
panic = "abort"

[[bin]]
name = "my-contract"
path = "src/main.rs"

[features]
default = []
```

```rust
// src/main.rs
#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]

use ckb_std::{
    ckb_constants::Source,
    ckb_types::{bytes::Bytes, prelude::*},
    high_level::{load_script, load_script_hash},
    debug, error,
};

use crate::error::Error;

mod error;

pub fn main() -> Result<(), Error> {
    // Load script arguments
    let script = load_script()?;
    let args: Bytes = script.args().unpack();
    
    // Contract-specific logic
    validate_transaction(&args)?;
    
    Ok(())
}

fn validate_transaction(args: &Bytes) -> Result<(), Error> {
    debug!("Validating transaction with args: {:?}", args);
    
    // Implementation here
    
    Ok(())
}

// Standard panic handler
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        error!(
            "panic occurred at {}:{}: {}",
            location.file(),
            location.line(),
            info.message().unwrap_or(&"unknown panic message")
        );
    } else {
        error!("panic occurred: {}", info.message().unwrap_or(&"unknown panic message"));
    }
    loop {}
}

// Memory allocation error handler
#[alloc_error_handler]
fn alloc_error_handler(_: core::alloc::Layout) -> ! {
    error!("out of memory");
    loop {}
}

// Language items
#[lang = "eh_personality"]
extern "C" fn eh_personality() {}
```

**Reference:** `resources/ckb-script-templates/contract/src/main.rs`

### 3. Error Handling Pattern

```rust
// src/error.rs
use ckb_std::error::SysError;

#[repr(i8)]
pub enum Error {
    IndexOutOfBound = 1,
    ItemMissing,
    LengthNotEnough,
    Encoding,
    InvalidArgs = 5,
    InvalidWitness,
    InvalidTransaction,
    // Custom errors
    AmountOverflow = 21,
    InsufficientBalance,
    InvalidRecipient,
}

impl From<SysError> for Error {
    fn from(err: SysError) -> Self {
        match err {
            SysError::IndexOutOfBound => Error::IndexOutOfBound,
            SysError::ItemMissing => Error::ItemMissing,
            SysError::LengthNotEnough(_) => Error::LengthNotEnough,
            SysError::Encoding => Error::Encoding,
            SysError::Unknown(err_code) => panic!("unexpected sys error {}", err_code),
        }
    }
}
```

### 4. Testing Framework

```rust
// tests/src/lib.rs
use super::*;
use ckb_tool::ckb_types::{
    bytes::Bytes,
    core::{TransactionBuilder, TransactionView},
    packed::*,
    prelude::*,
};
use ckb_tool::{ckb_error::assert_error_eq, ckb_script::ScriptError};

const MAX_CYCLES: u64 = 10_000_000;

#[test]
fn test_basic_validation() {
    let mut context = Context::default();
    
    // Deploy contract
    let contract_bin = Loader::default().load_binary("my-contract");
    let contract_out_point = context.deploy_cell(contract_bin);
    let contract_dep = CellDep::new_builder()
        .out_point(contract_out_point)
        .dep_type(DepType::Code.into())
        .build();
    
    // Create test inputs and outputs
    let input = CellInput::new_builder()
        .previous_output(
            context.create_cell(
                CellOutput::new_builder()
                    .capacity(1000u64.pack())
                    .lock(
                        Script::new_builder()
                            .code_hash(Byte32::from_slice(&[0u8; 32]).unwrap())
                            .hash_type(ScriptHashType::Data.into())
                            .build(),
                    )
                    .build(),
                Bytes::new(),
            )
        )
        .build();
    
    let output = CellOutput::new_builder()
        .capacity(999u64.pack())
        .lock(
            Script::new_builder()
                .code_hash(Byte32::from_slice(&[1u8; 32]).unwrap())
                .hash_type(ScriptHashType::Data.into())
                .build(),
        )
        .type_(Some(
            Script::new_builder()
                .code_hash(contract_out_point.tx_hash())
                .hash_type(ScriptHashType::Data.into())
                .args(Bytes::from(vec![42u8]).pack())
                .build(),
        ).pack())
        .build();
    
    // Build transaction
    let tx = TransactionBuilder::default()
        .input(input)
        .output(output)
        .output_data(Bytes::new().pack())
        .cell_dep(contract_dep)
        .build();
    
    // Verify transaction
    let cycles = context.verify_tx(&tx, MAX_CYCLES).expect("pass");
    println!("cycles: {}", cycles);
}

#[test]
fn test_invalid_args() {
    let mut context = Context::default();
    
    // Similar setup but with invalid arguments
    let contract_bin = Loader::default().load_binary("my-contract");
    let contract_out_point = context.deploy_cell(contract_bin);
    
    // ... build transaction with invalid args ...
    
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
    assert_error_eq!(err, ScriptError::ValidationFailure(Error::InvalidArgs as i8));
}
```

**Reference:** `resources/ckb-script-templates/contract/tests/src/lib.rs`

### 5. Advanced Memory Management

For contracts requiring precise memory control:

```rust
// src/memory.rs
use core::alloc::{GlobalAlloc, Layout};

// Custom allocator for deterministic memory usage
struct DeterministicAllocator;

unsafe impl GlobalAlloc for DeterministicAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Custom allocation logic
        ckb_std::syscalls::sys_brk(layout.size()).unwrap_or(core::ptr::null_mut())
    }
    
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Custom deallocation - often no-op in blockchain contracts
    }
}

#[global_allocator]
static ALLOCATOR: DeterministicAllocator = DeterministicAllocator;

// Stack frame reordering for optimization
#[inline(never)]
pub fn optimized_validation(data: &[u8]) -> Result<(), Error> {
    // Force specific stack layout
    let _local_buffer: [u8; 256] = [0; 256];
    
    // Validation logic with controlled memory access patterns
    validate_data_structure(data)?;
    
    Ok(())
}
```

**Reference:** `resources/ckb-script-templates/stack-reorder-contract/`

### 6. C Integration Pattern

For integrating existing C libraries:

```rust
// src/ffi.rs
extern "C" {
    fn c_crypto_function(
        input: *const u8,
        input_len: usize,
        output: *mut u8,
        output_len: *mut usize,
    ) -> i32;
}

pub fn crypto_operation(input: &[u8]) -> Result<Vec<u8>, Error> {
    let mut output = vec![0u8; 64]; // Adjust size as needed
    let mut output_len = output.len();
    
    let result = unsafe {
        c_crypto_function(
            input.as_ptr(),
            input.len(),
            output.as_mut_ptr(),
            &mut output_len,
        )
    };
    
    if result == 0 {
        output.truncate(output_len);
        Ok(output)
    } else {
        Err(Error::InvalidArgs)
    }
}
```

```c
// c/crypto.c
#include "ckb_syscalls.h"

int c_crypto_function(
    const uint8_t* input,
    size_t input_len,
    uint8_t* output,
    size_t* output_len
) {
    // C implementation using CKB syscalls
    if (input_len < 32) {
        return -1; // Invalid input
    }
    
    // Perform cryptographic operation
    // ...
    
    *output_len = 32; // Set actual output length
    return 0; // Success
}
```

**Reference:** `resources/ckb-script-templates/c-wrapper-crate/`

## Build and Deployment Patterns

### 1. Reproducible Build System

```dockerfile
# Dockerfile.build
FROM nervos/ckb-riscv-gnu-toolchain:jammy-20230214

# Set reproducible build environment
ENV SOURCE_DATE_EPOCH=1234567890
ENV BUILD_DATE=2024-01-01
ENV TZ=UTC
ENV LC_ALL=C

WORKDIR /contract

# Copy source code
COPY . .

# Build with deterministic settings
RUN make clean && \
    make all \
    CC_VERSION_CHECK=false \
    ENABLE_DETERMINISTIC_BUILD=1

# Verify build reproducibility
RUN sha256sum build/*.bin > checksums.txt
```

```makefile
# Makefile
RISCV_GNU_TOOLCHAIN_VERSION := 20230214
TARGET := riscv64imac-unknown-none-elf
CC := $(TARGET)-gcc
AR := $(TARGET)-ar
OBJCOPY := $(TARGET)-objcopy
OBJDUMP := $(TARGET)-objdump

# Deterministic build flags
CFLAGS := -Os -DCKB_DECLARATION_ONLY -I deps/ckb-c-stdlib
CFLAGS += -Wall -Werror -Wno-nonnull -Wno-nonnull-compare -Wno-unused-function
CFLAGS += -fdata-sections -ffunction-sections
LDFLAGS := -Wl,--gc-sections -Wl,-static -fdata-sections -ffunction-sections

ifdef ENABLE_DETERMINISTIC_BUILD
CFLAGS += -frandom-seed=0 -fno-stack-protector
LDFLAGS += -Wl,--build-id=none
endif

# Build targets
all: build/contract.bin

build/contract.bin: c/contract.c
	@mkdir -p build
	$(CC) $(CFLAGS) $(LDFLAGS) -o $@ $<
	$(OBJCOPY) --only-keep-debug $@ $@.debug
	$(OBJCOPY) --strip-debug --strip-unneeded $@

# Clean build artifacts
clean:
	rm -rf build/

# Install dependencies
install-deps:
	git submodule update --init --recursive

.PHONY: all clean install-deps
```

### 2. Automated Testing Pipeline

```yaml
# .github/workflows/test.yml
name: Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
      with:
        submodules: recursive
        
    - name: Install RISC-V toolchain
      run: |
        wget https://github.com/nervosnetwork/ckb-riscv-gnu-toolchain/releases/download/jammy-20230214/riscv64-linux-jammy-20230214.tar.gz
        tar -xzf riscv64-linux-jammy-20230214.tar.gz
        echo "$(pwd)/riscv64-linux-jammy-20230214/bin" >> $GITHUB_PATH
        
    - name: Build contracts
      run: make all
      
    - name: Run tests
      run: cargo test --all
      
    - name: Check reproducible build
      run: |
        make clean && make all
        sha256sum build/*.bin > checksums1.txt
        make clean && make all  
        sha256sum build/*.bin > checksums2.txt
        diff checksums1.txt checksums2.txt
```

### 3. Development Workflow

```bash
#!/bin/bash
# scripts/dev-workflow.sh

# Development workflow script

set -e

echo "🚀 Starting CKB development workflow..."

# 1. Start OffCKB node
echo "📡 Starting OffCKB node..."
offckb node --reset &
NODE_PID=$!

# Wait for node to be ready
sleep 5

# 2. Build contracts
echo "🔨 Building contracts..."
make clean && make all

# 3. Deploy contracts
echo "📤 Deploying contracts..."
CONTRACT_HASH=$(offckb deploy build/contract.bin)
echo "Contract deployed with hash: $CONTRACT_HASH"

# 4. Run tests
echo "🧪 Running tests..."
cargo test --all

# 5. Integration tests
echo "🔗 Running integration tests..."
npm run test:integration

# 6. Generate documentation
echo "📚 Generating documentation..."
cargo doc --no-deps
mdbook build docs/

echo "✅ Development workflow completed successfully!"

# Cleanup
trap "kill $NODE_PID 2>/dev/null || true" EXIT
```

## Best Practices

### 1. Project Organization

```
my-ckb-project/
├── contracts/
│   ├── my-contract/
│   │   ├── src/
│   │   ├── tests/
│   │   └── Cargo.toml
│   └── shared-types/
├── frontend/
│   ├── components/
│   ├── pages/
│   └── utils/
├── scripts/
│   ├── deploy.sh
│   ├── test.sh
│   └── dev-workflow.sh
├── docs/
│   ├── contracts.md
│   └── api.md
└── docker/
    ├── Dockerfile.build
    └── docker-compose.yml
```

### 2. Configuration Management

```typescript
// config/development.ts
export const developmentConfig = {
  ckb: {
    nodeUrl: 'http://localhost:8114',
    indexerUrl: 'http://localhost:8116',
    networkType: 'dev',
  },
  contracts: {
    xudt: {
      codeHash: '0x...',
      hashType: 'type',
    },
    omnilock: {
      codeHash: '0x...',
      hashType: 'type',
    },
  },
  accounts: {
    deployer: process.env.DEPLOYER_PRIVATE_KEY,
    tester: process.env.TESTER_PRIVATE_KEY,
  },
};
```

### 3. Error Recovery and Debugging

```typescript
// utils/debug.ts
export class CKBDebugger {
  constructor(private provider: OffCKBProvider) {}
  
  async debugTransaction(txHash: string) {
    const tx = await this.provider.getTransaction(txHash);
    
    console.log('Transaction Details:');
    console.log('- Hash:', txHash);
    console.log('- Status:', tx.txStatus.status);
    console.log('- Block:', tx.txStatus.blockHash);
    
    if (tx.txStatus.status === 'rejected') {
      console.log('- Rejection Reason:', tx.txStatus.reason);
      await this.analyzeRejectionReason(tx.transaction);
    }
  }
  
  private async analyzeRejectionReason(tx: Transaction) {
    // Script execution analysis
    for (let i = 0; i < tx.inputs.length; i++) {
      try {
        const result = await this.provider.dryRunTransaction(tx, i);
        console.log(`Input ${i} cycles:`, result.cycles);
      } catch (error) {
        console.log(`Input ${i} failed:`, error.message);
      }
    }
  }
  
  async validateCell(outPoint: OutPoint) {
    const cell = await this.provider.getLiveCell(outPoint);
    
    if (!cell) {
      console.log('❌ Cell not found or already spent');
      return;
    }
    
    console.log('✅ Cell validation:');
    console.log('- Capacity:', cell.cellOutput.capacity);
    console.log('- Lock Script:', cell.cellOutput.lock);
    console.log('- Type Script:', cell.cellOutput.type);
    console.log('- Data:', cell.data);
  }
}
```

These development tools and templates provide a comprehensive foundation for building, testing, and deploying CKB applications efficiently and reliably.