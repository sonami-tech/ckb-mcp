## Description

CKB development tool selection, SDK recommendations, and modern workflows. Covers CCC as the primary SDK for frontend/backend development, ckb-script-templates for smart contracts (replacing deprecated Capsule), OffCKB for rapid prototyping with local devnet, CKB-SDK-Rust for backend services, and protocol-specific tools (Omnilock, Spore, CoTA, RGB++). Includes migration guides from Lumos to CCC, tool selection guidelines, testing frameworks, security best practices, and performance optimization.

## Tool Selection Guidelines

### Use CCC When:
- **All new projects** (strongly recommended by Nervos)
- Building dApps, wallets, or production applications
- Need modern wallet integration (MetaMask, Unisat, OKX, JoyID)
- Want simplified transaction construction
- Require multi-chain support
- Following official Nervos examples and tutorials

### Use CKB-SDK-Rust When:
- Building backend services in Rust
- Need direct RPC client and HD wallet management
- Server-side cell collection and transaction building

### Use ckb-script-templates When:
- Creating new smart contracts
- Need modern Rust tooling with cargo workspaces
- Building production scripts

### Use Lumos When:
- **Legacy maintenance only**: Working with existing Lumos codebases
- Gradual migration from legacy code (migrate to CCC when possible)
- **Not recommended for any new development**

### Avoid:
- **Capsule** (deprecated, use ckb-script-templates)
- **Old manual toolchains** (use modern Rust)

## Essential Development Tools

### 1. OffCKB Development Environment

**Key Features:**
- **Local Devnet**: Instant blockchain with 20 pre-funded test accounts
- **Project Templates**: Next.js, React, Remix integration with CKB
- **Built-in Scripts**: xUDT, Omnilock, AnyoneCanPay, Spore contracts
- **Hot Reload**: Automatic contract redeployment on changes
- **Modern Workflow**: Integration with ckb-script-templates and CCC SDK

**Installation & Usage:**
```bash
# Install OffCKB
npm install -g @offckb/cli

# Create new full-stack project
offckb create my-dapp-project
# Select Next.js or Remix template when prompted

# Or create script-only project
offckb create --script my-script-project

# Start local development blockchain
offckb node

# Deploy contracts to devnet/testnet
offckb deploy --network devnet
```

**Project Structure:**
```
my-dapp-project/
├── contracts/           # Smart contracts (ckb-script-templates)
│   └── hello-world/
│       └── src/main.rs
├── frontend/           # Next.js/React frontend
│   ├── app/
│   └── offckb.config.ts
└── Makefile           # Build automation
```

**Reference:** Official Nervos Quick Start Guide

### 2. ckb-script-templates

**Key Features:**
- **Production-ready Templates**: Workspace, basic contract, atomics support
- **Modern Rust Tooling**: Cargo workspaces and advanced patterns
- **Integration**: Works seamlessly with OffCKB environment
- **Performance Optimized**: Efficient RISC-V compilation

**Usage:**
```bash
# Generate new contract from template
cargo generate --git https://github.com/cryptape/ckb-script-templates.git

# Available templates:
# 1. workspace - Multi-contract workspace (recommended)
# 2. contract - Single Rust contract
# 3. atomics-contract - Atomic operations support
# 4. c-wrapper-crate - C integration patterns

# Within OffCKB project:
make generate  # Add new contract
make build     # Build contracts
make test      # Run tests
```

**Reference:** `resources/ckb-script-templates/`

### 3. CKB CLI

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

### 4. CKB Debugger

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

### 1. CCC (Common Chain Connector) - Primary SDK

**Key Features:**
- **Universal Platform**: Works in Node.js (backend) and browsers (frontend)
- **Multi-wallet Support**: MetaMask, Unisat, OKX, JoyID integration
- **React Integration**: Hooks and components for React applications
- **Type Safety**: Full TypeScript support with CKB types
- **Cross-chain**: Bitcoin/Ethereum wallet compatibility via Omnilock
- **Transaction Building**: Complete transaction construction and management

**For Backend Development (Node.js):**
```typescript
import { ccc } from "@ckb-ccc/core";

// Initialize client for backend services
const client = new ccc.ClientPublicTestnet();
const signer = new ccc.SignerCkbPrivateKey(client, privateKey);

// Build and send transaction
const tx = ccc.Transaction.from({
    outputs: [{ lock: toLock, capacity: ccc.fixedPointFrom(amount) }],
});

await tx.completeInputsByCapacity(signer);
await tx.completeFeeBy(signer);
const txHash = await signer.sendTransaction(tx);
```

**For Frontend Development (React):**

```bash
npm install @ckb-ccc/core @ckb-ccc/connector-react
```

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

### 2. CKB-SDK-Rust (Backend/Server Development)

**Key Features:**
- **Transaction Building**: High-level transaction construction for applications
- **Cell Collection**: Automated UTXO management for backends
- **RPC Client**: Comprehensive CKB node interaction
- **Wallet Integration**: HD wallet and key management for services
- **Network Abstraction**: Mainnet/testnet/devnet support

**Installation:**
```toml
[dependencies]
ckb-sdk = "3.2.0"
ckb-types = "0.118"
```

**Basic Usage:**
```rust
use ckb_sdk::rpc::CkbRpcClient;
use ckb_sdk::traits::{DefaultCellCollector, SecpCkbRawKeySigner};

// Initialize client for backend service
let mut ckb_client = CkbRpcClient::new("https://testnet.ckb.dev/rpc");

// Build transfer transaction
let tx = builder
    .build_unlocked(
        &mut cell_collector,
        &cell_dep_resolver,
        &header_dep_resolver,
        &tx_dep_provider,
        &balancer,
        &unlockers,
    )
    .unwrap();
```

