## Description

Comprehensive guide to developing Digital Objects (DOBs) on CKB using Spore protocol extensions. Covers DOB/0 and DOB/1 implementations, pattern configuration, DNA generation, decoder selection, content storage strategies, and best practices for production deployment. Essential for creating composable, extensible digital assets with rich rendering capabilities.

## Introduction

Digital Objects (DOBs) extend the Spore protocol to create composable, extensible digital assets with rich rendering capabilities. This guide covers practical implementation patterns for DOB/0 and DOB/1 protocols.

## Core Concepts

### DOB Protocol Versions

**DOB/0**: Basic trait-based rendering with pattern templates
**DOB/1**: Advanced composable rendering with background/icon combinations

### Key Components

- **Cluster**: Container for DOB configuration and patterns
- **Pattern**: Defines how DNA translates to visual traits
- **DNA**: Random seed that determines trait generation
- **Decoder**: Script that renders DOB based on DNA and patterns

## DOB/0 Implementation

### Pattern Configuration

```typescript
const dob0Pattern: ccc.spore.dob.PatternElementDob0[] = [
  {
    traitName: "BackgroundColor",
    dobType: "String",
    dnaOffset: 0,
    dnaLength: 1,
    patternType: "options",
    traitArgs: ["red", "blue", "green", "black", "white"],
  },
  {
    traitName: "Type",
    dobType: "Number", 
    dnaOffset: 1,
    dnaLength: 1,
    patternType: "range",
    traitArgs: [10, 50],
  },
  {
    traitName: "Timestamp",
    dobType: "Number",
    dnaOffset: 2,
    dnaLength: 4,
    patternType: "rawNumber",
  },
];
```

### Cluster Creation

```typescript
function generateClusterDescriptionUnderDobProtocol() {
  const clusterDescription = "A simple loot cluster";
  
  const dob0: ccc.spore.dob.Dob0 = {
    description: clusterDescription,
    dob: {
      ver: 0,
      decoder: ccc.spore.dob.getDecoder(client, "dob0"),
      pattern: dob0Pattern,
    },
  };

  return ccc.spore.dob.encodeClusterDescriptionForDob0(dob0);
}

// Create cluster
const { tx: clusterTx, id: clusterId } = await ccc.spore.createSporeCluster({
  signer,
  data: {
    name: "Simple loot", 
    description: generateClusterDescriptionUnderDobProtocol(),
  },
});
```

### DOB Minting

```typescript
function generateSimpleDNA(length: number): string {
  return Array.from(
    { length }, 
    () => Math.floor(Math.random() * 16).toString(16)
  ).join('');
}

// Create spore with DOB content
const { tx: sporeTx, id: sporeId } = await ccc.spore.createSpore({
  signer,
  data: {
    contentType: "dob/0",
    content: ccc.bytesFrom(`{ "dna": "${generateSimpleDNA(16)}" }`, "utf8"),
    clusterId: clusterId,
  },
  clusterMode: "clusterCell",
});
```

## Pattern Types

### Options Pattern
```typescript
{
  traitName: "BackgroundColor",
  dobType: "String",
  dnaOffset: 0,
  dnaLength: 1,
  patternType: "options",
  traitArgs: ["red", "blue", "green", "black", "white"],
}
```

### Range Pattern  
```typescript
{
  traitName: "Level",
  dobType: "Number",
  dnaOffset: 1,
  dnaLength: 1,
  patternType: "range",
  traitArgs: [1, 100], // Min and max values
}
```

### Raw Number Pattern
```typescript
{
  traitName: "Timestamp", 
  dobType: "Number",
  dnaOffset: 2,
  dnaLength: 4,
  patternType: "rawNumber", // Direct numeric value from DNA
}
```

### Raw String Pattern
```typescript
{
  traitName: "Signature",
  dobType: "String", 
  dnaOffset: 6,
  dnaLength: 8,
  patternType: "rawString", // Direct hex string from DNA
}
```

## Content Storage Strategies

### BTCFS Storage (Recommended)
```typescript
// For decentralized, permanent storage
const btcfsUrl = "btcfs://i0/abc123..."; // Content uploaded to BTCFS
```

### IPFS Storage
```typescript
// For decentralized storage with pinning
const ipfsUrl = "ipfs://QmXyz..."; // Ensure content is pinned
```

### Regular HTTP Links
```typescript
// For development/testing only
const httpUrl = "https://example.com/image.png"; // Not recommended for production
```

## DNA Generation Best Practices

### Deterministic DNA
```typescript
// Generate DNA from transaction context for uniqueness
function generateDeterministicDNA(txHash: string, outputIndex: number): string {
  const combined = txHash + outputIndex.toString();
  const hash = blake2b(combined);
  return hash.slice(0, 32); // 16 bytes = 32 hex chars
}
```

### Pseudo-Random DNA  
```typescript
// For testing and development
function generatePseudoRandomDNA(length: number): string {
  return Array.from(
    { length },
    () => Math.floor(Math.random() * 16).toString(16)
  ).join('');
}
```

