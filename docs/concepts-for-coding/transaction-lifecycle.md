# Transaction Lifecycle and Construction Workflows (CCC Modern Approach)

## Description

Complete transaction construction workflow using modern CCC SDK, covering automated and manual approaches for building, signing, broadcasting, and monitoring CKB transactions. Includes cell collection, capacity management, dependency handling, error handling, and confirmation monitoring. Features deprecation guidance for Capsule and recommended 2024 development stack.

## Purpose
Complete workflow for transaction construction from creation to confirmation using **CCC (recommended)**. This pattern demonstrates:
- Modern, simplified transaction building process
- Automatic input and output cell management  
- Smart capacity calculation and fee handling
- Dependency management (cell deps)
- Signature generation and witness handling
- Broadcasting and confirmation monitoring

## Why Use CCC for Transaction Construction
- **Simplified API**: Intuitive transaction building vs manual complexity
- **Automatic Management**: Auto-handles cells, fees, capacity validation
- **Error Prevention**: Built-in validations prevent common mistakes
- **Modern Tooling**: Better TypeScript support and debugging
- **Production Ready**: Battle-tested in major CKB applications

## Modern Transaction Workflow (CCC)

### Quick Start: Simple Transfer (Recommended)
```typescript
import { ccc } from "@ckb-ccc/ccc";

// Most transactions can be built this simply with CCC
const signer = new ccc.SignerCkbPrivateKey(client, privateKey);
const { script: toLock } = await ccc.Address.fromString(toAddress, client);

const tx = ccc.Transaction.from({
  outputs: [{ lock: toLock, capacity: ccc.fixedPointFrom(amount) }],
});

await tx.completeInputsByCapacity(signer);  // Auto-collects inputs
await tx.completeFeeBy(signer);             // Auto-calculates fees  
const txHash = await signer.sendTransaction(tx); // Auto-signs and sends

// Monitor confirmation
await client.waitTransaction(txHash);
console.log("Transaction confirmed!");
```

## Advanced Manual Workflow (When Needed)

### Step 1: Create Empty Transaction
```typescript
import { ccc } from "@ckb-ccc/ccc";

// Initialize empty transaction scaffold
const tx = ccc.Transaction.from({
    version: "0x0",
    cellDeps: [],
    headerDeps: [],
    inputs: [],
    outputs: [],
    outputsData: [],
    witnesses: []
});

console.log("Empty transaction created");
```

### Step 2: Add Input Cells (Cell Collection)
```typescript
// Collect live cells for inputs
async function addInputCells(
    tx: ccc.Transaction,
    signer: ccc.Signer,
    requiredCapacity: bigint
): Promise<bigint> {
    const address = await signer.getRecommendedAddress();
    const cells = await signer.client.getCells({
        script: address.script,
        scriptType: "lock",
    }, "asc", "0x64");

    let collectedCapacity = 0n;
    
    for (const cell of cells.objects) {
        // Add cell as input
        tx.inputs.push({
            previousOutput: cell.outPoint,
            since: "0x0"
        });

        // Add corresponding witness placeholder
        tx.witnesses.push("0x");
        
        collectedCapacity += BigInt(cell.cellOutput.capacity);
        
        if (collectedCapacity >= requiredCapacity) {
            break;
        }
    }

    if (collectedCapacity < requiredCapacity) {
        throw new Error(`Insufficient capacity: need ${requiredCapacity}, found ${collectedCapacity}`);
    }

    return collectedCapacity;
}

// Usage
const inputCapacity = await addInputCells(tx, signer, requiredCapacity);
console.log(`Added inputs with ${inputCapacity} capacity`);
```

