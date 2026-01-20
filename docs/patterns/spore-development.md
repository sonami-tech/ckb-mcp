## Description

Development patterns for building applications with the Spore Protocol on CKB. Content preparation, capacity management, error handling, transaction batching, marketplace integration, collection management, wallet integration, and performance optimization with production-ready code examples.

## Related Resources

- [ckb://docs/protocols/spore-protocol](ckb://docs/protocols/spore-protocol) - Core protocol specification for Spore digital objects on CKB blockchain
- [ckb://docs/api-reference/spore-sdk-examples](ckb://docs/api-reference/spore-sdk-examples) - Spore SDK reference with TypeScript examples for digital object creation
- [ckb://docs/protocols/spore-digital-objects](ckb://docs/protocols/spore-digital-objects) - Spore digital objects protocol specification

Common development patterns and best practices for building applications with the Spore Protocol on CKB.

**Spore Protocol is recommended for high-value NFTs and digital assets that require fully on-chain data storage.** Use Spore when data permanence, metadata integrity, and 100% on-chain content are critical to your application. For cost-effective NFT solutions where low transaction costs are more important than flexibility, consider CoTA Protocol instead.

## Development Setup

### Environment Configuration

```typescript
import { config, predefined } from '@spore-sdk/core';

// Development/Testing
config.initializeConfig(predefined.Aggron4.TESTNET);

// Production
config.initializeConfig(predefined.Lina.MAINNET);

// Custom configuration
config.initializeConfig({
  lumos: {
    SCRIPTS: {
      // Custom script configurations
    },
    PREFIX: 'ckt', // or 'ckb' for mainnet
  },
  ckbNodeUrl: 'https://testnet.ckbapp.dev/rpc',
  ckbIndexerUrl: 'https://testnet.ckbapp.dev/indexer',
});
```

### Dependencies and Imports

```typescript
// Core SDK functions
import {
  createSpore,
  transferSpore,
  meltSpore,
  createCluster,
  transferCluster,
} from '@spore-sdk/core';

// Helper utilities
import {
  getSporeById,
  getClusterById,
  findSporesByCluster,
  calculateSporeCapacity,
} from '@spore-sdk/helpers';

// Transaction building
import { TransactionSkeleton } from '@ckb-lumos/helpers';
import { Indexer } from '@ckb-lumos/ckb-indexer';
```

## Basic Patterns

### 1. Content Preparation

```typescript
// Text content
const textContent = {
  contentType: 'text/plain',
  content: new TextEncoder().encode('Hello, Spore!'),
};

// JSON metadata
const jsonContent = {
  contentType: 'application/json',
  content: new TextEncoder().encode(JSON.stringify({
    name: 'My Digital Asset',
    description: 'A unique digital collectible',
    attributes: {
      rarity: 'legendary',
      power: 100,
    },
  })),
};

// Image content (from file upload)
const imageContent = {
  contentType: 'image/png',
  content: new Uint8Array(await file.arrayBuffer()),
};
```

### 2. Capacity Calculation

```typescript
import { calculateSporeCapacity } from '@spore-sdk/helpers';

function estimateSporeCapacity(contentSize: number, hasCluster = false): number {
  const baseCapacity = hasCluster ? 93 : 61; // CKB
  const contentCapacity = Math.ceil(contentSize / 8); // 8 bytes per CKB
  return baseCapacity + contentCapacity;
}

// Calculate exact capacity needed
const requiredCapacity = calculateSporeCapacity({
  contentType: 'image/png',
  content: imageData,
  clusterId: clusterOutPoint,
});
```

### 3. Error Handling Pattern

```typescript
async function createSporeWithErrorHandling(sporeData: SporeData) {
  try {
    const { txSkeleton } = await createSpore({
      data: sporeData,
      toLock: ownerLock,
      fromInfos: fundingSources,
    });
    
    return await signAndSendTransaction(txSkeleton);
    
  } catch (error) {
    switch (error.code) {
      case 'INSUFFICIENT_CAPACITY':
        throw new Error(`Need at least ${error.required} CKB, have ${error.available}`);
      
      case 'INVALID_CONTENT_TYPE':
        throw new Error(`Content type "${error.contentType}" not supported`);
      
      case 'CONTENT_TOO_LARGE':
        throw new Error(`Content size ${error.size} exceeds maximum ${error.maxSize}`);
      
      case 'INVALID_LOCK_SCRIPT':
        throw new Error('Invalid owner lock script provided');
      
      default:
        throw new Error(`Spore creation failed: ${error.message}`);
    }
  }
}
```

### 4. Transaction Batching

```typescript
// Create multiple Spores in a single transaction
async function batchCreateSpores(sporeDataList: SporeData[], toLock: Script) {
  let txSkeleton = TransactionSkeleton();
  
  for (const sporeData of sporeDataList) {
    const { txSkeleton: sporeTx } = await createSpore({
      data: sporeData,
      toLock,
      fromInfos: fundingSources,
      skipValidation: true, // Skip individual validation
    });
    
    txSkeleton = mergeTxSkeletons(txSkeleton, sporeTx);
  }
  
  // Validate and optimize the combined transaction
  return await validateAndOptimize(txSkeleton);
}
```

## Advanced Patterns

### 1. Zero-Fee Transfer Pattern

```typescript
// Recipient-funded transfer (zero cost to sender)
async function recipientFundedTransfer(
  sporeOutPoint: OutPoint,
  newOwnerLock: Script,
  recipientCapacity: OutPoint[]
) {
  const { txSkeleton } = await transferSpore({
    outPoint: sporeOutPoint,
    toLock: newOwnerLock,
    fromInfos: recipientCapacity, // Recipient provides capacity
  });
  
  // Recipient signs the transaction
  return await signTransactionAsRecipient(txSkeleton, newOwnerLock);
}
```

### 2. Marketplace Integration

```typescript
// Marketplace sale with automatic royalty distribution
class SporeMarketplace {
  async listSpore(
    sporeOutPoint: OutPoint,
    priceInCKB: number,
    royaltyRecipient?: Script,
    royaltyPercent = 5
  ) {
    // Create marketplace listing cell
    const listingData = {
      sporeId: sporeOutPoint,
      price: priceInCKB,
      seller: await getSporeOwner(sporeOutPoint),
      royalty: royaltyRecipient ? { recipient: royaltyRecipient, percent: royaltyPercent } : null,
    };
    
    return await this.createListing(listingData);
  }
  
  async purchaseSpore(
    listingOutPoint: OutPoint,
    buyerLock: Script
  ) {
    const listing = await this.getListing(listingOutPoint);
    
    // Calculate payments
    const totalPrice = listing.price;
    const royaltyAmount = listing.royalty 
      ? Math.floor(totalPrice * listing.royalty.percent / 100)
      : 0;
    const sellerAmount = totalPrice - royaltyAmount;
    
    // Build purchase transaction
    let txSkeleton = TransactionSkeleton();
    
    // Transfer Spore to buyer
    const { txSkeleton: transferTx } = await transferSpore({
      outPoint: listing.sporeId,
      toLock: buyerLock,
      fromInfos: [buyerCapacitySource],
    });
    
    txSkeleton = mergeTxSkeleton(txSkeleton, transferTx);
    
    // Add payment outputs
    if (royaltyAmount > 0) {
      txSkeleton = addPaymentOutput(txSkeleton, listing.royalty.recipient, royaltyAmount);
    }
    txSkeleton = addPaymentOutput(txSkeleton, listing.seller, sellerAmount);
    
    return txSkeleton;
  }
}
```

### 3. Collection Management

```typescript
class SporeCollection {
  private clusterId: OutPoint;
  
  constructor(clusterId: OutPoint) {
    this.clusterId = clusterId;
  }
  
  async addSporeToCollection(sporeData: Omit<SporeData, 'clusterId'>) {
    return await createSpore({
      data: {
        ...sporeData,
        clusterId: this.clusterId,
      },
      toLock: await this.getCollectionOwner(),
      fromInfos: await this.getCollectionFunding(),
    });
  }
  
  async listCollectionSpores(): Promise<Spore[]> {
    return await findSporesByCluster(this.clusterId);
  }
  
  async getCollectionStats() {
    const spores = await this.listCollectionSpores();
    return {
      totalSpores: spores.length,
      totalCapacity: spores.reduce((sum, spore) => sum + spore.capacity, 0),
      contentTypes: [...new Set(spores.map(s => s.data.contentType))],
      averageSize: spores.reduce((sum, s) => sum + s.data.content.length, 0) / spores.length,
    };
  }
}
```

### 4. Content Management Patterns

```typescript
// Content validation and optimization
class ContentManager {
  static validateContent(contentType: string, content: Uint8Array): boolean {
    const maxSize = 500 * 1024; // 500KB limit
    
    if (content.length > maxSize) {
      throw new Error(`Content too large: ${content.length} > ${maxSize}`);
    }
    
    // Validate content type
    const allowedTypes = [
      'text/plain',
      'application/json',
      'image/png',
      'image/jpeg',
      'image/gif',
      'image/svg+xml',
    ];
    
    if (!allowedTypes.includes(contentType)) {
      throw new Error(`Unsupported content type: ${contentType}`);
    }
    
    return true;
  }
  
  static async optimizeImage(imageData: Uint8Array, contentType: string): Promise<Uint8Array> {
    // Implement image compression logic
    if (contentType === 'image/png') {
      return await this.compressPNG(imageData);
    } else if (contentType === 'image/jpeg') {
      return await this.compressJPEG(imageData);
    }
    
    return imageData;
  }
  
  static generateThumbnail(imageData: Uint8Array, maxSize = 64): Uint8Array {
    // Generate small thumbnail for preview
    // Implementation depends on your image processing library
    return imageData; // Placeholder
  }
}
```

### 5. Event Monitoring

```typescript
// Monitor Spore-related events
class SporeMonitor {
  private indexer: Indexer;
  
  constructor(indexerUrl: string) {
    this.indexer = new Indexer(indexerUrl);
  }
  
  async watchSporeTransfers(sporeId: OutPoint, callback: (event: TransferEvent) => void) {
    const sporeTypeScript = await getSporeTypeScript();
    
    // Monitor cells with Spore type script
    this.indexer.subscribe({
      type_: sporeTypeScript,
      data: sporeId,
    }, (event) => {
      if (event.type === 'delete') {
        // Spore was consumed (transferred or melted)
        callback({
          type: 'transfer',
          sporeId,
          transaction: event.transaction,
          timestamp: Date.now(),
        });
      }
    });
  }
  
  async getSporeHistory(sporeId: OutPoint): Promise<TransferEvent[]> {
    // Query historical transactions involving this Spore
    const transactions = await this.querySporeTransactions(sporeId);
    
    return transactions.map(tx => ({
      type: this.determineEventType(tx),
      sporeId,
      transaction: tx.hash,
      timestamp: tx.timestamp,
      from: this.extractSender(tx),
      to: this.extractRecipient(tx),
    }));
  }
}
```

## Integration Patterns

### 1. Wallet Integration

```typescript
// Connect to CKB wallet
async function connectWallet(): Promise<WalletConnection> {
  if (typeof window !== 'undefined' && window.ckb) {
    // Browser wallet available
    await window.ckb.request({ method: 'wallet_enable' });
    
    return {
      type: 'browser',
      getAddresses: () => window.ckb.request({ method: 'wallet_getAddresses' }),
      signTransaction: (tx) => window.ckb.request({ 
        method: 'wallet_signTransaction', 
        params: [tx] 
      }),
    };
  } else {
    // Use Node.js wallet or hardware wallet
    return await connectNodeWallet();
  }
}
```

### 2. React Integration

```typescript
// React hook for Spore operations
function useSpore(sporeId?: OutPoint) {
  const [spore, setSpore] = useState<Spore | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  
  const createSpore = useCallback(async (data: SporeData) => {
    setLoading(true);
    setError(null);
    
    try {
      const result = await createSporeWithErrorHandling(data);
      setSpore(result);
      return result;
    } catch (err) {
      setError(err.message);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);
  
  const transferSpore = useCallback(async (toLock: Script) => {
    if (!spore) throw new Error('No Spore to transfer');
    
    setLoading(true);
    try {
      await recipientFundedTransfer(spore.outPoint, toLock, []);
      // Refresh Spore data after transfer
      const updatedSpore = await getSporeById(spore.outPoint);
      setSpore(updatedSpore);
    } catch (err) {
      setError(err.message);
      throw err;
    } finally {
      setLoading(false);
    }
  }, [spore]);
  
  useEffect(() => {
    if (sporeId) {
      getSporeById(sporeId).then(setSpore).catch(setError);
    }
  }, [sporeId]);
  
  return {
    spore,
    loading,
    error,
    createSpore,
    transferSpore,
  };
}
```

## Testing Patterns

### 1. Unit Testing

```typescript
// Test Spore creation
describe('Spore Creation', () => {
  beforeEach(async () => {
    // Setup test environment
    config.initializeConfig(predefined.Aggron4.TESTNET);
  });
  
  it('should create a text Spore', async () => {
    const sporeData = {
      contentType: 'text/plain',
      content: new TextEncoder().encode('Test content'),
    };
    
    const { txSkeleton } = await createSpore({
      data: sporeData,
      toLock: testLock,
      fromInfos: [testCapacityCell],
    });
    
    expect(txSkeleton.get('outputs').size).toBe(2); // Spore + change
    expect(txSkeleton.get('inputs').size).toBeGreaterThan(0);
  });
  
  it('should handle insufficient capacity error', async () => {
    const largeContent = new Uint8Array(1000000); // 1MB content
    
    await expect(createSpore({
      data: {
        contentType: 'application/octet-stream',
        content: largeContent,
      },
      toLock: testLock,
      fromInfos: [smallCapacityCell], // Insufficient capacity
    })).rejects.toThrow('INSUFFICIENT_CAPACITY');
  });
});
```

### 2. Integration Testing

```typescript
// End-to-end Spore lifecycle test
describe('Spore Lifecycle', () => {
  let sporeOutPoint: OutPoint;
  
  it('should create, transfer, and melt a Spore', async () => {
    // Create
    const createResult = await createSpore({
      data: testSporeData,
      toLock: ownerLock,
      fromInfos: fundingSources,
    });
    
    const createTx = await sendTransaction(createResult.txSkeleton);
    sporeOutPoint = extractSporeOutPoint(createTx);
    
    // Transfer
    const transferResult = await transferSpore({
      outPoint: sporeOutPoint,
      toLock: newOwnerLock,
      fromInfos: [newOwnerCapacity],
    });
    
    await sendTransaction(transferResult.txSkeleton);
    
    // Melt
    const meltResult = await meltSpore({
      outPoint: sporeOutPoint,
      fromInfos: [newOwnerCapacity],
    });
    
    const meltTx = await sendTransaction(meltResult.txSkeleton);
    
    // Verify Spore no longer exists
    await expect(getSporeById(sporeOutPoint)).rejects.toThrow('NOT_FOUND');
  });
});
```

## Performance Optimization

### 1. Capacity Management

```typescript
// Efficient capacity management
class CapacityManager {
  private capacityPool: OutPoint[] = [];
  
  async maintainCapacityPool(minCapacity = 1000) {
    const currentCapacity = await this.getTotalCapacity();
    
    if (currentCapacity < minCapacity) {
      await this.requestMoreCapacity(minCapacity - currentCapacity);
    }
  }
  
  async optimizeCapacityUsage(txSkeleton: TransactionSkeleton): TransactionSkeleton {
    // Minimize change outputs by selecting optimal capacity cells
    return await this.selectOptimalCapacityCells(txSkeleton);
  }
}
```

### 2. Batch Operations

```typescript
// Batch multiple Spore operations
async function batchSporeOperations(operations: SporeOperation[]) {
  const batches = chunkArray(operations, 10); // Process in batches of 10
  const results = [];
  
  for (const batch of batches) {
    const batchTx = await buildBatchTransaction(batch);
    const result = await sendTransaction(batchTx);
    results.push(result);
    
    // Wait between batches to avoid overwhelming the network
    await delay(5000);
  }
  
  return results;
}
```

This comprehensive guide covers the essential patterns for developing with the Spore Protocol, from basic operations to advanced integration scenarios.