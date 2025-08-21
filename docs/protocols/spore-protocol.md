# Spore Protocol

## Description

Core protocol specification for Spore digital objects on CKB blockchain defining structure, validation rules, and interaction patterns for on-chain digital assets. Covers cluster creation, spore minting, content addressing, metadata management, and ownership transfer mechanisms. Includes technical specifications for content validation, data encoding, capacity requirements, and cross-compatibility with other CKB protocols. Essential reference for developers implementing Spore-compatible applications.

## Overview

The Spore Protocol is a comprehensive framework for creating, managing, and transferring fully on-chain digital objects (DOBs) on the CKB blockchain. Unlike traditional NFTs that rely on external storage, Spore assets store all data directly on-chain, providing permanent, censorship-resistant digital ownership.

## Core Concepts

### Digital Object Backing (DOB)

Spore implements true on-chain asset storage where:
- All content data is encoded directly on the blockchain.
- Assets have intrinsic value through CKB capacity requirements.
- Content is immutable and permanently accessible.
- No external dependencies for asset integrity.

### Value Mechanism

- **Capital Preservation**: CKBytes used for storage are locked, not spent
- **Melt to Redeem**: Holders can "melt" Spores to reclaim underlying CKB
- **Intrinsic Value**: Assets have baseline value from capacity requirements
- **Growth Potential**: Value increases as network utilization grows

## Cell Types

### Spore Cell

The fundamental unit storing digital content:

```yaml
data:
    content_type: Bytes    # MIME type (e.g., "image/png")
    content: Bytes         # Actual content data
    cluster_id: BytesOpt   # Optional cluster association
type:
    hash_type: "data1"
    code_hash: SPORE_TYPE_DATA_HASH
    args: SPORE_ID
lock:
    <user_defined>
```

#### Data Structure (Molecule Schema)

```rust
table SporeData {
    content_type: Bytes,
    content: Bytes,
    cluster_id: BytesOpt,
}
```

#### Key Properties

- **Content Type**: MIME type following RFC 2046 standard
  - Basic: `image/png`, `text/plain`, `application/json`
  - Extended: `image/png;immortal=true` (indestructible NFT)
- **Content**: Raw binary data of the digital asset
- **Cluster ID**: Optional reference to a Spore Cluster
- **Spore ID**: `hash(transaction.inputs[0] | output_index)`

### Spore Cluster Cell

Organizational structure for grouping related Spores:

```yaml
data:
    name: Bytes           # Cluster name
    description: Bytes    # Cluster description
type:
    hash_type: "data1"
    code_hash: CLUSTER_TYPE_DATA_HASH
    args: CLUSTER_ID
lock:
    <user_defined>
```

#### Data Structure (Molecule Schema)

```rust
table ClusterData {
    name: Bytes,
    description: Bytes,
}

// Version 2 with mutant support
table ClusterDataV2 {
    name: Bytes,
    description: Bytes,
    mutant_id: BytesOpt,
}
```

#### Key Properties

- **Indestructible**: Cannot be destroyed once created
- **Immutable**: Data cannot be modified after creation
- **Cluster ID**: `hash(transaction.inputs[0] | output_index)`
- **Public/Private**: Access control for Spore association

## Protocol Rules

### Spore Creation Rules

1. **Type Script Validation**: Must use official Spore type script
2. **ID Generation**: Spore ID derived from first input and output index
3. **Content Validation**: Content type must be valid MIME type
4. **Capacity Requirements**: Sufficient CKB to store all data

### Cluster Association Rules

When `cluster_id` is set in a Spore:

1. **Cluster Existence**: Referenced cluster must exist in CellDep
2. **Authorization**: One of the following must be true:
   - Cluster cell appears in both inputs and outputs (owner consent)
   - Lock proxy cell with same lock as cluster exists in inputs/outputs
3. **Lock Compatibility**: Cluster lock must be unlockable
4. **Consistency**: Same cluster args in outputs must have matching lock

### Transfer Rules

- **Zero-Fee Transfers**: Recipients don't need CKB for gas fees
- **Built-in Fuel**: Each Spore contains capacity for future operations
- **Ownership**: Standard CKB lock script validation applies
- **Atomic Operations**: Multiple Spores can be transferred atomically

## Advanced Features

### Immortal Spores

Spores marked with `immortal=true` parameter:
- Cannot be destroyed or melted.
- Permanent on-chain storage guaranteed.
- Higher capacity requirements for permanence.

### DOB/0 Protocol Integration

Enhanced Spores with decoder configuration:

