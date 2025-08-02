# iCKB Development Patterns

## Description

Build applications with iCKB liquid staking protocol for enhanced CKB yield and liquidity. Learn system state management, conversion estimation, order lifecycle management, automated trading strategies, and portfolio rebalancing. Covers advanced patterns for liquidity provision, error handling, testing strategies, and production deployment of iCKB-integrated applications.

This guide covers development patterns and best practices for building applications with the iCKB protocol on CKB.

## Development Setup

### Environment Configuration

```typescript
import { ccc } from "@ckb-ccc/core";
import { IckbSdk } from "@ickb/sdk";

// Initialize CCC client
const client = new ccc.ClientPublicMainnet(); // or ClientPublicTestnet

// Initialize iCKB SDK
const sdk = new IckbSdk();
```

### Dependencies

```bash
# Core dependencies
npm install @ckb-ccc/core @ickb/sdk

# Additional utilities
npm install @ickb/utils @ickb/dao @ickb/order
```

## Core Patterns

### 1. System State Management

```typescript
// Get current protocol state
async function getSystemState(client: ccc.Client, userLocks: ccc.Script[]) {
  const { system, user } = await sdk.getL1State(client, userLocks);
  
  return {
    // Pool liquidity
    poolCkb: system.poolCkb,
    poolUdt: system.poolUdt,
    
    // Exchange rates
    ckbToUdt: system.info.ckbToUdt,
    udtToCkb: system.info.udtToCkb,
    
    // User positions
    userOrders: user.orders,
    
    // NervosDAO state
    daoAr: system.daoAr,
    tipEpoch: system.tipEpoch,
  };
}
```

### 2. Conversion Estimation

```typescript
// Estimate conversion amounts and fees
function estimateConversion(
  isCkb2Udt: boolean,
  inputAmount: string,
  systemState: SystemState
) {
  const amounts = isCkb2Udt 
    ? { ckb: inputAmount }
    : { udt: inputAmount };
    
  const estimate = IckbSdk.estimate(isCkb2Udt, amounts, systemState);
  
  return {
    inputAmount: estimate.input,
    outputAmount: estimate.output,
    exchangeRate: estimate.ratio,
    minimumOutput: estimate.minOutput,
    fees: estimate.fee,
    slippage: estimate.slippage,
  };
}
```

### 3. Order Creation Pattern

```typescript
// Create limit order for CKB to iCKB conversion
async function createCkbToUdtOrder(
  sdk: IckbSdk,
  signer: ccc.Signer,
  ckbAmount: string,
  minUdtAmount?: string
) {
  // Create transaction
  const tx = ccc.Transaction.from({});
  const smartTx = new ccc.SmartTransaction(tx);
  
  // Get current system state
  const { system } = await sdk.getL1State(client, [await signer.getRecommendedAddressObj()]);
  
  // Prepare order info
  const orderInfo = {
    ckbToUdt: system.info.ckbToUdt,
    udtToCkb: system.info.udtToCkb,
    ckbMinMatchLog: 20, // Minimum 1 CKB matches
  };
  
  // Create order
  await sdk.request(
    smartTx,
    signer,
    orderInfo,
    { ckb: ckbAmount }
  );
  
  // Sign and send
  await smartTx.completeFeeBy(signer);
  const signedTx = await signer.signTransaction(smartTx.tx);
  const txHash = await client.sendTransaction(signedTx);
  
  return txHash;
}
```

### 4. Order Monitoring

