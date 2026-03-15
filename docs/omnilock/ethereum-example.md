## Description

Complete Ethereum wallet integration example for Omnilock showing MetaMask connection, signature generation, witness construction, and transaction building with proper authentication flags and recovery ID handling.

## Related Resources

- Omnilock Protocol: ckb://docs/omnilock/protocol
- Omnilock Development: ckb://docs/omnilock/development
- Troubleshooting: ckb://docs/omnilock/errors

## Complete Ethereum Wallet Example

```typescript
import { ccc } from "@ckb-ccc/core";
import { ethers } from "ethers";

// Constants
const OMNILOCK_CODE_HASH = "0xf329effd1c475a2978453c8600e1eaf0bc2087ee093c3ee64cc96ec6847752cb"; // Testnet

async function ethereumOmnilockExample() {
  // Connect to MetaMask
  const provider = new ethers.providers.Web3Provider(window.ethereum);
  await provider.send("eth_requestAccounts", []);
  const ethSigner = provider.getSigner();
  const ethAddress = await ethSigner.getAddress();
  
  // Initialize CKB client
  const client = new ccc.ClientPublicTestnet();
  
  // Create Omnilock args for Ethereum
  const omnilockArgs = buildEthereumOmnilockArgs(ethAddress);
  
  // Build lock script
  const omnilockScript = {
    codeHash: OMNILOCK_CODE_HASH,
    hashType: "type",
    args: omnilockArgs
  };
  
  // Create transaction
  const tx = ccc.Transaction.from({
    outputs: [{
      lock: omnilockScript,
      capacity: ccc.fixedPointFrom("100")
    }],
    outputsData: ["0x"]
  });
  
  // Sign with Ethereum wallet
  const signedTx = await signEthereumOmnilock(tx, ethSigner);
  
  // Send transaction
  const txHash = await client.sendTransaction(signedTx);
  console.log("Transaction sent:", txHash);
}

function buildEthereumOmnilockArgs(ethAddress: string): string {
  // Remove 0x prefix and convert to lowercase
  const address = ethAddress.toLowerCase().replace("0x", "");
  
  // Omnilock args structure for Ethereum:
  // - Auth flag: 0x01 (1 byte)
  // - Ethereum address: 20 bytes
  // - Omnilock flags: 0x00 (1 byte) - no special modes
  // - Padding: zeros to make 32 bytes total
  
  const authFlag = "01"; // Ethereum auth
  const omnilockFlags = "00"; // No special modes
  const padding = "00".repeat(10); // Pad to 32 bytes
  
  return "0x" + authFlag + address + omnilockFlags + padding;
}

async function signEthereumOmnilock(
  tx: ccc.Transaction,
  ethSigner: ethers.Signer
): Promise<ccc.Transaction> {
  // Calculate signing message hash
  const message = tx.signingHasher.hash();
  
  // Ethereum personal sign
  const signature = await ethSigner.signMessage(message);
  
  // Parse signature components
  const sig = ethers.utils.splitSignature(signature);
  
  // Build Omnilock witness
  const witness = buildEthereumWitness(sig);
  
  // Add witness to transaction
  tx.witnesses[0] = witness;
  
  return tx;
}

function buildEthereumWitness(sig: ethers.utils.Signature): Uint8Array {
  // Omnilock Ethereum witness structure:
  // - Lock: 65 bytes signature (r: 32, s: 32, recovery: 1)
  // - Input type: empty
  // - Output type: empty
  
  const witness = new Uint8Array(65);
  
  // Copy r (32 bytes)
  const r = ethers.utils.arrayify(sig.r);
  witness.set(r, 0);
  
  // Copy s (32 bytes)
  const s = ethers.utils.arrayify(sig.s);
  witness.set(s, 32);
  
  // Set recovery ID (1 byte)
  witness[64] = sig.recoveryParam || 0;
  
  // Pack as WitnessArgs
  return ccc.WitnessArgs.from({
    lock: witness,
    inputType: undefined,
    outputType: undefined
  }).toBytes();
}

// Advanced: Ethereum Display Mode (shows address in wallet)
async function ethereumDisplayModeExample() {
  const provider = new ethers.providers.Web3Provider(window.ethereum);
  const ethSigner = provider.getSigner();
  
  // Use display mode flag 0x12 instead of 0x01
  const omnilockArgs = buildDisplayModeArgs(await ethSigner.getAddress());
  
  // Sign with typed data for better UX
  const signature = await signTypedData(ethSigner);
  
  return signature;
}

function buildDisplayModeArgs(ethAddress: string): string {
  const address = ethAddress.toLowerCase().replace("0x", "");
  const authFlag = "12"; // Ethereum display mode
  const omnilockFlags = "00";
  const padding = "00".repeat(10);
  
  return "0x" + authFlag + address + omnilockFlags + padding;
}

async function signTypedData(signer: ethers.Signer): Promise<string> {
  const domain = {
    name: "CKB Omnilock",
    version: "1",
    chainId: 1,
  };
  
  const types = {
    CKBTransaction: [
      { name: "hash", type: "bytes32" },
      { name: "amount", type: "uint256" },
      { name: "recipient", type: "address" }
    ]
  };
  
  const value = {
    hash: "0x" + "00".repeat(32),
    amount: ethers.utils.parseEther("100"),
    recipient: await signer.getAddress()
  };
  
  return signer._signTypedData(domain, types, value);
}

// Unlock Omnilock cells with Ethereum wallet
async function unlockWithEthereum(
  omnilockCell: ccc.Cell,
  ethSigner: ethers.Signer
): Promise<ccc.Transaction> {
  const client = new ccc.ClientPublicTestnet();
  
  // Build spending transaction
  const tx = ccc.Transaction.from({
    inputs: [{
      previousOutput: omnilockCell.outPoint,
      since: "0x0"
    }],
    outputs: [{
      lock: await getReceiverLock(),
      capacity: omnilockCell.capacity - ccc.fixedPointFrom("0.001") // Minus fee
    }],
    outputsData: ["0x"]
  });
  
  // Add Omnilock cell dep
  tx.cellDeps.push({
    outPoint: {
      txHash: "0x9b819793a64463aed77c615d6cb226eea5487ccfc0783043a8a3d0c12e3e2d8f",
      index: "0x0"
    },
    depType: "code"
  });
  
  // Calculate message for signing
  const message = calculateSigningMessage(tx, 0);
  
  // Sign with Ethereum wallet
  const ethSignature = await ethSigner.signMessage(message);
  
  // Build and add witness
  tx.witnesses[0] = buildEthereumWitness(
    ethers.utils.splitSignature(ethSignature)
  );
  
  return tx;
}

// Helper: Calculate signing message for specific input
function calculateSigningMessage(
  tx: ccc.Transaction,
  inputIndex: number
): Uint8Array {
  const hasher = new ccc.CKBHasher();
  
  // Hash transaction excluding witnesses
  hasher.update(serializeTxWithoutWitnesses(tx));
  
  // Hash witness placeholder for signing input
  const witnessPlaceholder = new Uint8Array(65); // 65 bytes for signature
  hasher.update(serializeWitnessArgs({
    lock: witnessPlaceholder,
    inputType: undefined,
    outputType: undefined
  }));
  
  // Hash remaining witnesses as-is
  for (let i = inputIndex + 1; i < tx.inputs.length; i++) {
    hasher.update(tx.witnesses[i] || new Uint8Array());
  }
  
  return hasher.digest();
}
```

