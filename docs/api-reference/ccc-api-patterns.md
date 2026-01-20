## Description

Practical API patterns for CCC SDK (CKB Common Chains Connector), covering client initialization, transaction construction, multi-wallet integration, and cross-chain compatibility. Includes TypeScript examples for address operations, balance queries, script building, error handling, and advanced transaction patterns. Modern CKB dApp development with support for Ethereum, Bitcoin, and other blockchain wallets.

## Client Initialization
```typescript
import { ccc } from "@ckb-ccc/core";

// Testnet client
const client = new ccc.ClientPublicTestnet();

// Mainnet client  
const client = new ccc.ClientPublicMainnet();

// Custom RPC client (extend ClientJsonRpc for custom endpoints)
// For most use cases, use the preset clients above
```

## Account and Address Operations
```typescript
// Create signer from private key
const signer = new ccc.SignerCkbPrivateKey(client, privateKey);

// Get secp256k1 address
const addressObj = await signer.getAddressObjSecp256k1();
const address = addressObj.toString();
const lockScript = addressObj.script;

// Parse address to script
const addr = await ccc.Address.fromString(address, client);
const script = addr.script;

// Generate address from script
const generatedAddr = ccc.Address.fromScript(script, client);
```

## Balance and Cell Queries
```typescript
// Get balance by lock scripts
const balance = await client.getBalance([lockScript]);

// Get live cells
const cells = await client.getCells({
  script: lockScript,
  scriptType: "lock"
}, "asc", "0x64");

// Get cell capacity
const capacity = await client.getCellsCapacity({
  script: lockScript,
  scriptType: "lock"
});
```

## Transaction Construction
```typescript
// Create basic transaction
const tx = ccc.Transaction.from({
  outputs: [
    {
      lock: toLockScript,
      capacity: ccc.fixedPointFrom("100")
    }
  ],
  outputsData: [new Uint8Array()],
});

// Add cell deps
tx.cellDeps.push({
  outPoint: {
    txHash: "0x...",
    index: "0x0"
  },
  depType: "depGroup"
});

// Complete inputs automatically
await tx.completeInputsByCapacity(signer);

// Add fee
await tx.completeFeeBy(signer, 1000);
```

## Transaction Sending
```typescript
// Sign and send transaction
const txHash = await signer.sendTransaction(tx);

// Just sign (don't send)
await signer.signTransaction(tx);

// Send pre-signed transaction
const txHash = await client.sendTransaction(tx);
```

## Script Building
```typescript
// Create script
const script: ccc.Script = {
  codeHash: "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
  hashType: "type",
  args: "0x..."
};

// Create script from known scripts (recommended)
const secp256k1Script = await ccc.Script.fromKnownScript(
  client,
  ccc.KnownScript.Secp256k1Blake160,
  args
);

// Create xUDT type script
const xudtScript = await ccc.Script.fromKnownScript(
  client,
  ccc.KnownScript.XUdt,
  ownerLockHash
);

// Create Omnilock script
const omnilockScript = await ccc.Script.fromKnownScript(
  client,
  ccc.KnownScript.OmniLock,
  omnilockArgs
);

// Available KnownScript values:
// - KnownScript.Secp256k1Blake160
// - KnownScript.Secp256k1Multisig
// - KnownScript.AnyoneCanPay
// - KnownScript.OmniLock
// - KnownScript.XUdt
// - KnownScript.NervosDao
// - KnownScript.JoyId
// - KnownScript.NostrLock
// - KnownScript.TypeId
// - KnownScript.COTA

// Manual script templates (for reference)
const SECP256K1_BLAKE160 = {
  codeHash: "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
  hashType: "type" as const
};

const OMNILOCK = {
  codeHash: "0x00000000000000000000000000000000000000000000000000545950455f4944",
  hashType: "type" as const  
};

const XUDT = {
  codeHash: "0x50bd8d6680b8b9cf98b73f3c08faf8b2a21914311954118ad6609be6e78a1b95",
  hashType: "data1" as const
};
```

## Units and Conversion
```typescript
// Convert CKB to Shannon (smallest unit)
const shannon = ccc.fixedPointFrom("100"); // 100 CKB = 100 * 10^8 Shannon

// Convert Shannon to CKB
const ckb = ccc.fixedPointToString(shannon);

// Manual conversion
const ckbAmount = Number(shannon) / 100000000;
```