```typescript
// Monitor order status and maturity
class OrderMonitor {
  private orders: Map<string, OrderCell> = new Map();
  
  async trackOrder(orderId: string, orderCell: OrderCell) {
    this.orders.set(orderId, orderCell);
    await this.checkOrderStatus(orderId);
  }
  
  async checkOrderStatus(orderId: string) {
    const order = this.orders.get(orderId);
    if (!order) return;
    
    const { system } = await sdk.getL1State(client, []);
    const maturityTime = IckbSdk.maturity(order, system);
    
    if (maturityTime && Date.now() > maturityTime) {
      console.log(`Order ${orderId} is ready for claiming`);
      await this.claimOrder(orderId);
    }
  }
  
  async claimOrder(orderId: string) {
    // Implementation for order claiming
    const order = this.orders.get(orderId);
    if (!order) return;
    
    // Create claim transaction
    const tx = ccc.Transaction.from({});
    const smartTx = new ccc.SmartTransaction(tx);
    
    // Add order cell as input
    smartTx.addInput(order.outPoint, order.output);
    
    // Add appropriate outputs based on order type
    // ... claim logic implementation
    
    return smartTx;
  }
}
```

## Advanced Patterns

### 1. Liquidity Pool Integration

```typescript
// Check pool liquidity before operations
async function checkPoolLiquidity(
  requestedAmount: string,
  isCkb2Udt: boolean,
  systemState: SystemState
): Promise<{
  hasLiquidity: boolean;
  availableAmount: string;
  estimatedDelay?: number;
}> {
  const poolAmount = isCkb2Udt ? systemState.poolCkb : systemState.poolUdt;
  const requested = BigInt(requestedAmount);
  const available = BigInt(poolAmount);
  
  if (available >= requested) {
    return {
      hasLiquidity: true,
      availableAmount: requestedAmount,
    };
  }
  
  // Estimate delay based on bot rebalancing cycle
  const shortfall = requested - available;
  const estimatedDelay = Math.ceil(Number(shortfall) / 100000) * 60; // 60s per rebalancing cycle
  
  return {
    hasLiquidity: false,
    availableAmount: poolAmount,
    estimatedDelay,
  };
}
```

### 2. Two-Phase Deposit Management

```typescript
// Handle two-phase deposit process
class DepositManager {
  private pendingReceipts: Map<string, ReceiptData> = new Map();
  
  async initiateDeposit(
    signer: ccc.Signer,
    ckbAmount: string
  ): Promise<string> {
    // Phase 1: Create deposit receipt
    const tx = ccc.Transaction.from({});
    const smartTx = new ccc.SmartTransaction(tx);
    
    // Add CKB input
    await smartTx.addInputsByAmount(signer, ccc.fixedPointFrom(ckbAmount));
    
    // Create receipt output
    const receiptData: ReceiptData = {
      depositQuantity: Math.floor(Number(ckbAmount) / 100000), // 100k CKB per deposit
      depositAmount: BigInt(ckbAmount),
    };
    
    // Add receipt cell
    smartTx.addOutput({
      lock: await signer.getRecommendedAddressObj(),
      type: getIckbReceiptType(),
      data: encodeReceiptData(receiptData),
    }, ccc.fixedPointFrom("142")); // Base capacity
    
    await smartTx.completeFeeBy(signer);
    const signedTx = await signer.signTransaction(smartTx.tx);
    const txHash = await client.sendTransaction(signedTx);
    
    // Track receipt for phase 2
    this.pendingReceipts.set(txHash, receiptData);
    
    return txHash;
  }
  
  async completeDeposit(
    receiptTxHash: string,
    signer: ccc.Signer
  ): Promise<string> {
    // Phase 2: Convert receipt to iCKB tokens
    const receiptData = this.pendingReceipts.get(receiptTxHash);
    if (!receiptData) throw new Error("Receipt not found");
    
    // Wait for next block to get updated AR
    await this.waitForNextBlock();
    
    // Get current AR for token calculation
    const { system } = await sdk.getL1State(client, []);
    const tokenAmount = this.calculateTokenAmount(receiptData, system.daoAr);
    
    // Create conversion transaction
    const tx = ccc.Transaction.from({});
    const smartTx = new ccc.SmartTransaction(tx);
    
    // Add receipt as input
    const receiptOutPoint = { txHash: receiptTxHash, index: "0x0" };
    smartTx.addInput(receiptOutPoint, await client.getCell(receiptOutPoint));
    
    // Add iCKB token output
    smartTx.addOutput({
      lock: await signer.getRecommendedAddressObj(),
      type: getIckbTokenType(),
      data: ccc.numLeToBytes(tokenAmount, 16),
    }, ccc.fixedPointFrom("142"));
    
    await smartTx.completeFeeBy(signer);
    const signedTx = await signer.signTransaction(smartTx.tx);
    const txHash = await client.sendTransaction(signedTx);
    
    // Clean up tracking
    this.pendingReceipts.delete(receiptTxHash);
    
    return txHash;
  }
  
  private calculateTokenAmount(
    receiptData: ReceiptData,
    currentAr: bigint
  ): bigint {
    // Token amount = deposit amount * (current AR / initial AR)
    const initialAr = BigInt("1000000000000000000"); // AR when deposit was made
    return receiptData.depositAmount * currentAr / initialAr;
  }
  
  private async waitForNextBlock(): Promise<void> {
    const currentTip = await client.getTipHeader();
    const currentNumber = currentTip.number;
    
    // Poll for next block
    while (true) {
      await new Promise(resolve => setTimeout(resolve, 5000));
      const newTip = await client.getTipHeader();
      if (newTip.number > currentNumber) break;
    }
  }
}
```

