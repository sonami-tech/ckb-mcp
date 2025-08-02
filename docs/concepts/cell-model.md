# CKB Cell Model

## Description

This document provides a comprehensive introduction to the CKB Cell Model, the foundational data structure of the Nervos CKB blockchain. It explains how CKB generalizes Bitcoin's UTXO model to enable flexible smart contract programming through cells that contain capacity, lock scripts, type scripts, and arbitrary data. The guide covers cell structure, state management, transaction model, conservation rules, and programming patterns. It also discusses key advantages like parallel processing and deterministic execution, common use cases including tokens and DeFi applications, and the deposit-based storage economics where 1 CKB equals 1 byte of storage. This is essential reading for developers building on CKB who need to understand how to design, create, and manage cells effectively.

## Overview

The CKB Cell Model is a generalized version of Bitcoin's UTXO (Unspent Transaction Output) model that enables more flexible programming patterns and state management on CKB blockchain.

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
- **Purpose**: Determines how much data the cell can store
- **Economics**: 1 CKB token = 1 byte of storage space
- **Requirement**: Must be sufficient to store lock, type, and data
- **Minimum**: 61 CKBytes for an empty cell

#### State Rent and Storage Economics
CKB operates on a **deposit-based storage model**, not a "pay once, store forever" system:
- **Deposit Required**: Each byte of data storage requires 1 CKB as a deposit
- **Locked Funds**: The deposit remains locked while the data exists in the blockchain state
- **Refundable**: When data is removed (cell consumed), the deposit is returned to the owner
- **Inflation Impact**: Over time, the deposit's value decreases slightly due to CKB's inflation mechanism, which effectively pays for the state rent
- **Contract Design**: Smart contracts must account for capacity requirements and deposit economics when managing state

### Lock Script
- **Purpose**: Controls who can consume this cell as input
- **Function**: Similar to Bitcoin's scriptPubKey
- **Common types**: Secp256k1 signatures, multi-sig, time locks
- **Execution**: Runs when cell is used as transaction input

### Type Script (Optional)
- **Purpose**: Validates the data stored in the cell
- **Function**: Enforces rules about cell data format and state transitions
- **Use cases**: Tokens, smart contracts, complex state validation
- **Execution**: Runs when cell is created or destroyed

### Data
- **Purpose**: Stores arbitrary application data
- **Format**: Raw bytes, can contain any information
- **Examples**: Token amounts, contract state, user data
- **Limit**: Constrained by cell capacity

## Cell States

### Live Cells
- **Status**: Unspent, available for consumption
- **Usage**: Can be referenced as transaction inputs
- **Index**: Tracked by CKB indexer for efficient queries

### Dead Cells
- **Status**: Already consumed/spent
- **Usage**: Cannot be used as inputs (only historical reference)
- **Purpose**: Maintain transaction history and verification

## UTXO vs Cell Model

| Aspect | Bitcoin UTXO | CKB Cell |
|--------|--------------|-----------|
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

### 1. **Parallel Processing**
- Independent cells can be processed simultaneously
- No global state contention like account-based models

### 2. **Deterministic Execution**
- Cell states are immutable once created
- Predictable transaction outcomes

### 3. **Flexible Storage**
- Store any data structure in cell data field
- Capacity-based storage economics

### 4. **Composability**
- Mix different lock and type scripts
- Create complex applications from simple components

### 5. **Security**
- Each cell protected by cryptographic lock script
- Type scripts prevent invalid state transitions

## Common Use Cases

- **Cryptocurrencies**: Store token balances and transfer rules
- **NFTs**: Store unique digital assets and metadata
- **DeFi**: Implement decentralized exchanges and lending protocols
- **Identity**: Store verifiable credentials and identity data
- **Gaming**: Manage in-game assets and state
- **Supply Chain**: Track goods and verify authenticity

## Development Workflow

1. **Design**: Define cell structure and scripts
2. **Implement**: Write lock and type scripts
3. **Test**: Verify script logic and state transitions
4. **Deploy**: Publish scripts to CKB network
5. **Interact**: Create transactions that use your cells

The Cell Model provides a powerful foundation for building decentralized applications with flexible state management and strong security guarantees.