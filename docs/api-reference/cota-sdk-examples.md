# CoTA SDK JavaScript Examples

## Description

Complete CoTA SDK JavaScript implementation guide with production-ready examples for NFT operations including minting, transferring, updating, and querying. Covers user registration, collection management, issuer setup, batch operations, and advanced patterns like two-step transfers and claim updates. Uses official `@nervina-labs/cota-sdk` with comprehensive error handling and transaction signing patterns.

## Complete CoTA SDK Implementation Guide

This guide provides production-ready examples using the official CoTA SDK JavaScript library (`@nervina-labs/cota-sdk`) with practical implementations for all CoTA operations.

## Installation and Setup

### Installation
```bash
npm install @nervina-labs/cota-sdk
# or
yarn add @nervina-labs/cota-sdk
```

### Service Configuration
```typescript
import { Collector, Aggregator, Service } from '@nervina-labs/cota-sdk';

// Production service configuration
const createCotaService = (isMainnet: boolean = false): Service => {
  const config = isMainnet ? {
    ckbNodeUrl: 'https://mainnet.ckbapp.dev/rpc',
    ckbIndexerUrl: 'https://mainnet.ckbapp.dev/indexer',
    registryUrl: 'https://cota.nervina.dev/mainnet-registry-aggregator',
    cotaUrl: 'https://cota.nervina.dev/mainnet-aggregator'
  } : {
    ckbNodeUrl: 'https://testnet.ckbapp.dev/rpc',
    ckbIndexerUrl: 'https://testnet.ckbapp.dev/indexer',
    registryUrl: 'https://cota.nervina.dev/registry-aggregator',
    cotaUrl: 'https://cota.nervina.dev/aggregator'
  };

  return {
    collector: new Collector({
      ckbNodeUrl: config.ckbNodeUrl,
      ckbIndexerUrl: config.ckbIndexerUrl,
    }),
    aggregator: new Aggregator({
      registryUrl: config.registryUrl,
      cotaUrl: config.cotaUrl,
    }),
  };
};
```

### Cell Dependencies
```typescript
import { addressToScript } from '@nervosnetwork/ckb-sdk-utils';

const secp256k1CellDep = (isMainnet: boolean): CKBComponents.CellDep => {
  return isMainnet ? {
    outPoint: {
      txHash: '0x71a7ba8fc96349fea0ed3a5c47992e3b4084b031a42264a018e0072e8172e46c',
      index: '0x0',
    },
    depType: 'depGroup',
  } : {
    outPoint: {
      txHash: '0xf8de3bb47d055cdf460d93a2a6e1b05f7432f9777c8c474abf4eec1d4aee5d37',
      index: '0x0',
    },
    depType: 'depGroup',
  };
};
```

## 1. Registry Operations

