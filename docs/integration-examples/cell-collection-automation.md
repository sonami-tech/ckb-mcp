# Cell Collection Automation Pattern (CCC Focused)

## Description

Production-ready automation pattern for cell collection and capacity management using modern CCC APIs. Provides comprehensive examples of automated cell discovery, capacity requirement calculation, change cell handling, and efficient cell selection algorithms. Includes both built-in CCC methods and custom collection logic for complex scenarios. Essential pattern for building robust CKB applications that need automated transaction construction and capacity management.

## Purpose  
Production-ready pattern for automatically collecting live cells to satisfy transaction capacity requirements. This pattern demonstrates:
- Automated cell discovery using modern CCC APIs
- Capacity requirement calculation
- Minimum cell capacity validation (61 CKB minimum)
- Change cell management
- Efficient cell selection algorithms

## Modern Approach: Use CCC Built-in Methods (Recommended)

### Automatic Cell Collection with CCC
```typescript
import { ccc } from "@ckb-ccc/ccc";

// CCC handles cell collection automatically - this is the recommended approach
const signer = new ccc.SignerCkbPrivateKey(client, privateKey);

const tx = ccc.Transaction.from({
  outputs: [{ 
    lock: recipientLockScript, 
    capacity: ccc.fixedPointFrom(amount) 
  }],
});

// CCC automatically handles:
// - Cell collection
// - Change cell creation
// - Fee calculation
// - Capacity validation
await tx.completeInputsByCapacity(signer);
await tx.completeFeeBy(signer);

const txHash = await signer.sendTransaction(tx);
```

## Manual Cell Collection (Advanced Use Cases)

### CCC Cell Collection API
```typescript
import { ccc } from "@ckb-ccc/ccc";

/**
 * Manual cell collection using CCC's lower-level APIs
 * Use this when you need custom cell selection logic
 */
async function collectCellsWithCCC(
    client: ccc.Client,
    lockScript: ccc.Script,
    requiredCapacity: bigint
): Promise<ccc.Cell[]> {
    const cells = await client.getCells({
        script: lockScript,
        scriptType: "lock",
        filter: {
            scriptLenRange: [0, 1], // Only cells without type scripts
        }
    }, "asc", "0x64"); // Order ascending, limit 100
    
    let collected: ccc.Cell[] = [];
    let totalCapacity = 0n;
    
    for (const cell of cells.objects) {
        collected.push(cell);
        totalCapacity += BigInt(cell.cellOutput.capacity);
        
        if (totalCapacity >= requiredCapacity) {
            break;
        }
    }
    
    if (totalCapacity < requiredCapacity) {
        throw new Error(`Insufficient capacity: need ${requiredCapacity}, found ${totalCapacity}`);
    }
    
    return collected;
}
```

## Legacy Lumos Implementation (For Reference Only)

### Basic Cell Collection (Lumos - Not Recommended)
```typescript
import { CellCollector, Indexer } from "@ckb-lumos/ckb-indexer";
import { Script, Cell, Hash, HexString } from "@ckb-lumos/base";

/**
 * Collects cells for use as capacity from the specified lock script
 * @param indexer - Running Lumos Indexer instance
 * @param lockScript - Lock script to search for cells
 * @param capacityRequired - Amount needed in Shannon (CKB * 10^8)
 * @returns Object with collected cells and total capacity
 */
async function collectCapacity(
    indexer: Indexer,
    lockScript: Script,
    capacityRequired: bigint
): Promise<{inputCells: Cell[], inputCapacity: bigint}> {
    const query = {
        lock: lockScript,
        type: null  // Only collect cells without type scripts
    };
    
    const cellCollector = new CellCollector(indexer, query);
    
    let inputCells: Cell[] = [];
    let inputCapacity = 0n;
    
    // Collect cells until we have enough capacity
    for await (const cell of cellCollector.collect()) {
        inputCells.push(cell);
        inputCapacity += BigInt(cell.cell_output.capacity);
        
        if (inputCapacity >= capacityRequired) {
            break;
        }
    }
    
    if (inputCapacity < capacityRequired) {
        throw new Error(
            `Insufficient capacity: need ${capacityRequired}, found ${inputCapacity}`
        );
    }
    
    return { inputCells, inputCapacity };
}
```

