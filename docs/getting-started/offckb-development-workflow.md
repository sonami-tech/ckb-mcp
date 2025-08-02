# OffCKB Development Workflow

## Description

Streamline CKB development with OffCKB's one-command local blockchain environment. Set up instant devnet with 20 pre-funded accounts, deploy contracts, test dApps, and debug transactions locally. Master project templates, account management, script deployment, and testing workflows for rapid CKB application prototyping and development before deploying to testnet or mainnet.

OffCKB provides one-command local CKB development environment with pre-deployed scripts and funded accounts.

## Installation

```bash
npm install -g @offckb/cli
```

## Core Commands

### Initialize Development Environment
```bash
offckb init
```
Creates local devnet with:
- CKB node on port 8114
- Indexer on port 8116  
- 20 pre-funded accounts (10,000 CKB each)
- Pre-deployed system scripts

### Account Management
```bash
# List all accounts with balances
offckb accounts

# Show specific account details
offckb account <address>

# Pre-funded test accounts (private keys in account/keys)
# Account 0: ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqvwg2cen8extgq8s5puft8vf40px3f9c7c2g5ufy
```

### Script Deployment
```bash
# Deploy custom script
offckb deploy --file <path/to/binary>

# List deployed scripts  
offckb scripts

# Pre-deployed scripts:
# - Secp256k1 Blake160 (system)
# - Omnilock
# - Simple UDT
# - xUDT
# - Spore
# - Cluster
```

### Project Templates
```bash
# Create new dApp project
offckb create <project-name>

# Templates available:
# - next-js: Next.js + CCC SDK
# - vite-react: Vite + React + CCC
# - node-script: Node.js script template
# - rust-contract: Capsule contract template
```

## Development Workflow

### 1. Start Local Environment
```bash
# Start devnet (runs in background)
offckb init

# Verify node is running
curl -X POST http://localhost:8114 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"get_tip_block_number","params":[],"id":1}'
```

### 2. Create Project
```bash
offckb create my-ckb-app
cd my-ckb-app
npm install
npm run dev
```

### 3. Use Pre-funded Accounts
```javascript
// In your app
import { accounts } from './offckb.config.js';

const testAccount = accounts[0];
console.log('Address:', testAccount.address);
console.log('Private Key:', testAccount.privateKey);
// Balance: 10,000 CKB
```

### 4. Deploy Contracts
```bash
# Build contract
cd contracts/my-contract
capsule build --release

# Deploy to local devnet
offckb deploy --file build/release/my-contract

# Output:
# Contract deployed!
# tx_hash: 0x...
# index: 0
# type_id: 0x...
```

### 5. Interact with Scripts
```javascript
// Use deployed script in transaction
const typeScript = {
  codeHash: deploymentInfo.codeHash,
  hashType: deploymentInfo.hashType,
  args: '0x'
};

const tx = helpers.TransactionSkeleton({});
tx = await common.transfer(
  tx,
  [testAccount.address],
  recipientAddress,
  BigInt(100 * 10**8),
  undefined,
  undefined,
  { config: OFFCKB_CONFIG }
);
```

## Configuration

### offckb.config.js
```javascript
export const config = {
  network: 'devnet',
  rpcUrl: 'http://localhost:8114',
  indexerUrl: 'http://localhost:8116',
  accounts: [
    {
      address: 'ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqvwg2cen8extgq8s5puft8vf40px3f9c7c2g5ufy',
      privateKey: '0x6c9ed03816e31...'
    }
    // ... 19 more accounts
  ],
  scripts: {
    secp256k1Blake160: {
      codeHash: '0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8',
      hashType: 'type',
      txHash: '0x...',
      index: '0x0',
      depType: 'depGroup'
    },
    omnilock: {
      codeHash: '0x00000000000000000000000000000000000000000000000000545950455f4944',
      hashType: 'type',
      txHash: '0x...',
      index: '0x0',
      depType: 'code'
    },
    xudt: {
      codeHash: '0x50bd8d6680b8b9cf98b73f3c08faf8b2a21914311954118ad6609be6e78a1b95',
      hashType: 'data1',
      txHash: '0x...',
      index: '0x0',
      depType: 'code'
    }
  }
};
```

