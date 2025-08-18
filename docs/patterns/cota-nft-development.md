# CoTA NFT Development Patterns

## Description

Build cost-effective NFT applications using CoTA (Compact Token Aggregator) protocol on CKB. Master user registration, collection definition, batch minting, two-phase transfers, NFT updates, and marketplace integration. Learn advanced patterns for gaming assets, error handling, fee management, and batch processing optimization for scalable NFT development.

## Complete CoTA NFT Development Workflow

CoTA (Compact Token Aggregator) provides an extremely cost-effective solution for NFT development on CKB. This guide covers practical patterns for building NFT applications using the CoTA protocol.

**CoTA Protocol is recommended for cost-effective NFT applications where low cost is more important than flexibility.** Use CoTA for gaming assets, membership tokens, high-volume NFT applications, and projects where transaction costs are a primary concern. For high-value NFTs requiring fully on-chain data storage and maximum metadata integrity, consider Spore Protocol instead.

## Prerequisites and Setup

### Required Services
```typescript
// CoTA infrastructure endpoints (Testnet)
const COTA_CONFIG = {
  ckbNodeUrl: "https://testnet.ckbapp.dev/rpc",
  ckbIndexerUrl: "https://testnet.ckbapp.dev/indexer", 
  registryAggregatorUrl: "https://cota.nervina.dev/registry-aggregator",
  cotaAggregatorUrl: "https://cota.nervina.dev/aggregator"
};
```

### SDK Installation
```bash
npm install @nervina-labs/cota-sdk-js
```

### Basic Setup
```typescript
import { CotaSDK } from "@nervina-labs/cota-sdk-js";

const cotaSDK = new CotaSDK({
  ckb: {
    nodeUrl: COTA_CONFIG.ckbNodeUrl,
    indexerUrl: COTA_CONFIG.ckbIndexerUrl
  },
  aggregator: {
    registryUrl: COTA_CONFIG.registryAggregatorUrl,
    cotaUrl: COTA_CONFIG.cotaAggregatorUrl
  }
});
```

## Core Development Patterns

### 1. User Registration Pattern
Every user must register exactly one CoTA cell before any operations:

```typescript
class CoTAUserManager {
  async ensureUserRegistered(lockScript: Script): Promise<boolean> {
    // Check if user already has CoTA cell
    const isRegistered = await this.checkRegistration(lockScript);
    
    if (!isRegistered) {
      return this.registerUser(lockScript);
    }
    
    return true;
  }
  
  private async checkRegistration(lockScript: Script): Promise<boolean> {
    try {
      const registryInfo = await cotaSDK.registry.getRegistryInfo({
        lockScript
      });
      return registryInfo.registered;
    } catch (error) {
      return false;
    }
  }
  
  private async registerUser(lockScript: Script): Promise<boolean> {
    try {
      const registryTx = await cotaSDK.registry.build({
        lockScript,
        fee: 1000n // CKB fee in shannons
      });
      
      await registryTx.send();
      
      // Wait for confirmation
      await this.waitForConfirmation(registryTx.hash);
      return true;
    } catch (error) {
      console.error("Registration failed:", error);
      return false;
    }
  }
}
```

### 2. NFT Collection Definition Pattern
Define collections with comprehensive metadata:

```typescript
interface NFTCollectionConfig {
  name: string;
  description?: string;
  image: string;
  totalSupply?: number; // 0 for unlimited
  audio?: string;
  video?: string;
  model?: string;
  properties?: string;
  characteristics?: [string, number][];
}

class CoTACollectionManager {
  async defineCollection(
    issuerLockScript: Script,
    config: NFTCollectionConfig
  ): Promise<string> {
    // Ensure issuer is registered
    await this.ensureUserRegistered(issuerLockScript);
    
    const defineTx = await cotaSDK.define.build({
      lockScript: issuerLockScript,
      cotaInfo: {
        name: config.name,
        description: config.description || "",
        image: config.image,
        audio: config.audio,
        video: config.video,
        model: config.model,
        characteristic: config.characteristics || [],
        properties: config.properties
      },
      total: config.totalSupply || 0,
      fee: 2000n
    });
    
    await defineTx.send();
    await this.waitForConfirmation(defineTx.hash);
    
    // Extract cota_id from transaction
    return this.extractCotaId(defineTx);
  }
  
  private extractCotaId(defineTx: any): string {
    // CoTA ID = hash(tx.inputs[0].out_point | first_cota_output_index)[0..20]  
    const firstInput = defineTx.transaction.inputs[0];
    const cotaOutputIndex = this.findFirstCotaOutputIndex(defineTx.transaction);
    
    return blake2b_256(firstInput.previousOutput.serialize() + cotaOutputIndex)
      .slice(0, 40); // First 20 bytes as hex
  }
}
```

