# Spore Protocol: Digital Objects (DOBs) Development

## Description

Protocol for true on-chain digital objects with complete ownership, redeemable value, and zero-fee transfers. Covers DOB/0 protocol family, decoder configurations, trait systems, content management, cluster organization, GraphQL integration, and practical implementation patterns for immutable digital assets.

## What is Spore Protocol?

Spore Protocol is designed for **on-chain digital objects (DOBs)** with true ownership, redeemable value, and zero-fee transfers. Unlike traditional NFTs that store metadata off-chain, Spore ensures complete on-chain ownership and storage.

## Key Advantages over Traditional NFTs

| Feature | Traditional NFTs | Spore DOBs |
|---------|------------------|------------|
| **Storage** | Off-chain (URLs, IPFS) | 100% on-chain |
| **Ownership** | Token ID + metadata link | Complete on-chain ownership |
| **Value** | Market speculation | Intrinsic + redeemable value |
| **Transfers** | Gas fees required | Zero-fee transfers |
| **Content Types** | Limited (mostly images) | Multi-format support |
| **Immutability** | Depends on hosting | Guaranteed immutable |

## Technical Implementation

### Spore Cell Structure
```rust
// Spore cell contains complete digital object
SporeData {
    content_type: String,    // MIME type (image/jpeg, text/plain, etc.)
    content: Vec<u8>,        // Actual content data (up to 500KB)
    cluster_id: Option<H256>, // Optional cluster grouping
}
```

## Digital Object Baseline (DOB) Protocol Family

The DOB protocol family extends Spore Protocol with standardized decoding and trait systems for digital objects.

### DOB/0 Protocol

DOB/0 is the first protocol in the DOB family, providing universal decoder configuration and interface standards.

#### Configuration Format

Decoders are configured in the Spore Cluster's `description` field as JSON:

```json
{
  "description": "This is the description for cluster",
  "dob": {
    "ver": 0,
    "decoder": {
      "type": "code_hash",  // or "type_id"
      "hash": "0x...",
    },
    "pattern": [
      ["Age", "Number", 1, 1, "range", [0, 100]],
      ["Rarity", "String", 1, 1, "enum", ["Common", "Rare", "Epic"]]
    ]
  }
}
```

**Configuration Fields:**
- `dob.ver`: Protocol version (always 0 for DOB/0)
- `dob.decoder.type`: How to locate decoder (`code_hash` or `type_id`)
- `dob.decoder.hash`: Hash value for decoder location
- `dob.pattern`: Trait definition patterns (decoder-specific)

#### Creating DOB/0 Clusters

```javascript
import { createCluster } from '@spore-sdk/api';
import { bytifyRawString } from '@spore-sdk/helpers/buffer';

const dob_metadata = {
  description: 'Gaming Character Collection',
  dob: {
    ver: 0,
    decoder: {
      type: 'code_hash',
      hash: '0x1234...abcd',
    },
    pattern: [
      ["Strength", "Number", 1, 1, "range", [1, 100]],
      ["Class", "String", 1, 1, "enum", ["Warrior", "Mage", "Archer"]],
      ["Element", "String", 0, 3, "enum", ["Fire", "Water", "Earth"]]
    ]
  }
};

const { txSkeleton, outputIndex } = await createCluster({
  data: {
    name: 'Fantasy RPG Characters',
    description: bytifyRawString(JSON.stringify(dob_metadata)),
  },
  fromInfos: [account.address],
  toLock: account.lock
});
```

#### DOB/0 Decoder Interface

**Input Parameters:**
- `DNA`: Hexadecimal string of the digital object's data
- `Pattern`: UTF-8 encoded pattern data from cluster configuration

**Output Format:**
```json
[
  { 
    "name": "Strength", 
    "traits": [{ "Number": 85 }] 
  },
  { 
    "name": "Class", 
    "traits": [{ "String": "Warrior" }] 
  },
  { 
    "name": "Element", 
    "traits": [
      { "String": "Fire" },
      { "String": "Earth" }
    ] 
  }
]
```