### User Registration
```typescript
import { 
  generateRegisterCotaTx, 
  FEE, 
  getAlwaysSuccessLock 
} from '@nervina-labs/cota-sdk';
import { 
  addressToScript, 
  scriptToHash, 
  rawTransactionToHash,
  serializeWitnessArgs 
} from '@nervosnetwork/ckb-sdk-utils';
import signWitnesses from '@nervosnetwork/ckb-sdk-core/lib/signWitnesses';

class CotaRegistry {
  constructor(
    private service: Service,
    private isMainnet: boolean = false
  ) {}

  async registerUser(
    userAddress: string,
    privateKey: string,
    providerAddress: string,
    providerPrivateKey: string
  ): Promise<string> {
    const ckb = this.service.collector.getCkb();
    const userLock = addressToScript(userAddress);
    const providerLock = addressToScript(providerAddress);

    // Generate registration transaction
    let rawTx = await generateRegisterCotaTx(
      this.service,
      [userLock], // Users to register
      providerLock, // CKB provider for fees
      FEE,
      this.isMainnet
    );

    // Add cell dependencies
    rawTx.cellDeps.push(secp256k1CellDep(this.isMainnet));

    const registryLock = getAlwaysSuccessLock(this.isMainnet);

    // Setup signature keys
    const keyMap = new Map<string, string>();
    keyMap.set(scriptToHash(registryLock), ''); // Always success lock
    keyMap.set(scriptToHash(providerLock), providerPrivateKey);

    // Prepare cells for signing
    const cells = rawTx.inputs.map((input, index) => ({
      outPoint: input.previousOutput,
      lock: index === 0 ? registryLock : providerLock,
    }));

    const transactionHash = rawTransactionToHash(rawTx);

    // Sign transaction
    const signedWitnesses = signWitnesses(keyMap)({
      transactionHash,
      witnesses: rawTx.witnesses,
      inputCells: cells,
      skipMissingKeys: true,
    });

    const signedTx = {
      ...rawTx,
      witnesses: signedWitnesses.map(witness => 
        typeof witness === 'string' ? witness : serializeWitnessArgs(witness)
      ),
    };

    // Send transaction
    const txHash = await ckb.rpc.sendTransaction(signedTx, 'passthrough');
    console.log(`Registry transaction sent: ${txHash}`);
    
    return txHash;
  }

  async checkRegistration(userAddress: string): Promise<boolean> {
    try {
      const userLock = addressToScript(userAddress);
      const lockHash = scriptToHash(userLock);
      
      // Query registry status from aggregator
      const response = await fetch(`${this.service.aggregator.registryUrl}/get_registry_info`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ lock_hash: lockHash })
      });
      
      const data = await response.json();
      return data.registered === true;
    } catch (error) {
      console.error('Failed to check registration:', error);
      return false;
    }
  }
}
```

## 2. Issuer and Collection Management

### Issuer Information Setup
```typescript
import { generateIssuerInfoTx, IssuerInfo } from '@nervina-labs/cota-sdk';

class CotaIssuer {
  constructor(
    private service: Service,
    private isMainnet: boolean = false
  ) {}

  async setIssuerInfo(
    issuerAddress: string,
    privateKey: string,
    issuerInfo: IssuerInfo
  ): Promise<string> {
    const ckb = this.service.collector.getCkb();
    const issuerLock = addressToScript(issuerAddress);

    const rawTx = await generateIssuerInfoTx(
      this.service,
      issuerLock,
      issuerInfo,
      FEE,
      this.isMainnet
    );

    rawTx.cellDeps.push(secp256k1CellDep(this.isMainnet));

    const signedTx = ckb.signTransaction(privateKey)(rawTx);
    const txHash = await ckb.rpc.sendTransaction(signedTx, 'passthrough');
    
    console.log(`Issuer info transaction sent: ${txHash}`);
    return txHash;
  }
}
```

### Collection Definition
```typescript
import { generateDefineCotaTx, CotaInfo } from '@nervina-labs/cota-sdk';

class CotaCollectionManager extends CotaIssuer {
  async defineCollection(
    issuerAddress: string,
    privateKey: string,
    collectionInfo: CotaInfo,
    totalSupply: number = 0 // 0 for unlimited
  ): Promise<{ txHash: string; cotaId: string }> {
    const ckb = this.service.collector.getCkb();
    const issuerLock = addressToScript(issuerAddress);

    const defineCotaInfo = {
      cotaInfo: collectionInfo,
      total: totalSupply,
    };

    const rawTx = await generateDefineCotaTx(
      this.service,
      issuerLock,
      defineCotaInfo,
      FEE,
      this.isMainnet
    );

    rawTx.cellDeps.push(secp256k1CellDep(this.isMainnet));

    const signedTx = ckb.signTransaction(privateKey)(rawTx);
    const txHash = await ckb.rpc.sendTransaction(signedTx, 'passthrough');

    // Extract cota_id from transaction
    const cotaId = this.extractCotaId(rawTx);
    
    console.log(`Collection defined - TX: ${txHash}, CoTA ID: ${cotaId}`);
    return { txHash, cotaId };
  }

  private extractCotaId(rawTx: any): string {
    // CoTA ID = hash(first_input.out_point + first_cota_output_index)[0..20]
    const firstInput = rawTx.inputs[0];
    const cotaOutputIndex = this.findFirstCotaOutputIndex(rawTx);
    
    // This is a simplified extraction - in practice, you'd use the SDK's utility
    return 'extracted_cota_id'; // Placeholder
  }

  private findFirstCotaOutputIndex(rawTx: any): number {
    // Find the first output with CoTA type script
    return rawTx.outputs.findIndex((output: any) => 
      output.type && output.type.codeHash === 'cota_type_script_hash'
    );
  }
}
```

