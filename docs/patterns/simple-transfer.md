# Simple CKB Transfer Pattern (CCC - Recommended)

## Description

Modern frontend pattern for transferring CKB using the CCC SDK (recommended over Lumos). Demonstrates account generation, balance checking, intuitive transaction construction, automatic fee handling, and transaction sending with TypeScript type safety and simplified APIs for dApp development.

## Purpose
Modern frontend/dApp pattern for transferring CKB using the **CCC SDK (recommended)**. Shows:
- Account generation from private key
- Balance checking
- Intuitive transaction construction
- Automatic fee handling
- Transaction sending with modern APIs

## Why Use CCC Over Lumos
- **Simpler API**: More intuitive transaction building
- **Better TypeScript**: Full type safety and auto-completion  
- **Automatic Management**: Auto-completes inputs and fees
- **Modern Wallets**: Built-in support for multi-chain wallets
- **Active Development**: Newest SDK with ongoing improvements

## Complete Working Example (CCC)

```typescript
import { ccc } from "@ckb-ccc/ccc";

type Account = {
  lockScript: ccc.Script;
  address: string;
  pubKey: string;
};

// Initialize CCC client (configure for your network)
const cccClient = new ccc.ClientPublicTestnet();

export const generateAccountFromPrivateKey = async (
  privKey: string
): Promise<Account> => {
  const signer = new ccc.SignerCkbPrivateKey(cccClient, privKey);
  const lock = await signer.getAddressObjSecp256k1();
  return {
    lockScript: lock.script,
    address: lock.toString(),
    pubKey: signer.publicKey,
  };
};

export async function capacityOf(address: string): Promise<bigint> {
  const addr = await ccc.Address.fromString(address, cccClient);
  let balance = await cccClient.getBalance([addr.script]);
  return balance;
}

export async function transfer(
  toAddress: string,
  amountInCKB: string,
  signerPrivateKey: string
): Promise<string> {
  const signer = new ccc.SignerCkbPrivateKey(cccClient, signerPrivateKey);
  const { script: toLock } = await ccc.Address.fromString(toAddress, cccClient);

  // Build transaction with output
  const tx = ccc.Transaction.from({
    outputs: [{ lock: toLock }],
    outputsData: [],
  });

  // Set output capacity
  tx.outputs.forEach((output, i) => {
    if (output.capacity > ccc.fixedPointFrom(amountInCKB)) {
      throw new Error(`Insufficient capacity at output ${i}`);
    }
    output.capacity = ccc.fixedPointFrom(amountInCKB);
  });

  // Complete inputs and fees automatically
  await tx.completeInputsByCapacity(signer);
  await tx.completeFeeBy(signer, 1000);
  
  // Sign and send
  const txHash = await signer.sendTransaction(tx);
  console.log(`Transaction sent: https://pudge.explorer.nervos.org/transaction/${txHash}`);

  return txHash;
}

// Utility functions
export function shannonToCKB(amount: bigint): bigint {
  return amount / 100000000n;
}

export async function wait(seconds: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, seconds * 1000));
}
```

## Key Patterns
1. **Client Setup**: Initialize CCC client for target network
2. **Account Generation**: Use `SignerCkbPrivateKey` with secp256k1
3. **Address Conversion**: `Address.fromString()` for lock script extraction
4. **Balance Query**: `getBalance()` with lock script array
5. **Transaction Building**: `Transaction.from()` with outputs structure
6. **Automatic Completion**: `completeInputsByCapacity()` and `completeFeeBy()`
7. **Units**: CKB = 10^8 Shannon (smallest unit)

## Network Configuration
```typescript
// Testnet
const client = new ccc.ClientPublicTestnet();

// Mainnet
const client = new ccc.ClientPublicMainnet();

// Custom node
const client = new ccc.Client({
  url: "https://your-node.com/rpc",
});
```

## Error Handling
```typescript
try {
  const txHash = await transfer(toAddress, amount, privateKey);
  console.log("Transfer successful:", txHash);
} catch (error) {
  console.error("Transfer failed:", error.message);
  // Handle specific error types
  if (error.message.includes("Insufficient")) {
    console.log("Not enough CKB balance");
  }
}
```

## Legacy Lumos Approach (Not Recommended for New Projects)

If you must use Lumos for existing projects:

```typescript
import { generateDefaultScriptInfos } from "@ckb-ccc/lumos-patches";
import { registerCustomLockScriptInfos } from "@ckb-lumos/common-scripts";
import { TransactionSkeleton } from "@ckb-lumos/helpers";

// Apply CCC patches for wallet compatibility
registerCustomLockScriptInfos(generateDefaultScriptInfos());

// More complex transaction building with Lumos
let txSkeleton = new TransactionSkeleton({
    cellProvider: indexer,
});
txSkeleton = await common.transfer(
    txSkeleton,
    fromAddresses,
    toAddress,
    amount
);
```

**Note**: While Lumos still works, CCC provides a much better developer experience and is actively maintained for modern CKB development.

## When to Use CCC (Recommended)
- **All new projects** - CCC is the current recommended SDK
- Basic CKB transfers in dApps
- Wallet implementations requiring multi-chain support
- Payment processing with modern APIs
- Any production application requiring reliability

## When to Use Lumos (Legacy)
- Existing Lumos codebases that are hard to migrate
- Projects requiring specific Lumos-only features
- Gradual migration scenarios

## Migration Recommendation
**For new projects**: Always use CCC
**For existing projects**: Consider migrating to CCC for better maintainability and features