### 3. Batch Minting Pattern
Efficiently mint multiple NFTs in single transactions:

```typescript
interface MintRequest {
  recipientLockScript: Script;
  tokenIndex: number;
  state?: string;          // Custom state data
  characteristic?: string; // Custom characteristic data
}

class CoTAMintingManager {
  async batchMint(
    issuerLockScript: Script,
    cotaId: string,
    mintRequests: MintRequest[],
    batchSize: number = 50 // Optimal batch size
  ): Promise<string[]> {
    const txHashes: string[] = [];
    
    // Process in batches to avoid transaction size limits
    for (let i = 0; i < mintRequests.length; i += batchSize) {
      const batch = mintRequests.slice(i, i + batchSize);
      const txHash = await this.mintBatch(issuerLockScript, cotaId, batch);
      txHashes.push(txHash);
      
      // Small delay to avoid overwhelming the network
      await this.delay(1000);
    }
    
    return txHashes;
  }
  
  private async mintBatch(
    issuerLockScript: Script,
    cotaId: string,
    batch: MintRequest[]
  ): Promise<string> {
    const mintTx = await cotaSDK.mint.build({
      lockScript: issuerLockScript,
      cotaId,
      withdrawals: batch.map(request => ({
        tokenIndex: request.tokenIndex,
        state: request.state || "0x00",
        characteristic: request.characteristic || "0x" + "00".repeat(20),
        toLockScript: request.recipientLockScript
      })),
      fee: 3000n + BigInt(batch.length * 100) // Dynamic fee based on batch size
    });
    
    await mintTx.send();
    return mintTx.hash;
  }
  
  // Generate sequential token indices
  generateTokenIndices(startIndex: number, count: number): number[] {
    return Array.from({length: count}, (_, i) => startIndex + i);
  }
  
  // Generate random characteristics for procedural NFTs
  generateRandomCharacteristic(): string {
    const randomBytes = new Uint8Array(20);
    crypto.getRandomValues(randomBytes);
    return "0x" + Array.from(randomBytes)
      .map(b => b.toString(16).padStart(2, '0'))
      .join('');
  }
}
```

### 4. Transfer and Claim Pattern
Handle the two-step transfer process:

```typescript
class CoTATransferManager {
  // Option A: Two-step transfer (withdrawal + claim)
  async initiateTransfer(
    senderLockScript: Script,
    cotaId: string,
    tokenIndex: number,
    recipientLockScript: Script
  ): Promise<string> {
    const withdrawTx = await cotaSDK.withdraw.build({
      lockScript: senderLockScript,
      cotaId,
      tokenIndex,
      toLockScript: recipientLockScript,
      fee: 2000n
    });
    
    await withdrawTx.send();
    return withdrawTx.hash;
  }
  
  async completeTransfer(
    recipientLockScript: Script,
    cotaId: string,
    tokenIndex: number,
    withdrawalTxHash: string
  ): Promise<string> {
    // Get withdrawal proof from aggregator
    const withdrawalProof = await this.getWithdrawalProof(
      cotaId,
      tokenIndex,
      withdrawalTxHash
    );
    
    const claimTx = await cotaSDK.claim.build({
      lockScript: recipientLockScript,
      cotaId,
      tokenIndex,
      withdrawalProof,
      fee: 2000n
    });
    
    await claimTx.send();
    return claimTx.hash;
  }
  
  // Option B: Direct transfer (combines both steps)
  async directTransfer(
    currentOwnerLockScript: Script,
    cotaId: string,
    tokenIndex: number,
    newOwnerLockScript: Script
  ): Promise<string> {
    const transferTx = await cotaSDK.transfer.build({
      lockScript: currentOwnerLockScript,
      cotaId,
      tokenIndex,
      toLockScript: newOwnerLockScript,
      fee: 3000n // Higher fee for combined operation
    });
    
    await transferTx.send();
    return transferTx.hash;
  }
  
  private async getWithdrawalProof(
    cotaId: string,
    tokenIndex: number,
    txHash: string
  ): Promise<any> {
    // Query aggregator for withdrawal proof
    const response = await fetch(`${COTA_CONFIG.cotaAggregatorUrl}/withdrawal_proof`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        cota_id: cotaId,
        token_index: tokenIndex,
        tx_hash: txHash
      })
    });
    
    return response.json();
  }
}
```