**Trait Rules:**
- Each trait has a unique `name` as identifier
- Multiple same-name traits are combined into arrays
- Single-element `traits` arrays represent single values
- Each trait element has exactly one key-value pair for type and value

### DOB/0 Development Examples

#### Custom Decoder Implementation

```rust
use ckb_std::{
    ckb_constants::Source,
    high_level::{load_cell_data, load_script},
};

// DOB/0 decoder for gaming characters
pub fn decode_gaming_character(dna: &[u8], pattern: &str) -> Result<Vec<Trait>, Error> {
    let mut traits = Vec::new();
    
    // Parse DNA to extract character attributes
    if dna.len() >= 4 {
        let strength = u32::from_le_bytes(dna[0..4].try_into()?);
        traits.push(Trait {
            name: "Strength".to_string(),
            traits_data: vec![TraitValue::Number(strength as u64)],
        });
    }
    
    if dna.len() >= 8 {
        let class_id = u32::from_le_bytes(dna[4..8].try_into()?);
        let class_name = match class_id % 3 {
            0 => "Warrior",
            1 => "Mage",
            _ => "Archer",
        };
        traits.push(Trait {
            name: "Class".to_string(),
            traits_data: vec![TraitValue::String(class_name.to_string())],
        });
    }
    
    Ok(traits)
}

#[derive(Debug)]
struct Trait {
    name: String,
    traits_data: Vec<TraitValue>,
}

#[derive(Debug)]
enum TraitValue {
    String(String),
    Number(u64),
    Boolean(bool),
}
```

#### DOB/0 Client Integration

```javascript
// Fetch and decode DOB traits
async function decodeDOB(sporeId, clusterData) {
  const dob_config = JSON.parse(clusterData.description);
  
  if (dob_config.dob?.ver === 0) {
    // Locate decoder using configuration
    const decoder = await findDecoder(dob_config.dob.decoder);
    
    // Load spore DNA data  
    const dna = await getSporeData(sporeId);
    
    // Call decoder with DNA and pattern
    const traits = await decoder.decode(dna, dob_config.dob.pattern);
    
    return traits;
  }
  
  throw new Error("Unsupported DOB version");
}

async function findDecoder(decoderConfig) {
  if (decoderConfig.type === "code_hash") {
    // Find cell with matching data hash
    return await findCellByDataHash(decoderConfig.hash);
  } else if (decoderConfig.type === "type_id") {
    // Find cell with matching type script args
    return await findCellByTypeId(decoderConfig.hash);
  }
  
  throw new Error("Unknown decoder type");
}
```

### Creating Spores with CCC
```typescript
import { ccc } from "@ckb-ccc/ccc";
import { spore } from "@ckb-ccc/spore";

// Create a digital object (image, document, etc.)
const sporeData = {
    contentType: "image/png",
    content: await fs.readFile("artwork.png"),
    clusterId: undefined, // or specify cluster
};

const { tx } = await spore.createSpore({
    data: sporeData,
    toLock: recipientLockScript,
    fromInfos: [senderInfo],
    config: spore.predefined.TESTNET,
});

// Send transaction
const txHash = await signer.sendTransaction(tx);
```

### Cluster Management
```typescript
// Create a cluster for organizing related spores
const clusterData = {
    name: "My Art Collection",
    description: "Digital art series",
};

const { tx: clusterTx } = await spore.createCluster({
    data: clusterData,
    toLock: ownerLockScript,
    fromInfos: [ownerInfo],
});

const clusterTxHash = await signer.sendTransaction(clusterTx);
```

## Spore Protocol Integration Patterns

### Reading Spore Content
```typescript
// Fetch and decode spore data
const sporeCell = await client.getCell(sporeOutPoint);
const sporeData = spore.unpackSporeData(sporeCell.cellOutput.type?.args);

// Access content
const contentType = sporeData.contentType; // "image/jpeg"
const contentBytes = sporeData.content;    // Raw bytes
const clusterId = sporeData.clusterId;     // Optional cluster
```