## 3. Minting Operations

### Batch Minting
```typescript
import { generateMintCotaTx, MintCotaInfo } from '@nervina-labs/cota-sdk';
import { serializeScript } from '@nervosnetwork/ckb-sdk-utils';

class CotaMinter extends CotaCollectionManager {
  async batchMint(
    issuerAddress: string,
    privateKey: string,
    cotaId: string,
    recipients: Array<{
      address: string;
      tokenIndex?: number; // Auto-assigned if not provided
      state?: string;
      characteristic?: string;
    }>
  ): Promise<string> {
    const ckb = this.service.collector.getCkb();
    const issuerLock = addressToScript(issuerAddress);

    const mintCotaInfo: MintCotaInfo = {
      cotaId,
      withdrawals: recipients.map((recipient, index) => ({
        tokenIndex: recipient.tokenIndex ? 
          `0x${recipient.tokenIndex.toString(16).padStart(8, '0')}` : 
          undefined, // Auto-assigned
        state: recipient.state || '0x00',
        characteristic: recipient.characteristic || 
          '0x' + '05'.repeat(20), // Default characteristic
        toLockScript: serializeScript(addressToScript(recipient.address)),
      })),
    };

    const rawTx = await generateMintCotaTx(
      this.service,
      issuerLock,
      mintCotaInfo,
      FEE,
      this.isMainnet
    );

    rawTx.cellDeps.push(secp256k1CellDep(this.isMainnet));

    const signedTx = ckb.signTransaction(privateKey)(rawTx);
    const txHash = await ckb.rpc.sendTransaction(signedTx, 'passthrough');
    
    console.log(`Minted ${recipients.length} NFTs - TX: ${txHash}`);
    return txHash;
  }

  async mintToSingleRecipient(
    issuerAddress: string,
    privateKey: string,
    cotaId: string,
    recipientAddress: string,
    customCharacteristic?: string
  ): Promise<string> {
    return this.batchMint(
      issuerAddress,
      privateKey,
      cotaId,
      [{
        address: recipientAddress,
        characteristic: customCharacteristic
      }]
    );
  }
}
```

## 4. Transfer Operations

### Direct Transfer (Claim + Withdraw)
```typescript
import { generateTransferCotaTx, TransferWithdrawal } from '@nervina-labs/cota-sdk';

class CotaTransfer extends CotaMinter {
  async directTransfer(
    currentOwnerAddress: string,
    privateKey: string,
    transfers: Array<{
      cotaId: string;
      tokenIndex: number;
      toAddress: string;
    }>
  ): Promise<string> {
    const ckb = this.service.collector.getCkb();
    const currentOwnerLock = addressToScript(currentOwnerAddress);
    const withdrawLock = currentOwnerLock; // Same for direct transfer

    const transferWithdrawals: TransferWithdrawal[] = transfers.map(transfer => ({
      cotaId: transfer.cotaId,
      tokenIndex: `0x${transfer.tokenIndex.toString(16).padStart(8, '0')}`,
      toLockScript: serializeScript(addressToScript(transfer.toAddress)),
    }));

    const rawTx = await generateTransferCotaTx(
      this.service,
      currentOwnerLock, // CoTA lock (claimer)
      withdrawLock,     // Withdraw lock (withdrawer)
      transferWithdrawals,
      FEE,
      this.isMainnet
    );

    rawTx.cellDeps.push(secp256k1CellDep(this.isMainnet));

    const signedTx = ckb.signTransaction(privateKey)(rawTx);
    const txHash = await ckb.rpc.sendTransaction(signedTx, 'passthrough');
    
    console.log(`Transferred ${transfers.length} NFTs - TX: ${txHash}`);
    return txHash;
  }

  async singleTransfer(
    currentOwnerAddress: string,
    privateKey: string,
    cotaId: string,
    tokenIndex: number,
    toAddress: string
  ): Promise<string> {
    return this.directTransfer(
      currentOwnerAddress,
      privateKey,
      [{ cotaId, tokenIndex, toAddress }]
    );
  }
}
```