### Advanced Cell Collection with Change Management
```typescript
interface CollectionResult {
    inputCells: Cell[];
    inputCapacity: bigint;
    changeCapacity: bigint;
    needsChange: boolean;
}

/**
 * Advanced cell collection with proper change cell handling
 */
async function collectCapacityWithChange(
    indexer: Indexer,
    lockScript: Script,
    outputCapacity: bigint,
    feeCapacity: bigint = 1000n // 0.00001 CKB default fee
): Promise<CollectionResult> {
    const MIN_CELL_CAPACITY = 6100000000n; // 61 CKB minimum
    const totalRequired = outputCapacity + feeCapacity;
    
    // First, try to collect exact amount
    const { inputCells, inputCapacity } = await collectCapacity(
        indexer,
        lockScript,
        totalRequired
    );
    
    const changeCapacity = inputCapacity - totalRequired;
    
    // Check if change amount can form a valid cell
    if (changeCapacity > 0n && changeCapacity < MIN_CELL_CAPACITY) {
        // Need to collect more to make change cell valid
        const additionalRequired = MIN_CELL_CAPACITY - changeCapacity;
        
        const additional = await collectCapacity(
            indexer,
            lockScript,
            additionalRequired
        );
        
        return {
            inputCells: [...inputCells, ...additional.inputCells],
            inputCapacity: inputCapacity + additional.inputCapacity,
            changeCapacity: changeCapacity + additional.inputCapacity,
            needsChange: true
        };
    }
    
    return {
        inputCells,
        inputCapacity,
        changeCapacity,
        needsChange: changeCapacity > 0n
    };
}
```

### CCC SDK Cell Collection Pattern
```typescript
import { ccc } from "@ckb-ccc/core";

/**
 * Cell collection using CCC SDK
 */
async function collectCellsWithCCC(
    client: ccc.Client,
    lockScript: ccc.Script,
    requiredCapacity: bigint
): Promise<ccc.Cell[]> {
    const cells = await client.getCells({
        script: lockScript,
        scriptType: "lock",
        filter: {
            scriptLenRange: [0, 1], // Only cells without type scripts
        }
    }, "asc", "0x64"); // Order ascending, limit 100
    
    let collected: ccc.Cell[] = [];
    let totalCapacity = 0n;
    
    for (const cell of cells.objects) {
        collected.push(cell);
        totalCapacity += BigInt(cell.cellOutput.capacity);
        
        if (totalCapacity >= requiredCapacity) {
            break;
        }
    }
    
    if (totalCapacity < requiredCapacity) {
        throw new Error(`Insufficient capacity: need ${requiredCapacity}, found ${totalCapacity}`);
    }
    
    return collected;
}
```

