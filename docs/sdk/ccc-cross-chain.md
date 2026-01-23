## Description

CKB Common Connector (CCC) SDK's cross-chain wallet integration capabilities. Build CKB applications that work seamlessly with Bitcoin, Ethereum, and Nostr wallets. Unified signer interfaces, transaction patterns, and practical examples for multi-chain dApp development on CKB.

## Overview

The CCC SDK provides a unified interface for integrating multiple blockchain wallets with CKB applications. This enables users to interact with CKB using their existing Bitcoin, Ethereum, or other blockchain wallets.

## Supported Wallet Types

### Bitcoin Wallets
- OKX Wallet
- UniSat
- Xverse
- UTXO Global (multi-chain UTXO support)

### Ethereum Wallets
- EIP-6963 compliant wallets (MetaMask, etc.)
- Any EIP-1193 compatible provider

### Other Protocols
- JoyID (WebAuthn-based)
- Nostr (NIP-07 protocol)

## Package Structure

Each wallet type is provided as a separate package:
- `@ckb-ccc/okx` - OKX wallet
- `@ckb-ccc/uni-sat` - UniSat wallet
- `@ckb-ccc/xverse` - Xverse wallet
- `@ckb-ccc/utxo-global` - UTXO Global wallet
- `@ckb-ccc/eip6963` - EIP-6963 Ethereum wallets
- `@ckb-ccc/nip07` - Nostr NIP-07 wallets
- `@ckb-ccc/joy-id` - JoyID wallet

## Bitcoin Wallet Integration

### OKX Wallet

```typescript
import { ccc } from "@ckb-ccc/ccc";
import { Okx } from "@ckb-ccc/okx";

// Initialize client
const client = new ccc.ClientPublicTestnet();

// Create OKX Bitcoin signer
// OKX provides separate providers for different chains
const signer = new Okx.BitcoinSigner(client, {
  bitcoin: window.okxwallet?.bitcoin,
  bitcoinTestnet: window.okxwallet?.bitcoinTestnet,
});

// Connect and get CKB address
await signer.connect();
const addresses = await signer.getAddressObjs();
const ckbAddress = addresses[0].toString();
```

### UniSat Wallet

```typescript
import { ccc } from "@ckb-ccc/ccc";
import { UniSat } from "@ckb-ccc/uni-sat";

const client = new ccc.ClientPublicTestnet();
const signer = new UniSat.Signer(client, window.unisat);

await signer.connect();
const address = await signer.getAddressObjs();
```

### Transaction Signing with Bitcoin Wallet

```typescript
// Build CKB transaction
const tx = ccc.Transaction.from({
  outputs: [{
    lock: recipientLock,
    capacity: ccc.fixedPointFrom(100)
  }],
  outputsData: [new Uint8Array()]
});

// Complete and sign with Bitcoin wallet
await tx.completeInputsByCapacity(signer);
await tx.completeFeeBy(signer);
const txHash = await signer.sendTransaction(tx);
```

## Ethereum Wallet Integration

### EIP-6963 Wallet Discovery

The EIP-6963 standard provides a way to discover all available Ethereum wallets. CCC uses event-based discovery:

```typescript
import { ccc } from "@ckb-ccc/ccc";
import { Eip6963 } from "@ckb-ccc/eip6963";

const client = new ccc.ClientPublicTestnet();

// Create signer factory for wallet discovery
const factory = new Eip6963.SignerFactory(client);

// Subscribe to discover available wallets
const unsubscribe = factory.subscribeSigners((signer, detail) => {
  console.log("Discovered wallet:", detail?.info.name);
  console.log("Wallet icon:", detail?.info.icon);

  // Connect to this wallet
  signer.connect().then(() => {
    console.log("Connected to", detail?.info.name);
  });
});

// Later: stop listening for new wallets
unsubscribe();
```

### Direct EIP-1193 Provider

```typescript
import { Eip6963 } from "@ckb-ccc/eip6963";

// Connect directly to window.ethereum (legacy approach)
if (window.ethereum) {
  const signer = new Eip6963.Signer(client, window.ethereum);
  await signer.connect();
  const evmAddress = await signer.getEvmAccount();
  const ckbAddresses = await signer.getAddressObjs();
}
```

### Message Signing

```typescript
// Sign arbitrary messages (useful for authentication)
const message = "Login to CKB dApp";
const signature = await signer.signMessageRaw(message);
```

## Nostr Wallet Integration (NIP-07)

```typescript
import { ccc } from "@ckb-ccc/ccc";
import { Nip07 } from "@ckb-ccc/nip07";

const client = new ccc.ClientPublicTestnet();

// Create Nostr signer (requires NIP-07 compatible extension)
const signer = new Nip07.Signer(client, window.nostr);

await signer.connect();
const publicKey = await signer.getNostrPublicKey();
const addresses = await signer.getAddressObjs();
```