## Testing Workflow

### Unit Tests
```bash
# Run contract tests
cd contracts/my-contract
capsule test

# Run integration tests
npm test
```

### Integration Testing
```javascript
// test/integration.test.js
import { initOffCKB, accounts, deployContract } from '../offckb.config.js';

beforeAll(async () => {
  await initOffCKB();
});

test('deploy and interact with contract', async () => {
  // Deploy contract
  const deployment = await deployContract('./build/release/my-contract');
  
  // Create transaction using deployed contract
  const tx = await createTransaction({
    from: accounts[0],
    to: accounts[1],
    amount: BigInt(100 * 10**8),
    type: deployment.typeScript
  });
  
  // Send transaction
  const txHash = await rpc.sendTransaction(tx);
  
  // Wait for confirmation
  await waitForTransaction(txHash);
});
```

## Debugging

### View Logs
```bash
# CKB node logs
tail -f ~/.offckb/ckb.log

# Indexer logs  
tail -f ~/.offckb/indexer.log
```

### Debug Transactions
```bash
# Get transaction details
offckb tx <tx_hash>

# Decode transaction
offckb decode-tx <tx_hex>

# Trace script execution
offckb trace <tx_hash>
```

### Common Issues

**Port Already in Use**
```bash
# Stop existing OffCKB instance
offckb stop

# Or use custom ports
offckb init --ckb-port 8115 --indexer-port 8117
```

**Script Deployment Failed**
```bash
# Check binary is RISC-V
file build/release/my-contract
# Should show: ELF 64-bit LSB executable, UCB RISC-V

# Verify script size
ls -lh build/release/my-contract
# Should be < 600KB
```

## Advanced Usage

### Custom Network Configuration
```javascript
// offckb.config.js
export const customConfig = {
  network: 'custom',
  rpcUrl: process.env.CKB_RPC_URL,
  indexerUrl: process.env.CKB_INDEXER_URL,
  accounts: loadAccountsFromEnv(),
  scripts: loadScriptsFromFile('./deployments.json')
};
```

### Script Debugging
```javascript
// Enable debug mode for detailed execution traces
const tx = await createTransaction({
  // ... transaction details
  debug: true
});

// Logs will show:
// - Script execution cycles
// - Memory usage
// - Syscall traces
```

### Multi-Contract Projects
```bash
# Deploy multiple contracts
offckb deploy --file build/release/lock_script
offckb deploy --file build/release/type_script

# Export deployment info
offckb export-config > deployments.json
```

## Best Practices

1. **Use Test Accounts**: Never use mainnet private keys in development.
2. **Reset State**: Run `offckb clean && offckb init` to reset blockchain state.
3. **Version Control**: Add `.offckb/` to `.gitignore`.
4. **Script Caching**: Cache deployed script info in `offckb.config.js`.
5. **Parallel Testing**: Use different account indices for parallel tests.

## Migration to Testnet/Mainnet

```javascript
// Update config for testnet
export const testnetConfig = {
  network: 'testnet',
  rpcUrl: 'https://testnet.ckb.dev/rpc',
  indexerUrl: 'https://testnet.ckb.dev/indexer',
  accounts: [], // Use real testnet accounts
  scripts: {
    // Use testnet script deployments
    secp256k1Blake160: TESTNET_SCRIPTS.SECP256K1_BLAKE160,
    omnilock: TESTNET_SCRIPTS.OMNILOCK,
    xudt: TESTNET_SCRIPTS.XUDT
  }
};
```

OffCKB streamlines CKB development by providing instant local environment setup, pre-funded accounts, and common script deployments. Use it for rapid prototyping, testing, and development before deploying to testnet or mainnet.