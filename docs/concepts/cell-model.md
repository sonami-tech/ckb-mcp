# Cell Model

## Description

A Cell is a single piece of state stored on the blockchain, governed by scripts (smart contracts) that dictate the rules under which it can be changed. This mechanism is the foundation of all programmable functionality on CKB, including things such as tokens, NFTs, and DeFi. CKB generalizes Bitcoin's UTXO model with cells containing capacity, lock scripts, type scripts, and arbitrary data. Covers cell structure, state management, transaction model, conservation rules, programming patterns, parallel processing, deterministic execution, and deposit-based storage economics where 1 CKB equals 1 byte of storage.

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

### Lock Script
- Controls who can consume this cell as input
- Similar to Bitcoin's scriptPubKey
- Common types: Secp256k1 signatures, multi-sig, time locks
- Runs when cell is used as transaction input

For a detailed understanding of how lock scripts are generated from private keys and the complete transformation chain (Private Key → Public Key → Lock Arg → Lock Script → Lock Hash → Address), see the [Lock Value Relationships](ckb://docs/concepts/lock-value-relationships) guide.

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
1. **Capacity Conservation**: Total input capacity ≥ total output capacity
2. **Data Validation**: Type scripts validate state transitions
3. **Authorization**: Lock scripts authorize cell consumption

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