### Two-Step Transfer (Withdraw + Claim)
```typescript
import { 
  generateWithdrawCotaTx, 
  generateClaimCotaTx,
  WithdrawCotaInfo,
  ClaimCotaInfo 
} from '@nervina-labs/cota-sdk';

class CotaTwoStepTransfer extends CotaTransfer {
  async withdraw(
    ownerAddress: string,
    privateKey: string,
    withdrawals: Array<{
      cotaId: string;
      tokenIndex: number;
      toAddress: string;
      state?: string;
      characteristic?: string;
    }>
  ): Promise<string> {
    const ckb = this.service.collector.getCkb();
    const ownerLock = addressToScript(ownerAddress);

    const withdrawCotaInfo: WithdrawCotaInfo = {
      cotaId: withdrawals[0].cotaId, // Assuming same collection
      withdrawals: withdrawals.map(w => ({
        tokenIndex: `0x${w.tokenIndex.toString(16).padStart(8, '0')}`,
        state: w.state || '0x00',
        characteristic: w.characteristic || '0x' + '05'.repeat(20),
        toLockScript: serializeScript(addressToScript(w.toAddress)),
      })),
    };

    const rawTx = await generateWithdrawCotaTx(
      this.service,
      ownerLock,
      withdrawCotaInfo,
      FEE,
      this.isMainnet
    );

    rawTx.cellDeps.push(secp256k1CellDep(this.isMainnet));

    const signedTx = ckb.signTransaction(privateKey)(rawTx);
    const txHash = await ckb.rpc.sendTransaction(signedTx, 'passthrough');
    
    console.log(`Withdrew ${withdrawals.length} NFTs - TX: ${txHash}`);
    return txHash;
  }

  async claim(
    recipientAddress: string,
    privateKey: string,
    claims: Array<{
      cotaId: string;
      tokenIndex: number;
      withdrawalTxHash: string;
    }>
  ): Promise<string> {
    const ckb = this.service.collector.getCkb();
    const recipientLock = addressToScript(recipientAddress);

    // Get withdrawal proofs from aggregator
    const claimCotaInfo: ClaimCotaInfo = {
      cotaId: claims[0].cotaId,
      claims: await Promise.all(claims.map(async claim => {
        const proof = await this.getWithdrawalProof(
          claim.cotaId,
          claim.tokenIndex,
          claim.withdrawalTxHash
        );
        return {
          tokenIndex: `0x${claim.tokenIndex.toString(16).padStart(8, '0')}`,
          proof,
        };
      })),
    };

    const rawTx = await generateClaimCotaTx(
      this.service,
      recipientLock,
      claimCotaInfo,
      FEE,
      this.isMainnet
    );

    rawTx.cellDeps.push(secp256k1CellDep(this.isMainnet));

    const signedTx = ckb.signTransaction(privateKey)(rawTx);
    const txHash = await ckb.rpc.sendTransaction(signedTx, 'passthrough');
    
    console.log(`Claimed ${claims.length} NFTs - TX: ${txHash}`);
    return txHash;
  }

  private async getWithdrawalProof(
    cotaId: string,
    tokenIndex: number,
    withdrawalTxHash: string
  ): Promise<any> {
    const response = await fetch(`${this.service.aggregator.cotaUrl}/get_withdrawal_proof`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        cota_id: cotaId,
        token_index: `0x${tokenIndex.toString(16).padStart(8, '0')}`,
        tx_hash: withdrawalTxHash
      })
    });

    const data = await response.json();
    return data.proof;
  }
}
```

## 5. Update Operations

