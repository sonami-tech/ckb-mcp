# Deploying Binaries to CKB Blockchain

## Description

Complete guide for deploying compiled smart contract binaries to CKB blockchain using multiple deployment methods. Covers OffCKB deployment workflow, manual deployment with CKB-CLI, transaction construction for code cells, and script referencing patterns. Includes practical examples of binary storage, capacity calculations, deployment verification, and integration with development tools. Essential resource for developers moving from development to production deployment of CKB smart contracts.

## Overview

Deploying binaries to CKB involves storing compiled smart contract code in blockchain cells, where it can be referenced and executed by future transactions. Unlike account-based blockchains, CKB uses a cell-based model where contract code lives as data in cells.

## Deployment Process Overview

```
Source Code → RISC-V Binary → Cell Data → Script Reference → Execution
```

The complete deployment workflow consists of:

1. **Compilation**: Build source code to RISC-V executable binary
2. **Deployment**: Store binary in a cell's data field on-chain
3. **Referencing**: Create script structures that reference the deployed code
4. **Execution**: Use the script in transactions for validation

## Method 1: Using OffCKB (Recommended)

OffCKB provides the most streamlined deployment experience for modern CKB development.

### Setup OffCKB

```bash
# Install OffCKB CLI globally
npm install -g @offckb/cli

# Start local devnet
offckb node

# Check available commands
offckb deploy --help
```

### Build and Deploy

```bash
# 1. Build your contract (example with Rust)
cargo build --target=riscv64imac-unknown-none-elf --release

# 2. Deploy to devnet
offckb deploy --network devnet --target ./build/release

# 3. Deploy to testnet (requires funding)
offckb deploy --network testnet --target ./build/release

# 4. Check deployed contracts
offckb my-scripts --network devnet
```

### OffCKB Deployment Output

```bash
contract MY-SCRIPT deployed, tx hash: 0x9f55da2b555cdc4412945ff8827b7e77508c84f0...
wait 4 blocks..
contract MY-SCRIPT deployed successfully!
  - tx hash: 0x9f55da2b555cdc4412945ff8827b7e77508c84f0...
  - code hash: 0x8209891745eb858abd6f5e53c99b4f101bca221bd150a2ece58a389b7b4f8fa7
  - tx index: 0x0
```

## Method 2: Manual Deployment

For advanced users or when integrating with custom workflows, manual deployment provides full control.

### Step 1: Compile Binary

```bash
# Rust example
cargo build --target=riscv64imac-unknown-none-elf --release

# C example with RISC-V toolchain
riscv64-unknown-elf-gcc -Os -DCKB_NO_MMU -D__riscv_soft_float \
    -D__riscv_float_abi_soft -o contract.bin contract.c \
    -Wl,-static -fdata-sections -ffunction-sections -Wl,--gc-sections -Wl,-s
```

### Step 2: Calculate Deployment Costs

```javascript
const fs = require('fs');
const utils = require('@nervosnetwork/ckb-sdk-utils');

function calculateDeploymentCost(binaryPath) {
    const binary = fs.readFileSync(binaryPath);
    const binarySize = binary.length;
    
    // CKB capacity calculation: 1 CKB = 1 byte storage
    const cellOverhead = 61; // Minimum cell size in CKBytes
    const requiredCapacity = BigInt(binarySize + cellOverhead) * 100000000n; // Convert to Shannon
    
    console.log(`Binary size: ${binarySize} bytes`);
    console.log(`Required capacity: ${requiredCapacity / 100000000n} CKB`);
    
    return requiredCapacity;
}
```

### Step 3: Deploy Binary

