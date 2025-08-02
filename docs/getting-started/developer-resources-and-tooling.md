# CKB Developer Resources and Tooling

## Description

Discover essential CKB development tools, frameworks, and SDKs to build blockchain applications efficiently. This comprehensive guide covers Capsule for smart contracts, CKB-SDK-Rust for blockchain interactions, CCC for modern TypeScript development, OffCKB for rapid prototyping, and protocol-specific tools like Omnilock, Spore, and CoTA. Learn modern development workflows, testing frameworks, security best practices, and performance optimization techniques for professional CKB development.

This comprehensive guide covers all available CKB development resources, tools, and frameworks. **Modern CKB development emphasizes Rust** for smart contracts while supporting multiple languages for different use cases.

## Essential Development Tools

### 1. Capsule Framework (Primary Development Tool)

**Purpose:** Official CKB smart contract development framework with **Rust-first approach**.

**Key Features:**
- **Multi-language Support**: Rust (primary), C, AssemblyScript, C++, Lua
- **Project Templates**: Pre-configured project structures
- **Testing Framework**: Comprehensive contract testing
- **Deployment Tools**: Mainnet/testnet deployment automation
- **Debugging Support**: Transaction simulation and debugging

**Installation:**
```bash
# Install Capsule
cargo install ckb-capsule

# Create new Rust project
capsule new my-contract --template=rust
cd my-contract

# Build contract
capsule build

# Run tests
capsule test

# Deploy to testnet
capsule deploy --address <your-address> --fee 0.1
```

**Project Structure:**
```
my-contract/
├── contracts/
│   └── my-contract/
│       ├── src/
│       │   └── main.rs          # Main contract logic (Rust)
│       └── Cargo.toml
├── tests/
│   └── src/
│       └── tests.rs             # Integration tests
├── deployment.toml              # Deployment configuration
└── capsule.toml                # Project configuration
```

**Reference:** `resources/CKB-Developer-Resource/README.md`

### 2. CKB CLI

**Purpose:** Command-line interface for CKB blockchain interactions.

**Key Operations:**
```bash
# Account management
ckb-cli account new
ckb-cli account list
ckb-cli account import --privkey-path <key-file>

# Wallet operations
ckb-cli wallet transfer --to-address <address> --capacity 100
ckb-cli wallet get-capacity --address <address>

# DAO operations
ckb-cli dao deposit --capacity 1000 --from-account <account>
ckb-cli dao prepare --out-point <tx-hash:index>
ckb-cli dao withdraw --out-point <tx-hash:index>

# Transaction debugging
ckb-cli tx send --tx-file transaction.json
ckb-cli util key-info --privkey-path <key-file>
```

### 3. CKB Debugger

**Purpose:** Advanced debugging for CKB scripts and transactions.

**Features:**
- **Script Execution Tracing**: Step-by-step execution analysis
- **Cycle Counting**: Gas usage optimization
- **Memory Inspection**: Debug memory access patterns
- **Syscall Monitoring**: Track system call usage

**Usage:**
```bash
# Debug transaction
ckb-debugger --tx-file <transaction.json> --script-group-type lock

# Profile script execution
ckb-debugger --tx-file <transaction.json> --script-group-type type --mode profile

# Trace execution with GDB
ckb-debugger --tx-file <transaction.json> --mode gdb
```

## Software Development Kits (SDKs)

### 1. CKB-SDK-Rust (Primary Rust SDK)

**Purpose:** **Primary SDK for Rust developers** - comprehensive blockchain interaction library.

**Key Features:**
- **Transaction Building**: High-level transaction construction
- **Cell Collection**: Automated UTXO management
- **Script Integration**: Support for all major script types
- **Wallet Integration**: HD wallet and key management
- **Network Abstraction**: Mainnet/testnet/devnet support

**Installation:**
```toml
[dependencies]
ckb-sdk = "3.0"
ckb-types = "0.118"
```