## JoyID Integration (WebAuthn)

```typescript
import { ccc } from "@ckb-ccc/ccc";
import { JoyId } from "@ckb-ccc/joy-id";

const client = new ccc.ClientPublicTestnet();

// Create JoyID CKB signer
const signer = new JoyId.CkbSigner(
  client,
  "My dApp Name",              // App name shown in JoyID
  "https://example.com/icon.png" // App icon
);

// WebAuthn-based authentication
await signer.connect();

// Biometric signing
const tx = ccc.Transaction.from({ /* ... */ });
await tx.completeInputsByCapacity(signer);
const signedTx = await signer.signOnlyTransaction(tx);
```

## Multi-Wallet Support Pattern

```typescript
import { ccc } from "@ckb-ccc/ccc";
import { Okx } from "@ckb-ccc/okx";
import { UniSat } from "@ckb-ccc/uni-sat";
import { Eip6963 } from "@ckb-ccc/eip6963";
import { Nip07 } from "@ckb-ccc/nip07";

class MultiWalletManager {
  private client: ccc.Client;
  private signers: Map<string, ccc.Signer> = new Map();

  constructor() {
    this.client = new ccc.ClientPublicTestnet();
  }

  async addWallet(type: string): Promise<ccc.Signer | null> {
    let signer: ccc.Signer;

    switch(type) {
      case 'okx':
        if (!window.okxwallet?.bitcoin) return null;
        signer = new Okx.BitcoinSigner(this.client, {
          bitcoin: window.okxwallet.bitcoin,
          bitcoinTestnet: window.okxwallet.bitcoinTestnet,
        });
        break;
      case 'unisat':
        if (!window.unisat) return null;
        signer = new UniSat.Signer(this.client, window.unisat);
        break;
      case 'metamask':
        if (!window.ethereum) return null;
        signer = new Eip6963.Signer(this.client, window.ethereum);
        break;
      case 'nostr':
        if (!window.nostr) return null;
        signer = new Nip07.Signer(this.client, window.nostr);
        break;
      default:
        return null;
    }

    await signer.connect();
    this.signers.set(type, signer);
    return signer;
  }

  getSigner(type: string): ccc.Signer | undefined {
    return this.signers.get(type);
  }
}
```

## Best Practices

### Wallet Detection

```typescript
function detectAvailableWallets(): string[] {
  const wallets: string[] = [];

  // Check Bitcoin wallets
  if (window.okxwallet?.bitcoin) wallets.push('okx');
  if (window.unisat) wallets.push('unisat');

  // Check Ethereum
  if (window.ethereum) wallets.push('ethereum');

  // Check Nostr
  if (window.nostr) wallets.push('nostr');

  return wallets;
}
```

### Error Handling

```typescript
async function safeConnect(signer: ccc.Signer): Promise<boolean> {
  try {
    await signer.connect();
    return true;
  } catch (error: any) {
    if (error.code === 4001) {
      // User rejected connection
      console.log("User rejected wallet connection");
      return false;
    }
    throw error;
  }
}
```

### Connection State

```typescript
// Check if signer is connected
const connected = await signer.isConnected();

// Listen for account changes (if supported)
const unsubscribe = signer.onReplaced(() => {
  console.log("Account changed, re-fetch addresses");
});

// Cleanup when done
unsubscribe();
```

## Integration Examples

### DeFi Application

```typescript
async function swapTokens(
  signer: ccc.Signer,
  fromToken: string,
  toToken: string,
  amount: bigint
) {
  // Build swap transaction
  const swapTx = await buildSwapTransaction(fromToken, toToken, amount);

  // Complete inputs and fee using any wallet type
  await swapTx.completeInputsByCapacity(signer);
  await swapTx.completeFeeBy(signer);

  // Sign and send
  return signer.sendTransaction(swapTx);
}
```

### NFT Marketplace

```typescript
async function listNFT(nftId: string, price: bigint) {
  // Works with any connected signer
  const addresses = await signer.getAddressObjs();
  const ownerLock = addresses[0].script;

  const listingTx = await createNFTListing(nftId, price, ownerLock);
  return signer.sendTransaction(listingTx);
}
```

## Troubleshooting

### Common Issues

1. **Wallet Not Detected**: Ensure wallet extension is installed and unlocked
2. **Connection Rejected**: User clicked "Reject" - handle error code 4001
3. **Wrong Network**: Check client network matches wallet network preference
4. **Signing Failures**: Verify transaction format and cell dependencies

### Debug Tips

```typescript
// Get signer type for debugging
console.log("Signer type:", signer.type);  // e.g., "BTC", "EVM", "CKB"

// Check addresses
const addrs = await signer.getAddressObjs();
addrs.forEach((addr, i) => {
  console.log(`Address ${i}:`, addr.toString());
  console.log(`Lock script:`, addr.script);
});
```