```json
{
  "description": "Cluster description",
  "dob": {
    "ver": 0,
    "decoder": {
      "type": "code_hash",
      "hash": "0x..."
    },
    "pattern": []
  }
}
```

### Mutant Support

Extension mechanism for dynamic content:
- Mutant scripts can modify Spore content.
- Preserves original data integrity.
- Enables programmable NFT behavior.

## Capacity Economics

### Storage Costs

- **1 CKB = 1 Byte**: Direct storage cost relationship
- **Minimum Cell**: 61 CKB base requirement
- **Content Size**: Additional capacity for actual content
- **Overhead**: Type script, lock script, and metadata

### Economic Model

```
Total Capacity = Base (61 CKB) + Content Size + Metadata Size + Scripts
```

### Value Dynamics

- **Locked Capital**: CKB stored in cell, not spent
- **Redemption**: "Melt" to recover underlying CKB
- **Appreciation**: Value grows with network adoption
- **Utility**: Permanent storage provides intrinsic value

## Transaction Examples

### Single Spore Minting

```yaml
CellDep:
  - Spore Type Script Cell
Inputs:
  - Normal CKB Cell (capacity source)
Outputs:
  - Spore Cell:
      capacity: [content_size + overhead] CKB
      type: spore_type_script
      lock: owner_lock
      data: SporeData { content_type, content, cluster_id }
Witnesses:
  - Valid signature
```

### Cluster Creation

```yaml
CellDep:
  - Cluster Type Script Cell
Inputs:
  - Normal CKB Cell (capacity source)
Outputs:
  - Cluster Cell:
      capacity: [metadata_size + overhead] CKB
      type: cluster_type_script
      lock: owner_lock
      data: ClusterData { name, description }
Witnesses:
  - Valid signature
```

### Clustered Spore Creation

```yaml
CellDep:
  - Spore Type Script Cell
  - Target Cluster Cell
Inputs:
  - Normal CKB Cell (capacity source)
  - Cluster Cell (for authorization)
Outputs:
  - Spore Cell (with cluster_id set)
  - Cluster Cell (unchanged)
Witnesses:
  - Valid signatures
```

## Developer Integration

### SDK Usage

```typescript
import { createSpore, createCluster } from '@spore-sdk/api';

// Create cluster
const cluster = await createCluster({
  data: {
    name: 'My Collection',
    description: 'A collection of digital art'
  }
});

// Create spore in cluster
const spore = await createSpore({
  data: {
    contentType: 'image/png',
    content: imageBytes,
    clusterId: cluster.id
  }
});
```

### Contract Integration

```rust
use spore_types::{SporeData, ClusterData};

// Validate spore data
let spore_data: SporeData = load_data()?;
validate_content_type(&spore_data.content_type())?;
validate_content_size(&spore_data.content())?;

// Check cluster association
if let Some(cluster_id) = spore_data.cluster_id() {
    validate_cluster_authorization(cluster_id)?;
}
```

## Security Considerations

### Content Validation

- MIME type format validation.
- Content size limits.
- Malicious content detection.
- Resource consumption limits.

### Authorization

- Lock script security.
- Cluster permission model.
- Proxy authorization patterns.
- Multi-signature support.

### Economic Security

- Spam prevention through capacity costs.
- Value preservation mechanisms.
- Network resource utilization.
- Long-term sustainability.

## Protocol Extensions

### Custom Content Types

Developers can extend content types:
- Custom MIME parameters.
- Application-specific formats.
- Cross-protocol compatibility.
- Metadata standards.

### Mutant Integration

Dynamic content modification:
- Programmable NFT behavior.
- State transitions.
- Interactive experiences.
- Composable functionality.

### Cross-Chain Compatibility

Bridge mechanisms:
- Asset representation.
- Value preservation.
- Metadata translation.
- Protocol interoperability.

## Implementation Status

### Mainnet Deployment

- Spore Type Script: Deployed and verified
- Cluster Type Script: Deployed and verified
- SDK Libraries: Production ready
- Documentation: Comprehensive guides available

### Network Statistics

- Total Spores Created: Growing ecosystem
- Storage Utilization: Efficient capacity usage
- Developer Adoption: Active community
- Protocol Upgrades: Backwards compatible

## Resources

- **Official Website**: https://spore.pro
- **Documentation**: https://docs.spore.pro
- **SDK Repository**: https://github.com/sporeprotocol/spore-sdk
- **Contract Source**: https://github.com/sporeprotocol/spore-contract
- **Demo Application**: https://a-simple-demo.spore.pro

The Spore Protocol represents a significant advancement in on-chain digital asset management, providing developers with powerful tools for creating truly decentralized, permanent, and valuable digital objects on the CKB blockchain.