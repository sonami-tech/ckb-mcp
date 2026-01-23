## Description

Streamline CKB development with OffCKB's one-command local blockchain environment. Set up instant devnet with 20 pre-funded accounts, deploy contracts, test dApps, and debug transactions locally. Master project templates, account management, script deployment, and testing workflows for rapid CKB application prototyping and development before deploying to testnet or mainnet.

OffCKB provides one-command local CKB development environment with pre-deployed scripts and funded accounts.

## Installation

```bash
npm install -g @offckb/cli
```

## Core Commands

### Start Development Environment
```bash
offckb node
```
Starts local devnet with:
- CKB node on port 8114
- Indexer on port 8116
- 20 pre-funded accounts
- Pre-deployed system scripts

### Account Management
```bash
# List all pre-funded accounts
offckb accounts

# Check balance of specific address
offckb balance <address>
offckb balance <address> --network testnet

# Pre-funded test accounts are defined in account/account.json
```

### Script Deployment
```bash
# Deploy scripts from current directory to devnet
offckb deploy

# Deploy scripts from specific path
offckb deploy --target <path/to/binary>

# Deploy to testnet
offckb deploy --network testnet --privkey <your-private-key>

# Deploy with Type ID (upgradable)
offckb deploy --type-id

# Print system scripts info
offckb system-scripts
offckb system-scripts --network testnet
offckb system-scripts --export-style lumos  # or ccc, system
offckb system-scripts -o scripts.json  # Export to JSON file
```

### Project Templates
```bash
# Create new dApp project
offckb create <project-name>

# Templates available:
# - next-js: Next.js + CCC SDK
# - remix-vite: Remix + Vite + CCC SDK
# - node-script: Node.js script template
# - script-only: Rust contract template (ckb-script-templates)
```

## Development Workflow

### 1. Start Local Environment
```bash
# Start devnet (runs in background)
offckb node

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
// In your app with CCC
import { ccc } from "@ckb-ccc/core";
import offckb from './offckb.config.ts';

const testAccount = offckb.accounts[0];
const client = new ccc.ClientPublicTestnet();
const signer = new ccc.SignerCkbPrivateKey(client, testAccount.privateKey);

console.log('Address:', testAccount.address);
// Balance: 10,000 CKB
```

### 4. Deploy Contracts
```bash
# Build contract
cd contracts/my-contract
make build

# Deploy to local devnet
offckb deploy --network devnet

# Output:
# Contract deployed!
# tx_hash: 0x...
# index: 0
# type_id: 0x...
```

### 5. Interact with Scripts
```javascript
// Use deployed script in transaction with CCC
import { ccc } from "@ckb-ccc/core";

const typeScript = {
  codeHash: deploymentInfo.codeHash,
  hashType: deploymentInfo.hashType,
  args: '0x'
};

// Build transaction with CCC
const tx = ccc.Transaction.from({
  outputs: [{
    lock: recipientLock,
    capacity: ccc.fixedPointFrom(100), // 100 CKB
    type: typeScript
  }]
});

await tx.completeInputsByCapacity(signer);
await tx.completeFeeBy(signer);
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
make test

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
  const deployment = await deployContract('./build/my-contract');
  
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

### Debug Transactions
```bash
# Debug a transaction by its hash (shows all script executions)
offckb debug --tx-hash <tx_hash>

# Debug a specific cell script in a transaction
offckb debug --tx-hash <tx_hash> --single-script "input:0:lock"
offckb debug --tx-hash <tx_hash> --single-script "output:1:type"

# Debug with a replacement binary (for testing changes)
offckb debug --tx-hash <tx_hash> --single-script "input:0:lock" --bin ./my-script

# Debug on different networks
offckb debug --tx-hash <tx_hash> --network testnet
```

### Raw CKB Debugger
```bash
# Access the underlying CKB Standalone Debugger directly
offckb debugger --help
offckb debugger <debugger-args>
```

### View Logs
```bash
# CKB node logs (location depends on platform)
tail -f ~/.offckb/devnet/ckb.log
```

### Common Issues

**Port Already in Use**
```bash
# Clean and restart devnet
offckb clean
offckb node
```

**Script Deployment Failed**
```bash
# Check binary is RISC-V
file build/my-contract
# Should show: ELF 64-bit LSB executable, UCB RISC-V

# Verify script size
ls -lh build/my-contract
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
# Build all contracts in workspace
make build

# Deploy all contracts from build output
offckb deploy --target ./build --network devnet -o ./deployment

# Deployment records are written to the output folder as JSON files
# Use these files in your app to reference deployed scripts
```

## Best Practices

1. **Use Test Accounts**: Never use mainnet private keys in development.
2. **Reset State**: Run `offckb clean && offckb node` to reset blockchain state.
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