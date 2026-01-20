## Description

Transaction construction error debugging covering insufficient capacity, missing cell deps (including devnet resolution), RBF/cell contention in concurrent operations, indexer synchronization lag, invalid witnesses, script execution failures, header dependency issues, cycle limits, and fee calculation with comprehensive solutions and debugging strategies.

## Related Resources

- Transaction Patterns: ckb://docs/patterns/transaction-building-patterns
- Transaction Structure: ckb://docs/concepts/transaction-structure
- Script Errors: ckb://docs/troubleshooting/common-script-errors

## Common Transaction Building Errors

### ERROR: Insufficient Capacity

**Error Code**: `InsufficientCapacity`

**Cause**: Output cells don't have minimum required capacity for their data/type

**Quick Reference**: For complete capacity calculation details, see [Cell Capacity Calculation](ckb://docs/concepts-for-coding/cell-lifecycle)

**Troubleshooting Example**:
```typescript
// Calculate minimum capacity: capacity_field + data + lock + type
function calculateMinCapacity(cell: Cell): bigint {
  const base = 8n; // Capacity field
  const dataSize = BigInt(cell.data.length);
  const lockSize = BigInt(cell.lock.args.length / 2 + 33); // code_hash (32) + hash_type (1)
  const typeSize = cell.type ? BigInt(cell.type.args.length / 2 + 33) : 0n;

  const minCapacity = (base + dataSize + lockSize + typeSize) * 100000000n;

  if (cell.capacity < minCapacity) {
    throw new Error(`Need ${minCapacity} shannons, got ${cell.capacity}`);
  }

  return minCapacity;
}
```

### ERROR: Missing Cell Deps

**Error Code**: `CellDepNotFound`

**Common Missing Dependencies**:
```typescript
const COMMON_CELL_DEPS = {
  // System scripts
  SECP256K1_BLAKE160: {
    outPoint: {
      txHash: "0x71a7ba8fc96349fea0ed3a5c47992e3b4084b031a42264a018e0072e8172e46c",
      index: "0x0"
    },
    depType: "depGroup"
  },
  
  // Mainnet scripts
  OMNILOCK: {
    outPoint: {
      txHash: "0x9b819793a64463aed77c615d6cb226eea5487ccfc0783043a8a3d0c12e3e2d8f",
      index: "0x0"
    },
    depType: "code"
  },
  
  XUDT: {
    outPoint: {
      txHash: "0xc07844ce21b38e4b071dd0e1ee3b0e27afd8d7532491327f39b786343f558ab7",
      index: "0x0"
    },
    depType: "code"
  }
};

// Add required deps based on scripts used
function addRequiredCellDeps(tx: TransactionSkeleton): TransactionSkeleton {
  const scripts = extractUsedScripts(tx);

  for (const script of scripts) {
    const dep = COMMON_CELL_DEPS[script.name];
    if (dep && !hasCellDep(tx, dep)) {
      tx = tx.update("cellDeps", deps => deps.push(dep));
    }
  }

  return tx;
}
```

**TransactionFailedToResolve on Custom Networks**:

When deploying on devnet or custom networks, you may see:
```
TransactionFailedToResolve: Unknown(OutPoint(0x...))
```

This occurs when SDKs use hardcoded mainnet/testnet cell deps that don't exist on your network. The solution is network detection:

```typescript
// Detect network and load correct system scripts
async function getSystemScriptCellDeps(rpc: RPC): Promise<CellDep[]> {
  const genesisBlock = await rpc.getBlockByNumber("0x0");
  const genesisHash = genesisBlock.header.hash;

  // Check if known network
  const KNOWN_NETWORKS = {
    "0x92b197aa1fba0f63633922c61c92375c9c074a93e85963554f5499fe1450d0e5": "mainnet",
    "0x10639e0895502b5688a6be8cf69460d76541bfa4821629d86d62ba0aae3f9606": "testnet",
  };

  if (KNOWN_NETWORKS[genesisHash]) {
    return getWellKnownCellDeps(KNOWN_NETWORKS[genesisHash]);
  }

  // Custom network - extract deps from genesis
  // System scripts are in genesis.transactions[1]
  const systemTx = genesisBlock.transactions[1];

  return [{
    outPoint: {
      txHash: systemTx.hash,
      index: "0x0"  // dep_group at index 0
    },
    depType: "depGroup"
  }];
}
```

### ERROR: RBF / Cell Contention

**Error Code**: `RBFRejected`, `PoolRejectedRBF`, `PoolRejectedDuplicatedTransaction`

**Cause**: Multiple operations attempting to spend the same cells (UTXO contention), causing Replace-By-Fee conflicts.

**Common Symptoms**:
```
PoolRejectedRBF(Reject(LowFeeRate(...)))
PoolRejectedDuplicatedTransaction
TransactionFailedToResolve (when inputs already spent)
```

**Root Cause**: When multiple tests or operations run concurrently and share a UTXO pool, they may both try to build transactions using the same unspent cells. The first transaction enters the mempool, and subsequent transactions conflict with it.

**Solution A - Serialization (wait for confirmation)**:
```typescript
// Wait for transaction confirmation and indexer sync
async function deployWithConfirmation(data: Uint8Array, rpc: RPC): Promise<string> {
  // Build and send transaction
  const tx = await buildDeployTransaction(data);
  const txHash = await rpc.sendTransaction(tx);

  // Poll for confirmation
  let confirmed = false;
  while (!confirmed) {
    const txStatus = await rpc.getTransaction(txHash);
    if (txStatus?.txStatus?.status === 'committed') {
      confirmed = true;
      console.log(`Transaction ${txHash} confirmed at block ${txStatus.txStatus.blockHash}`);
    }
    await sleep(1000);
  }

  // Wait for indexer to catch up
  const blockNumber = await getBlockNumber(txStatus.txStatus.blockHash);
  await waitForIndexerSync(rpc, blockNumber);

  return txHash;
}

async function waitForIndexerSync(rpc: RPC, targetBlock: bigint) {
  while (true) {
    const indexerTip = await rpc.getIndexerTip();
    if (BigInt(indexerTip.blockNumber) >= targetBlock) {
      break;
    }
    await sleep(500);
  }
}
```

**Solution B - Isolation (separate UTXO pools)**:
```typescript
// Give each test its own funding cells
async function setupIsolatedTest(): Promise<TestContext> {
  const privateKey = generatePrivateKey();
  const address = privateKeyToAddress(privateKey);

  // Fund this address with dedicated cells
  await fundAddress(address, 1000_00000000n);

  // Each test uses its own isolated UTXOs
  return { privateKey, address };
}

// Run tests in parallel with isolated contexts
await Promise.all([
  testWithContext(await setupIsolatedTest()),
  testWithContext(await setupIsolatedTest()),
  testWithContext(await setupIsolatedTest()),
]);
```

### ERROR: Indexer Lag / Stale Data

**Error Code**: `CellNotFound`, stale balance queries, missing recently created cells

**Cause**: Indexer database lags behind blockchain consensus, causing queries to return stale or incomplete data.

**Symptoms**:
```
- Transaction confirmed but cells not visible in queries
- Balance shows old value after transaction
- Recently deployed cell not found by out_point
- "Cell not found" errors for newly created cells
```

**Root Cause**: CKB uses a separate indexing service that processes blocks asynchronously from consensus. After a block is mined, the indexer needs time to:
1. Receive the new block
2. Process all transactions
3. Update its database
4. Make data available for queries

**Solution - Poll Indexer Tip**:
```typescript
// Wait for indexer to process a specific block
async function waitForIndexerSync(
  rpc: RPC,
  targetBlockNumber: bigint,
  timeoutMs: number = 30000
): Promise<void> {
  const startTime = Date.now();

  while (Date.now() - startTime < timeoutMs) {
    const tip = await rpc.getIndexerTip();
    const indexerBlock = BigInt(tip.blockNumber);

    if (indexerBlock >= targetBlockNumber) {
      console.log(`Indexer synced to block ${indexerBlock}`);
      return;
    }

    console.log(`Waiting for indexer: ${indexerBlock} / ${targetBlockNumber}`);
    await sleep(500);
  }

  throw new Error(`Indexer sync timeout after ${timeoutMs}ms`);
}

// Use after operations that modify state
async function deployAndVerify(data: Uint8Array, rpc: RPC) {
  // Deploy transaction
  const tx = await buildTransaction(data);
  const txHash = await rpc.sendTransaction(tx);

  // Wait for confirmation
  const receipt = await waitForConfirmation(txHash, rpc);
  const blockNumber = await getBlockNumber(receipt.blockHash);

  // Critical: Wait for indexer to process the block
  await waitForIndexerSync(rpc, blockNumber);

  // Now queries will see the new state
  const cell = await rpc.getCellByOutPoint({
    txHash: txHash,
    index: "0x0"
  });

  console.log("Cell deployed and indexed:", cell);
}
```

**Prevention Pattern**:
```typescript
// Always sync indexer before dependent operations
class CKBClient {
  async sendTransactionAndSync(tx: Transaction): Promise<string> {
    const txHash = await this.rpc.sendTransaction(tx);
    const receipt = await this.waitForConfirmation(txHash);

    // Automatically sync indexer
    const blockNum = await this.getBlockNumber(receipt.blockHash);
    await this.waitForIndexerSync(blockNum);

    return txHash;
  }
}
```

### ERROR: Invalid Witness Structure

**Error Code**: `InvalidWitness`

**Cause**: Witness doesn't match expected format for lock script

**Witness Debugging**:
```typescript
// Validate witness matches input count
function validateWitnesses(tx: Transaction) {
  // Must have at least as many witnesses as inputs
  if (tx.witnesses.length < tx.inputs.length) {
    throw new Error(`Need ${tx.inputs.length} witnesses, got ${tx.witnesses.length}`);
  }
  
  // First witness in group must be WitnessArgs
  for (let i = 0; i < tx.inputs.length; i++) {
    const witness = tx.witnesses[i];
    
    try {
      // Try to parse as WitnessArgs
      const args = WitnessArgs.unpack(witness);
      console.log(`Witness ${i}:`, {
        lock: args.lock ? `${args.lock.length} bytes` : 'empty',
        inputType: args.inputType ? `${args.inputType.length} bytes` : 'empty',
        outputType: args.outputType ? `${args.outputType.length} bytes` : 'empty'
      });
    } catch (e) {
      console.warn(`Witness ${i} is not WitnessArgs format`);
    }
  }
}
```

### ERROR: Script Execution Failed

**Error Code**: `ScriptExecutionFailed(-1)`

**Common Causes and Debug Steps**:
```typescript
// 1. Wrong script args
async function debugScriptArgs(lockScript: Script) {
  console.log("Script:", {
    codeHash: lockScript.codeHash,
    hashType: lockScript.hashType,
    args: lockScript.args
  });
  
  // Validate args length
  if (lockScript.codeHash === SECP256K1_BLAKE160.codeHash) {
    if (lockScript.args.length !== 42) { // "0x" + 40 hex chars
      throw new Error("Secp256k1 lock requires 20-byte args");
    }
  }
}

// 2. Signature verification failed
function debugSignature(tx: Transaction, privateKey: string) {
  const message = tx.signingHasher.hash();
  const signature = secp256k1.sign(message, privateKey);
  
  console.log("Signing details:", {
    message,
    publicKey: secp256k1.getPublicKey(privateKey),
    signature: signature.toHex(),
    recoveryId: signature.recovery
  });
}

// 3. Type script validation failed
async function debugTypeScript(tx: Transaction) {
  for (const output of tx.outputs) {
    if (output.type) {
      console.log("Type script:", output.type);
      console.log("Cell data:", output.data);
      
      // Validate data format matches type script requirements
      if (output.type.codeHash === XUDT.codeHash) {
        if (output.data.length < 32) { // 16 bytes amount minimum
          throw new Error("xUDT requires at least 16 bytes data");
        }
      }
    }
  }
}
```

### ERROR: Header Dep Not Found

**Error Code**: `HeaderDepNotFound`

**Cause**: Referenced block header doesn't exist or not mature enough

**Solution**:
```typescript
// Ensure header is mature (4 epochs old)
async function addHeaderDep(
  tx: TransactionSkeleton,
  blockHash: string
): Promise<TransactionSkeleton> {
  const header = await rpc.getHeader(blockHash);
  const tipHeader = await rpc.getTipHeader();
  
  const headerEpoch = parseEpoch(header.epoch);
  const tipEpoch = parseEpoch(tipHeader.epoch);
  
  const epochDiff = tipEpoch.number - headerEpoch.number;
  
  if (epochDiff < 4) {
    throw new Error(`Header needs ${4 - epochDiff} more epochs to mature`);
  }
  
  return tx.update("headerDeps", deps => deps.push(blockHash));
}
```

### ERROR: Cycles Limit Exceeded

**Error Code**: `CyclesExceeded`

**Cause**: Transaction requires more computational cycles than allowed

**Debug and Optimize**:
```typescript
// Estimate cycles before sending
async function estimateCycles(tx: Transaction): Promise<bigint> {
  const result = await rpc.dryRunTransaction(tx);
  console.log("Cycle usage:", result.cycles);
  
  const MAX_CYCLES = 3_500_000_000n; // 3.5B cycles max
  
  if (BigInt(result.cycles) > MAX_CYCLES) {
    // Try to optimize:
    // 1. Reduce input/output count
    // 2. Simplify witness data
    // 3. Use more efficient scripts
    console.error("Transaction too complex:", {
      used: result.cycles,
      max: MAX_CYCLES,
      ratio: Number(BigInt(result.cycles) * 100n / MAX_CYCLES) + '%'
    });
  }
  
  return BigInt(result.cycles);
}
```

### ERROR: Invalid Since Value

**Error Code**: `InvalidSince`

**Cause**: Since field has invalid format or references future time

**Since Field Format**:
```typescript
// Since field encoding (64-bit)
// Bit 63: relative flag (0=absolute, 1=relative)
// Bit 62: metric flag (0=block, 1=epoch)
// Bits 0-55: value

function encodeSince(value: {
  relative: boolean;
  metric: 'block' | 'epoch';
  value: bigint;
}): string {
  let since = value.value;
  
  if (value.relative) {
    since |= 0x8000000000000000n; // Set bit 63
  }
  
  if (value.metric === 'epoch') {
    since |= 0x4000000000000000n; // Set bit 62
  }
  
  return "0x" + since.toString(16).padStart(16, '0');
}

// Common patterns
const SINCE_VALUES = {
  immediate: "0x0",
  afterBlock: (n: number) => encodeSince({
    relative: true,
    metric: 'block',
    value: BigInt(n)
  }),
  afterEpoch: (n: number) => encodeSince({
    relative: true,
    metric: 'epoch',
    value: BigInt(n)
  })
};
```

### ERROR: Fee Too Low

**Error Code**: `FeeTooLow`

**Cause**: Transaction fee below minimum relay fee

**Fee Calculation**:
```typescript
function calculateFee(tx: Transaction): bigint {
  const txSize = BigInt(serializeTransaction(tx).length);
  const feeRate = 1000n; // 1000 shannons per KB
  
  const minFee = txSize * feeRate / 1000n;
  
  const inputCapacity = sumInputCapacity(tx);
  const outputCapacity = sumOutputCapacity(tx);
  const actualFee = inputCapacity - outputCapacity;
  
  if (actualFee < minFee) {
    console.error(`Fee too low: ${actualFee} < ${minFee}`);
    console.log(`Increase fee by ${minFee - actualFee} shannons`);
  }
  
  return actualFee;
}
```

## Transaction Debugging Checklist

```typescript
async function debugTransaction(tx: Transaction) {
  const checks = {
    structure: checkTransactionStructure(tx),
    capacity: checkCapacityBalance(tx),
    cellDeps: checkCellDeps(tx),
    witnesses: checkWitnesses(tx),
    scripts: await checkScripts(tx),
    cycles: await estimateCycles(tx),
    fee: checkFee(tx),
    serialization: checkSerialization(tx)
  };
  
  console.table(checks);
  
  // Run local validation
  try {
    const result = await rpc.dryRunTransaction(tx);
    console.log("Dry run successful:", result);
  } catch (e) {
    console.error("Dry run failed:", e);
    // Parse error for specific validation failure
    const errorCode = parseErrorCode(e.message);
    console.log("Error code:", errorCode);
  }
}
```

## Prevention Best Practices

1. **Always validate capacity**: Check minimum capacity before creating cells
2. **Include all dependencies**: Add cell deps for all scripts used, detect network and load correct deps for devnet
3. **Avoid cell contention**: Either serialize operations or use isolated UTXO pools for parallel operations
4. **Wait for indexer sync**: Poll indexer tip after transactions before dependent operations
5. **Match witness count**: Ensure witnesses align with inputs
6. **Test locally first**: Use dry run before broadcasting
7. **Handle all script types**: Different scripts have different requirements
8. **Check epoch maturity**: Headers need 4 epochs to mature
9. **Monitor cycle usage**: Complex transactions may exceed limits
10. **Calculate accurate fees**: Use dynamic fee calculation based on size