## Testing Utils

```typescript
// Test Ethereum Omnilock locally
async function testEthereumOmnilock() {
  // Create test wallet
  const testWallet = ethers.Wallet.createRandom();
  console.log("Test wallet address:", testWallet.address);
  
  // Build Omnilock args
  const args = buildEthereumOmnilockArgs(testWallet.address);
  console.log("Omnilock args:", args);
  
  // Generate test signature
  const testMessage = "0x" + "ff".repeat(32);
  const signature = await testWallet.signMessage(testMessage);
  console.log("Test signature:", signature);
  
  // Verify signature locally
  const recovered = ethers.utils.verifyMessage(testMessage, signature);
  console.log("Signature valid:", recovered === testWallet.address);
}

// Batch unlock multiple Omnilock cells
async function batchUnlockEthereum(
  cells: ccc.Cell[],
  ethSigner: ethers.Signer
): Promise<ccc.Transaction> {
  // Group cells by lock script
  const grouped = cells.reduce((acc, cell) => {
    const key = cell.cellOutput.lock.hash();
    if (!acc[key]) acc[key] = [];
    acc[key].push(cell);
    return acc;
  }, {});
  
  // Build transaction with all inputs
  const tx = ccc.Transaction.from({
    inputs: cells.map(cell => ({
      previousOutput: cell.outPoint,
      since: "0x0"
    })),
    outputs: [{
      lock: await getReceiverLock(),
      capacity: cells.reduce((sum, cell) => 
        sum + BigInt(cell.capacity), 0n
      ) - ccc.fixedPointFrom("0.01") // Minus fee
    }],
    outputsData: ["0x"]
  });
  
  // Sign once for each lock group
  for (const [lockHash, groupCells] of Object.entries(grouped)) {
    const firstIndex = cells.indexOf(groupCells[0]);
    const message = calculateSigningMessage(tx, firstIndex);
    const signature = await ethSigner.signMessage(message);
    
    tx.witnesses[firstIndex] = buildEthereumWitness(
      ethers.utils.splitSignature(signature)
    );
    
    // Other cells in group use empty witness
    for (let i = 1; i < groupCells.length; i++) {
      const index = cells.indexOf(groupCells[i]);
      tx.witnesses[index] = "0x";
    }
  }
  
  return tx;
}
```