```javascript
const CKB = require('@nervosnetwork/ckb-sdk-core').default;
const utils = require('@nervosnetwork/ckb-sdk-utils');
const fs = require('fs');

async function deployBinary(binaryPath, privateKey, nodeUrl) {
    const ckb = new CKB(nodeUrl);
    const binary = fs.readFileSync(binaryPath);
    
    // Get deployer information
    const publicKey = ckb.utils.privateKeyToPublicKey(privateKey);
    const publicKeyHash = `0x${ckb.utils.blake160(publicKey, 'hex')}`;
    
    // Load secp256k1 dependency
    const secp256k1Dep = await ckb.loadSecp256k1Dep();
    
    const lockScript = {
        hashType: secp256k1Dep.hashType,
        codeHash: secp256k1Dep.codeHash,
        args: publicKeyHash
    };
    
    // Get unspent cells for funding
    const lockHash = ckb.utils.scriptToHash(lockScript);
    const unspentCells = await ckb.loadCells({ lockHash });
    const totalCapacity = unspentCells.reduce((sum, cell) => sum + BigInt(cell.capacity), 0n);
    
    // Calculate required capacity
    const binaryCapacity = BigInt(binary.length) * 100000000n + 6100000000n; // Binary + cell overhead
    const fee = 100000000n; // 1 CKB fee
    
    if (totalCapacity < binaryCapacity + fee) {
        throw new Error('Insufficient funds for deployment');
    }
    
    // Create deployment transaction
    const transaction = {
        version: '0x0',
        cellDeps: [{
            outPoint: secp256k1Dep.outPoint,
            depType: 'depGroup'
        }],
        headerDeps: [],
        inputs: unspentCells.map(cell => ({
            previousOutput: cell.outPoint,
            since: '0x0'
        })),
        outputs: [
            {
                // Code cell with minimal lock (data-only)
                lock: {
                    codeHash: '0x0000000000000000000000000000000000000000000000000000000000000000',
                    hashType: 'data',
                    args: '0x'
                },
                type: null,
                capacity: '0x' + binaryCapacity.toString(16)
            },
            {
                // Change cell
                lock: lockScript,
                type: null,
                capacity: '0x' + (totalCapacity - binaryCapacity - fee).toString(16)
            }
        ],
        witnesses: [{
            lock: '',
            inputType: '',
            outputType: ''
        }],
        outputsData: [
            utils.bytesToHex(binary), // Binary data in first output
            '0x'                      // Empty data for change cell
        ]
    };
    
    // Sign and send transaction
    const signedTransaction = ckb.signTransaction(privateKey)(transaction);
    const txHash = await ckb.rpc.sendTransaction(signedTransaction, 'passthrough');
    
    // Calculate code hash for future reference
    const codeHash = ckb.utils.blake2b(binary);
    
    return {
        txHash,
        codeHash: utils.bytesToHex(codeHash),
        outPoint: {
            txHash,
            index: '0x0'
        }
    };
}

// Usage
deployBinary('./build/contract.bin', '0xprivate_key', 'http://127.0.0.1:8114')
    .then(result => {
        console.log('Deployment successful:', result);
    })
    .catch(console.error);
```

## Step 4: Reference Deployed Code

After deployment, reference the code in your scripts:

```javascript
// Create script that uses deployed code
const script = {
    codeHash: '0x8209891745eb858abd6f5e53c99b4f101bca221bd150a2ece58a389b7b4f8fa7', // From deployment
    hashType: 'data',      // Always 'data' for deployed binaries
    args: '0x1234...'      // Script arguments
};

// Include in transaction cell deps
const cellDeps = [{
    outPoint: {
        txHash: '0x9f55da2b555cdc4412945ff8827b7e77508c84f0...', // Deployment tx hash
        index: '0x0'  // Output index containing the binary
    },
    depType: 'code'  // Indicates this is executable code
}];
```

## Capacity Economics

### Capacity Calculation

```javascript
function calculateCapacity(binarySize) {
    const cellOverhead = 61;  // Base cell size in CKBytes
    const totalBytes = binarySize + cellOverhead;
    const capacityInShannon = totalBytes * 100000000; // 1 CKB = 100,000,000 Shannon
    
    return {
        bytes: totalBytes,
        ckb: totalBytes,
        shannon: capacityInShannon
    };
}

// Example: 10KB binary
const cost = calculateCapacity(10240);
console.log(`Cost: ${cost.ckb} CKB (${cost.shannon} Shannon)`);
```

### Cost Optimization Tips

1. **Minimize Binary Size**:
   - Use release builds with optimizations
   - Strip debug symbols
   - Remove unused code paths

2. **Efficient Compilation**:
   ```bash
   # Rust optimization flags
   cargo build --target=riscv64imac-unknown-none-elf --release
   
   # Additional size optimization in Cargo.toml
   [profile.release]
   lto = true
   codegen-units = 1
   panic = 'abort'
   ```

3. **Shared Libraries**: Consider using existing deployed contracts rather than deploying duplicates.

## Network Deployment Guide

### Devnet Deployment

```bash
# Start local devnet
offckb node

# Deploy with pre-funded accounts
offckb deploy --network devnet
```

**Advantages**:
- Free CKB for testing
- Fast block times
- Complete control over environment
- Integrated debugging tools

### Testnet Deployment

```bash
# Get testnet CKB from faucet
# Visit: https://faucet.nervos.org/

# Deploy to testnet
offckb deploy --network testnet
```

**Requirements**:
- Testnet CKB tokens for fees
- Internet connection
- Compatible with mainnet environment

