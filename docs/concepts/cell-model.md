## Description

CKB's cell model: the generalized UTXO that underpins all on-chain state. Covers cell structure (capacity, lock, type, data), live/dead states, UTXO comparison, transaction model with conservation rules, and programming patterns for state storage, tokens, and contract deployment. Also covers capacity management and storage economics, script execution context for lock and type scripts, advanced transaction patterns (multi-input aggregation, state updates, batch operations), advanced cell patterns (shared state, proxy/delegate, time-locked cells), and cell collection optimization.

## Basic Cell Structure

Every cell in CKB contains four components:

```typescript
interface Cell {
  capacity: number;    // Storage space in CKBytes (1 CKB = 1 byte storage)
  lock: Script;        // Defines who can unlock/spend this cell
  type?: Script;       // Optional script that validates cell data
  data: Bytes;         // Arbitrary data stored in the cell
}
```

### Capacity
- 1 CKB token = 1 byte of storage space
- Must be sufficient to store lock, type, and data
- Minimum: 61 CKBytes for an empty cell

#### State Rent and Storage Economics
CKB uses a **deposit-based storage model**:
- Each byte requires 1 CKB as deposit
- Deposit remains locked while data exists
- Deposit returned when cell consumed
- Inflation mechanism effectively pays for state rent
- Smart contracts must account for capacity requirements

**Real-world Storage Costs**:
- Chinese novel (780K words): ~1.56M CKB tokens
- High-resolution image (1MB): 1M CKB tokens
- Smart contract code (10KB): 10K CKB tokens