### 3. Automated Trading Bot

```typescript
// Simple arbitrage bot for iCKB/CKB price differences
class IckbArbitrageBot {
  private readonly client: ccc.Client;
  private readonly signer: ccc.Signer;
  private readonly minProfitMargin: number;
  
  constructor(client: ccc.Client, signer: ccc.Signer, minProfit = 0.005) {
    this.client = client;
    this.signer = signer;
    this.minProfitMargin = minProfit; // 0.5% minimum profit
  }
  
  async runArbitrageLoop() {
    while (true) {
      try {
        await this.checkArbitrageOpportunity();
        await new Promise(resolve => setTimeout(resolve, 30000)); // 30s intervals
      } catch (error) {
        console.error("Arbitrage error:", error);
        await new Promise(resolve => setTimeout(resolve, 60000)); // Wait longer on error
      }
    }
  }
  
  private async checkArbitrageOpportunity() {
    const { system } = await sdk.getL1State(this.client, []);
    
    // Calculate price difference
    const ckbToUdtRate = Number(system.info.ckbToUdt.numerator) / Number(system.info.ckbToUdt.denominator);
    const udtToCkbRate = Number(system.info.udtToCkb.numerator) / Number(system.info.udtToCkb.denominator);
    
    const priceDifference = (ckbToUdtRate * udtToCkbRate) - 1;
    
    if (Math.abs(priceDifference) > this.minProfitMargin) {
      if (priceDifference > 0) {
        // Profit from CKB → iCKB → CKB
        await this.executeCkbToUdtToCkbArbitrage(system);
      } else {
        // Profit from iCKB → CKB → iCKB  
        await this.executeUdtToCkbToUdtArbitrage(system);
      }
    }
  }
  
  private async executeCkbToUdtToCkbArbitrage(system: SystemState) {
    const arbitrageAmount = "100000000000"; // 1000 CKB
    
    // Step 1: CKB → iCKB
    const ckbToUdtTx = await createCkbToUdtOrder(
      this.sdk,
      this.signer,
      arbitrageAmount
    );
    
    // Wait for completion and get iCKB amount
    const iCkbAmount = await this.waitForOrderCompletion(ckbToUdtTx);
    
    // Step 2: iCKB → CKB
    const udtToCkbTx = await this.createUdtToCkbOrder(
      this.signer,
      iCkbAmount
    );
    
    // Track profit
    const finalCkb = await this.waitForOrderCompletion(udtToCkbTx);
    const profit = BigInt(finalCkb) - BigInt(arbitrageAmount);
    
    console.log(`Arbitrage profit: ${profit} CKB`);
  }
  
  private async createUdtToCkbOrder(
    signer: ccc.Signer,
    udtAmount: string
  ): Promise<string> {
    const tx = ccc.Transaction.from({});
    const smartTx = new ccc.SmartTransaction(tx);
    
    const { system } = await sdk.getL1State(this.client, []);
    
    const orderInfo = {
      ckbToUdt: system.info.ckbToUdt,
      udtToCkb: system.info.udtToCkb,
      ckbMinMatchLog: 20,
    };
    
    await sdk.request(
      smartTx,
      signer,
      orderInfo,
      { udt: udtAmount }
    );
    
    await smartTx.completeFeeBy(signer);
    const signedTx = await signer.signTransaction(smartTx.tx);
    return await this.client.sendTransaction(signedTx);
  }
  
  private async waitForOrderCompletion(txHash: string): Promise<string> {
    // Implementation to wait for order completion and return output amount
    // This would involve monitoring the transaction and its outputs
    // Simplified for brevity
    return "0";
  }
}
```

