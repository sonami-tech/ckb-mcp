## Description

Layer-1.5 account-based token management solution using Sparse Merkle Trees for ultra-low-cost NFT/FT operations on CKB. Covers SMT key-value structures, two-step transfer model, registration workflows, batch operations, infrastructure requirements, aggregator services, development patterns, and production deployment strategies.

## Related Resources

- [ckb://docs/cota/development](ckb://docs/cota/development) - Build cost-effective NFT applications using CoTA protocol
- [ckb://docs/cota/sdk-examples](ckb://docs/cota/sdk-examples) - CoTA SDK JavaScript implementation guide with production-ready examples
- [ckb://docs/cota/infrastructure](ckb://docs/cota/infrastructure) - Comprehensive deployment guide for CoTA infrastructure

## Protocol Overview

CoTA (Compact Token Aggregator) is a layer-1.5 **account-based fungible/non-fungible token and key-value data management solution** on Nervos CKB. The protocol uses Sparse Merkle Trees (SMT) to manage enormous amounts of data with constant on-chain storage space (32 bytes).

### Key Advantages
- **Extremely low cost**: Batch operations for NFTs with minimal on-chain footprint
- **Account model**: Provides account-like experience on UTXO-based CKB
- **Two-step transfers**: Eliminates security risks of simultaneous sender/receiver updates
- **Unlimited scalability**: SMT compression allows handling millions of tokens

## Core Architecture

### CoTA Cell Structure
```yaml
# CoTA cell data structure
data:
    version: byte
    smt_root: byte32          # All user data compressed into 32 bytes
type:
    code_hash: cota_type
    args: lockscript_hash[0..20]  # Must match cell lock script
lock:
    # User's lock script
```

### Two-Step Operation Design
One of CoTA's innovations is the two-step transfer model that solves the simultaneous update problem:

1. **Withdrawal**: Sender removes token from their SMT and creates withdrawal proof
2. **Claim**: Receiver adds token to their SMT using the withdrawal proof

This eliminates the need for anyone-can-pay locks and their associated security risks.

```
Sender SMT:     [Token A] → [Withdrawal Proof] → [Empty]
                     ↓
Receiver SMT:   [Empty] → [Claim Token A] → [Token A]
```

## SMT Key-Value Structure

CoTA uses a standardized key-value structure in the SMT with different types identified by 2-byte prefixes:

### NFT Operations
| Type | 1st Byte | 2nd Byte | Description |
|------|----------|----------|-------------|
| cota-NFT-define | 0x81 | 0x00 | NFT collection definition |
| cota-NFT-hold | 0x81 | 0x01 | Currently owned NFTs |
| cota-NFT-withdrawal | 0x81 | 0x02 | NFT withdrawal records |
| cota-NFT-claim | 0x81 | 0x03 | NFT claim records |

### Fungible Token Operations
| Type | 1st Byte | 2nd Byte | Description |
|------|----------|----------|-------------|
| cota-FT-define | 0x82 | 0x00 | FT collection definition |
| cota-FT-hold | 0x82 | 0x01 | Currently held FT amounts |
| cota-FT-withdrawal | 0x82 | 0x02 | FT withdrawal records |
| cota-FT-claim | 0x82 | 0x03 | FT claim records |

## Development Workflow

### 1. Registration (Required First Step)
Every user must register exactly one CoTA cell before any operations:

```typescript
import { CotaSDK } from "@nervina-labs/cota-sdk-js";

// Register CoTA cell - only done once per address
const registerTx = await cotaSDK.registry.build({
  lockScript: userLockScript,
  // Registration parameters
});
```

### 2. Collection Definition (For Issuers)
Define NFT or FT collections with metadata:

```typescript
// Define NFT collection
const defineTx = await cotaSDK.define.build({
  lockScript: issuerLockScript,
  cotaInfo: {
    name: "My NFT Collection",
    image: "https://example.com/collection.jpg",
    description: "A unique NFT collection",
    // Additional metadata fields
  }
});
```

### 3. Minting Operations
Mint tokens to recipients:

```typescript
// Mint NFTs to multiple recipients
const mintTx = await cotaSDK.mint.build({
  lockScript: issuerLockScript,
  cotaId: collectionId,
  recipients: [
    {
      toLockScript: recipient1Lock,
      tokenIndex: 1,
      state: "0x00",
      characteristic: "0x..." // Custom NFT data
    },
    // Additional recipients...
  ]
});
```

### 4. Transfer Operations

#### Option A: Two-Step Transfer (Withdrawal + Claim)
```typescript
// Step 1: Sender withdraws NFT
const withdrawTx = await cotaSDK.withdraw.build({
  lockScript: senderLockScript,
  cotaId: collectionId,
  tokenIndex: 1,
  toLockScript: receiverLockScript
});

// Step 2: Receiver claims NFT (can be done anytime)
const claimTx = await cotaSDK.claim.build({
  lockScript: receiverLockScript,
  withdrawalProof: withdrawalProofFromAggregator
});
```

#### Option B: Direct Transfer (Combines Both Steps)
```typescript
// Direct transfer for immediate ownership change
const transferTx = await cotaSDK.transfer.build({
  lockScript: currentOwnerLock,
  cotaId: collectionId,
  tokenIndex: 1,
  toLockScript: newOwnerLock
});
```

### 5. Update Operations
Modify NFT characteristics after owning them:

```typescript
const updateTx = await cotaSDK.update.build({
  lockScript: ownerLockScript,
  cotaId: collectionId,
  tokenIndex: 1,
  state: "0x01", // New state
  characteristic: "0x..." // Updated characteristics
});
```

## Infrastructure Requirements

### Aggregator Services
CoTA requires aggregator services to manage SMT operations:

```typescript
// Testnet endpoints
const aggregatorConfig = {
  registryAggregator: "https://cota.nervina.dev/registry-aggregator",
  cotaAggregator: "https://cota.nervina.dev/aggregator",
  ckbNode: "https://testnet.ckbapp.dev/rpc",
  ckbIndexer: "https://testnet.ckbapp.dev/indexer"
};
```

### Global Registry
- Ensures one CoTA cell per address to prevent double-spending.
- Maintains global SMT of all registered addresses.
- Provides registration proofs for new users.

## Advanced Features

### Batch Operations
Process multiple tokens in single transactions:

```typescript
// Batch mint multiple NFTs
const batchMintTx = await cotaSDK.mint.build({
  lockScript: issuerLockScript,
  cotaId: collectionId,
  recipients: Array.from({length: 100}, (_, i) => ({
    toLockScript: recipientLocks[i],
    tokenIndex: i + 1
  }))
});
```

### Extension Data Storage
Store arbitrary key-value data in CoTA cells:

```yaml
# Extension data type
extension-data: 0xF0, 0x00  # User-defined data storage
```

### Fungible Token Support
Handle fungible tokens with decimal precision:

```typescript
// Define fungible token
const ftDefineTx = await cotaSDK.defineFT.build({
  lockScript: issuerLockScript,
  total: 1000000n, // Total supply
  decimal: 8,      // Decimal places
  name: "My Token",
  symbol: "MTK"
});
```

## Error Handling

### Common Error Codes
```typescript
// Registry errors
const REGISTRY_ERRORS = {
  ALREADY_REGISTERED: "Address already has CoTA cell",
  INVALID_LOCK_HASH: "Lock hash not in registry SMT"
};

// Operation errors  
const OPERATION_ERRORS = {
  INSUFFICIENT_BALANCE: "Not enough tokens to withdraw",
  INVALID_WITHDRAWAL: "Withdrawal proof invalid or already claimed",
  TOKEN_NOT_FOUND: "Token not found in hold leaves"
};
```

### Error Recovery Patterns
```typescript
try {
  const tx = await cotaSDK.transfer.build(transferParams);
  await tx.send();
} catch (error) {
  if (error.code === "INSUFFICIENT_BALANCE") {
    // Handle insufficient balance
    console.log("Need to claim pending tokens first");
  } else if (error.code === "INVALID_WITHDRAWAL") {
    // Handle invalid withdrawal
    console.log("Withdrawal may have been claimed already");
  }
}
```

## Production Deployment

### Self-Hosted Aggregators
For production applications, consider running your own aggregator services:

```yaml
# Docker compose for CoTA infrastructure
version: '3'
services:
  cota-aggregator:
    image: nervinalabs/cota-aggregator
    environment:
      - CKB_NODE_URL=http://ckb-node:8114
      - DATABASE_URL=postgresql://...
  
  registry-aggregator:
    image: nervinalabs/cota-registry-aggregator
    environment:
      - CKB_NODE_URL=http://ckb-node:8114
      - DATABASE_URL=postgresql://...
```

### Performance Optimization
- **Batch operations**: Group multiple token operations into single transactions
- **Proof caching**: Cache SMT proofs to reduce aggregator calls
- **Connection pooling**: Reuse connections to CKB node and indexer

## Use Cases and Applications

### NFT Marketplaces
- **Mass minting**: Create thousands of NFTs with minimal on-chain cost
- **Instant transfers**: Two-step model enables immediate transfer confirmation
- **Rich metadata**: Store extensive NFT characteristics on-chain

### Gaming Assets
- **In-game items**: Represent game items as NFTs with mutable characteristics
- **Achievement tokens**: Issue achievement NFTs to players
- **Economic systems**: Create fungible game currencies

### DeFi Applications
- **Loyalty tokens**: Issue FTs representing loyalty points or rewards
- **Governance tokens**: Create voting tokens with account-based management
- **Yield farming**: Distribute reward tokens efficiently

### Identity and Credentials
- **Digital certificates**: Issue verifiable credentials as NFTs
- **Membership tokens**: Create membership or access tokens
- **Professional credentials**: Issue professional certifications

## Migration and Integration

### From Other Standards
When migrating from other token standards:

```typescript
// Migration helper for SUDT to CoTA
const migrationHelper = {
  async migrateFromSUDT(sudtCells) {
    // 1. Register CoTA cell if not exists
    await this.ensureRegistered();
    
    // 2. Define equivalent FT collection
    const cotaId = await this.defineFTCollection({
      total: calculateTotalFromSUDT(sudtCells),
      decimal: 0 // SUDT typically uses no decimals
    });
    
    // 3. Mint equivalent amounts to users
    return this.batchMintFT(cotaId, sudtBalances);
  }
};
```

### Integration Patterns
```typescript
// Integration with existing dApps
class CoTAIntegration {
  async integrateWithExistingApp() {
    // 1. Add CoTA cell check to user onboarding
    const hasCoTACell = await this.checkCoTARegistration(userAddress);
    if (!hasCoTACell) {
      await this.promptRegistration();
    }
    
    // 2. Replace direct token transfers with CoTA operations
    await this.replaceTransferLogic();
    
    // 3. Add claim flow for receiving tokens
    await this.addClaimFlow();
  }
}
```

CoTA protocol provides a comprehensive solution for scalable token management on CKB, offering significant cost savings and improved user experience compared to traditional UTXO-based token standards.