**Basic Usage:**
```rust
use ckb_sdk::{
    CkbRpcClient, HttpRpcClient,
    traits::{DefaultCellCollector, DefaultTransactionDependencyProvider},
    tx_builder::CapacityBalancer,
    unlock::SecpCkbRawKeySigner,
    Address, NetworkType,
};

// Initialize SDK
let mut sdk = CkbSdkManager::new("http://localhost:8114", NetworkType::Dev);

// Build and send transaction
let tx = sdk.build_transfer_transaction(
    &from_address,
    &to_address, 
    1000000000, // 10 CKB
    1000,       // fee rate
    private_key,
).await?;

let tx_hash = sdk.send_transaction(&tx).await?;
```

**Reference:** `resources/ckb-sdk-rust/`

### 2. CCC (Common Chain Connector) - TypeScript/JavaScript

**Purpose:** **Modern JavaScript/TypeScript SDK** for frontend applications.

**Key Features:**
- **Multi-wallet Support**: MetaMask, Unisat, OKX, JoyID integration
- **React Integration**: Hooks and components for React applications
- **Type Safety**: Full TypeScript support with CKB types
- **Cross-chain**: Bitcoin/Ethereum wallet compatibility via Omnilock

**Installation:**
```bash
npm install @ckb-ccc/core @ckb-ccc/connector-react
```

**React Integration:**
```typescript
import { ccc } from "@ckb-ccc/connector-react";

// Wallet connection component
function WalletConnector() {
    const { connect, disconnect, wallet, signer } = ccc.useCcc();
    
    return (
        <div>
            {wallet ? (
                <button onClick={disconnect}>Disconnect {wallet.name}</button>
            ) : (
                <button onClick={() => connect(ccc.WalletType.MetaMask)}>
                    Connect MetaMask
                </button>
            )}
        </div>
    );
}

// Transaction component
function TransferComponent() {
    const { signer } = ccc.useCcc();
    
    const sendTransaction = async () => {
        if (!signer) return;
        
        const tx = ccc.Transaction.from({
            inputs: [],
            outputs: [
                {
                    capacity: ccc.fixedPointFrom(100), // 100 CKB
                    lock: await signer.getRecommendedAddressObj(),
                }
            ],
            outputsData: ["0x"],
        });
        
        await tx.completeInputsByCapacity(signer);
        await tx.completeFeeBy(signer, 1000n);
        
        const txHash = await signer.sendTransaction(tx);
        console.log("Transaction sent:", txHash);
    };
    
    return (
        <button onClick={sendTransaction}>Send 100 CKB</button>
    );
}
```

**Reference:** `resources/nervdao/` (production example)

### 3. Lumos Framework (Legacy - Maintenance Mode)

**Status:** Maintenance mode - **use CCC for new projects**.

**Purpose:** Comprehensive TypeScript framework for CKB development.

**Note:** While still functional, new projects should use CCC for better wallet integration and modern TypeScript patterns.

**Reference:** `resources/lumos/`

## Development Frameworks and Templates

### 1. OffCKB Development Environment

**Purpose:** **Rapid prototyping and development** with pre-configured environment.

**Features:**
- **Local Devnet**: Instant blockchain with 20 pre-funded accounts
- **Project Templates**: Next.js, React, Remix integration
- **Built-in Scripts**: xUDT, Omnilock, AnyoneCanPay, Spore
- **Hot Reload**: Automatic contract redeployment

**Setup:**
```bash
# Install OffCKB
npm install -g @offckb/cli

# Create new project
offckb init my-dapp --template nextjs
cd my-dapp

# Start development environment
offckb node &  # Start local blockchain
npm run dev    # Start frontend
```

**Reference:** `resources/offckb/`

### 2. CKB Script Templates

**Purpose:** Production-ready **Rust contract templates** with advanced features.