For detailed capacity calculation formulas and lifecycle management, see [Cell Lifecycle](ckb://docs/concepts/cell-lifecycle).

### Lock Script
- Controls who can consume this cell as input
- Similar to Bitcoin's scriptPubKey
- Common types: Secp256k1 signatures, multi-sig, time locks
- Runs when cell is used as transaction input

For a detailed understanding of how lock scripts are generated from private keys and the complete transformation chain (Private Key -> Public Key -> Lock Arg -> Lock Script -> Lock Hash -> Address), see the [Lock Value Relationships](ckb://docs/concepts/lock-values) guide.

### Type Script (Optional)
- Validates the data stored in the cell
- Enforces rules about cell data format and state transitions
- Use cases: Tokens, smart contracts, complex state validation
- Runs when cell is created or destroyed

### Data
- Stores arbitrary application data as raw bytes
- Examples: Token amounts, contract state, user data
- Constrained by cell capacity

## Cell States

### Live Cells
- Unspent, available for consumption
- Can be referenced as transaction inputs
- Tracked by CKB indexer for efficient queries

### Dead Cells
- Already consumed/spent
- Cannot be used as inputs (only historical reference)
- Maintain transaction history and verification

## UTXO vs Cell Model

| **Aspect** | **Bitcoin UTXO** | **CKB Cell** |
| **Data Storage** | Limited (scriptPubKey) | Arbitrary data via data field |
| **Programmability** | Basic scripting | Full Turing-complete scripts |
| **State Management** | Stateless | Stateful via type scripts |
| **Flexibility** | Fixed structure | Flexible with lock + type + data |

## Transaction Model

### Inputs and Outputs
```rust
// Transaction structure
struct Transaction {
    inputs: Vec<CellInput>,      // References to existing live cells
    outputs: Vec<CellOutput>,    // New cells being created
    outputs_data: Vec<Bytes>,    // Data for each output cell
    witnesses: Vec<Bytes>,       // Signatures and proofs
}
```

### Conservation Rules
1. **Capacity Conservation**: Total input capacity >= total output capacity
2. **Data Validation**: Type scripts validate state transitions
3. **Authorization**: Lock scripts authorize cell consumption

```typescript
// Capacity conservation validation
const validateCapacityConservation = (tx: Transaction) => {
  const inputCapacity = tx.inputs.reduce((sum, input) =>
    sum + getInputCell(input).capacity, 0n);

  const outputCapacity = tx.outputs.reduce((sum, output) =>
    sum + output.capacity, 0n);

  const txFee = inputCapacity - outputCapacity;

  // Fee must be non-negative
  return txFee >= 0n;
};
```

## Script Execution Context

### Lock Script Validation
```typescript
// Lock script determines spending conditions
interface LockScriptExecution {
  // Available during execution:
  currentTransaction: Transaction;    // Full transaction context
  inputIndex: number;                // Which input is being validated
  witness: HexString;                // Associated witness data

  // Validation result:
  isValid: boolean;                  // Must return true to allow spending
}
```

### Type Script Validation
```typescript
// Type script enforces data validation rules
interface TypeScriptExecution {
  // Validates both inputs and outputs with same type script
  inputCells: Cell[];               // All inputs with this type
  outputCells: Cell[];              // All outputs with this type

  // Common validation patterns:
  // - Token supply conservation
  // - State transition rules
  // - Data format requirements
}
```

## Programming Patterns

### State Storage
```typescript
// Store application state in cell data
const stateCell = {
  capacity: 1000,
  lock: userLockScript,
  type: contractTypeScript,
  data: encodeState({ balance: 100, nonce: 5 })
};
```

### Asset Representation
```typescript
// User Defined Token (UDT)
const tokenCell = {
  capacity: 144,
  lock: ownerLockScript,
  type: udtTypeScript,
  data: encodeAmount(1000000) // Token amount
};
```

### Contract Deployment
```typescript
// Deploy contract code
const contractCell = {
  capacity: codeSize + 61,
  lock: deployerLockScript,
  type: null,
  data: contractBinary
};
```

## Advanced Transaction Patterns

### Multi-Input Aggregation
```typescript
// Combining multiple small cells into fewer larger cells
const aggregateSmallCells = async (userCells: Cell[]) => {
  const transaction = {
    inputs: userCells.map(cell => ({
      previousOutput: cell.outPoint,
      since: "0x0"
    })),

    outputs: [{
      capacity: userCells.reduce((sum, cell) =>
        sum + BigInt(cell.capacity), 0n),
      lock: userCells[0].lock, // Same owner
      type: null,
      data: "0x" // Empty data
    }],

    witnesses: [/* signature data */]
  };

  return transaction;
};
```

### State Update Pattern
```typescript
// Updating cell data while preserving ownership
const updateCellData = async (
  originalCell: Cell,
  newData: string
) => {
  const updatedCell: Cell = {
    capacity: originalCell.capacity,
    lock: originalCell.lock,        // Same ownership
    type: originalCell.type,        // Same validation
    data: hexify(newData)           // New data
  };

  // Ensure capacity still sufficient
  if (!validateCellCapacity(updatedCell)) {
    throw new Error("Insufficient capacity for new data");
  }

  return buildUpdateTransaction(originalCell, updatedCell);
};
```

## Advanced Cell Patterns

### Shared State Cells
```typescript
// Global state cell for protocol-level data
interface SharedStateCell extends Cell {
  // Multiple users can reference but only authorized can modify
  lock: ProtocolOwnerScript;        // Protocol-controlled
  type: StateValidationScript;      // Enforces state rules
  data: EncodedProtocolState;       // Shared protocol state
}
```

### Proxy and Delegate Patterns
```typescript
// Cell that references code stored elsewhere
interface ProxyCell extends Cell {
  lock: StandardLockScript;         // User ownership
  type: ProxyTypeScript;           // Points to actual logic
  data: {
    targetCodeHash: HexString;      // Reference to actual code
    parameters: EncodedArgs;        // Configuration parameters
  }
}
```

### Time-Locked Cells
```typescript
// Cells that can only be spent after specific conditions
interface TimeLockCell extends Cell {
  lock: {
    codeHash: TimeLockScriptHash;
    args: {
      unlockTime: Timestamp;        // When cell becomes spendable
      ownerPubkeyHash: HexString;   // Who can spend after unlock
    }
  };
}
```

## Transaction Optimization

### Cell Collection Optimization
```typescript
const optimizedCellSelection = (
  availableCells: Cell[],
  targetCapacity: bigint
) => {
  // Sort by capacity to minimize fragmentation
  const sortedCells = availableCells.sort((a, b) =>
    Number(BigInt(b.capacity) - BigInt(a.capacity))
  );

  // Use greedy algorithm for optimal selection
  const selected: Cell[] = [];
  let remaining = targetCapacity;

  for (const cell of sortedCells) {
    const capacity = BigInt(cell.capacity);
    if (capacity <= remaining) {
      selected.push(cell);
      remaining -= capacity;

      if (remaining === 0n) break;
    }
  }

  return selected;
};
```

### Batch Operations
```typescript
// Process multiple operations in single transaction
const batchTransfer = async (transfers: TransferRequest[]) => {
  const transaction = {
    inputs: collectAllRequiredInputs(transfers),
    outputs: [
      ...transfers.map(createTransferOutput),
      ...generateChangeOutputs(transfers)
    ],
    witnesses: generateBatchWitnesses(transfers)
  };

  return optimizeTransaction(transaction);
};
```

## Key Advantages

- **Parallel Processing**: Independent cells can be processed simultaneously
- **Deterministic Execution**: Cell states are immutable once created
- **Flexible Storage**: Store any data structure in cell data field
- **Composability**: Mix different lock and type scripts
- **Security**: Each cell protected by cryptographic lock script

## Common Use Cases

- Store token balances and transfer rules
- Store unique digital assets and metadata
- Implement decentralized exchanges and lending protocols
- Store verifiable credentials and identity data
- Manage in-game assets and state
- Track goods and verify authenticity

## Development Workflow

1. **Design**: Define cell structure and scripts
2. **Implement**: Write lock and type scripts
3. **Test**: Verify script logic and state transitions
4. **Deploy**: Publish scripts to CKB network
5. **Interact**: Create transactions that use your cells