### Step 3: Add Output Cells
```typescript
// Add output cells for recipients
function addOutputCells(
    tx: ccc.Transaction,
    outputs: Array<{address: string, capacity: bigint, data?: Uint8Array}>
) {
    for (const output of outputs) {
        // Parse address to get lock script
        const address = ccc.Address.fromString(output.address, signer.client);
        
        // Add output cell
        tx.outputs.push({
            capacity: ccc.numToHex(output.capacity),
            lock: address.script,
            type: undefined // No type script for basic transfer
        });

        // Add corresponding output data
        tx.outputsData.push(output.data || new Uint8Array());
    }
}

// Usage
addOutputCells(tx, [
    { 
        address: "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsq...",
        capacity: 10000000000n // 100 CKB
    }
]);
```

### Step 4: Add Change Cell and Calculate Fees
```typescript
async function addChangeAndFees(
    tx: ccc.Transaction,
    signer: ccc.Signer,
    inputCapacity: bigint,
    outputCapacity: bigint,
    feeRate: bigint = 1000n
): Promise<void> {
    const MIN_CELL_CAPACITY = 6100000000n; // 61 CKB minimum

    // Calculate approximate fee (will be refined later)
    const approximateFee = BigInt(tx.getSize()) * feeRate;
    const changeCapacity = inputCapacity - outputCapacity - approximateFee;

    if (changeCapacity > MIN_CELL_CAPACITY) {
        // Add change cell
        const changeAddress = await signer.getRecommendedAddress();
        tx.outputs.push({
            capacity: ccc.numToHex(changeCapacity),
            lock: changeAddress.script,
            type: undefined
        });
        tx.outputsData.push(new Uint8Array());
    } else if (changeCapacity > 0n && changeCapacity < MIN_CELL_CAPACITY) {
        // Need to collect more capacity for valid change cell
        const additionalCapacity = MIN_CELL_CAPACITY - changeCapacity;
        await addInputCells(tx, signer, additionalCapacity);
        
        // Recalculate and add change
        const newChangeCapacity = changeCapacity + additionalCapacity;
        const changeAddress = await signer.getRecommendedAddress();
        tx.outputs.push({
            capacity: ccc.numToHex(newChangeCapacity),
            lock: changeAddress.script,
            type: undefined
        });
        tx.outputsData.push(new Uint8Array());
    }
    // If changeCapacity <= 0, it goes to miners as fee
}
```

### Step 5: Add Dependencies (Cell Deps)
```typescript
// Add required cell dependencies
function addCellDeps(tx: ccc.Transaction, deps: Array<{
    outPoint: ccc.OutPoint,
    depType: "code" | "depGroup"
}>) {
    for (const dep of deps) {
        tx.cellDeps.push({
            outPoint: dep.outPoint,
            depType: dep.depType
        });
    }
}

// Common system cell deps (secp256k1 signature verification)
const SECP256K1_BLAKE160_SIGHASH_ALL_DEP = {
    outPoint: {
        txHash: "0x71a7ba8fc96349fea0ed3a5c47992e3b4084b031a42264a018e0072e8172e46c",
        index: "0x0"
    },
    depType: "depGroup" as const
};

addCellDeps(tx, [SECP256K1_BLAKE160_SIGHASH_ALL_DEP]);
```

### Step 6: Generate Signatures
```typescript
async function signTransaction(
    tx: ccc.Transaction,
    signer: ccc.Signer
): Promise<void> {
    // CCC SDK handles signature generation automatically
    await signer.signTransaction(tx);
    console.log("Transaction signed successfully");
}

// Manual signing process (for understanding)
async function manualSign(
    tx: ccc.Transaction,
    privateKey: string,
    inputIndices: number[]
): Promise<void> {
    // Generate signing message hash
    const signingHash = tx.hash();
    
    // Sign with private key
    const signature = signMessage(signingHash, privateKey);
    
    // Add signature to witness
    for (const index of inputIndices) {
        tx.witnesses[index] = ccc.WitnessArgs.from({
            lock: signature,
            inputType: undefined,
            outputType: undefined
        }).toBytes();
    }
}
```