**Available Templates:**
- **Basic Contract**: Standard RISC-V contract structure
- **Workspace**: Multi-contract development environment
- **Atomics Contract**: Atomic operations without hardware support
- **C Wrapper**: FFI integration patterns
- **Stack Reorder**: Custom memory management

**Usage:**
```bash
# Generate new contract from template
cargo generate --git https://github.com/cryptape/ckb-script-templates.git

# Select template:
# 1. workspace - Multi-contract workspace
# 2. contract - Single Rust contract
# 3. atomics-contract - Atomic operations
# 4. c-wrapper-crate - C integration
```

**Reference:** `resources/ckb-script-templates/`

## Protocol-Specific Tools and SDKs

### 1. Omnilock Development

**Purpose:** Universal lock script supporting multiple signature algorithms.

**Supported Signatures:**
- **CKB Native**: secp256k1 with Blake2b
- **Ethereum**: ECDSA with Keccak256 
- **Bitcoin**: ECDSA with Bitcoin message format
- **WebAuthn**: Passkey authentication
- **Custom**: Extensible authentication methods

**Integration Example:**
```rust
use omnilock_sdk::{OmnilockConfig, OmnilockBuilder};

// Ethereum wallet integration
let omnilock_config = OmnilockConfig::new_ethereum(ethereum_address);
let lock_script = omnilock_config.build_script();

// Bitcoin wallet integration
let omnilock_config = OmnilockConfig::new_bitcoin(bitcoin_pubkey);
let lock_script = omnilock_config.build_script();
```

**Reference:** `resources/omnilock/`

### 2. Spore Protocol SDK

**Purpose:** Digital object protocol for NFTs and digital assets.

**Features:**
- **Content Storage**: On-chain content with content-type specification
- **Transfer Mechanics**: Secure ownership transfer patterns
- **Batch Operations**: Efficient multi-NFT operations
- **Metadata Support**: Rich metadata and attributes

**Usage:**
```typescript
import { SporeSDK } from "@spore-protocol/core";

const spore = new SporeSDK({
    networkType: "testnet",
    nodeUrl: "https://testnet.ckb.dev",
});

// Create new Spore NFT
const sporeData = {
    content: new Uint8Array([/* image data */]),
    contentType: "image/png",
    clusterId: undefined, // Optional cluster grouping
};

const tx = await spore.createSpore(sporeData, signer);
const txHash = await signer.sendTransaction(tx);
```

**Reference:** `resources/spore-sdk/`

### 3. CoTA NFT Development

**Purpose:** Compact Token Aggregator for efficient NFT management.

**Features:**
- **Aggregation**: Batch multiple NFTs into single cells
- **Registry System**: Decentralized NFT metadata registry
- **Efficient Transfers**: Reduced transaction costs
- **Metadata Indexing**: Off-chain indexing integration

**SDK Usage:**
```typescript
import { CotaSDK } from "@nervina-labs/cota-sdk";

const cota = new CotaSDK({
    ckbNodeUrl: "https://testnet.ckb.dev",
    cotaUrl: "https://cota.nervina.dev/testnet",
});

// Mint CoTA NFT
const mintTx = await cota.mint({
    cotaType: "0x...", // CoTA type script
    tokenIndex: "0x00",
    characteristic: "0x...", // NFT characteristics
    configure: "0x01", // Configuration flags
});
```

**Reference:** `resources/cota-sdk-js/`

### 4. RGB++ Asset Protocol

**Purpose:** Bitcoin-compatible asset issuance on CKB.

**Features:**
- **Bitcoin Binding**: Assets backed by Bitcoin UTXOs
- **Cross-chain Transfers**: Bitcoin ↔ CKB asset movement
- **Programmable Assets**: Smart contract integration
- **Trustless Bridge**: No centralized custody

**Development Status:** Emerging protocol - check latest documentation

## Testing and Debugging Tools

### 1. CKB Testnet Integration

**Testnet Endpoints:**
- **RPC**: https://testnet.ckb.dev
- **Indexer**: https://testnet.ckb.dev/indexer  
- **Explorer**: https://pudge.explorer.nervos.org