### Optimized Cell Selection Algorithm
```typescript
interface CellWithCapacity {
    cell: Cell;
    capacity: bigint;
}

/**
 * Optimized cell selection using different strategies
 */
class CellSelector {
    constructor(private indexer: Indexer) {}
    
    /**
     * Select cells using "largest first" strategy to minimize inputs
     */
    async selectLargestFirst(
        lockScript: Script,
        targetCapacity: bigint
    ): Promise<Cell[]> {
        const query = { lock: lockScript, type: null };
        const cellCollector = new CellCollector(this.indexer, query);
        
        // Collect all available cells first
        const availableCells: CellWithCapacity[] = [];
        for await (const cell of cellCollector.collect()) {
            availableCells.push({
                cell,
                capacity: BigInt(cell.cell_output.capacity)
            });
        }
        
        // Sort by capacity (largest first)
        availableCells.sort((a, b) => 
            a.capacity > b.capacity ? -1 : 1
        );
        
        // Select minimum number of cells
        const selected: Cell[] = [];
        let totalCapacity = 0n;
        
        for (const { cell, capacity } of availableCells) {
            selected.push(cell);
            totalCapacity += capacity;
            
            if (totalCapacity >= targetCapacity) {
                break;
            }
        }
        
        if (totalCapacity < targetCapacity) {
            throw new Error("Insufficient total capacity");
        }
        
        return selected;
    }
    
    /**
     * Select cells using "best fit" strategy to minimize change
     */
    async selectBestFit(
        lockScript: Script,
        targetCapacity: bigint
    ): Promise<Cell[]> {
        const query = { lock: lockScript, type: null };
        const cellCollector = new CellCollector(this.indexer, query);
        
        const availableCells: CellWithCapacity[] = [];
        for await (const cell of cellCollector.collect()) {
            availableCells.push({
                cell,
                capacity: BigInt(cell.cell_output.capacity)
            });
        }
        
        // Try to find exact match first
        const exactMatch = availableCells.find(c => c.capacity === targetCapacity);
        if (exactMatch) {
            return [exactMatch.cell];
        }
        
        // Use knapsack-like algorithm for best fit
        return this.knapsackSelection(availableCells, targetCapacity);
    }
    
    private knapsackSelection(
        cells: CellWithCapacity[],
        target: bigint
    ): Cell[] {
        // Simplified knapsack for demonstration
        // In production, use more sophisticated algorithm
        const selected: Cell[] = [];
        let remaining = target;
        
        // Sort cells by efficiency (capacity/overhead ratio)
        cells.sort((a, b) => a.capacity > b.capacity ? -1 : 1);
        
        for (const { cell, capacity } of cells) {
            if (capacity <= remaining) {
                selected.push(cell);
                remaining -= capacity;
                
                if (remaining === 0n) break;
            }
        }
        
        return selected;
    }
}
```

### Indexer Setup and Configuration
```typescript
/**
 * Initialize indexer for cell collection
 */
function setupIndexer(): Indexer {
    const NODE_URL = "http://127.0.0.1:8114/";
    const INDEXER_URL = "http://127.0.0.1:8114/"; // Same as node in new versions
    
    const indexer = new Indexer(INDEXER_URL, NODE_URL);
    
    // Wait for indexer to sync
    return indexer;
}

/**
 * Verify indexer is working
 */
async function verifyIndexer(indexer: Indexer): Promise<void> {
    try {
        const tip = await indexer.tip();
        console.log("Indexer synced to block:", tip.block_number);
    } catch (error) {
        throw new Error("Indexer not accessible: " + error.message);
    }
}
```

### Production Cell Collection Service
```typescript
class CellCollectionService {
    constructor(
        private indexer: Indexer,
        private selector: CellSelector
    ) {}
    
    /**
     * Collect cells for a transaction with all validations
     */
    async collectForTransaction(
        fromLockScript: Script,
        outputs: { lockScript: Script, capacity: bigint }[],
        feeCapacity: bigint = 1000n
    ): Promise<{
        inputCells: Cell[],
        changeOutput?: { lockScript: Script, capacity: bigint }
    }> {
        const MIN_CELL_CAPACITY = 6100000000n; // 61 CKB
        
        // Calculate total output capacity needed
        const totalOutputCapacity = outputs.reduce(
            (sum, output) => sum + output.capacity,
            0n
        );
        
        const totalRequired = totalOutputCapacity + feeCapacity;
        
        // Collect input cells
        const inputCells = await this.selector.selectLargestFirst(
            fromLockScript,
            totalRequired
        );
        
        // Calculate input capacity
        const inputCapacity = inputCells.reduce(
            (sum, cell) => sum + BigInt(cell.cell_output.capacity),
            0n
        );
        
        // Calculate change
        const changeCapacity = inputCapacity - totalRequired;
        
        if (changeCapacity > 0n) {
            if (changeCapacity < MIN_CELL_CAPACITY) {
                // Collect additional capacity for valid change cell
                const additional = await this.selector.selectLargestFirst(
                    fromLockScript,
                    MIN_CELL_CAPACITY - changeCapacity
                );
                
                const finalChangeCapacity = changeCapacity + 
                    additional.reduce((sum, cell) => 
                        sum + BigInt(cell.cell_output.capacity), 0n
                    );
                
                return {
                    inputCells: [...inputCells, ...additional],
                    changeOutput: {
                        lockScript: fromLockScript,
                        capacity: finalChangeCapacity
                    }
                };
            } else {
                return {
                    inputCells,
                    changeOutput: {
                        lockScript: fromLockScript,
                        capacity: changeCapacity
                    }
                };
            }
        }
        
        return { inputCells };
    }
}
```