### 4. Portfolio Management

```typescript
// Manage iCKB holdings with automatic rebalancing
class IckbPortfolioManager {
  private readonly targetRatio: number; // Target iCKB/CKB ratio
  private readonly rebalanceThreshold: number;
  
  constructor(targetRatio = 0.5, threshold = 0.1) {
    this.targetRatio = targetRatio;
    this.rebalanceThreshold = threshold;
  }
  
  async analyzePortfolio(
    client: ccc.Client,
    userLocks: ccc.Script[]
  ): Promise<PortfolioAnalysis> {
    const { system, user } = await sdk.getL1State(client, userLocks);
    
    // Calculate current holdings
    const ckbBalance = await this.getCkbBalance(client, userLocks);
    const iCkbBalance = await this.getIckbBalance(client, userLocks);
    
    const totalValue = ckbBalance + this.convertIckbToCkb(iCkbBalance, system);
    const currentRatio = this.convertIckbToCkb(iCkbBalance, system) / totalValue;
    
    const needsRebalancing = Math.abs(currentRatio - this.targetRatio) > this.rebalanceThreshold;
    
    return {
      ckbBalance,
      iCkbBalance,
      totalValue,
      currentRatio,
      targetRatio: this.targetRatio,
      needsRebalancing,
      recommendedAction: this.getRecommendedAction(currentRatio),
    };
  }
  
  async rebalancePortfolio(
    client: ccc.Client,
    signer: ccc.Signer
  ): Promise<string[]> {
    const analysis = await this.analyzePortfolio(client, [await signer.getRecommendedAddressObj()]);
    
    if (!analysis.needsRebalancing) {
      return [];
    }
    
    const transactions: string[] = [];
    
    if (analysis.currentRatio > this.targetRatio) {
      // Too much iCKB, convert some to CKB
      const excessIckb = (analysis.currentRatio - this.targetRatio) * analysis.totalValue;
      const tx = await this.createUdtToCkbOrder(signer, excessIckb.toString());
      transactions.push(tx);
    } else {
      // Too little iCKB, convert some CKB to iCKB
      const neededCkb = (this.targetRatio - analysis.currentRatio) * analysis.totalValue;
      const tx = await createCkbToUdtOrder(sdk, signer, neededCkb.toString());
      transactions.push(tx);
    }
    
    return transactions;
  }
  
  private getRecommendedAction(currentRatio: number): string {
    if (currentRatio > this.targetRatio + this.rebalanceThreshold) {
      return "Convert some iCKB to CKB";
    } else if (currentRatio < this.targetRatio - this.rebalanceThreshold) {
      return "Convert some CKB to iCKB";
    }
    return "Portfolio is balanced";
  }
  
  private convertIckbToCkb(iCkbAmount: bigint, system: SystemState): bigint {
    const rate = BigInt(system.info.udtToCkb.numerator) / BigInt(system.info.udtToCkb.denominator);
    return iCkbAmount * rate;
  }
}
```

## Error Handling Patterns

### 1. Comprehensive Error Types