### Step 7: Broadcast Transaction
```typescript
async function broadcastTransaction(
    tx: ccc.Transaction,
    client: ccc.Client
): Promise<string> {
    try {
        // Send transaction to network
        const txHash = await client.sendTransaction(tx);
        console.log(`Transaction broadcasted: ${txHash}`);
        return txHash;
    } catch (error) {
        console.error("Failed to broadcast transaction:", error);
        throw error;
    }
}
```

### Step 8: Wait for Confirmation
```typescript
async function waitForConfirmation(
    txHash: string,
    client: ccc.Client,
    maxWaitTime: number = 300000 // 5 minutes
): Promise<void> {
    const startTime = Date.now();
    
    while (Date.now() - startTime < maxWaitTime) {
        try {
            const txStatus = await client.getTransaction(txHash);
            
            if (txStatus && txStatus.txStatus) {
                switch (txStatus.txStatus.status) {
                    case "committed":
                        console.log(`Transaction confirmed in block: ${txStatus.txStatus.blockHash}`);
                        return;
                    case "rejected":
                        throw new Error(`Transaction rejected: ${txStatus.txStatus.reason}`);
                    case "pending":
                        console.log("Transaction pending in mempool...");
                        break;
                    case "proposed":
                        console.log("Transaction proposed by miner...");
                        break;
                }
            }
        } catch (error) {
            console.log("Transaction not found yet, waiting...");
        }
        
        // Wait 5 seconds before checking again
        await new Promise(resolve => setTimeout(resolve, 5000));
    }
    
    throw new Error("Transaction confirmation timeout");
}
```

## Complete Transaction Construction Example
```typescript
async function buildCompleteTransaction(
    signer: ccc.Signer,
    toAddress: string,
    amount: bigint
): Promise<string> {
    // Step 1: Create empty transaction
    const tx = ccc.Transaction.from({
        version: "0x0",
        cellDeps: [],
        headerDeps: [],
        inputs: [],
        outputs: [],
        outputsData: [],
        witnesses: []
    });

    // Step 2: Calculate requirements
    const feeCapacity = 1000n; // 0.00001 CKB
    const minChangeCapacity = 6100000000n; // 61 CKB
    const totalRequired = amount + feeCapacity + minChangeCapacity;

    // Step 3: Collect input cells
    const inputCapacity = await addInputCells(tx, signer, totalRequired);

    // Step 4: Add output cell
    addOutputCells(tx, [{
        address: toAddress,
        capacity: amount
    }]);

    // Step 5: Add change and handle fees
    await addChangeAndFees(tx, signer, inputCapacity, amount, 1000n);

    // Step 6: Add cell dependencies
    addCellDeps(tx, [SECP256K1_BLAKE160_SIGHASH_ALL_DEP]);

    // Step 7: Sign transaction
    await signTransaction(tx, signer);

    // Step 8: Broadcast
    const txHash = await broadcastTransaction(tx, signer.client);

    // Step 9: Wait for confirmation
    await waitForConfirmation(txHash, signer.client);

    return txHash;
}

// Usage
const txHash = await buildCompleteTransaction(
    signer,
    "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsq...",
    10000000000n // 100 CKB
);
console.log("Transaction completed:", txHash);
```

## Transaction States and Monitoring
```typescript
enum TransactionStatus {
    Unknown = "unknown",
    Pending = "pending",    // In mempool, waiting to be mined
    Proposed = "proposed",  // Included in proposed block
    Committed = "committed", // Confirmed in blockchain
    Rejected = "rejected"   // Rejected by network
}

class TransactionMonitor {
    constructor(private client: ccc.Client) {}

    async getStatus(txHash: string): Promise<TransactionStatus> {
        try {
            const result = await this.client.getTransaction(txHash);
            return result?.txStatus?.status as TransactionStatus || TransactionStatus.Unknown;
        } catch {
            return TransactionStatus.Unknown;
        }
    }

    async waitForStatus(
        txHash: string,
        targetStatus: TransactionStatus,
        timeout: number = 300000
    ): Promise<void> {
        const startTime = Date.now();
        
        while (Date.now() - startTime < timeout) {
            const status = await this.getStatus(txHash);
            
            if (status === targetStatus) {
                return;
            }
            
            if (status === TransactionStatus.Rejected) {
                throw new Error("Transaction was rejected");
            }
            
            await new Promise(resolve => setTimeout(resolve, 5000));
        }
        
        throw new Error(`Timeout waiting for status: ${targetStatus}`);
    }
}
```