### Weighted DNA Generation
```typescript
// Generate DNA with specific trait distributions
function generateWeightedDNA(traitWeights: Record<string, number[]>): string {
  let dna = "";
  let offset = 0;
  
  for (const [trait, weights] of Object.entries(traitWeights)) {
    const random = Math.random();
    let selected = 0;
    let cumulative = 0;
    
    for (let i = 0; i < weights.length; i++) {
      cumulative += weights[i];
      if (random <= cumulative) {
        selected = i;
        break;
      }
    }
    
    dna += selected.toString(16);
    offset++;
  }
  
  return dna.padEnd(32, '0'); // Pad to required length
}
```

## Production Deployment Workflow

### Phase 1: MVP Validation
1. **Design patterns** with temporary HTTP links
2. **Test rendering** on testnet across platforms (JoyID, Omiga, etc.)
3. **Use CCC Playground** for rapid iteration
4. **Validate compatibility** before proceeding

### Phase 2: Integration Testing
1. **Upload media to BTCFS/IPFS** at least 1 week before launch
2. **Update cluster configuration** with permanent links  
3. **Test DNA generation** and trait distributions
4. **Verify rendering quality** across all platforms

### Phase 3: Pre-Production
1. **Create test cluster on mainnet** with production media
2. **Use distinctive names** (e.g., "⚠️ Test Collection")
3. **Deploy application** with restricted access
4. **Test complete workflow** end-to-end

### Phase 4: Production Launch
1. **Prepare emergency response** plans
2. **Launch mid-week** for better monitoring
3. **Monitor error logs** and platform compatibility
4. **Respond to user feedback** proactively

## Advanced Patterns

### Conditional Trait Generation
```typescript
// Generate traits based on other trait values
function generateConditionalTraits(dna: string): any {
  const baseType = parseInt(dna[0], 16) % 3; // 0, 1, or 2
  
  if (baseType === 0) {
    // Fire type gets fire-related traits
    return {
      element: "Fire",
      color: ["red", "orange", "yellow"][parseInt(dna[1], 16) % 3],
      power: 80 + (parseInt(dna[2], 16) % 20)
    };
  } else if (baseType === 1) {
    // Water type gets water-related traits  
    return {
      element: "Water",
      color: ["blue", "cyan", "turquoise"][parseInt(dna[1], 16) % 3],
      power: 70 + (parseInt(dna[2], 16) % 30)
    };
  }
  // Earth type...
}
```

### Multi-Layer Composition (DOB/1)
```typescript
// DOB/1 supports background + icon composition
const dob1Pattern = {
  backgroundLayer: {
    type: "btcfs",
    variations: ["bg1.png", "bg2.png", "bg3.png"]
  },
  iconLayer: {
    type: "svg", 
    variations: ["icon1.svg", "icon2.svg", "icon3.svg"]
  }
};
```

## Error Handling

### Common Issues
```typescript
// Validate pattern configuration
function validatePattern(pattern: PatternElementDob0[]): void {
  let totalLength = 0;
  
  for (const element of pattern) {
    if (element.dnaOffset + element.dnaLength > 32) {
      throw new Error(`Pattern ${element.traitName} exceeds DNA length`);
    }
    totalLength = Math.max(totalLength, element.dnaOffset + element.dnaLength);
  }
  
  if (totalLength > 32) {
    throw new Error("Total pattern length exceeds 32 bytes");
  }
}

// Validate DNA format
function validateDNA(dna: string): void {
  if (!/^[0-9a-fA-F]+$/.test(dna)) {
    throw new Error("DNA must be valid hexadecimal");
  }
  
  if (dna.length % 2 !== 0) {
    throw new Error("DNA length must be even");
  }
}
```

## Platform Compatibility

### Supported Platforms
- **JoyID**: Full DOB/0 and DOB/1 support
- **Omiga**: Full DOB/0 and DOB/1 support  
- **CKB Explorer**: Full DOB/0 and DOB/1 support
- **Mobit**: Full DOB/0 and DOB/1 support
- **Dobby**: Full DOB/0 and DOB/1 support

### Compatibility Testing
```typescript
// Test DOB rendering across platforms
async function testPlatformCompatibility(sporeId: string) {
  const platforms = [
    `https://testnet.joyid.dev/nft/${sporeId}`,
    `https://test.omiga.io/info/dobs/${sporeTypeHash}`,
    `https://testnet.explorer.nervos.org/nft-info/${clusterTypeHash}/${sporeId}`,
    `https://mobit.app/dob/${sporeId}?chain=ckb`,
    `https://test-dobby.entrust3.com/item-detail_ckb/${sporeId}`
  ];
  
  console.log("Test rendering on these platforms:");
  platforms.forEach(url => console.log(url));
}
```

## Resources

- **DOB/0 Decoder**: Universal decoder for basic patterns  
- **DOB/1 Decoder**: Advanced decoder for composition
- **CCC Playground**: https://live.ckbccc.com/ - Online development environment
- **BTCFS Upload**: For decentralized content storage
- **Spore Documentation**: https://docs.spore.pro