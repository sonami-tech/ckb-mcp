## Description

Guide to Script-Sourced Rich Information (SSRI) framework in the CCC SDK. Shows how smart contracts can provide metadata and advanced functions directly on-chain. Covers UDT token metadata, custom script functions, and dynamic information retrieval. Essential for developers building tokens and smart contracts that need to expose rich metadata without off-chain dependencies.

## Related Resources

- [ckb://docs/protocols/ssri](ckb://docs/protocols/ssri) - Extension protocol enabling CKB scripts to provide rich information through off-chain execution
- [ckb://docs/patterns/ssri-implementation-guide](ckb://docs/patterns/ssri-implementation-guide) - Implementation guide for Script-Sourced Rich Information in CKB smart contracts
- [ckb://docs/tools/ssri-server](ckb://docs/tools/ssri-server) - Comprehensive integration guide for SSRI server enabling off-chain CKB script execution

## Overview

SSRI (Script-Sourced Rich Information) is a framework that enables CKB smart contracts to provide metadata and execute functions directly on-chain. This eliminates the need for off-chain metadata storage and enables truly decentralized token and contract information.

## Core Concepts

### What is SSRI?

SSRI allows smart contracts to:
- Provide metadata (name, symbol, decimals, icons) directly from the contract
- Execute custom functions that return dynamic information
- Support complex queries without requiring off-chain infrastructure

### How It Works

1. **Contract Implementation**: Smart contracts implement SSRI-compatible functions
2. **On-Chain Execution**: CCC SDK executes these functions in CKB-VM
3. **Result Decoding**: SDK decodes the returned data into usable formats

## SSRI for UDT Tokens

### Basic Token Metadata

```typescript
import { ccc } from "@ckb-ccc/ccc";
import { Udt } from "@ckb-ccc/udt";

// Create UDT instance with SSRI support
// Requires: code cell OutPoint and the UDT type script
const udt = new Udt(
  { txHash: "0x...", index: 0 },  // Code cell out point
  tokenTypeScript                  // UDT type script
);

// Fetch token metadata via SSRI (separate method calls)
const nameRes = await udt.name();
const symbolRes = await udt.symbol();
const decimalsRes = await udt.decimals();

console.log({
  name: nameRes.res,        // "My Token" or undefined
  symbol: symbolRes.res,    // "MTK" or undefined
  decimals: decimalsRes.res // 8n or undefined (as bigint)
});
```

### Icon Retrieval

```typescript
// Get token icon as data URI
const iconRes = await udt.icon();
if (iconRes.res) {
  // icon is a data URI that can be used directly in img tags
  // e.g., "data:image/svg+xml;base64,..."
  document.getElementById('token-icon').src = iconRes.res;
}
```

### UDT Transfer

```typescript
import { Udt } from "@ckb-ccc/udt";

const udt = new Udt(codeOutPoint, typeScript);

// Transfer tokens to multiple recipients
const { res: tx } = await udt.transfer(
  signer,
  [
    { to: recipientLock1, amount: 100n },
    { to: recipientLock2, amount: 200n }
  ]
);

// Complete the transaction
const completedTx = await udt.completeBy(tx, signer);
await completedTx.completeInputsByCapacity(signer);
await completedTx.completeFeeBy(signer);
const txHash = await signer.sendTransaction(completedTx);
```

## Implementing SSRI in Smart Contracts

### Contract Structure

For a contract to support SSRI, it must handle specific function selectors:

```rust
// Rust smart contract example
use ckb_std::ckb_types::bytes::Bytes;

const SSRI_NAME: u32 = 0x00000001;
const SSRI_SYMBOL: u32 = 0x00000002;
const SSRI_DECIMALS: u32 = 0x00000003;
const SSRI_ICON: u32 = 0x00000004;

pub fn handle_ssri(function_id: u32) -> Result<Bytes, Error> {
    match function_id {
        SSRI_NAME => Ok(Bytes::from("My Token")),
        SSRI_SYMBOL => Ok(Bytes::from("MTK")),
        SSRI_DECIMALS => Ok(Bytes::from(vec![8])),
        SSRI_ICON => Ok(load_icon_data()),
        _ => Err(Error::UnknownFunction)
    }
}
```

### Advanced SSRI Functions

```rust
// Custom functions beyond standard metadata
const SSRI_TOTAL_SUPPLY: u32 = 0x00000010;
const SSRI_CIRCULATING_SUPPLY: u32 = 0x00000011;
const SSRI_HOLDERS_COUNT: u32 = 0x00000012;

pub fn handle_advanced_ssri(function_id: u32) -> Result<Bytes, Error> {
    match function_id {
        SSRI_TOTAL_SUPPLY => {
            let supply = calculate_total_supply();
            Ok(supply.to_le_bytes().to_vec().into())
        },
        SSRI_CIRCULATING_SUPPLY => {
            let circulating = calculate_circulating_supply();
            Ok(circulating.to_le_bytes().to_vec().into())
        },
        SSRI_HOLDERS_COUNT => {
            let count = count_token_holders();
            Ok(count.to_le_bytes().to_vec().into())
        },
        _ => handle_ssri(function_id)
    }
}
```

## Using SSRI with CCC SDK

### SSRI Executor Setup

```typescript
import { ssri } from "@ckb-ccc/ssri";

// Create JSON-RPC executor connected to SSRI server
const executor = new ssri.ExecutorJsonRpc("https://ssri-server.example.com/");

// Or create executor with custom requestor
const executor = new ssri.ExecutorJsonRpc(
  "https://ssri-server.example.com/",
  { timeout: 30000 }
);
```

### Direct SSRI Calls

```typescript
import { ssri } from "@ckb-ccc/ssri";

// Execute arbitrary SSRI function using the executor
const result = await executor.runScript(
  codeOutPoint,           // Code cell OutPoint
  "UDT.totalSupply",      // Method name (e.g., "UDT.name", "UDT.symbol")
  [],                     // Arguments as hex strings
  { script: contractScript }  // Context
);

// Result contains response and cell dependencies
const totalSupply = ccc.numFromBytes(result.res);
console.log("Cell deps used:", result.cellDeps);
```

### Creating SSRI-Compatible UDTs

```typescript
// Deploy UDT with SSRI metadata
const udtScript = await deploySSRIUDT({
  name: "My Token",
  symbol: "MTK",
  decimals: 8,
  icon: await loadIcon('./token-icon.svg'),
  // Additional SSRI functions
  customFunctions: {
    0x00000010: getTotalSupplyFunction(),
    0x00000011: getCirculatingSupplyFunction()
  }
});

const udt = new ccc.udt.Udt(udtScript, client);
```

### Caching SSRI Results

```typescript
class SSRICache {
  private cache = new Map<string, any>();
  
  async getTokenInfo(udt: ccc.udt.Udt): Promise<TokenInfo> {
    const key = udt.script.hash();
    
    if (this.cache.has(key)) {
      return this.cache.get(key);
    }
    
    const info = await udt.getInfo();
    this.cache.set(key, info);
    return info;
  }
}
```

## SSRI Patterns

### Using Executor for Custom Queries

```typescript
import { ssri } from "@ckb-ccc/ssri";
import { ccc } from "@ckb-ccc/ccc";

// Create executor
const executor = new ssri.ExecutorJsonRpc("https://ssri-server.example.com/");

// Query custom SSRI method
async function getTokenPrice(
  codeOutPoint: ccc.OutPointLike,
  tokenScript: ccc.Script
): Promise<bigint> {
  const result = await executor.runScript(
    codeOutPoint,
    "Token.price",  // Custom SSRI method
    [],
    { script: tokenScript }
  );

  return ccc.numFromBytes(result.res);
}
```

### Governance Parameters

```typescript
// Fetch on-chain governance settings via SSRI
async function getGovernanceParams(
  executor: ssri.Executor,
  codeOutPoint: ccc.OutPointLike,
  daoScript: ccc.Script
) {
  const [thresholdRes, durationRes] = await Promise.all([
    executor.runScript(codeOutPoint, "DAO.votingThreshold", [], { script: daoScript }),
    executor.runScript(codeOutPoint, "DAO.proposalDuration", [], { script: daoScript })
  ]);

  return {
    votingThreshold: ccc.numFromBytes(thresholdRes.res),
    proposalDuration: ccc.numFromBytes(durationRes.res)
  };
}
```

### Access Control Information

```typescript
// Query role-based permissions
async function checkPermissions(
  executor: ssri.Executor,
  codeOutPoint: ccc.OutPointLike,
  contractScript: ccc.Script,
  userLockHash: ccc.Hex
): Promise<{ admin: boolean; minter: boolean }> {
  const [isAdminRes, isMinterRes] = await Promise.all([
    executor.runScript(
      codeOutPoint,
      "Access.isAdmin",
      [userLockHash],
      { script: contractScript }
    ),
    executor.runScript(
      codeOutPoint,
      "Access.isMinter",
      [userLockHash],
      { script: contractScript }
    )
  ]);

  return {
    admin: ccc.bytesFrom(isAdminRes.res)[0] === 1,
    minter: ccc.bytesFrom(isMinterRes.res)[0] === 1
  };
}
```

## Best Practices

### Error Handling

```typescript
import { Udt } from "@ckb-ccc/udt";
import { ssri } from "@ckb-ccc/ssri";

async function safeGetTokenInfo(udt: Udt) {
  try {
    const [nameRes, symbolRes, decimalsRes] = await Promise.all([
      udt.name(),
      udt.symbol(),
      udt.decimals()
    ]);

    return {
      name: nameRes.res ?? 'Unknown Token',
      symbol: symbolRes.res ?? 'UNK',
      decimals: decimalsRes.res ?? 8n
    };
  } catch (error) {
    if (error instanceof ssri.ExecutorErrorExecutionFailed) {
      // SSRI not supported, return defaults
      return {
        name: 'Unknown Token',
        symbol: 'UNK',
        decimals: 8n
      };
    }
    throw error;
  }
}
```

### Performance Optimization

```typescript
// Batch SSRI calls for efficiency using Promise.all
async function batchSSRICalls(
  executor: ssri.Executor,
  codeOutPoint: ccc.OutPointLike,
  script: ccc.Script,
  methods: string[]
): Promise<ssri.ExecutorResponse<ccc.Hex>[]> {
  // Execute all methods in parallel
  return Promise.all(
    methods.map(method =>
      executor.runScript(codeOutPoint, method, [], { script })
    )
  );
}

// Usage
const results = await batchSSRICalls(
  executor,
  codeOutPoint,
  tokenScript,
  ["UDT.name", "UDT.symbol", "UDT.decimals", "UDT.icon"]
);
```

### Validation

```typescript
// Validate SSRI responses
function validateTokenInfo(info: any): boolean {
  return (
    typeof info.name === 'string' &&
    typeof info.symbol === 'string' &&
    typeof info.decimals === 'number' &&
    info.decimals >= 0 &&
    info.decimals <= 255
  );
}
```

## Integration Examples

### Token Explorer

```typescript
import { Udt } from "@ckb-ccc/udt";

async function displayTokenDetails(
  codeOutPoint: ccc.OutPointLike,
  typeScript: ccc.Script
) {
  const udt = new Udt(codeOutPoint, typeScript);

  // Fetch all metadata in parallel
  const [nameRes, symbolRes, decimalsRes, iconRes] = await Promise.all([
    udt.name(),
    udt.symbol(),
    udt.decimals(),
    udt.icon()
  ]);

  // Display in UI
  updateUI({
    name: nameRes.res ?? 'Unknown',
    symbol: symbolRes.res ?? 'UNK',
    decimals: Number(decimalsRes.res ?? 8n),
    icon: iconRes.res || '/default-token-icon.svg'
  });
}
```

### DeFi Integration

```typescript
import { Udt } from "@ckb-ccc/udt";

// Automatic token detection in DeFi app
async function addTokenToPool(
  codeOutPoint: ccc.OutPointLike,
  tokenScript: ccc.Script
) {
  const udt = new Udt(codeOutPoint, tokenScript);

  // Verify token has SSRI metadata
  const nameRes = await udt.name();
  const symbolRes = await udt.symbol();

  if (!nameRes.res || !symbolRes.res) {
    throw new Error('Token must provide name and symbol via SSRI');
  }

  // Add to liquidity pool
  await liquidityPool.addToken({
    script: tokenScript,
    name: nameRes.res,
    symbol: symbolRes.res
  });
}
```

## Troubleshooting

### Common Issues

1. **SSRI Not Supported**: Contract doesn't implement SSRI functions
2. **Invalid Response**: Contract returns malformed data
3. **Execution Failure**: Contract logic errors during SSRI execution
4. **Gas Limits**: Complex SSRI functions may exceed cycle limits

### Debugging

```typescript
import { ssri } from "@ckb-ccc/ssri";
import { ccc } from "@ckb-ccc/ccc";

// Debug SSRI calls
async function debugSSRI(
  executor: ssri.Executor,
  codeOutPoint: ccc.OutPointLike,
  script: ccc.Script,
  method: string
) {
  console.log(`Calling SSRI method: ${method}`);

  try {
    const result = await executor.runScript(
      codeOutPoint,
      method,
      [],
      { script }
    );

    console.log('Raw result:', result.res);
    console.log('Cell deps:', result.cellDeps);
    console.log('As UTF-8:', ccc.bytesTo(result.res, "utf8"));
  } catch (error) {
    if (error instanceof ssri.ExecutorErrorExecutionFailed) {
      console.error('SSRI execution failed:', error.message);
    } else if (error instanceof ssri.ExecutorErrorDecode) {
      console.error('Failed to decode response:', error.message);
    } else {
      console.error('Unknown error:', error);
    }
  }
}
```