## Error Handling
```typescript
try {
  const result = await signer.sendTransaction(tx);
} catch (error) {
  if (error.message.includes("InsufficientBalance")) {
    console.log("Not enough CKB");
  } else if (error.message.includes("InvalidTransaction")) {
    console.log("Transaction validation failed");
  } else {
    console.log("Unknown error:", error.message);
  }
}
```

## Multi-Wallet Integration
```typescript
// JoyID CKB wallet integration
import { JoyId } from "@ckb-ccc/joy-id";
const joyIdSigner = new JoyId.CkbSigner(
  client,
  "My App",           // App name
  "https://example.com/icon.png"  // App icon
);

// EIP-6963 Ethereum wallet discovery (MetaMask, etc.)
import { Eip6963 } from "@ckb-ccc/eip6963";
const eip6963Signer = new Eip6963.Signer(client, window.ethereum);

// UniSat Bitcoin wallet
import { UniSat } from "@ckb-ccc/uni-sat";
const unisatSigner = new UniSat.Signer(client, window.unisat);

// Connect to wallet
await joyIdSigner.connect();
const address = await joyIdSigner.getAddressObj();

// Using SignersFactory for wallet discovery
import { Eip6963 } from "@ckb-ccc/eip6963";
const factory = new Eip6963.SignersFactory(client);
factory.subscribeSigners((signer, detail) => {
  console.log("Discovered wallet:", detail?.info.name);
  // detail contains wallet metadata (name, icon, etc.)
});
```

## Advanced Transaction Patterns
```typescript
// Multi-output transaction
const tx = ccc.Transaction.from({
  outputs: [
    { lock: recipient1Lock, capacity: ccc.fixedPointFrom("100") },
    { lock: recipient2Lock, capacity: ccc.fixedPointFrom("200") },
    { lock: changeLock, capacity: ccc.fixedPointFrom("50") }
  ],
  outputsData: [new Uint8Array(), new Uint8Array(), new Uint8Array()]
});

// Transaction with data
const messageData = new TextEncoder().encode("Hello CKB");
const tx = ccc.Transaction.from({
  outputs: [{
    lock: recipientLock,
    capacity: ccc.fixedPointFrom(100 + messageData.length)
  }],
  outputsData: [messageData]
});

// Type script transaction (e.g., UDT transfer)
const tx = ccc.Transaction.from({
  outputs: [{
    lock: recipientLock,
    type: udtTypeScript,
    capacity: ccc.fixedPointFrom("142")
  }],
  outputsData: [ccc.numLeToBytes(1000000, 16)] // UDT amount
});
```

## Cell Collection and Filtering
```typescript
// Collect cells with filter
const collector = client.findCells({
  script: lockScript,
  scriptType: "lock",
  filter: {
    outputCapacityRange: ["0x0", "0xffffffffffffffff"]
  }
});

// Process cells in batches
for await (const cell of collector) {
  console.log("Cell capacity:", cell.cellOutput.capacity);
  console.log("Cell data:", cell.outputData);
}

// Find cells with type script
const udtCells = await client.getCells({
  script: udtTypeScript,
  scriptType: "type",
  filter: {
    script: ownerLockScript
  }
});
```

## Common Patterns
```typescript
// Check if address is valid
try {
  await ccc.Address.fromString(address, client);
  console.log("Valid address");
} catch {
  console.log("Invalid address");
}

// Wait for transaction confirmation
const txHash = await signer.sendTransaction(tx);
let confirmed = false;
while (!confirmed) {
  try {
    const txStatus = await client.getTransaction(txHash);
    if (txStatus && txStatus.txStatus.status === "committed") {
      confirmed = true;
    }
  } catch {
    // Transaction not found yet
  }
  await new Promise(resolve => setTimeout(resolve, 1000));
}

// Auto-completion with fee rate
await tx.completeInputsByCapacity(signer);
await tx.completeFeeBy(signer, 1000); // 1000 shannons per KB

// Manual witness handling
tx.witnesses[0] = ccc.WitnessArgs.from({
  lock: "0x" + "00".repeat(65), // Placeholder for signature
  inputType: udtTransferWitness,
  outputType: null
}).toBytes();
```