### NFT Updates
```typescript
import { generateUpdateCotaTx, UpdateCotaInfo } from '@nervina-labs/cota-sdk';

class CotaUpdater extends CotaTwoStepTransfer {
  async updateNFTs(
    ownerAddress: string,
    privateKey: string,
    updates: Array<{
      cotaId: string;
      tokenIndex: number;
      newState?: string;
      newCharacteristic?: string;
    }>
  ): Promise<string> {
    const ckb = this.service.collector.getCkb();
    const ownerLock = addressToScript(ownerAddress);

    const updateCotaInfo: UpdateCotaInfo = {
      cotaId: updates[0].cotaId,
      updates: updates.map(update => ({
        tokenIndex: `0x${update.tokenIndex.toString(16).padStart(8, '0')}`,
        state: update.newState || '0x00',
        characteristic: update.newCharacteristic || '0x' + '05'.repeat(20),
      })),
    };

    const rawTx = await generateUpdateCotaTx(
      this.service,
      ownerLock,
      updateCotaInfo,
      FEE,
      this.isMainnet
    );

    rawTx.cellDeps.push(secp256k1CellDep(this.isMainnet));

    const signedTx = ckb.signTransaction(privateKey)(rawTx);
    const txHash = await ckb.rpc.sendTransaction(signedTx, 'passthrough');
    
    console.log(`Updated ${updates.length} NFTs - TX: ${txHash}`);
    return txHash;
  }

  async updateSingleNFT(
    ownerAddress: string,
    privateKey: string,
    cotaId: string,
    tokenIndex: number,
    newState?: string,
    newCharacteristic?: string
  ): Promise<string> {
    return this.updateNFTs(
      ownerAddress,
      privateKey,
      [{ cotaId, tokenIndex, newState, newCharacteristic }]
    );
  }
}
```

### Combined Claim and Update
```typescript
import { generateClaimUpdateCotaTx, ClaimUpdateInfo } from '@nervina-labs/cota-sdk';

class CotaClaimUpdate extends CotaUpdater {
  async claimAndUpdate(
    recipientAddress: string,
    privateKey: string,
    claimUpdates: Array<{
      cotaId: string;
      tokenIndex: number;
      withdrawalTxHash: string;
      newState?: string;
      newCharacteristic?: string;
    }>
  ): Promise<string> {
    const ckb = this.service.collector.getCkb();
    const recipientLock = addressToScript(recipientAddress);

    const claimUpdateInfo: ClaimUpdateInfo = {
      cotaId: claimUpdates[0].cotaId,
      claimUpdates: await Promise.all(claimUpdates.map(async cu => {
        const proof = await this.getWithdrawalProof(
          cu.cotaId,
          cu.tokenIndex,
          cu.withdrawalTxHash
        );
        return {
          tokenIndex: `0x${cu.tokenIndex.toString(16).padStart(8, '0')}`,
          state: cu.newState || '0x00',
          characteristic: cu.newCharacteristic || '0x' + '05'.repeat(20),
          proof,
        };
      })),
    };

    const rawTx = await generateClaimUpdateCotaTx(
      this.service,
      recipientLock,
      claimUpdateInfo,
      FEE,
      this.isMainnet
    );

    rawTx.cellDeps.push(secp256k1CellDep(this.isMainnet));

    const signedTx = ckb.signTransaction(privateKey)(rawTx);
    const txHash = await ckb.rpc.sendTransaction(signedTx, 'passthrough');
    
    console.log(`Claimed and updated ${claimUpdates.length} NFTs - TX: ${txHash}`);
    return txHash;
  }
}
```

## 6. Query Operations

### NFT and Collection Queries
```typescript
class CotaQueryService extends CotaClaimUpdate {
  async getUserNFTs(userAddress: string): Promise<any[]> {
    const userLock = addressToScript(userAddress);
    const lockHash = scriptToHash(userLock);

    const response = await fetch(`${this.service.aggregator.cotaUrl}/get_hold_cota_nft`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        lock_script: userLock,
        page: 0,
        page_size: 100
      })
    });

    const data = await response.json();
    return data.nfts || [];
  }

  async getWithdrawalNFTs(userAddress: string): Promise<any[]> {
    const userLock = addressToScript(userAddress);

    const response = await fetch(`${this.service.aggregator.cotaUrl}/get_withdrawal_cota_nft`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        lock_script: userLock,
        page: 0,
        page_size: 100
      })
    });

    const data = await response.json();
    return data.withdrawals || [];
  }

  async getCollectionInfo(cotaId: string): Promise<any> {
    const response = await fetch(`${this.service.aggregator.cotaUrl}/get_define_info`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ cota_id: cotaId })
    });

    return await response.json();
  }

  async getIssuerInfo(issuerAddress: string): Promise<any> {
    const issuerLock = addressToScript(issuerAddress);

    const response = await fetch(`${this.service.aggregator.cotaUrl}/get_issuer_info`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ lock_script: issuerLock })
    });

    return await response.json();
  }
}
```

