## Description

Complete xUDT token minting example demonstrating owner mode setup, minting transactions, token amount encoding, extension configuration, and proper cell dep management with TypeScript code.

## Related Resources

- xUDT Protocol: ckb-dev-context://protocols/xudt-protocol
- Token Creation: ckb-dev-context://patterns/token-creation
- Troubleshooting: ckb-dev-context://troubleshooting/xudt-errors

## Complete xUDT Minting Example

```typescript
import { ccc } from "@ckb-ccc/core";
import { blockchain } from "@ckb-lumos/base";

// xUDT constants
const XUDT_CODE_HASH = {
  mainnet: "0x50bd8d6680b8b9cf98b73f3c08faf8b2a21914311954118ad6609be6e78a1b95",
  testnet: "0x25c29dc317811a6f6f3985a7a9ebc4838bd388d19d0feeecf0bcd60f6c0975bb"
};

interface XudtConfig {
  name: string;
  symbol: string;
  decimals: number;
  totalSupply: bigint;
  ownerLock: ccc.Script;
}

// Step 1: Deploy xUDT with owner mode
async function deployXudt(config: XudtConfig): Promise<string> {
  const client = new ccc.ClientPublicTestnet();
  const signer = new ccc.SignerCkbPrivateKey(client, process.env.PRIVATE_KEY!);
  
  // Create unique type ID for the token
  const typeId = generateTypeId();
  
  // Build xUDT type script
  const xudtType = {
    codeHash: XUDT_CODE_HASH.testnet,
    hashType: "type" as const,
    args: typeId // Type ID as args
  };
  
  // Encode initial supply with owner mode extension
  const cellData = encodeXudtData(config.totalSupply, {
    ownerMode: true,
    ownerLockHash: config.ownerLock.hash()
  });
  
  // Create deployment transaction
  const tx = ccc.Transaction.from({
    outputs: [{
      lock: config.ownerLock,
      type: xudtType,
      capacity: calculateMinCapacity(cellData)
    }],
    outputsData: [cellData]
  });
  
  // Add type ID cell dep
  tx.cellDeps.push(getTypeIdCellDep());
  
  // Complete and sign
  await tx.completeInputsByCapacity(signer);
  await tx.completeFeeBy(signer, 1000);
  const signedTx = await signer.signTransaction(tx);
  
  // Send transaction
  const txHash = await client.sendTransaction(signedTx);
  console.log("xUDT deployed:", {
    txHash,
    typeScript: xudtType,
    totalSupply: config.totalSupply.toString()
  });
  
  return txHash;
}

// Step 2: Mint additional tokens (owner only)
async function mintXudt(
  xudtTypeScript: ccc.Script,
  mintAmount: bigint,
  ownerSigner: ccc.Signer
): Promise<string> {
  const client = ownerSigner.client;
  
  // Find existing xUDT cell owned by minter
  const ownerLock = await ownerSigner.getAddressObj();
  const xudtCells = await findXudtCells(xudtTypeScript, ownerLock.script);
  
  if (xudtCells.length === 0) {
    throw new Error("No xUDT cells found for owner");
  }
  
  const inputCell = xudtCells[0];
  const currentAmount = decodeXudtAmount(inputCell.data);
  const newAmount = currentAmount + mintAmount;
  
  // Build minting transaction
  const tx = ccc.Transaction.from({
    inputs: [{
      previousOutput: inputCell.outPoint,
      since: "0x0"
    }],
    outputs: [{
      lock: ownerLock.script,
      type: xudtTypeScript,
      capacity: inputCell.capacity
    }],
    outputsData: [
      encodeXudtData(newAmount, {
        ownerMode: true,
        ownerLockHash: ownerLock.script.hash()
      })
    ]
  });
  
  // Add xUDT cell dep
  tx.cellDeps.push(getXudtCellDep());
  
  // Complete and sign
  await tx.completeInputsByCapacity(ownerSigner);
  await tx.completeFeeBy(ownerSigner, 1000);
  const signedTx = await ownerSigner.signTransaction(tx);
  
  const txHash = await client.sendTransaction(signedTx);
  console.log("Minted tokens:", {
    txHash,
    amount: mintAmount.toString(),
    newTotal: newAmount.toString()
  });
  
  return txHash;
}

// Step 3: Transfer tokens to users
async function transferXudt(
  xudtTypeScript: ccc.Script,
  fromSigner: ccc.Signer,
  toAddress: string,
  amount: bigint
): Promise<string> {
  const client = fromSigner.client;
  const toLock = ccc.Address.fromString(toAddress, client).script;
  
  // Find sender's xUDT cells
  const fromLock = await fromSigner.getAddressObj();
  const senderCells = await findXudtCells(xudtTypeScript, fromLock.script);
  
  // Collect enough tokens
  const { cells, totalAmount } = collectXudtCells(senderCells, amount);
  
  if (totalAmount < amount) {
    throw new Error(`Insufficient balance: ${totalAmount} < ${amount}`);
  }
  
  const change = totalAmount - amount;
  
  // Build transfer transaction
  const tx = ccc.Transaction.from({
    inputs: cells.map(cell => ({
      previousOutput: cell.outPoint,
      since: "0x0"
    })),
    outputs: [
      // Recipient output
      {
        lock: toLock,
        type: xudtTypeScript,
        capacity: calculateMinCapacity(encodeXudtAmount(amount))
      },
      // Change output (if any)
      ...(change > 0n ? [{
        lock: fromLock.script,
        type: xudtTypeScript,
        capacity: calculateMinCapacity(encodeXudtAmount(change))
      }] : [])
    ],
    outputsData: [
      encodeXudtAmount(amount),
      ...(change > 0n ? [encodeXudtAmount(change)] : [])
    ]
  });
  
  // Add cell deps
  tx.cellDeps.push(getXudtCellDep());
  tx.cellDeps.push(getSecp256k1CellDep());
  
  // Complete and sign
  await tx.completeInputsByCapacity(fromSigner);
  await tx.completeFeeBy(fromSigner, 1000);
  const signedTx = await fromSigner.signTransaction(tx);
  
  const txHash = await client.sendTransaction(signedTx);
  console.log("Transfer complete:", {
    txHash,
    to: toAddress,
    amount: amount.toString()
  });
  
  return txHash;
}

// Helper: Encode xUDT data with extensions
function encodeXudtData(
  amount: bigint,
  extensions?: {
    ownerMode?: boolean;
    ownerLockHash?: string;
    regulatory?: RegulatoryConfig;
  }
): Uint8Array {
  const data: number[] = [];
  
  // Encode amount as 16-byte little-endian
  const amountBytes = new Uint8Array(16);
  const view = new DataView(amountBytes.buffer);
  view.setBigUint64(0, amount & 0xFFFFFFFFFFFFFFFFn, true);
  view.setBigUint64(8, amount >> 64n, true);
  data.push(...amountBytes);
  
  // Add extensions if present
  if (extensions?.ownerMode && extensions.ownerLockHash) {
    // Owner mode flag
    data.push(0x01);
    // Owner lock hash (32 bytes)
    const lockHash = extensions.ownerLockHash.replace("0x", "");
    const hashBytes = Buffer.from(lockHash, "hex");
    data.push(...hashBytes);
  }
  
  if (extensions?.regulatory) {
    // Regulatory extension
    data.push(0x02);
    data.push(extensions.regulatory.flags);
    // Add regulatory data...
  }
  
  return new Uint8Array(data);
}

// Helper: Decode xUDT amount from cell data
function decodeXudtAmount(data: Uint8Array): bigint {
  if (data.length < 16) {
    throw new Error("Invalid xUDT data: too short");
  }
  
  const view = new DataView(data.buffer, data.byteOffset, 16);
  const low = view.getBigUint64(0, true);
  const high = view.getBigUint64(8, true);
  
  return (high << 64n) | low;
}

// Helper: Find xUDT cells
async function findXudtCells(
  typeScript: ccc.Script,
  lockScript?: ccc.Script
): Promise<ccc.Cell[]> {
  const client = new ccc.ClientPublicTestnet();
  
  const searchParams: any = {
    script: typeScript,
    scriptType: "type"
  };
  
  if (lockScript) {
    searchParams.filter = {
      script: lockScript
    };
  }
  
  const collector = client.findCells(searchParams);
  const cells: ccc.Cell[] = [];
  
  for await (const cell of collector) {
    cells.push(cell);
  }
  
  return cells;
}

// Helper: Collect cells for transfer
function collectXudtCells(
  cells: ccc.Cell[],
  requiredAmount: bigint
): { cells: ccc.Cell[]; totalAmount: bigint } {
  const sorted = cells.sort((a, b) => {
    const amountA = decodeXudtAmount(a.data);
    const amountB = decodeXudtAmount(b.data);
    return Number(amountB - amountA); // Sort descending
  });
  
  const collected: ccc.Cell[] = [];
  let totalAmount = 0n;
  
  for (const cell of sorted) {
    collected.push(cell);
    totalAmount += decodeXudtAmount(cell.data);
    
    if (totalAmount >= requiredAmount) {
      break;
    }
  }
  
  return { cells: collected, totalAmount };
}

// Helper: Calculate minimum capacity
// Simplified calculation for xUDT. See ckb-dev-context://concepts-for-coding/cell-lifecycle for details
function calculateMinCapacity(data: Uint8Array): bigint {
  const base = 8; // Capacity field
  const dataSize = data.length;
  const lockSize = 53; // Secp256k1 lock: code_hash (32) + hash_type (1) + args (20)
  const typeSize = 65; // xUDT type: code_hash (32) + hash_type (1) + unique_id (32)

  return BigInt(base + dataSize + lockSize + typeSize) * 100000000n;
}

// Helper: Get cell dependencies
function getXudtCellDep(): ccc.CellDep {
  return {
    outPoint: {
      txHash: "0xc07844ce21b38e4b071dd0e1ee3b0e27afd8d7532491327f39b786343f558ab7",
      index: "0x0"
    },
    depType: "code"
  };
}

function getSecp256k1CellDep(): ccc.CellDep {
  return {
    outPoint: {
      txHash: "0x71a7ba8fc96349fea0ed3a5c47992e3b4084b031a42264a018e0072e8172e46c",
      index: "0x0"
    },
    depType: "depGroup"
  };
}

function getTypeIdCellDep(): ccc.CellDep {
  return {
    outPoint: {
      txHash: "0x00000000000000000000000000000000000000000000000000000000000000",
      index: "0x0"
    },
    depType: "code"
  };
}

// Helper: Generate unique type ID
function generateTypeId(): string {
  // In production, use first input's outpoint hash
  // This is a placeholder
  return "0x" + "00".repeat(32);
}

// Advanced: Burn tokens (owner only)
async function burnXudt(
  xudtTypeScript: ccc.Script,
  burnAmount: bigint,
  ownerSigner: ccc.Signer
): Promise<string> {
  const ownerLock = await ownerSigner.getAddressObj();
  const xudtCells = await findXudtCells(xudtTypeScript, ownerLock.script);
  
  const inputCell = xudtCells[0];
  const currentAmount = decodeXudtAmount(inputCell.data);
  
  if (currentAmount < burnAmount) {
    throw new Error("Insufficient balance for burn");
  }
  
  const newAmount = currentAmount - burnAmount;
  
  const tx = ccc.Transaction.from({
    inputs: [{
      previousOutput: inputCell.outPoint,
      since: "0x0"
    }],
    outputs: newAmount > 0n ? [{
      lock: ownerLock.script,
      type: xudtTypeScript,
      capacity: inputCell.capacity
    }] : [],
    outputsData: newAmount > 0n ? [
      encodeXudtData(newAmount, {
        ownerMode: true,
        ownerLockHash: ownerLock.script.hash()
      })
    ] : []
  });
  
  // If burning all tokens, convert to normal cell
  if (newAmount === 0n) {
    tx.outputs.push({
      lock: ownerLock.script,
      type: undefined,
      capacity: inputCell.capacity
    });
    tx.outputsData.push("0x");
  }
  
  tx.cellDeps.push(getXudtCellDep());
  
  await tx.completeInputsByCapacity(ownerSigner);
  await tx.completeFeeBy(ownerSigner, 1000);
  const signedTx = await ownerSigner.signTransaction(tx);
  
  const txHash = await ownerSigner.client.sendTransaction(signedTx);
  console.log("Burned tokens:", {
    txHash,
    burned: burnAmount.toString(),
    remaining: newAmount.toString()
  });
  
  return txHash;
}
```

## Usage Example

```typescript
async function main() {
  // Deploy new xUDT token
  const ownerSigner = new ccc.SignerCkbPrivateKey(
    new ccc.ClientPublicTestnet(),
    process.env.OWNER_KEY!
  );
  
  const config: XudtConfig = {
    name: "My Token",
    symbol: "MTK",
    decimals: 18,
    totalSupply: 1000000n * 10n ** 18n, // 1 million tokens
    ownerLock: await ownerSigner.getAddressObj().script
  };
  
  const deployTx = await deployXudt(config);
  console.log("Token deployed:", deployTx);
  
  // Mint additional tokens
  const xudtType = {
    codeHash: XUDT_CODE_HASH.testnet,
    hashType: "type" as const,
    args: "0x..." // Type ID from deployment
  };
  
  await mintXudt(xudtType, 500000n * 10n ** 18n, ownerSigner);
  
  // Transfer to users
  await transferXudt(
    xudtType,
    ownerSigner,
    "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsq...",
    1000n * 10n ** 18n
  );
}

main().catch(console.error);
```