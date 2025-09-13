## Description

CKB Common Connector (CCC) SDK's cross-chain wallet integration capabilities. Build CKB applications that work seamlessly with Bitcoin, Ethereum, and Nostr wallets. Unified signer interfaces, transaction patterns, and practical examples for multi-chain dApp development on CKB.

## Overview

The CCC SDK provides a unified interface for integrating multiple blockchain wallets with CKB applications. This enables users to interact with CKB using their existing Bitcoin, Ethereum, or other blockchain wallets.

## Supported Wallet Types

### Bitcoin Wallets
- OKX Wallet.
- UniSat.
- Xverse.
- UTXO Global (multi-chain UTXO support).

### Ethereum Wallets
- EIP-6963 compliant wallets.
- MetaMask and other standard Ethereum wallets.

### Other Protocols
- JoyID (WebAuthn-based).
- Nostr (NIP-07 protocol).

## Unified Signer Interface

All wallet types implement a common signer interface:

```typescript
// Initialize any supported wallet type
const btcSigner = new ccc.SignerBtc(client, provider);
const ethSigner = new ccc.SignerEip6963(client, provider);
const nostrSigner = new ccc.SignerNip07(client, provider);

// Common operations work across all signers
await signer.connect();
const addresses = await signer.getAddresses();
const signedTx = await signer.signTransaction(tx);
```

## Bitcoin Wallet Integration

### Basic Setup

```typescript
import { ccc } from "@ckb-ccc/ccc";

// Initialize Bitcoin signer with OKX wallet
const client = new ccc.ClientPublicTestnet();
const signer = new ccc.okx.SignerOkx(client);

// Connect and get CKB address from Bitcoin wallet
await signer.connect();
const ckbAddress = await signer.getRecommendedAddress();
```

### Transaction Signing

```typescript
// Build CKB transaction
const tx = ccc.Transaction.from({
  outputs: [{
    lock: recipientLock,
    capacity: ccc.fixedPointFrom(100)
  }]
});

// Complete and sign with Bitcoin wallet
await tx.completeInputsByCapacity(signer);
await tx.completeFeeBy(signer);
const txHash = await signer.sendTransaction(tx);
```

## Ethereum Wallet Integration

### EIP-6963 Discovery

```typescript
// Discover available Ethereum wallets
const providers = await ccc.eip6963.getProviders();

// Connect to first available wallet
if (providers.length > 0) {
  const signer = new ccc.SignerEip6963(client, providers[0]);
  await signer.connect();
}
```

### Message Signing

```typescript
// Sign arbitrary messages (useful for authentication)
const message = "Login to CKB dApp";
const signature = await signer.signMessage(message);
```

## Cross-Chain Transaction Patterns

### Multi-Wallet Support

```typescript
class MultiWalletManager {
  private signers: Map<string, ccc.Signer> = new Map();
  
  async addWallet(type: string, provider: any) {
    let signer: ccc.Signer;
    
    switch(type) {
      case 'bitcoin-okx':
        signer = new ccc.okx.SignerOkx(this.client, provider);
        break;
      case 'ethereum':
        signer = new ccc.SignerEip6963(this.client, provider);
        break;
      case 'nostr':
        signer = new ccc.SignerNip07(this.client, provider);
        break;
    }
    
    await signer.connect();
    this.signers.set(type, signer);
  }
}
```

### UTXO Chain Bridge Pattern

```typescript
// Bridge Bitcoin UTXO to CKB cell
const utxoGlobalSigner = new ccc.utxoGlobal.SignerUtxoGlobal(
  client,
  provider,
  { chain: 'BTC' }
);

// Ensure correct network
await utxoGlobalSigner.ensureNetwork();

// Create corresponding CKB transaction
const bridgeTx = await createBridgeTransaction(utxoData);
await utxoGlobalSigner.signTransaction(bridgeTx);
```

## WebAuthn Integration with JoyID

```typescript
// Initialize JoyID signer
const joySigner = new ccc.joyId.SignerJoyId(client);

// WebAuthn-based authentication
await joySigner.connect();

// Biometric signing
const tx = await buildTransaction();
const signedTx = await joySigner.signTransaction(tx);
```

## Best Practices

### Network Compatibility

Always ensure network compatibility when using cross-chain wallets:

```typescript
try {
  await signer.ensureNetwork();
} catch (error) {
  console.error("Network mismatch:", error);
  // Handle network switching or show error to user
}
```

### Error Handling

```typescript
async function safeConnect(signer: ccc.Signer) {
  try {
    await signer.connect();
    return true;
  } catch (error) {
    if (error.code === 4001) {
      // User rejected connection
      return false;
    }
    throw error;
  }
}
```

### Wallet Detection

```typescript
function detectAvailableWallets() {
  const wallets = [];
  
  // Check Bitcoin wallets
  if (window.okxwallet) wallets.push({ type: 'okx', name: 'OKX Wallet' });
  if (window.unisat) wallets.push({ type: 'unisat', name: 'UniSat' });
  
  // Check Ethereum wallets via EIP-6963
  ccc.eip6963.getProviders().then(providers => {
    providers.forEach(p => wallets.push({ type: 'ethereum', name: p.info.name }));
  });
  
  return wallets;
}
```

## Advanced Features

### Custom Signing Strategies

```typescript
class CustomMultiSigSigner extends ccc.Signer {
  constructor(
    private signers: ccc.Signer[],
    private threshold: number
  ) {
    super();
  }
  
  async signTransaction(tx: ccc.Transaction): Promise<ccc.Transaction> {
    const signatures = [];
    
    for (let i = 0; i < this.threshold; i++) {
      const sig = await this.signers[i].signTransaction(tx);
      signatures.push(sig);
    }
    
    return combineSignatures(tx, signatures);
  }
}
```

### Cross-Chain Event Monitoring

```typescript
// Monitor events across chains
const monitor = new CrossChainMonitor();

monitor.on('btc-transaction', async (btcTx) => {
  // Trigger corresponding CKB action
  const ckbTx = await createMirrorTransaction(btcTx);
  await signer.sendTransaction(ckbTx);
});
```

## Integration Examples

### DeFi Application

```typescript
// Allow users to interact with CKB DeFi using any wallet
async function swapTokens(fromToken: string, toToken: string, amount: bigint) {
  const wallet = await selectWallet(); // UI wallet selector
  const signer = createSigner(wallet);
  
  const swapTx = await buildSwapTransaction(fromToken, toToken, amount);
  await swapTx.completeInputsByCapacity(signer);
  
  return signer.sendTransaction(swapTx);
}
```

### NFT Marketplace

```typescript
// List NFT for sale using Bitcoin wallet
async function listNFT(nftId: string, price: bigint) {
  const btcSigner = new ccc.okx.SignerOkx(client);
  await btcSigner.connect();
  
  const listingTx = await createNFTListing(nftId, price);
  return btcSigner.sendTransaction(listingTx);
}
```

## Troubleshooting

### Common Issues

1. **Wallet Not Detected**: Ensure wallet extension is installed and unlocked
2. **Network Mismatch**: Use `ensureNetwork()` to verify correct network
3. **Signing Failures**: Check wallet permissions and transaction format
4. **Connection Timeouts**: Implement retry logic with exponential backoff

### Debug Mode

```typescript
// Enable detailed logging
ccc.setDebugMode(true);

// Monitor wallet events
signer.on('connect', () => console.log('Wallet connected'));
signer.on('disconnect', () => console.log('Wallet disconnected'));
signer.on('accountsChanged', (accounts) => console.log('Accounts:', accounts));
```