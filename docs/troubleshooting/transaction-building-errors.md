## Description

Transaction construction error debugging covering insufficient capacity, missing cell deps, invalid witnesses, script execution failures, header dependency issues, cycle limit exceeded, fee calculation errors, and serialization problems with solutions and debugging strategies.

## Related Resources

- Transaction Patterns: ckb-dev-context://patterns/transaction-building-patterns
- Transaction Structure: ckb-dev-context://concepts/transaction-structure
- Script Errors: ckb-dev-context://troubleshooting/common-script-errors

## Common Transaction Building Errors

### ERROR: Insufficient Capacity

**Error Code**: `InsufficientCapacity`

**Cause**: Output cells don't have minimum required capacity for their data/type

**Minimum Capacity Calculation**:
```typescript
// Minimum capacity = 8 (capacity) + data_size + type_script_size + lock_script_size
function calculateMinCapacity(cell: Cell): bigint {
  const base = 8n; // Capacity field itself
  const dataSize = BigInt(cell.data.length);
  const lockSize = BigInt(cell.lock.args.length / 2 + 33); // 33 for code_hash + hash_type
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
2. **Include all dependencies**: Add cell deps for all scripts used
3. **Match witness count**: Ensure witnesses align with inputs
4. **Test locally first**: Use dry run before broadcasting
5. **Handle all script types**: Different scripts have different requirements
6. **Check epoch maturity**: Headers need 4 epochs to mature
7. **Monitor cycle usage**: Complex transactions may exceed limits
8. **Calculate accurate fees**: Use dynamic fee calculation based on size