```typescript
enum IckbErrorType {
  INSUFFICIENT_LIQUIDITY = "INSUFFICIENT_LIQUIDITY",
  INVALID_ORDER_AMOUNT = "INVALID_ORDER_AMOUNT",
  ORDER_EXPIRED = "ORDER_EXPIRED",
  SLIPPAGE_EXCEEDED = "SLIPPAGE_EXCEEDED",
  NETWORK_ERROR = "NETWORK_ERROR",
  CONTRACT_ERROR = "CONTRACT_ERROR",
}

class IckbError extends Error {
  constructor(
    public type: IckbErrorType,
    message: string,
    public details?: any
  ) {
    super(message);
    this.name = "IckbError";
  }
}
```

### 2. Robust Transaction Handling

```typescript
async function executeIckbTransaction<T>(
  operation: () => Promise<T>,
  retries = 3,
  delay = 1000
): Promise<T> {
  for (let attempt = 1; attempt <= retries; attempt++) {
    try {
      return await operation();
    } catch (error) {
      console.warn(`Attempt ${attempt} failed:`, error.message);
      
      if (attempt === retries) {
        throw new IckbError(
          IckbErrorType.NETWORK_ERROR,
          `Operation failed after ${retries} attempts`,
          error
        );
      }
      
      // Exponential backoff
      await new Promise(resolve => setTimeout(resolve, delay * Math.pow(2, attempt - 1)));
    }
  }
  
  throw new Error("Unexpected error in transaction execution");
}
```

### 3. Validation Helpers

```typescript
// Validate order parameters
function validateOrderParams(
  amount: string,
  systemState: SystemState,
  isCkb2Udt: boolean
): void {
  const amountBigInt = BigInt(amount);
  const minAmount = BigInt("100000000"); // 1 CKB minimum
  
  if (amountBigInt < minAmount) {
    throw new IckbError(
      IckbErrorType.INVALID_ORDER_AMOUNT,
      `Amount ${amount} is below minimum ${minAmount}`
    );
  }
  
  // Check pool liquidity
  const poolAmount = isCkb2Udt ? systemState.poolCkb : systemState.poolUdt;
  if (BigInt(poolAmount) < amountBigInt) {
    throw new IckbError(
      IckbErrorType.INSUFFICIENT_LIQUIDITY,
      `Requested ${amount}, pool has ${poolAmount}`
    );
  }
}
```

## Testing Patterns

### 1. Mock System State

```typescript
// Create mock system state for testing
function createMockSystemState(overrides: Partial<SystemState> = {}): SystemState {
  return {
    poolCkb: "1000000000000", // 10,000 CKB
    poolUdt: "10000000000000000", // 10,000 iCKB
    info: {
      ckbToUdt: { numerator: BigInt(1), denominator: BigInt(1) },
      udtToCkb: { numerator: BigInt(1), denominator: BigInt(1) },
      ckbMinMatchLog: 20,
    },
    daoAr: BigInt("1000000000000000000"),
    tipEpoch: { number: BigInt(2000), index: BigInt(0), length: BigInt(1800) },
    ...overrides,
  };
}
```

### 2. Integration Tests

```typescript
describe("iCKB Integration", () => {
  let client: ccc.Client;
  let signer: ccc.Signer;
  let sdk: IckbSdk;
  
  beforeEach(async () => {
    client = new ccc.ClientPublicTestnet();
    signer = new ccc.SignerCkbPrivateKey(client, testPrivateKey);
    sdk = new IckbSdk();
  });
  
  it("should complete CKB to iCKB conversion", async () => {
    const initialBalance = await getCkbBalance(client, [await signer.getRecommendedAddressObj()]);
    
    const txHash = await createCkbToUdtOrder(
      sdk,
      signer,
      "100000000000" // 1000 CKB
    );
    
    // Wait for transaction confirmation
    await waitForTransaction(client, txHash);
    
    // Verify iCKB tokens received
    const iCkbBalance = await getIckbBalance(client, [await signer.getRecommendedAddressObj()]);
    expect(iCkbBalance).toBeGreaterThan(0);
  });
});
```

This comprehensive guide provides the essential patterns for building applications with the iCKB protocol, from basic conversions to advanced trading strategies and portfolio management.