## Key Patterns Explained

### 1. Capacity Management Flow
```typescript
// Pattern: Always calculate capacity requirements upfront
const outputCapacity = calculateOutputCapacity();
const feeCapacity = estimateFee();
const minChangeCapacity = 6100000000n;
const totalRequired = outputCapacity + feeCapacity + minChangeCapacity;
```

### 2. Input Collection Strategy
```typescript
// Pattern: Collect more than needed, handle change properly
const collected = await collectCells(totalRequired);
const change = collected - outputCapacity - fee;

if (change > 0n && change < MIN_CAPACITY) {
    // Collect more to make valid change cell
    await collectAdditionalCells(MIN_CAPACITY - change);
}
```

### 3. Dependency Management
```typescript
// Pattern: Add all required dependencies before signing
const requiredDeps = [
    SECP256K1_DEP,     // For signature verification
    SCRIPT_DEP,        // For custom scripts
    // ... other deps
];
tx.cellDeps.push(...requiredDeps);
```

### 4. Error Handling Throughout Lifecycle
```typescript
// Pattern: Handle errors at each step
try {
    await addInputs();
    await addOutputs();
    await sign();
    const hash = await broadcast();
    await waitForConfirmation(hash);
} catch (error) {
    if (error.message.includes("Insufficient")) {
        // Handle capacity issues
    } else if (error.message.includes("signature")) {
        // Handle signing issues
    }
    // ... other error types
}
```

## When to Use This Pattern
- **All CKB transaction construction** scenarios
- **Wallet implementations** requiring reliable transaction building
- **DApp backends** automating transaction workflows
- **Complex transactions** with multiple inputs/outputs
- **Production applications** needing robust transaction handling

## Script Development: Capsule Deprecation Notice

### ⚠️ Important: Capsule is Deprecated

**Capsule is no longer maintained**. For new script development:

```bash
# OLD (Deprecated - Don't Use)
cargo install ckb-capsule
capsule new my-script

# NEW (Recommended - Use This)
cargo install ckb-script-templates
# Follow ckb-script-templates documentation
```

### Migration from Capsule

If you have existing Capsule projects:

1. **For new scripts**: Use `ckb-script-templates`
2. **For testing**: Continue using `ckb-testtool` (still maintained)
3. **For existing scripts**: Consider gradual migration

### What Happened to Capsule?

- **Split functionality**:
  - Project management → `ckb-script-templates`
  - Testing utilities → `ckb-testtool` (maintained)
- **Reason**: Better separation of concerns and modern tooling support
- **Timeline**: Deprecated as of 2024, use modern alternatives

## Summary: Modern CKB Development Stack (2024)

### Recommended for New Projects:
- **Frontend**: CCC SDK (`@ckb-ccc/ccc`)
- **Script Development**: ckb-script-templates
- **Testing**: ckb-testtool
- **Transaction Building**: CCC automatic methods

### Legacy but Supported:
- **Frontend**: Lumos (with CCC patches for wallets)
- **Script Development**: Manual Rust toolchain

### Deprecated (Avoid):
- **Capsule** (use ckb-script-templates instead)

This comprehensive lifecycle pattern ensures reliable, production-ready transaction construction using modern CKB development tools with proper error handling and confirmation monitoring.