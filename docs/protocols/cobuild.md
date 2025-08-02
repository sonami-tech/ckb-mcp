# CKB Transaction CoBuild Protocol

## Description

Collaborative transaction construction protocol for CKB enabling multi-party transaction building with standardized procedures. Covers message signing (EIP712-like), script interfaces with actions, composable witness layouts, BuildingPacket structures, validation rules, hash calculation methods, and migration strategies for enhanced UX and composability.

The CKB Transaction CoBuild Protocol enables collaborative construction of CKB transactions by multiple parties, providing standardized procedures for transaction building and signing with enhanced user experience and composability.

## Overview

CoBuild addresses key challenges in CKB application development:
- **Message Signing**: User-friendly signing standard similar to EIP712
- **Script Interface**: Standardized on-chain script interface with actions
- **Witness Layout**: Composable transaction witness layout replacing WitnessArgs
- **Collaborative Building**: Standard procedures and data formats for multi-party transaction construction

## Core Concepts

### Roles

The collaborative transaction building process involves four key roles:

- **Builder**: Provides basic transaction data (inputs, outputs, messages, witness data)
- **Asset Manager**: Manages CKB assets (CKByte, xUDT, Spore), provides asset operations
- **Fee Manager**: Analyzes on-chain fees, sets transaction fee/feerate
- **Signer**: Manages keypairs and signs transactions or messages

### Basic Flow

1. User initiates operation in CKB app
2. App generates Message based on user action
3. App builds Transaction reflecting the Message
4. App creates BuildingPacket containing transaction data
5. App sends BuildingPacket to wallet for signing
6. Wallet presents Message to user for confirmation
7. Upon approval, wallet signs and broadcasts transaction

## Data Structures

### Core Types

```rust
// Basic types
type Byte32 = [u8; 32];
type Hash = [u8; 32];
type ByteVec = Vec<u8>;
type String = Vec<u8>; // UTF-8 encoded

// Action represents a specific operation
struct Action {
    script_info_hash: Byte32,  // Script info hash
    script_hash: Byte32,       // Script hash
    data: ByteVec,             // Action data
}

// Message contains user actions
struct Message {
    actions: Vec<Action>,
}

// Script information
struct ScriptInfo {
    name: String,
    url: String,
    script_hash: Byte32,
    schema: String,
    message_type: String,
}
```

### BuildingPacket

The core data structure for transaction construction:

```rust
struct BuildingPacketV1 {
    // User operations as actions
    message: Message,
    
    // CKB transaction data
    payload: Transaction,
    
    // Cell data for transaction inputs
    resolved_inputs: ResolvedInputs,
    
    // Optional change output index for fee adjustment
    change_output: Option<u32>,
    
    // Script information for actions
    script_infos: Vec<ScriptInfo>,
    
    // Temporary data for lock scripts (e.g., multisig)
    lock_actions: Vec<Action>,
}
```

### WitnessLayout

Replaces WitnessArgs with a more composable structure:

```rust
enum WitnessLayout {
    SighashAll {
        seal: ByteVec,     // User signature
        message: Message,  // Transaction message
    },
    SighashAllOnly {
        seal: ByteVec,     // Signature only
    },
    Otx { /* ... */ },     // Open transaction
    OtxStart { /* ... */ }, // Open transaction start
}
```

## Implementation Examples

### Spore NFT Integration

Example Message schema for Spore NFT operations:

```rust
enum SporeAction {
    Mint {
        id: Byte32,
        to: Address,
        content_hash: Byte32,
    },
    Transfer {
        nft_id: Byte32,
        from: Option<Address>,
        to: Option<Address>,
    },
    Melt {
        id: Byte32,
    },
}
```

### Transaction Structure

Example Spore mint transaction using CoBuild:

```
inputs:
  - capacity: 1000 CKB
    lock: JoyID lock A
    
outputs:
  - capacity: 800 CKB
    lock: JoyID lock B
    type: Spore type script
    data: Spore data
  - capacity: 199 CKB (change)
    lock: JoyID lock A
    
witnesses:
  - WitnessLayout::SighashAll {
      seal: Signature for JoyID lock A,
      message: Message with Mint action
    }
```

## Validation Rules

### Lock Script Validation

1. Extract Message from witness
2. Calculate signing message hash according to CoBuild spec
3. Verify signature against calculated hash
4. Validate Message actions match transaction effects

### Type Script Validation

1. Extract Message from WitnessLayout
2. Verify Message matches transaction state transitions
3. Validate action data corresponds to script behavior
4. Ensure all required fields are present

## Hash Calculation

CoBuild defines specific hash calculation methods for signing:

1. Serialize transaction without signatures
2. Include Message in hash calculation
3. Apply script-specific hash transformations
4. Generate final signing message hash

## Benefits

- **User Experience**: Clear action presentation instead of raw transaction data
- **Composability**: Multiple scripts can collaborate in single transaction
- **Automation**: Standard procedures enable tool automation
- **Security**: Structured validation prevents common vulnerabilities
- **Flexibility**: Supports both CoBuild-only and legacy compatibility modes

## Migration Strategy

Scripts can support both CoBuild and legacy WitnessArgs:

1. Check witness format (WitnessLayout vs WitnessArgs)
2. Apply appropriate validation logic
3. Maintain backward compatibility
4. Gradually migrate to CoBuild-only mode

## Best Practices

- Always validate Message matches transaction effects
- Use descriptive action names and clear schemas
- Implement proper error handling for malformed data
- Support incremental adoption with legacy compatibility
- Document ScriptInfo thoroughly for developers