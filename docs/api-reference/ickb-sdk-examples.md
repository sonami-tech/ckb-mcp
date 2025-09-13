## Description

iCKB SDK reference with TypeScript examples for CKB-to-iCKB conversions, order management, and liquidity operations. Core methods like getL1State, estimate, request, and maturity calculations. Practical patterns for balance queries, transaction monitoring, price tracking, error handling, and advanced trading strategies with the iCKB protocol.

## Related Resources

- [ckb-dev-context://protocols/ickb-protocol](ckb-dev-context://protocols/ickb-protocol) - Revolutionary liquidity protocol tokenizing NervosDAO deposits
- [ckb-dev-context://patterns/ickb-development](ckb-dev-context://patterns/ickb-development) - Build applications with iCKB liquid staking protocol for enhanced CKB yield
- [ckb-dev-context://patterns/ickb-liquidity-patterns](ckb-dev-context://patterns/ickb-liquidity-patterns) - Advanced iCKB liquidity management with automated rebalancing algorithms
- [ckb-dev-context://troubleshooting/ickb-debugging](ckb-dev-context://troubleshooting/ickb-debugging) - Specialized debugging guide for iCKB protocol development

Examples of using the iCKB SDK for building applications with the iCKB protocol on CKB.

## Installation and Setup

### NPM Installation

```bash
# Core SDK and dependencies
npm install @ickb/sdk @ckb-ccc/core

# Optional utilities
npm install @ickb/utils @ickb/dao @ickb/order @ickb/core
```

### Basic Configuration

```typescript
import { ccc } from "@ckb-ccc/core";
import { IckbSdk } from "@ickb/sdk";

// Initialize CCC client
const client = new ccc.ClientPublicMainnet({
  url: "https://mainnet.ckbapp.dev/rpc",
});

// For testnet
const testnetClient = new ccc.ClientPublicTestnet({
  url: "https://testnet.ckbapp.dev/rpc",
});

// Initialize iCKB SDK
const sdk = new IckbSdk();
```

### Wallet Integration

```typescript
// Connect wallet signer
const signer = new ccc.SignerCkbPrivateKey(client, privateKey);

// Or use browser wallet
const signer = new ccc.SignerCkbPublicKey(client, publicKey);
```

## Core SDK Methods

### getL1State

Retrieves the current protocol state and user positions.

#### Parameters

```typescript
async getL1State(
  client: ccc.Client,
  locks: ccc.Script[]
): Promise<{
  system: SystemState;
  user: { orders: OrderGroup[] };
}>
```

#### Example

```typescript
// Get protocol state
const userLocks = [await signer.getRecommendedAddressObj()];
const { system, user } = await sdk.getL1State(client, userLocks);

console.log("System State:");
console.log(`Pool CKB: ${system.poolCkb}`);
console.log(`Pool iCKB: ${system.poolUdt}`);
console.log(`CKB to iCKB rate: ${system.info.ckbToUdt.numerator}/${system.info.ckbToUdt.denominator}`);
console.log(`iCKB to CKB rate: ${system.info.udtToCkb.numerator}/${system.info.udtToCkb.denominator}`);

console.log("User Orders:");
user.orders.forEach((orderGroup, index) => {
  console.log(`Order Group ${index}:`, orderGroup);
});
```

### estimate

Estimates conversion amounts and fees for CKB ↔ iCKB operations.

#### Parameters

```typescript
static estimate(
  isCkb2Udt: boolean,
  amounts: ValueComponents,
  system: SystemState
): EstimateResult
```

#### Examples

```typescript
// Estimate CKB to iCKB conversion
const ckbToUdtEstimate = IckbSdk.estimate(
  true, // CKB to iCKB
  { ckb: "100000000000" }, // 1000 CKB
  system
);

console.log("CKB to iCKB Estimate:");
console.log(`Input: ${ckbToUdtEstimate.input.ckb} CKB`);
console.log(`Output: ${ckbToUdtEstimate.output.udt} iCKB`);
console.log(`Fee: ${ckbToUdtEstimate.fee} CKB`);
console.log(`Exchange Rate: ${ckbToUdtEstimate.ratio}`);

// Estimate iCKB to CKB conversion
const udtToCkbEstimate = IckbSdk.estimate(
  false, // iCKB to CKB
  { udt: "100000000000000000" }, // 100 iCKB
  system
);

console.log("iCKB to CKB Estimate:");
console.log(`Input: ${udtToCkbEstimate.input.udt} iCKB`);
console.log(`Output: ${udtToCkbEstimate.output.ckb} CKB`);
console.log(`Fee: ${udtToCkbEstimate.fee} CKB`);
```

### request

Creates a limit order for conversion between CKB and iCKB.

#### Parameters

```typescript
async request(
  tx: ccc.SmartTransaction,
  user: ccc.Signer | ccc.Script,
  info: Info,
  amounts: ValueComponents
): Promise<void>
```

#### Examples

```typescript
// Create CKB to iCKB order
async function createCkbToIckbOrder(
  signer: ccc.Signer,
  ckbAmount: string
): Promise<string> {
  // Create transaction
  const tx = ccc.Transaction.from({});
  const smartTx = new ccc.SmartTransaction(tx);
  
  // Get current system state
  const { system } = await sdk.getL1State(client, [await signer.getRecommendedAddressObj()]);
  
  // Create order
  await sdk.request(
    smartTx,
    signer,
    system.info,
    { ckb: ckbAmount }
  );
  
  // Complete and send transaction
  await smartTx.completeFeeBy(signer);
  const signedTx = await signer.signTransaction(smartTx.tx);
  const txHash = await client.sendTransaction(signedTx);
  
  return txHash;
}

// Create iCKB to CKB order
async function createIckbToCkbOrder(
  signer: ccc.Signer,
  udtAmount: string
): Promise<string> {
  const tx = ccc.Transaction.from({});
  const smartTx = new ccc.SmartTransaction(tx);
  
  const { system } = await sdk.getL1State(client, [await signer.getRecommendedAddressObj()]);
  
  await sdk.request(
    smartTx,
    signer,
    system.info,
    { udt: udtAmount }
  );
  
  await smartTx.completeFeeBy(signer);
  const signedTx = await signer.signTransaction(smartTx.tx);
  return await client.sendTransaction(signedTx);
}
```

### maturity

Calculates when an order will be ready for claiming.

#### Parameters

```typescript
static maturity(
  order: OrderCell,
  system: SystemState
): bigint | undefined
```

#### Example

```typescript
// Check order maturity
const { user } = await sdk.getL1State(client, userLocks);

user.orders.forEach((orderGroup) => {
  orderGroup.orders.forEach((order) => {
    const maturityTime = IckbSdk.maturity(order, system);
    
    if (maturityTime) {
      const maturityDate = new Date(Number(maturityTime));
      const isReady = Date.now() > Number(maturityTime);
      
      console.log(`Order: ${order.outPoint.txHash}`);
      console.log(`Maturity: ${maturityDate.toISOString()}`);
      console.log(`Ready: ${isReady}`);
      
      if (isReady) {
        console.log("Order can be claimed!");
      }
    } else {
      console.log("Order maturity not available");
    }
  });
});
```

## Utility Functions

### Balance Queries

```typescript
// Get CKB balance
async function getCkbBalance(
  client: ccc.Client,
  locks: ccc.Script[]
): Promise<bigint> {
  let total = BigInt(0);
  
  for (const lock of locks) {
    const cells = client.findCellsByLock(lock, null, true);
    
    for await (const cell of cells) {
      total += BigInt(cell.capacity);
    }
  }
  
  return total;
}

// Get iCKB token balance
async function getIckbBalance(
  client: ccc.Client,
  locks: ccc.Script[]
): Promise<bigint> {
  let total = BigInt(0);
  const ickbType = getIckbTokenType(); // Helper to get iCKB type script
  
  for (const lock of locks) {
    const cells = client.findCellsByLock(lock, ickbType, true);
    
    for await (const cell of cells) {
      const amount = ccc.numFromBytes(cell.outputData, 16);
      total += amount;
    }
  }
  
  return total;
}

// Helper function to get iCKB token type script
function getIckbTokenType(): ccc.Script {
  return {
    codeHash: "0x00000000000000000000000000000000000000000000000000545950455f4944",
    hashType: "type",
    args: "0x...", // iCKB type script args
  };
}
```

### Transaction Monitoring

```typescript
// Wait for transaction confirmation
async function waitForTransaction(
  client: ccc.Client,
  txHash: string,
  timeout = 300000 // 5 minutes
): Promise<ccc.Transaction> {
  const startTime = Date.now();
  
  while (Date.now() - startTime < timeout) {
    try {
      const tx = await client.getTransaction(txHash);
      if (tx && tx.status === "committed") {
        return tx.transaction;
      }
    } catch (error) {
      // Transaction not found yet, continue waiting
    }
    
    await new Promise(resolve => setTimeout(resolve, 5000)); // Wait 5 seconds
  }
  
  throw new Error(`Transaction ${txHash} not confirmed within timeout`);
}

// Monitor order status
async function monitorOrderStatus(
  orderId: string,
  callback: (status: OrderStatus) => void
): Promise<void> {
  const checkInterval = 30000; // 30 seconds
  
  while (true) {
    try {
      const { system, user } = await sdk.getL1State(client, userLocks);
      
      // Find the specific order
      const order = user.orders
        .flatMap(group => group.orders)
        .find(o => o.outPoint.txHash === orderId);
      
      if (!order) {
        callback({ status: "not_found" });
        break;
      }
      
      const maturityTime = IckbSdk.maturity(order, system);
      const isReady = maturityTime && Date.now() > Number(maturityTime);
      
      callback({
        status: isReady ? "ready" : "pending",
        maturityTime: maturityTime ? Number(maturityTime) : undefined,
        order,
      });
      
      if (isReady) break;
      
    } catch (error) {
      callback({ status: "error", error: error.message });
    }
    
    await new Promise(resolve => setTimeout(resolve, checkInterval));
  }
}

interface OrderStatus {
  status: "pending" | "ready" | "not_found" | "error";
  maturityTime?: number;
  order?: OrderCell;
  error?: string;
}
```

## Advanced Usage Patterns

### Batch Operations

```typescript
// Create multiple orders in sequence
async function createBatchOrders(
  signer: ccc.Signer,
  orders: Array<{ isCkb2Udt: boolean; amount: string }>
): Promise<string[]> {
  const txHashes: string[] = [];
  
  for (const orderSpec of orders) {
    try {
      let txHash: string;
      
      if (orderSpec.isCkb2Udt) {
        txHash = await createCkbToIckbOrder(signer, orderSpec.amount);
      } else {
        txHash = await createIckbToCkbOrder(signer, orderSpec.amount);
      }
      
      txHashes.push(txHash);
      
      // Wait a bit between orders to avoid nonce conflicts
      await new Promise(resolve => setTimeout(resolve, 2000));
      
    } catch (error) {
      console.error(`Failed to create order:`, error);
      throw error;
    }
  }
  
  return txHashes;
}
```

### Price Monitoring

```typescript
// Monitor exchange rate changes
class IckbPriceMonitor {
  private lastRates: { ckbToUdt?: number; udtToCkb?: number } = {};
  private callbacks: ((rates: PriceUpdate) => void)[] = [];
  
  constructor(private updateInterval = 60000) {} // 1 minute default
  
  onPriceUpdate(callback: (rates: PriceUpdate) => void): void {
    this.callbacks.push(callback);
  }
  
  async start(): Promise<void> {
    while (true) {
      try {
        const { system } = await sdk.getL1State(client, []);
        
        const ckbToUdt = Number(system.info.ckbToUdt.numerator) / Number(system.info.ckbToUdt.denominator);
        const udtToCkb = Number(system.info.udtToCkb.numerator) / Number(system.info.udtToCkb.denominator);
        
        const priceUpdate: PriceUpdate = {
          ckbToUdt,
          udtToCkb,
          spread: (ckbToUdt * udtToCkb) - 1,
          timestamp: Date.now(),
          poolCkb: system.poolCkb,
          poolUdt: system.poolUdt,
        };
        
        // Check for significant changes
        const significantChange = 
          !this.lastRates.ckbToUdt ||
          Math.abs(ckbToUdt - this.lastRates.ckbToUdt) / this.lastRates.ckbToUdt > 0.001 ||
          Math.abs(udtToCkb - this.lastRates.udtToCkb!) / this.lastRates.udtToCkb! > 0.001;
        
        if (significantChange) {
          this.callbacks.forEach(callback => callback(priceUpdate));
          this.lastRates = { ckbToUdt, udtToCkb };
        }
        
      } catch (error) {
        console.error("Price monitoring error:", error);
      }
      
      await new Promise(resolve => setTimeout(resolve, this.updateInterval));
    }
  }
}

interface PriceUpdate {
  ckbToUdt: number;
  udtToCkb: number;
  spread: number;
  timestamp: number;
  poolCkb: string;
  poolUdt: string;
}
```

### Liquidity Analysis

```typescript
// Analyze pool liquidity and efficiency
class LiquidityAnalyzer {
  async analyzePoolState(system: SystemState): Promise<PoolAnalysis> {
    const poolCkb = BigInt(system.poolCkb);
    const poolUdt = BigInt(system.poolUdt);
    
    // Calculate pool utilization
    const totalLiquidity = poolCkb + this.convertUdtToCkb(poolUdt, system);
    const ckbRatio = Number(poolCkb) / Number(totalLiquidity);
    const udtRatio = Number(this.convertUdtToCkb(poolUdt, system)) / Number(totalLiquidity);
    
    // Calculate efficiency metrics
    const ckbToUdtRate = Number(system.info.ckbToUdt.numerator) / Number(system.info.ckbToUdt.denominator);
    const udtToCkbRate = Number(system.info.udtToCkb.numerator) / Number(system.info.udtToCkb.denominator);
    const spread = (ckbToUdtRate * udtToCkbRate) - 1;
    
    // Determine optimal trade sizes
    const optimalCkbTrade = this.calculateOptimalTradeSize(poolCkb, true);
    const optimalUdtTrade = this.calculateOptimalTradeSize(poolUdt, false);
    
    return {
      poolCkb: system.poolCkb,
      poolUdt: system.poolUdt,
      totalLiquidity: totalLiquidity.toString(),
      ckbRatio,
      udtRatio,
      ckbToUdtRate,
      udtToCkbRate,
      spread,
      optimalCkbTrade: optimalCkbTrade.toString(),
      optimalUdtTrade: optimalUdtTrade.toString(),
      recommendation: this.getRecommendation(ckbRatio, spread),
    };
  }
  
  private convertUdtToCkb(udtAmount: bigint, system: SystemState): bigint {
    const rate = BigInt(system.info.udtToCkb.numerator) / BigInt(system.info.udtToCkb.denominator);
    return udtAmount * rate;
  }
  
  private calculateOptimalTradeSize(poolAmount: bigint, isCkb: boolean): bigint {
    // Simple heuristic: 5% of pool size
    return poolAmount / BigInt(20);
  }
  
  private getRecommendation(ckbRatio: number, spread: number): string {
    if (spread > 0.01) {
      return "High spread - arbitrage opportunity available";
    } else if (ckbRatio > 0.7) {
      return "Pool has excess CKB - good time to convert CKB to iCKB";
    } else if (ckbRatio < 0.3) {
      return "Pool has excess iCKB - good time to convert iCKB to CKB";
    } else {
      return "Pool is well balanced";
    }
  }
}

interface PoolAnalysis {
  poolCkb: string;
  poolUdt: string;
  totalLiquidity: string;
  ckbRatio: number;
  udtRatio: number;
  ckbToUdtRate: number;
  udtToCkbRate: number;
  spread: number;
  optimalCkbTrade: string;
  optimalUdtTrade: string;
  recommendation: string;
}
```

## Error Handling

### Common Error Patterns

```typescript
// Handle iCKB-specific errors
async function safeIckbOperation<T>(
  operation: () => Promise<T>
): Promise<T> {
  try {
    return await operation();
  } catch (error) {
    if (error.message.includes("insufficient capacity")) {
      throw new Error("Insufficient CKB balance for transaction");
    } else if (error.message.includes("pool depleted")) {
      throw new Error("Pool liquidity insufficient - try smaller amount or wait for rebalancing");
    } else if (error.message.includes("slippage")) {
      throw new Error("Price moved unfavorably - retry with adjusted parameters");
    } else if (error.message.includes("maturity")) {
      throw new Error("Order not yet mature - please wait");
    } else {
      throw new Error(`iCKB operation failed: ${error.message}`);
    }
  }
}

// Retry with exponential backoff
async function retryIckbOperation<T>(
  operation: () => Promise<T>,
  maxRetries = 3,
  baseDelay = 1000
): Promise<T> {
  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      return await safeIckbOperation(operation);
    } catch (error) {
      if (attempt === maxRetries) {
        throw error;
      }
      
      const delay = baseDelay * Math.pow(2, attempt - 1);
      console.warn(`Attempt ${attempt} failed, retrying in ${delay}ms:`, error.message);
      await new Promise(resolve => setTimeout(resolve, delay));
    }
  }
  
  throw new Error("Unexpected error in retry logic");
}
```

## TypeScript Types

### Core Types

```typescript
// System state interface
interface SystemState {
  poolCkb: string;
  poolUdt: string;
  info: Info;
  daoAr: bigint;
  tipEpoch: {
    number: bigint;
    index: bigint;
    length: bigint;
  };
}

// Order info interface
interface Info {
  ckbToUdt: {
    numerator: bigint;
    denominator: bigint;
  };
  udtToCkb: {
    numerator: bigint;
    denominator: bigint;
  };
  ckbMinMatchLog: number;
}

// Value components for operations
interface ValueComponents {
  ckb?: string;
  udt?: string;
}

// Order cell interface
interface OrderCell {
  outPoint: {
    txHash: string;
    index: string;
  };
  output: ccc.CellOutput;
  outputData: string;
}

// Order group interface
interface OrderGroup {
  orders: OrderCell[];
  totalCkb: string;
  totalUdt: string;
}

// Estimate result interface
interface EstimateResult {
  input: ValueComponents;
  output: ValueComponents;
  fee: string;
  ratio: number;
  minOutput: ValueComponents;
  slippage: number;
}
```

This comprehensive API reference provides all the tools needed to integrate iCKB functionality into CKB applications, from basic conversions to advanced trading and monitoring systems.