### 5. NFT Update Pattern
Modify NFT characteristics after ownership:

```typescript
interface NFTUpdateRequest {
  cotaId: string;
  tokenIndex: number;
  newState?: string;
  newCharacteristic?: string;
}

class CoTAUpdateManager {
  async updateNFT(
    ownerLockScript: Script,
    updateRequest: NFTUpdateRequest
  ): Promise<string> {
    // First ensure NFT is claimed (in hold leaves)
    await this.ensureNFTClaimed(ownerLockScript, updateRequest.cotaId, updateRequest.tokenIndex);
    
    const updateTx = await cotaSDK.update.build({
      lockScript: ownerLockScript,
      cotaId: updateRequest.cotaId,
      tokenIndex: updateRequest.tokenIndex,
      state: updateRequest.newState || "0x00",
      characteristic: updateRequest.newCharacteristic || "0x" + "00".repeat(20),
      fee: 2000n
    });
    
    await updateTx.send();
    return updateTx.hash;
  }
  
  // Combine claim and update in single operation
  async claimAndUpdate(
    recipientLockScript: Script,
    cotaId: string,
    tokenIndex: number,
    withdrawalTxHash: string,
    updateData: { state?: string; characteristic?: string }
  ): Promise<string> {
    const withdrawalProof = await this.getWithdrawalProof(
      cotaId,
      tokenIndex,
      withdrawalTxHash
    );
    
    const claimUpdateTx = await cotaSDK.claimUpdate.build({
      lockScript: recipientLockScript,
      cotaId,
      tokenIndex,
      withdrawalProof,
      state: updateData.state || "0x00",
      characteristic: updateData.characteristic || "0x" + "00".repeat(20),
      fee: 3000n
    });
    
    await claimUpdateTx.send();
    return claimUpdateTx.hash;
  }
  
  private async ensureNFTClaimed(
    lockScript: Script,
    cotaId: string,
    tokenIndex: number
  ): Promise<void> {
    const holdInfo = await this.checkNFTHold(lockScript, cotaId, tokenIndex);
    
    if (!holdInfo.isHeld) {
      // Check for pending claims
      const pendingClaims = await this.getPendingClaims(lockScript, cotaId);
      const pendingClaim = pendingClaims.find(claim => 
        claim.tokenIndex === tokenIndex
      );
      
      if (pendingClaim) {
        // Auto-claim the NFT first
        await this.completeTransfer(
          lockScript,
          cotaId,
          tokenIndex,
          pendingClaim.withdrawalTxHash
        );
      } else {
        throw new Error(`NFT ${cotaId}:${tokenIndex} not owned by address`);
      }
    }
  }
}
```

### 6. Query and Management Pattern
Retrieve NFT information and manage collections:

```typescript
class CoTAQueryManager {
  async getUserNFTs(lockScript: Script): Promise<NFTInfo[]> {
    const response = await fetch(`${COTA_CONFIG.cotaAggregatorUrl}/hold_cota_nft`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        lock_script: lockScript,
        page: 0,
        page_size: 100
      })
    });
    
    const data = await response.json();
    return data.nfts || [];
  }
  
  async getPendingClaims(lockScript: Script): Promise<PendingClaim[]> {
    const response = await fetch(`${COTA_CONFIG.cotaAggregatorUrl}/pending_claims`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        lock_script: lockScript
      })
    });
    
    const data = await response.json();
    return data.claims || [];
  }
  
  async getCollectionInfo(cotaId: string): Promise<CollectionInfo> {
    const response = await fetch(`${COTA_CONFIG.cotaAggregatorUrl}/define_cota_nft`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        cota_id: cotaId
      })
    });
    
    return response.json();
  }
  
  async getCollectionStats(cotaId: string): Promise<CollectionStats> {
    const [defineInfo, holders] = await Promise.all([
      this.getCollectionInfo(cotaId),
      this.getCollectionHolders(cotaId)
    ]);
    
    return {
      totalSupply: defineInfo.total,
      totalMinted: defineInfo.issued,
      uniqueHolders: new Set(holders.map(h => h.lockScript)).size,
      floorPrice: await this.getFloorPrice(cotaId)
    };
  }
  
  private async getCollectionHolders(cotaId: string): Promise<any[]> {
    const response = await fetch(`${COTA_CONFIG.cotaAggregatorUrl}/holders`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        cota_id: cotaId,
        page_size: 1000
      })
    });
    
    const data = await response.json();
    return data.holders || [];
  }
}
```