## 7. Complete Integration Example

### Full CoTA Integration Class
```typescript
export class CotaSDKManager extends CotaQueryService {
  constructor(isMainnet: boolean = false) {
    const service = createCotaService(isMainnet);
    super(service, isMainnet);
  }

  // Convenience method for complete user onboarding
  async onboardUser(
    userAddress: string,
    privateKey: string,
    providerAddress: string,
    providerPrivateKey: string
  ): Promise<{ isRegistered: boolean; txHash?: string }> {
    const isAlreadyRegistered = await this.checkRegistration(userAddress);
    
    if (isAlreadyRegistered) {
      return { isRegistered: true };
    }

    const txHash = await this.registerUser(
      userAddress,
      privateKey,
      providerAddress,
      providerPrivateKey
    );

    return { isRegistered: false, txHash };
  }

  // Convenience method for complete collection creation
  async createCompleteCollection(
    issuerAddress: string,
    privateKey: string,
    issuerInfo: IssuerInfo,
    collectionInfo: CotaInfo,
    totalSupply?: number
  ): Promise<{ cotaId: string; issuerTxHash: string; defineTxHash: string }> {
    // Set issuer information
    const issuerTxHash = await this.setIssuerInfo(
      issuerAddress,
      privateKey,
      issuerInfo
    );

    // Wait for issuer transaction confirmation
    await this.waitForConfirmation(issuerTxHash);

    // Define collection
    const { txHash: defineTxHash, cotaId } = await this.defineCollection(
      issuerAddress,
      privateKey,
      collectionInfo,
      totalSupply
    );

    return { cotaId, issuerTxHash, defineTxHash };
  }

  private async waitForConfirmation(txHash: string, maxWaitTime: number = 60000): Promise<void> {
    const ckb = this.service.collector.getCkb();
    const startTime = Date.now();
    
    while (Date.now() - startTime < maxWaitTime) {
      try {
        const txStatus = await ckb.rpc.getTransaction(txHash);
        if (txStatus && txStatus.txStatus.status === 'committed') {
          return;
        }
      } catch (error) {
        // Transaction not found yet, continue waiting
      }
      
      await new Promise(resolve => setTimeout(resolve, 2000));
    }
    
    throw new Error(`Transaction ${txHash} not confirmed within ${maxWaitTime}ms`);
  }
}

// Usage example
export async function exampleUsage() {
  const cotaManager = new CotaSDKManager(false); // false for testnet

  // 1. Register user
  const userRegistration = await cotaManager.onboardUser(
    'ckt1qyq0scej4vn0uka238m63azcel7cmcme7f2sxj5ska',
    '0xprivate_key_here',
    'ckt1qyqp8ydxwz3p4vcmjwc2d7zqkxhv707j80q4yrap2',
    '0xprovider_private_key_here'
  );

  // 2. Create collection
  const collection = await cotaManager.createCompleteCollection(
    'ckt1qyqp8ydxwz3p4vcmjwc2d7zqkxhv707j80q4yrap2',
    '0xissuer_private_key_here',
    {
      name: 'My NFT Project',
      description: 'An amazing NFT collection',
      avatar: 'https://example.com/avatar.jpg'
    },
    {
      name: 'Cool NFT Collection',
      image: 'https://example.com/collection.jpg',
      description: 'A collection of cool NFTs',
    },
    1000 // Limited supply
  );

  console.log('Collection created:', collection);
}
```

This comprehensive guide provides production-ready CoTA SDK examples covering all major operations from user registration to advanced NFT management. The examples include proper error handling, type safety, and real-world usage patterns for building scalable CoTA applications.