**Note:** For smart contract development, use ckb-script-templates with ckb-std, not CKB-SDK-Rust.

**Reference:** `resources/ckb-sdk-rust/`

### 3. Lumos Framework (Deprecated)

**Status:** No longer actively recommended for new projects - migrate to CCC.

**Migration Path:**
```typescript
// Old Lumos approach
import { TransactionSkeleton } from "@ckb-lumos/helpers";

// New CCC approach (recommended)
import { ccc } from "@ckb-ccc/ccc";
const tx = ccc.Transaction.from({
  outputs: [{ lock: toLock, capacity: amount }],
});
```

**Migration Benefits:**
- Simpler API surface
- Better TypeScript support
- Automatic capacity/fee management
- Modern wallet integrations
- Unified development experience

### 4. Alternative Language Support

**For Non-Rust/TypeScript Backends:**
- **Direct RPC**: Any language with HTTP client can call CKB JSON-RPC API
- **Python, Go, Java**: Use direct RPC calls for blockchain interaction
- **Community SDKs**: Check ecosystem for language-specific implementations

## Modern Development Workflow

### Recommended Development Path

1. **Initialize**: Use OffCKB to create full-stack project
2. **Smart Contracts**: Develop with ckb-script-templates (Rust)
3. **Frontend/Backend**: Build with CCC SDK (TypeScript/JavaScript)
4. **Testing**: Use built-in testing frameworks and local devnet
5. **Deployment**: Deploy to testnet, then mainnet

### Full-Stack Development with OffCKB
```bash
# Create full-stack dApp (recommended)
offckb create my-dapp-project
# Choose Next.js or Remix template

# Or create script-only project
offckb create --script my-script-project

# Start local blockchain with pre-funded accounts
offckb node

# In separate terminal, start frontend
cd frontend
npm i && npm run dev

# Contract development commands
make generate  # Add new contract
make build     # Build contracts
make test      # Run tests

# Deploy to networks
offckb deploy --network devnet
offckb deploy --network testnet
```

**Integration Benefits:**
- **Pre-configured**: CCC SDK + ckb-script-templates integration
- **Live Reload**: Contract changes automatically deployed
- **Built-in Scripts**: Omnilock, Spore, xUDT contracts included
- **Testing Environment**: 20 pre-funded accounts for testing

## Protocol-Specific Tools and SDKs

### 1. Omnilock Development

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

**Recommended for:**
- **Art NFTs**: High-value collectibles where data permanence is critical
- **Premium Digital Assets**: Content requiring 100% on-chain storage
- **Digital Collectibles**: Items where metadata integrity is essential

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

**Recommended for:**
- **Gaming Assets**: In-game items and collectibles requiring low cost
- **Membership Tokens**: Access tokens and utility NFTs
- **High-Volume Applications**: Projects minting thousands of NFTs
- **Cost-Sensitive Projects**: Where transaction costs are a primary concern

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

### 4. Deprecated NFT Standards

**mNFT (Multi-purpose NFT)**
- **Status**: No longer in development, not recommended for new projects
- **Alternative**: Choose between Spore Protocol (high-value, fully on-chain) or CoTA Protocol (cost-effective, gaming)
- **Migration**: Existing mNFT projects should consider migrating to supported protocols

### 5. RGB++ Asset Protocol

**Features:**
- **Bitcoin Binding**: Assets backed by Bitcoin UTXOs
- **Cross-chain Transfers**: Bitcoin <-> CKB asset movement
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

## Development Best Practices

### Language Selection

**Smart Contracts** (ckb-script-templates + ckb-std):
- **Rust**: Recommended for all new smart contracts
- **C**: System-level development, maximum performance
- **Assembly**: Low-level optimizations when needed

**Application Development**:
- **TypeScript/JavaScript**: CCC SDK for frontend and backend
- **Rust**: CKB-SDK-Rust for backend services
- **Other Languages**: Direct RPC for Python, Go, Java, etc.

### Security Considerations

- **Smart Contracts**: Always validate script arguments and witness data
- **Integer Overflow**: Use checked arithmetic operations in Rust
- **Cycle Efficiency**: Optimize contracts for gas limits
- **Transaction Atomicity**: Consider multi-step operation safety
- **Key Management**: Secure private key handling in applications
- **Wallet Integration**: Use CCC's secure wallet connection patterns

### Performance Optimization

- **Contract Development**: Minimize syscalls and optimize RISC-V compilation
- **Cell Collection**: Efficient UTXO selection with CCC/CKB-SDK-Rust
- **Batch Operations**: Group multiple operations when possible
- **Frontend**: Use CCC's optimized transaction building
- **Caching**: Cache frequently accessed blockchain data
- **Local Development**: Use OffCKB for fast iteration cycles

## Community Resources

- **Developer Documentation**: https://docs.nervos.org
- **CCC Documentation**: https://docs.ckbccc.com/
- **CCC Playground**: https://live.ckbccc.com/
- **ckb-script-templates**: https://github.com/cryptape/ckb-script-templates
- **GitHub Organization**: https://github.com/nervosnetwork
- **Developer Discord**: https://discord.gg/nervos
- **CKB Academy**: https://academy.nervos.org
- **Developer Training**: `resources/developer-training-course-documentation/`
- **CKB Developer Resources**: `resources/CKB-Developer-Resource/`