## Advanced Patterns

### Marketplace Integration
```typescript
class CoTAMarketplace {
  async listNFTForSale(
    ownerLockScript: Script,
    cotaId: string,
    tokenIndex: number,
    price: bigint,
    marketplaceLockScript: Script
  ): Promise<string> {
    // Transfer to marketplace with special characteristic indicating sale
    const saleCharacteristic = this.encodeSaleInfo(price, ownerLockScript);
    
    return this.directTransferWithUpdate(
      ownerLockScript,
      cotaId,
      tokenIndex,
      marketplaceLockScript,
      { characteristic: saleCharacteristic }
    );
  }
  
  async completeSale(
    buyerLockScript: Script,
    cotaId: string,
    tokenIndex: number,
    payment: bigint
  ): Promise<string> {
    // Verify payment and transfer from marketplace to buyer
    await this.verifyPayment(buyerLockScript, payment);
    
    return this.directTransfer(
      this.marketplaceLockScript,
      cotaId,
      tokenIndex,
      buyerLockScript
    );
  }
}
```

### Gaming Asset Management
```typescript
class CoTAGameAssets {
  async mintGameItem(
    gameServerLockScript: Script,
    playerId: string,
    itemType: string,
    attributes: GameItemAttributes
  ): Promise<string> {
    const characteristic = this.encodeGameAttributes(attributes);
    const playerLockScript = this.getPlayerLockScript(playerId);
    
    return this.mintToPlayer(
      gameServerLockScript,
      this.gameItemsCotaId,
      playerLockScript,
      { characteristic }
    );
  }
  
  async upgradeItem(
    playerLockScript: Script,
    itemTokenIndex: number,
    newAttributes: GameItemAttributes
  ): Promise<string> {
    const newCharacteristic = this.encodeGameAttributes(newAttributes);
    
    return this.updateNFT(playerLockScript, {
      cotaId: this.gameItemsCotaId,
      tokenIndex: itemTokenIndex,
      newCharacteristic
    });
  }
}
```

## Best Practices

### 1. Error Handling and Retry Logic
```typescript
class CoTAErrorHandler {
  async withRetry<T>(
    operation: () => Promise<T>,
    maxRetries: number = 3,
    delayMs: number = 2000
  ): Promise<T> {
    for (let attempt = 1; attempt <= maxRetries; attempt++) {
      try {
        return await operation();
      } catch (error) {
        if (attempt === maxRetries) throw error;
        
        console.warn(`Attempt ${attempt} failed, retrying in ${delayMs}ms...`);
        await this.delay(delayMs);
        delayMs *= 2; // Exponential backoff
      }
    }
    
    throw new Error("Max retries exceeded");
  }
}
```

### 2. Transaction Fee Management
```typescript
class CoTAFeeManager {
  calculateOptimalFee(operationType: string, dataSize: number): bigint {
    const baseFees = {
      register: 1000n,
      define: 2000n,
      mint: 3000n,
      transfer: 2000n,
      claim: 2000n,
      update: 2000n
    };
    
    const baseFee = baseFees[operationType] || 2000n;
    const dataSizeFee = BigInt(Math.ceil(dataSize / 1000) * 100);
    
    return baseFee + dataSizeFee;
  }
}
```

### 3. Batch Processing Optimization
```typescript
class CoTABatchProcessor {
  async processLargeBatch<T>(
    items: T[],
    processor: (batch: T[]) => Promise<void>,
    batchSize: number = 50,
    concurrency: number = 3
  ): Promise<void> {
    const batches = this.chunkArray(items, batchSize);
    
    // Process batches with limited concurrency
    for (let i = 0; i < batches.length; i += concurrency) {
      const concurrentBatches = batches.slice(i, i + concurrency);
      await Promise.all(concurrentBatches.map(processor));
      
      // Rate limiting
      if (i + concurrency < batches.length) {
        await this.delay(1000);
      }
    }
  }
  
  private chunkArray<T>(array: T[], size: number): T[][] {
    return Array.from({length: Math.ceil(array.length / size)}, (_, i) =>
      array.slice(i * size, i * size + size)
    );
  }
}
```

CoTA provides a powerful and cost-effective solution for NFT development on CKB. By following these patterns and best practices, developers can build scalable NFT applications with minimal on-chain footprint and excellent user experience.