## Key Patterns Explained

### 1. Capacity Requirement Calculation
```typescript
// Pattern: Always account for all capacity needs
const outputCapacity = 10000000000n; // 100 CKB to send
const feeCapacity = 1000n;           // 0.00001 CKB fee
const minChangeCapacity = 6100000000n; // 61 CKB minimum for change

const totalRequired = outputCapacity + feeCapacity;
const safeRequired = totalRequired + minChangeCapacity; // Always assume change needed
```

### 2. Cell Collection Loop
```typescript
// Pattern: Collect until requirement met, with error handling
let collected: Cell[] = [];
let totalCapacity = 0n;

for await (const cell of cellCollector.collect()) {
    collected.push(cell);
    totalCapacity += getCellCapacity(cell);
    
    if (totalCapacity >= required) {
        break; // Found enough
    }
}

if (totalCapacity < required) {
    throw new Error("Insufficient capacity available");
}
```

### 3. Change Cell Validation
```typescript
// Pattern: Ensure change meets minimum capacity requirements
const change = inputCapacity - outputCapacity - fee;

if (change > 0n && change < MIN_CELL_CAPACITY) {
    // Need to collect more to make valid change cell
    return await collectAdditionalCapacity(MIN_CELL_CAPACITY - change);
}
```

## Recommendations for Cell Collection

### Preferred Approach: Use CCC Built-ins
```typescript
// Best practice - let CCC handle complexity
const tx = ccc.Transaction.from({
  outputs: [{ lock: toLock, capacity: amount }],
});
await tx.completeInputsByCapacity(signer); // Automatic cell collection
await tx.completeFeeBy(signer);            // Automatic fee management
```

### When to Use Manual Collection
- Custom cell selection algorithms needed
- Complex multi-asset transactions
- Specific UTXO management requirements
- Advanced DeFi applications

### Migration from Lumos
If migrating from Lumos-based cell collection:

```typescript
// Old Lumos approach (complex)
const indexer = new Indexer(nodeUrl);
const cellCollector = new CellCollector(indexer, query);
// ... manual cell collection logic

// New CCC approach (simple)
await tx.completeInputsByCapacity(signer);
```

## When to Use This Pattern

### Use CCC Automatic (Recommended):
- **Most applications** - CCC handles the complexity
- Simple transfers and payments
- Standard wallet functionality
- Rapid development needs

### Use Manual Collection (Advanced):
- **Custom cell selection algorithms** required
- **Multi-input transactions** requiring optimization
- **DApp backends** with specific UTXO strategies
- **Production applications** needing fine-grained control

## Integration Example (CCC Recommended)
```typescript
// Modern CCC approach - simple and reliable
import { ccc } from "@ckb-ccc/ccc";

const signer = new ccc.SignerCkbPrivateKey(client, privateKey);
const { script: toLock } = await ccc.Address.fromString(toAddress, client);

const tx = ccc.Transaction.from({
    outputs: [{ lock: toLock, capacity: ccc.fixedPointFrom(amount) }],
});

// CCC handles all the complexity automatically
await tx.completeInputsByCapacity(signer);
await tx.completeFeeBy(signer);

const txHash = await signer.sendTransaction(tx);
```

## Legacy Integration (Lumos - For Reference)
```typescript
// Old Lumos approach - more complex but still functional
const service = new CellCollectionService(indexer, new CellSelector(indexer));
const result = await service.collectForTransaction(/*...*/);
// ... manual transaction building
```

**Recommendation**: Use CCC for new projects. The automatic cell collection provides robust, production-ready functionality with much less complexity than manual approaches.