### Transfer Spores (Zero-Fee)
```typescript
// Transfer spore to new owner (zero fees for recipient)
const { tx } = await spore.transferSpore({
    outPoint: sporeOutPoint,
    toLock: newOwnerLockScript,
    fromInfos: [currentOwnerInfo],
});

// Current owner pays transaction fee
const txHash = await currentOwnerSigner.sendTransaction(tx);
```

### Melting Spores (Redeem Value)
```typescript
// Destroy spore and reclaim CKB capacity
const { tx } = await spore.meltSpore({
    outPoint: sporeOutPoint,
    changeAddress: ownerAddress,
    fromInfos: [ownerInfo],
});

// Owner receives back the CKB used for storage
const txHash = await ownerSigner.sendTransaction(tx);
```

## Content Type Support

### Supported Formats
```typescript
// Images
contentType: "image/jpeg"
contentType: "image/png"
contentType: "image/gif"
contentType: "image/svg+xml"

// Documents
contentType: "text/plain"
contentType: "text/html"
contentType: "application/json"
contentType: "application/pdf"

// Audio/Video
contentType: "audio/mpeg"
contentType: "video/mp4"

// Custom formats
contentType: "application/custom"
```

### Content Size Limits
- **Maximum**: 500KB per spore
- **Optimal**: Under 100KB for better performance
- **Cost**: Storage cost scales with content size

## Production Integration Patterns

### Spore-based NFT Marketplace
```typescript
class SporeMarketplace {
    async listSpore(sporeOutPoint: ccc.OutPoint, price: bigint) {
        // Create marketplace listing with price lock
        const listingTx = await this.createListingTransaction(
            sporeOutPoint, 
            price
        );
        return await this.signer.sendTransaction(listingTx);
    }
    
    async purchaseSpore(listingOutPoint: ccc.OutPoint) {
        // Purchase listed spore
        const purchaseTx = await this.createPurchaseTransaction(
            listingOutPoint
        );
        return await this.signer.sendTransaction(purchaseTx);
    }
}
```

### Content Management System
```typescript
class SporeContentManager {
    async publishContent(content: Uint8Array, metadata: any) {
        const sporeData = {
            contentType: this.detectContentType(content),
            content: content,
            clusterId: metadata.collectionId,
        };
        
        return await this.createAndPublishSpore(sporeData);
    }
    
    async getContentHistory(clusterId: string) {
        // Query all spores in a cluster
        return await this.querySporesByCluster(clusterId);
    }
}
```

## GraphQL Integration

### Spore GraphQL API
```typescript
// Query spores with GraphQL
const query = `
  query GetSpores($first: Int, $cluster: String) {
    spores(first: $first, cluster: $cluster) {
      id
      contentType
      clusterId
      capacity
      owner
      createdAt
    }
  }
`;

const spores = await sporeClient.query(query, {
    first: 10,
    cluster: clusterHash
});
```

## When to Use Spore Protocol

### Perfect for:
- **Digital art** with guaranteed on-chain storage
- **Documents** requiring immutable archival
- **Certificates** needing permanent verification
- **Content creation** platforms with true ownership
- **Gaming assets** with intrinsic value

### Consider alternatives for:
- **Large files** (>500KB) - use IPFS + metadata
- **High-frequency trading** - consider simpler UDT
- **Temporary content** - traditional storage may be cheaper

## Resources and Tools

### Development
- **Spore SDK**: https://github.com/sporeprotocol/spore-sdk
- **Spore Docs**: https://docs.spore.pro/
- **DOB Cookbook**: https://github.com/sporeprotocol/dob-cookbook

### Infrastructure
- **Spore GraphQL**: Query layer for spore data
- **DOB Decoder**: Standalone rendering server
- **Spore Demo**: Reference implementation

Spore Protocol provides the foundation for building applications with true digital ownership and on-chain content storage, enabling new possibilities for creators and users in the digital economy.