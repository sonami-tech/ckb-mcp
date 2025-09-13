## Description

Spore SDK reference with TypeScript examples for digital object creation, management, and NFT operations on CKB. createSpore, transferSpore, meltSpore, cluster management, content optimization, batch operations, and advanced patterns for building Spore-based applications with error handling and transaction monitoring.

## Related Resources

- [ckb-dev-context://protocols/spore-protocol](ckb-dev-context://protocols/spore-protocol) - Core protocol specification for Spore digital objects on CKB blockchain
- [ckb-dev-context://patterns/spore-development](ckb-dev-context://patterns/spore-development) - Development patterns for building applications with the Spore Protocol
- [ckb-dev-context://protocols/spore-digital-objects](ckb-dev-context://protocols/spore-digital-objects) - Spore digital objects protocol specification

Examples of using the Spore SDK for building applications with the Spore Protocol on CKB.

## Installation and Setup

### NPM Installation

```bash
# Core SDK
npm install @spore-sdk/core

# Helper utilities (optional)
npm install @spore-sdk/helpers

# Additional dependencies
npm install @ckb-lumos/base @ckb-lumos/helpers @ckb-lumos/ckb-indexer
```

### Configuration

```typescript
import { config, predefined } from '@spore-sdk/core';

// Testnet configuration
config.initializeConfig(predefined.Aggron4.TESTNET);

// Mainnet configuration  
config.initializeConfig(predefined.Lina.MAINNET);

// Custom configuration
config.initializeConfig({
  lumos: {
    SCRIPTS: {
      SPORE: {
        CODE_HASH: '0x4a4dce1df3dffff7f8b2cd7dff7303df3b6150c9788cb75dcf6747247132b9f5',
        HASH_TYPE: 'type',
        TX_HASH: '0x...',
        INDEX: '0x0',
        DEP_TYPE: 'code',
      },
      CLUSTER: {
        CODE_HASH: '0x7366a61534fa7c7e6225ecc0d828ea3b5366adec2b58206f2ee84995fe030075',
        HASH_TYPE: 'type',
        TX_HASH: '0x...',
        INDEX: '0x1',
        DEP_TYPE: 'code',
      },
    },
    PREFIX: 'ckt', // 'ckb' for mainnet
  },
  ckbNodeUrl: 'https://testnet.ckbapp.dev/rpc',
  ckbIndexerUrl: 'https://testnet.ckbapp.dev/indexer',
});
```

## Core API Functions

### createSpore

Creates a new Spore with specified content and metadata.

#### Parameters

```typescript
interface CreateSporeParams {
  data: {
    contentType: string;      // MIME type
    content: Uint8Array;      // Content bytes
    clusterId?: string;       // Optional cluster ID
  };
  toLock: string | Script;    // Owner lock script
  fromInfos: string[];       // Funding sources
  changeAddress?: string;     // Change output address
}
```

#### Examples

```typescript
import { createSpore } from '@spore-sdk/core';

// Create a text Spore
const textSpore = await createSpore({
  data: {
    contentType: 'text/plain',
    content: new TextEncoder().encode('Hello, Spore Protocol!'),
  },
  toLock: 'ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsq2qf8keemy2p5uu0g0gn8cd4ju23s5269qk8rg4r',
  fromInfos: ['ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsq2qf8keemy2p5uu0g0gn8cd4ju23s5269qk8rg4r'],
});

// Create an image Spore
const imageFile = document.getElementById('imageInput').files[0];
const imageBuffer = await imageFile.arrayBuffer();

const imageSpore = await createSpore({
  data: {
    contentType: 'image/png',
    content: new Uint8Array(imageBuffer),
  },
  toLock: ownerLockScript,
  fromInfos: [fundingAddress],
});

// Create a JSON metadata Spore
const metadata = {
  name: 'Rare Digital Collectible',
  description: 'A unique piece from the Genesis Collection',
  attributes: {
    rarity: 'legendary',
    power: 95,
    element: 'fire',
  },
  image: 'https://example.com/image.png', // External reference
};

const jsonSpore = await createSpore({
  data: {
    contentType: 'application/json',
    content: new TextEncoder().encode(JSON.stringify(metadata)),
  },
  toLock: ownerAddress,
  fromInfos: [fundingSource],
});

// Create a Spore within a cluster
const clusteredSpore = await createSpore({
  data: {
    contentType: 'text/plain',
    content: new TextEncoder().encode('Part of my collection'),
    clusterId: '0x...', // Existing cluster ID
  },
  toLock: ownerAddress,
  fromInfos: [fundingSource],
});
```

### transferSpore

Transfers ownership of a Spore to a new owner.

#### Parameters

```typescript
interface TransferSporeParams {
  outPoint: OutPoint;         // Spore cell to transfer
  toLock: string | Script;    // New owner lock script
  fromInfos?: string[];       // Capacity sources (optional for zero-fee)
  changeAddress?: string;     // Change output address
}
```

#### Examples

```typescript
import { transferSpore } from '@spore-sdk/core';

// Standard transfer (sender pays fees)
const transfer = await transferSpore({
  outPoint: {
    txHash: '0x...',
    index: '0x0',
  },
  toLock: 'ckt1...',  // New owner address
  fromInfos: ['ckt1...'], // Current owner provides capacity
});

// Zero-fee transfer (recipient pays)
const zeroFeeTransfer = await transferSpore({
  outPoint: sporeOutPoint,
  toLock: newOwnerAddress,
  fromInfos: [newOwnerAddress], // Recipient provides capacity
});

// Transfer with custom change address
const customChangeTransfer = await transferSpore({
  outPoint: sporeOutPoint,
  toLock: newOwnerAddress,
  fromInfos: [senderAddress],
  changeAddress: customChangeAddress,
});
```

### meltSpore

Destroys a Spore and reclaims its CKB capacity.

#### Parameters

```typescript
interface MeltSporeParams {
  outPoint: OutPoint;         // Spore cell to melt
  fromInfos: string[];       // Must include Spore owner
  changeAddress?: string;     // Where to send reclaimed CKB
}
```

#### Examples

```typescript
import { meltSpore } from '@spore-sdk/core';

// Basic melt operation
const melt = await meltSpore({
  outPoint: {
    txHash: '0x...',
    index: '0x0',
  },
  fromInfos: ['ckt1...'], // Spore owner address
});

// Melt with custom change address
const customMelt = await meltSpore({
  outPoint: sporeOutPoint,
  fromInfos: [ownerAddress],
  changeAddress: beneficiaryAddress,
});

// Calculate reclaimed capacity before melting
import { getSporeById } from '@spore-sdk/helpers';

const spore = await getSporeById(sporeOutPoint);
const reclaimedCapacity = spore.capacity;
console.log(`Will reclaim ${reclaimedCapacity} CKB`);

const meltTx = await meltSpore({
  outPoint: sporeOutPoint,
  fromInfos: [ownerAddress],
});
```

### createCluster

Creates a new Cluster for organizing related Spores.

#### Parameters

```typescript
interface CreateClusterParams {
  data: {
    name: Uint8Array;         // Cluster name
    description: Uint8Array;   // Cluster description
  };
  toLock: string | Script;    // Cluster owner
  fromInfos: string[];       // Funding sources
  changeAddress?: string;     // Change output address
}
```

#### Examples

```typescript
import { createCluster } from '@spore-sdk/core';

// Create a basic cluster
const cluster = await createCluster({
  data: {
    name: new TextEncoder().encode('Genesis Collection'),
    description: new TextEncoder().encode('The first collection of digital artifacts on CKB'),
  },
  toLock: 'ckt1...',
  fromInfos: ['ckt1...'],
});

// Create cluster with rich metadata
const detailedCluster = await createCluster({
  data: {
    name: new TextEncoder().encode('Pixel Art Masters'),
    description: new TextEncoder().encode(JSON.stringify({
      description: 'Curated collection of pixel art',
      category: 'art',
      tags: ['pixel', 'retro', 'gaming'],
      creator: 'PixelArtist',
      website: 'https://pixelart.example.com',
      maxSupply: 1000,
    })),
  },
  toLock: creatorAddress,
  fromInfos: [fundingSource],
});
```

### transferCluster

Transfers ownership of a Cluster to a new owner.

#### Parameters

```typescript
interface TransferClusterParams {
  outPoint: OutPoint;         // Cluster cell to transfer  
  toLock: string | Script;    // New owner lock script
  fromInfos?: string[];       // Capacity sources
  changeAddress?: string;     // Change output address
}
```

#### Examples

```typescript
import { transferCluster } from '@spore-sdk/core';

// Transfer cluster ownership
const clusterTransfer = await transferCluster({
  outPoint: clusterOutPoint,
  toLock: newOwnerAddress,
  fromInfos: [currentOwnerAddress],
});

// Zero-fee cluster transfer
const zeroFeeClusterTransfer = await transferCluster({
  outPoint: clusterOutPoint,
  toLock: newOwnerAddress,
  fromInfos: [newOwnerAddress], // New owner pays
});
```

## Helper Functions

### Query Functions

```typescript
import {
  getSporeById,
  getClusterById,
  findSporesByCluster,
  findSporesByAddress,
  findClustersByAddress,
} from '@spore-sdk/helpers';

// Get specific Spore
const spore = await getSporeById({
  txHash: '0x...',
  index: '0x0',
});

// Get specific Cluster
const cluster = await getClusterById({
  txHash: '0x...',
  index: '0x0',
});

// Find all Spores in a cluster
const clusterSpores = await findSporesByCluster({
  txHash: '0x...',
  index: '0x0',
});

// Find Spores owned by address
const mySpores = await findSporesByAddress('ckt1...');

// Find Clusters owned by address
const myClusters = await findClustersByAddress('ckt1...');
```

### Utility Functions

```typescript
import { 
  calculateSporeCapacity,
  validateSporeData,
  extractSporeContent,
  generateSporeId,
} from '@spore-sdk/helpers';

// Calculate required capacity
const requiredCapacity = calculateSporeCapacity({
  contentType: 'image/png',
  content: imageData,
  clusterId: clusterOutPoint,
});

// Validate Spore data
const isValid = validateSporeData({
  contentType: 'application/json',
  content: jsonData,
});

// Extract content from Spore
const { contentType, content } = extractSporeContent(spore);
const contentString = new TextDecoder().decode(content);

// Generate deterministic Spore ID
const sporeId = generateSporeId(txHash, outputIndex);
```

## Advanced Usage Patterns

### Batch Operations

```typescript
// Create multiple Spores efficiently
async function createSporeCollection(
  sporeDataList: SporeData[],
  ownerAddress: string,
  clusterOutPoint?: OutPoint
) {
  const results = [];
  
  for (const sporeData of sporeDataList) {
    const spore = await createSpore({
      data: {
        ...sporeData,
        clusterId: clusterOutPoint,
      },
      toLock: ownerAddress,
      fromInfos: [ownerAddress],
    });
    
    results.push(spore);
    
    // Small delay to avoid overwhelming the network
    await new Promise(resolve => setTimeout(resolve, 1000));
  }
  
  return results;
}
```

### Content Management

```typescript
// Content type detection and validation
function detectContentType(buffer: Uint8Array): string {
  // PNG signature
  if (buffer.slice(0, 8).every((byte, i) => 
    byte === [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A][i]
  )) {
    return 'image/png';
  }
  
  // JPEG signature
  if (buffer[0] === 0xFF && buffer[1] === 0xD8) {
    return 'image/jpeg';
  }
  
  // Try to parse as JSON
  try {
    const text = new TextDecoder().decode(buffer);
    JSON.parse(text);
    return 'application/json';
  } catch {
    // Fall back to plain text
    return 'text/plain';
  }
}

// Content optimization
async function optimizeContent(
  contentType: string, 
  content: Uint8Array
): Promise<Uint8Array> {
  const maxSize = 500 * 1024; // 500KB limit
  
  if (content.length <= maxSize) {
    return content;
  }
  
  // Implement compression based on content type
  switch (contentType) {
    case 'image/png':
    case 'image/jpeg':
      return await compressImage(content, maxSize);
    
    case 'application/json':
      return await compressJSON(content);
    
    default:
      throw new Error(`Content too large: ${content.length} > ${maxSize}`);
  }
}
```

### Error Handling

```typescript
// Comprehensive error handling
async function robustSporeCreation(sporeData: SporeData, ownerAddress: string) {
  try {
    // Validate content first
    validateSporeData(sporeData);
    
    // Check capacity requirements
    const requiredCapacity = calculateSporeCapacity(sporeData);
    const availableCapacity = await getAddressCapacity(ownerAddress);
    
    if (availableCapacity < requiredCapacity) {
      throw new Error(`Insufficient capacity: need ${requiredCapacity}, have ${availableCapacity}`);
    }
    
    // Create the Spore
    const result = await createSpore({
      data: sporeData,
      toLock: ownerAddress,
      fromInfos: [ownerAddress],
    });
    
    return result;
    
  } catch (error) {
    // Handle specific error types
    if (error.message.includes('INSUFFICIENT_CAPACITY')) {
      console.error('Not enough CKB to create Spore');
      // Suggest getting more CKB or reducing content size
    } else if (error.message.includes('INVALID_CONTENT_TYPE')) {
      console.error('Unsupported content type');
      // Suggest supported content types
    } else if (error.message.includes('CONTENT_TOO_LARGE')) {
      console.error('Content size exceeds limits');
      // Suggest compression or size reduction
    } else {
      console.error('Unexpected error:', error.message);
    }
    
    throw error;
  }
}
```

### Transaction Monitoring

```typescript
// Monitor transaction status
async function monitorTransaction(txHash: string): Promise<boolean> {
  const maxRetries = 30; // 5 minutes with 10s intervals
  let retries = 0;
  
  while (retries < maxRetries) {
    try {
      const tx = await ckbRpc.getTransaction(txHash);
      
      if (tx && tx.txStatus && tx.txStatus.status === 'committed') {
        console.log(`Transaction ${txHash} confirmed`);
        return true;
      }
      
      console.log(`Transaction ${txHash} status: ${tx?.txStatus?.status || 'pending'}`);
      
    } catch (error) {
      console.warn(`Error checking transaction ${txHash}:`, error.message);
    }
    
    await new Promise(resolve => setTimeout(resolve, 10000)); // Wait 10 seconds
    retries++;
  }
  
  console.error(`Transaction ${txHash} not confirmed after ${maxRetries} retries`);
  return false;
}

// Usage with Spore creation
const { txSkeleton } = await createSpore(sporeParams);
const signedTx = await signTransaction(txSkeleton);
const txHash = await sendTransaction(signedTx);

const confirmed = await monitorTransaction(txHash);
if (confirmed) {
  console.log('Spore created successfully!');
} else {
  console.error('Spore creation may have failed');
}
```

## TypeScript Types

### Core Types

```typescript
// Spore data structure
interface SporeData {
  contentType: string;
  content: Uint8Array;
  clusterId?: OutPoint;
}

// Cluster data structure
interface ClusterData {
  name: Uint8Array;
  description: Uint8Array;
}

// OutPoint reference
interface OutPoint {
  txHash: string;
  index: string;
}

// Script structure
interface Script {
  codeHash: string;
  hashType: 'type' | 'data';
  args: string;
}

// Spore cell
interface Spore {
  outPoint: OutPoint;
  capacity: string;
  lock: Script;
  type: Script;
  data: SporeData;
}

// Cluster cell
interface Cluster {
  outPoint: OutPoint;
  capacity: string;
  lock: Script;
  type: Script;
  data: ClusterData;
}
```

### API Response Types

```typescript
// Transaction skeleton (from Lumos)
interface TransactionSkeleton {
  inputs: List<Cell>;
  outputs: List<Cell>;
  cellDeps: List<CellDep>;
  headerDeps: List<Hash>;
  witnesses: List<string>;
}

// API response wrapper
interface SporeApiResponse<T> {
  data: T;
  txSkeleton: TransactionSkeleton;
}

// Query results
interface SporeQueryResult {
  spores: Spore[];
  total: number;
  hasMore: boolean;
}

interface ClusterQueryResult {
  clusters: Cluster[];
  total: number;
  hasMore: boolean;
}
```

Examples for integrating the Spore Protocol into CKB applications. Use these patterns as starting points and adapt them to specific use cases.