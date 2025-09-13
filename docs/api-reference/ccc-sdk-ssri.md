## Description

Guide to Script-Sourced Rich Information (SSRI) framework in the CCC SDK. Shows how smart contracts can provide metadata and advanced functions directly on-chain. Covers UDT token metadata, custom script functions, and dynamic information retrieval. Essential for developers building tokens and smart contracts that need to expose rich metadata without off-chain dependencies.

## Related Resources

- [ckb-dev-context://protocols/ssri](ckb-dev-context://protocols/ssri) - Extension protocol enabling CKB scripts to provide rich information through off-chain execution
- [ckb-dev-context://patterns/ssri-implementation](ckb-dev-context://patterns/ssri-implementation) - Implementation guide for Script-Sourced Rich Information in CKB smart contracts
- [ckb-dev-context://tools/ssri-server](ckb-dev-context://tools/ssri-server) - Comprehensive integration guide for SSRI server enabling off-chain CKB script execution

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

// Create UDT instance with SSRI support
const udt = new ccc.udt.Udt(
  tokenTypeScript,
  client
);

// Fetch token metadata via SSRI
const info = await udt.getInfo();
console.log({
  name: info.name,        // "My Token"
  symbol: info.symbol,    // "MTK"
  decimals: info.decimals // 8
});
```

### Icon Retrieval

```typescript
// Get token icon as data URI
const icon = await udt.getIcon();
if (icon) {
  // icon is a data URI that can be used directly in img tags
  // e.g., "data:image/svg+xml;base64,..."
  document.getElementById('token-icon').src = icon;
}
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

### Direct SSRI Calls

```typescript
// Execute arbitrary SSRI function
const result = await ccc.ssri.execute(
  client,
  {
    script: contractScript,
    functionId: 0x00000010, // SSRI_TOTAL_SUPPLY
    args: []
  }
);

const totalSupply = ccc.numFromBytes(result);
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

### Dynamic Pricing Information

```typescript
// SSRI function that returns current token price
const SSRI_CURRENT_PRICE = 0x00000020;

async function getTokenPrice(tokenScript: ccc.Script): Promise<bigint> {
  const result = await ccc.ssri.execute(client, {
    script: tokenScript,
    functionId: SSRI_CURRENT_PRICE,
    args: []
  });
  
  return ccc.numFromBytes(result);
}
```

### Governance Parameters

```typescript
// Fetch on-chain governance settings
const SSRI_VOTING_THRESHOLD = 0x00000030;
const SSRI_PROPOSAL_DURATION = 0x00000031;

async function getGovernanceParams(daoScript: ccc.Script) {
  const [threshold, duration] = await Promise.all([
    ccc.ssri.execute(client, {
      script: daoScript,
      functionId: SSRI_VOTING_THRESHOLD
    }),
    ccc.ssri.execute(client, {
      script: daoScript,
      functionId: SSRI_PROPOSAL_DURATION
    })
  ]);
  
  return {
    votingThreshold: ccc.numFromBytes(threshold),
    proposalDuration: ccc.numFromBytes(duration)
  };
}
```

### Access Control Information

```typescript
// Query role-based permissions
const SSRI_IS_ADMIN = 0x00000040;
const SSRI_IS_MINTER = 0x00000041;

async function checkPermissions(
  contractScript: ccc.Script,
  userLock: ccc.Script
): Promise<Permissions> {
  const args = userLock.hash();
  
  const [isAdmin, isMinter] = await Promise.all([
    ccc.ssri.execute(client, {
      script: contractScript,
      functionId: SSRI_IS_ADMIN,
      args
    }),
    ccc.ssri.execute(client, {
      script: contractScript,
      functionId: SSRI_IS_MINTER,
      args
    })
  ]);
  
  return {
    admin: isAdmin[0] === 1,
    minter: isMinter[0] === 1
  };
}
```

## Best Practices

### Error Handling

```typescript
async function safeGetTokenInfo(udt: ccc.udt.Udt) {
  try {
    return await udt.getInfo();
  } catch (error) {
    if (error.code === 'SSRI_NOT_SUPPORTED') {
      // Fallback to default values
      return {
        name: 'Unknown Token',
        symbol: 'UNK',
        decimals: 8
      };
    }
    throw error;
  }
}
```

### Performance Optimization

```typescript
// Batch SSRI calls for efficiency
async function batchSSRICalls(
  script: ccc.Script,
  functionIds: number[]
): Promise<any[]> {
  const calls = functionIds.map(id => ({
    script,
    functionId: id,
    args: []
  }));
  
  // Execute in parallel
  return Promise.all(
    calls.map(call => ccc.ssri.execute(client, call))
  );
}
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
async function displayTokenDetails(tokenAddress: string) {
  const script = ccc.Script.fromAddress(tokenAddress);
  const udt = new ccc.udt.Udt(script, client);
  
  // Fetch all metadata
  const [info, icon, totalSupply] = await Promise.all([
    udt.getInfo(),
    udt.getIcon(),
    ccc.ssri.execute(client, {
      script,
      functionId: 0x00000010 // TOTAL_SUPPLY
    })
  ]);
  
  // Display in UI
  updateUI({
    name: info.name,
    symbol: info.symbol,
    decimals: info.decimals,
    icon: icon || '/default-token-icon.svg',
    totalSupply: formatUnits(totalSupply, info.decimals)
  });
}
```

### DeFi Integration

```typescript
// Automatic token detection in DeFi app
async function addTokenToPool(tokenScript: ccc.Script) {
  const udt = new ccc.udt.Udt(tokenScript, client);
  
  // Verify token is SSRI-compatible
  let tokenInfo;
  try {
    tokenInfo = await udt.getInfo();
  } catch {
    throw new Error('Token must support SSRI for automatic listing');
  }
  
  // Add to liquidity pool
  await liquidityPool.addToken({
    script: tokenScript,
    ...tokenInfo
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
// Debug SSRI calls
async function debugSSRI(script: ccc.Script, functionId: number) {
  console.log(`Calling SSRI function ${functionId.toString(16)}`);
  
  try {
    const result = await ccc.ssri.execute(client, {
      script,
      functionId,
      args: []
    });
    
    console.log('Raw result:', result);
    console.log('Hex:', ccc.hexFrom(result));
    console.log('UTF-8:', new TextDecoder().decode(result));
  } catch (error) {
    console.error('SSRI execution failed:', error);
  }
}
```