### Mainnet Deployment

```bash
# Use mainnet RPC endpoints
# Requires real CKB tokens
offckb deploy --network mainnet
```

**Considerations**:
- Real economic costs
- Irreversible deployment
- Thorough testing recommended
- Security audit essential

## Common Deployment Patterns

### 1. Contract Factory Pattern

Deploy a factory contract that can create instances:

```rust
// Factory contract that deploys child contracts
pub fn deploy_child_contract(args: &[u8]) -> Result<Script, Error> {
    let child_code_hash = CHILD_CONTRACT_CODE_HASH;
    Ok(Script::new_builder()
        .code_hash(child_code_hash.pack())
        .hash_type(ScriptHashType::Data.into())
        .args(args.pack())
        .build())
}
```

### 2. Upgradeable Contracts

Use proxy patterns for upgradeable logic:

```javascript
// Proxy script that delegates to implementation
const proxyScript = {
    codeHash: PROXY_CODE_HASH,
    hashType: 'data',
    args: implementationCodeHash + adminLockHash
};
```

### 3. Library Deployment

Deploy reusable code libraries:

```javascript
// Deploy common library once
const mathLibrary = await deployBinary('./math_lib.bin', privateKey, nodeUrl);

// Reference in multiple contracts
const contractA = {
    // ... contract A code with math library dependency
    cellDeps: [mathLibrary.outPoint]
};
```

## Troubleshooting

### Common Deployment Issues

**1. Insufficient Capacity**:
```
Error: Insufficient funds for deployment
```
**Solution**: Ensure wallet has enough CKB (binary size + 61 CKB minimum + fees)

**2. Invalid Binary Format**:
```
Error: Invalid RISC-V binary
```
**Solution**: Verify compilation target and binary format:
```bash
file ./build/contract.bin
# Should show: ELF 64-bit LSB executable, UCB RISC-V
```

**3. Cell Deps Missing**:
```
Error: Unknown script code hash
```
**Solution**: Include deployment transaction outpoint in cell deps

**4. Network Issues**:
```
Error: Connection refused
```
**Solution**: Verify CKB node is running and accessible

### Debugging Deployed Scripts

```bash
# Use CKB debugger
offckb debug --tx-file transaction.json --script-group-type lock

# Enable debug logs in deployed script
export RUST_LOG=debug
```

### Verification Tools

```javascript
// Verify deployed code matches source
async function verifyDeployment(txHash, index, expectedBinary) {
    const transaction = await ckb.rpc.getTransaction(txHash);
    const deployedBinary = transaction.transaction.outputsData[index];
    const expectedHex = utils.bytesToHex(expectedBinary);
    
    return deployedBinary === expectedHex;
}
```

## Security Considerations

### Pre-Deployment Checklist

1. **Code Review**: Thoroughly audit smart contract logic
2. **Testing**: Comprehensive test coverage including edge cases
3. **Formal Verification**: Use formal methods for critical contracts
4. **Capacity Validation**: Ensure sufficient funds for deployment
5. **Network Selection**: Start with devnet, progress through testnet

### Post-Deployment Verification

1. **Code Integrity**: Verify deployed binary matches source
2. **Access Control**: Confirm only authorized parties can interact
3. **Monitoring**: Set up alerts for contract interactions
4. **Documentation**: Maintain deployment records and code hashes

### Best Practices

- **Never deploy untested code to mainnet**
- **Use minimal lock scripts for code cells (data-only)**
- **Keep deployment keys secure and separate from operational keys**
- **Document all deployments with version control tags**
- **Consider using multisig for production deployments**

## Advanced Topics

### Batch Deployment

Deploy multiple contracts in a single transaction:

```javascript
const batchDeployment = {
    outputs: [
        { /* contract A cell */ },
        { /* contract B cell */ },
        { /* contract C cell */ }
    ],
    outputsData: [
        contractABinary,
        contractBBinary,
        contractCBinary
    ]
};
```

### Cross-Chain Deployment

Deploy the same contract across multiple CKB networks:

```bash
# Deploy to all networks
offckb deploy --network devnet
offckb deploy --network testnet
offckb deploy --network mainnet

# Maintain deployment registry
cat deployments.json
{
  "MyContract": {
    "devnet": {
      "codeHash": "0x123...",
      "txHash": "0x456..."
    },
    "testnet": {
      "codeHash": "0x123...",
      "txHash": "0x789..."
    }
  }
}
```

Deploying binaries to CKB requires understanding the cell model, capacity economics, and proper tooling. Whether using OffCKB for convenience or manual deployment for control, following these patterns ensures successful and secure contract deployment.