**Faucet Access:**
- **Official Faucet**: https://faucet.nervos.org
- **OffCKB Faucet**: Built-in with 20 pre-funded accounts

### 2. Transaction Simulation

```rust
// Simulate transaction before sending
use ckb_sdk::traits::TransactionSimulator;

let simulation_result = sdk.simulate_transaction(&tx).await?;

println!("Estimated cycles: {}", simulation_result.cycles);
println!("Fee required: {} CKB", simulation_result.fee);

// Check for potential errors
if let Some(error) = simulation_result.error {
    eprintln!("Transaction would fail: {}", error);
}
```

### 3. Contract Testing Framework

```rust
#[cfg(test)]
mod integration_tests {
    use ckb_tool::ckb_types::{
        core::TransactionBuilder,
        packed::*,
        prelude::*,
    };
    
    #[test]
    fn test_contract_integration() {
        let mut context = Context::default();
        
        // Deploy contract
        let contract_bin = std::fs::read("target/riscv64imac-unknown-none-elf/release/my-contract")?;
        let contract_out_point = context.deploy_cell(contract_bin.into());
        
        // Test transaction
        let tx = build_test_transaction(&context, contract_out_point);
        let cycles = context.verify_tx(&tx, 10_000_000)?;
        
        assert!(cycles < 1_000_000, "Contract should be gas-efficient");
    }
}
```

## Community Resources and Support

### 1. Official Channels

- **Developer Documentation**: https://docs.nervos.org
- **GitHub Organization**: https://github.com/nervosnetwork
- **Developer Discord**: https://discord.gg/nervos
- **Forum**: https://talk.nervos.org

### 2. Educational Resources

- **CKB Academy**: https://academy.nervos.org
- **Developer Training**: Comprehensive course materials in `resources/developer-training-course-documentation/`
- **Interactive Examples**: Hands-on tutorials with working code
- **Video Tutorials**: Step-by-step development guides

### 3. Grant and Support Programs

- **Nervos Grants**: Funding for ecosystem projects
- **Hackathons**: Regular development competitions  
- **Mentorship**: Direct support from core developers
- **Incubation**: Acceleration programs for startups

**Reference:** `resources/CKB-Developer-Resource/`

## Development Best Practices

### 1. **Language Selection Guidelines**

**Rust** (Recommended for smart contracts):
- Type safety and memory safety
- Excellent tooling and package ecosystem
- ckb-std crate provides CKB-specific functionality
- Growing ecosystem of Rust-based contracts

**C** (System-level development):
- Maximum performance for critical paths
- Used in system scripts for deterministic execution
- Lower-level control when needed

**TypeScript/JavaScript** (Frontend and application layer):
- Rich wallet integration ecosystem
- React/Next.js development patterns
- Modern web development practices

### 2. **Development Workflow**

1. **Planning**: Use CKB's UTXO model considerations in design
2. **Prototyping**: Start with OffCKB for rapid iteration
3. **Development**: Use Capsule with Rust templates
4. **Testing**: Comprehensive unit and integration tests
5. **Deployment**: Testnet validation before mainnet
6. **Monitoring**: Transaction tracking and error handling

### 3. **Security Considerations**

- **Input Validation**: Always validate script arguments and witness data
- **Integer Overflow**: Use checked arithmetic operations
- **Gas Limits**: Optimize for cycle efficiency
- **Reentrancy**: Consider transaction atomicity
- **Key Management**: Secure private key handling

### 4. **Performance Optimization**

- **Cell Collection**: Efficient UTXO selection algorithms
- **Batch Operations**: Group multiple operations when possible
- **Caching**: Cache frequently accessed data
- **Minimal Syscalls**: Reduce blockchain interaction overhead

This comprehensive guide provides everything needed to start CKB development with modern tools and best practices, emphasizing Rust development while supporting the full ecosystem of available tools and frameworks.