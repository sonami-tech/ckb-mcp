# CKB Open Transaction (OTX) Protocol

## Description

Off-chain transaction construction protocol enabling collaborative assembly of CKB transactions through peer-to-peer networks. Covers partial transaction building, agent-based collection and relaying, batch processing, OTX/OtxStart witness layouts, validation rules, and integration with CoBuild for decentralized transaction coordination.

The Open Transaction (OTX) protocol enables off-chain construction and collaborative assembly of CKB transactions through a peer-to-peer network, allowing multiple parties to contribute to transaction building without direct coordination.

## Overview

OTX extends the CoBuild protocol to support:
- **Partial Transaction Building**: Create incomplete transactions that others can complete
- **P2P Transaction Assembly**: Agent-based network for OTX collection and relaying
- **Batch Processing**: Multiple OTXs packed into single CKB transactions
- **Decentralized Coordination**: No central authority required for transaction assembly

## Core Concepts

### Open Transaction Structure

An OTX shares the same structure as a CKB transaction but with special handling:

```rust
struct OpenTransaction {
    // Standard CKB transaction fields
    version: u32,
    cell_deps: Vec<CellDep>,
    header_deps: Vec<Byte32>,
    inputs: Vec<CellInput>,
    outputs: Vec<CellOutput>,
    outputs_data: Vec<Bytes>,
    witnesses: Vec<Bytes>,
    
    // OTX-specific handling in witnesses
    // Uses WitnessLayout::Otx or OtxStart variants
}
```

### Roles

- **OTX Creator**: Initiates partial transactions with specific intents
- **OTX Agent**: Collects, validates, and assembles OTXs into complete transactions
- **OTX Relayer**: Propagates OTXs through the P2P network

## OTX Workflow

### 1. Creation Phase

OTX creators build partial transactions:

```rust
// Example: Create OTX for token swap
let otx = OpenTransaction {
    inputs: vec![user_token_cell],
    outputs: vec![expected_output],
    witnesses: vec![
        WitnessLayout::OtxStart {
            start_input_cell: 0,
            start_output_cell: 0,
            start_cell_deps: 0,
            start_header_deps: 0,
        }
    ],
};
```

### 2. Propagation Phase

OTXs spread through the P2P network:
- Creators broadcast to connected agents
- Agents validate and relay to peers
- Network maintains OTX pool for assembly

### 3. Assembly Phase

Agents combine compatible OTXs:

```rust
// Batch multiple OTXs into single transaction
let combined_tx = Transaction {
    inputs: otx1.inputs + otx2.inputs + agent_inputs,
    outputs: otx1.outputs + otx2.outputs + agent_outputs,
    witnesses: transform_otx_witnesses(otx_witnesses),
};
```

## Witness Layout for OTX

### OtxStart

Marks the beginning of an OTX in combined transaction:

```rust
struct OtxStart {
    start_input_cell: u32,    // First input index for this OTX
    start_output_cell: u32,   // First output index for this OTX
    start_cell_deps: u32,     // First cell_dep index
    start_header_deps: u32,   // First header_dep index
}
```

### Otx

Contains the actual OTX witness data:

```rust
struct Otx {
    input_cells: u32,     // Number of inputs in this OTX
    output_cells: u32,    // Number of outputs in this OTX
    cell_deps: u32,       // Number of cell dependencies
    header_deps: u32,     // Number of header dependencies
    message: Message,     // CoBuild message for this OTX
}
```

## Batch Processing Example

Multiple OTXs combined into single transaction:

```
Transaction:
  inputs:
    0-1: OTX1 inputs (2 cells)
    2-3: OTX2 inputs (2 cells)
    4: Agent fee input
    
  outputs:
    0-1: OTX1 outputs
    2: OTX2 output
    3: Agent fee output
    
  witnesses:
    0: OtxStart for OTX1
    1: Otx data for OTX1
    2: OtxStart for OTX2
    3: Otx data for OTX2
    4: Agent signature
```

## Validation Rules

### OTX Validation

1. Verify OTX structure integrity
2. Check input cells exist and are unspent
3. Validate witness format follows OTX rules
4. Ensure Message matches intended operations

### Assembly Validation

1. Verify OTXs don't conflict (no double-spending)
2. Calculate correct index offsets for combination
3. Maintain OTX isolation in final transaction
4. Validate total fees cover network requirements

## Benefits

- **Composability**: Different applications can contribute to same transaction
- **Efficiency**: Batch processing reduces on-chain overhead
- **Flexibility**: Partial transactions enable complex multi-party operations
- **Decentralization**: No central coordinator required

## Security Considerations

- OTX creators must trust agents for fair inclusion
- Agents should validate OTXs before assembly
- Time limits prevent indefinite OTX pending
- Proper fee mechanisms incentivize honest behavior

## Implementation Guidelines

### For OTX Creators

1. Build partial transaction with clear intent
2. Include sufficient fee contribution
3. Set reasonable expiration time
4. Sign with appropriate witness layout

### For OTX Agents

1. Maintain OTX pool with validation
2. Implement fair selection algorithms
3. Optimize batch assembly for efficiency
4. Handle conflicts and failures gracefully

## Integration with CoBuild

OTX builds on CoBuild foundations:
- Uses same Message and Action structures
- Extends WitnessLayout with OTX variants
- Maintains compatibility with ScriptInfo
- Leverages BuildingPacket for complex assembly