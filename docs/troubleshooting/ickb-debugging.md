## Description

Specialized debugging guide for iCKB protocol development covering liquidity pool issues, conversion failures, order management problems, and integration troubleshooting. Provides diagnostic tools, error analysis techniques, and step-by-step solutions for common iCKB implementation challenges. Includes code examples for pool state monitoring, transaction debugging, slippage analysis, and SDK integration issues. Essential resource for developers building iCKB-based applications and resolving protocol-specific errors.

## Related Resources

- [ckb-dev-context://protocols/ickb-protocol](ckb-dev-context://protocols/ickb-protocol) - Revolutionary liquidity protocol tokenizing NervosDAO deposits
- [ckb-dev-context://patterns/ickb-development](ckb-dev-context://patterns/ickb-development) - Build applications with iCKB liquid staking protocol for enhanced CKB yield
- [ckb-dev-context://patterns/ickb-liquidity-patterns](ckb-dev-context://patterns/ickb-liquidity-patterns) - Advanced iCKB liquidity management with automated rebalancing algorithms
- [ckb-dev-context://api-reference/ickb-sdk-examples](ckb-dev-context://api-reference/ickb-sdk-examples) - iCKB SDK reference with TypeScript examples for conversions and order management

This guide covers common issues, debugging techniques, and troubleshooting patterns specific to the iCKB protocol implementation.

## Common Issues and Solutions

### 1. Pool Liquidity Issues

#### Insufficient Pool Liquidity
**Symptoms:**
- Conversion transactions fail with "pool depleted" error
- Long delays in order fulfillment
- High slippage on conversions

**Diagnosis:**
```typescript
async function diagnosePoolLiquidity(): Promise<LiquidityDiagnosis> {
  const { system } = await sdk.getL1State(client, []);
  
  const poolCkb = BigInt(system.poolCkb);
  const poolUdt = BigInt(system.poolUdt);
  const totalLiquidity = poolCkb + convertUdtToCkb(poolUdt, system);
  
  return {
    poolCkb: system.poolCkb,
    poolUdt: system.poolUdt,
    totalLiquidity: totalLiquidity.toString(),
    ckbRatio: Number(poolCkb) / Number(totalLiquidity),
    isHealthy: poolCkb > BigInt("100000000000") && poolUdt > BigInt("100000000000000"), // 1k CKB, 100 iCKB
    recommendation: poolCkb < BigInt("100000000000") 
      ? "Pool needs CKB liquidity - wait for bot rebalancing or try smaller amounts"
      : poolUdt < BigInt("100000000000000")
      ? "Pool needs iCKB liquidity - consider CKB to iCKB conversion"
      : "Pool is healthy"
  };
}
```

**Solutions:**
1. **Wait for bot rebalancing** (typically 60-second cycles)
2. **Reduce transaction amount** to fit available liquidity
3. **Check bot status** using debug endpoints
4. **Use two-phase process** for large amounts

#### Bot Rebalancing Delays
**Symptoms:**
- Pool remains imbalanced for extended periods
- Bot operations appear stuck
- Maturity estimates are incorrect

**Debug Bot Status:**
```typescript
async function debugBotStatus(): Promise<BotDiagnosis> {
  // Check bot capacity cells
  const botLocks = getBotLockScripts(); // From deployment config
  const botCapacities = await Promise.all(
    botLocks.map(async (lock) => {
      const cells = [];
      for await (const cell of client.findCellsByLock(lock, null, true)) {
        cells.push(cell);
      }
      return { lock, cells, totalCapacity: cells.reduce((sum, c) => sum + BigInt(c.capacity), 0n) };
    })
  );
  
  // Check bot withdrawal requests
  const withdrawalRequests = [];
  for (const botLock of botLocks) {
    const requests = await findWithdrawalRequests(client, botLock);
    withdrawalRequests.push(...requests);
  }
  
  return {
    botsActive: botCapacities.filter(b => b.totalCapacity > BigInt("130000000000000")).length,
    totalBotCapacity: botCapacities.reduce((sum, b) => sum + b.totalCapacity, 0n).toString(),
    pendingWithdrawals: withdrawalRequests.length,
    nextMaturityEpoch: Math.min(...withdrawalRequests.map(r => r.maturityEpoch)),
    lastRebalanceTime: await getLastRebalanceTime(botCapacities),
  };
}

async function getLastRebalanceTime(botCapacities: BotCapacity[]): Promise<number> {
  let latestTime = 0;
  
  for (const bot of botCapacities) {
    for (const cell of bot.cells) {
      try {
        const tx = await client.getTransaction(cell.outPoint.txHash);
        if (tx?.transaction.timestamp) {
          latestTime = Math.max(latestTime, Number(tx.transaction.timestamp));
        }
      } catch (error) {
        // Transaction might not be available
      }
    }
  }
  
  return latestTime;
}
```

### 2. Exchange Rate Calculation Errors

#### Incorrect AR (Accumulated Rate) Values
**Symptoms:**
- Token amounts don't match expected values
- Conversion rates seem wrong
- Receipt to token conversion fails

**Debug AR Calculation:**
```typescript
async function debugExchangeRate(): Promise<ExchangeRateDebug> {
  const tip = await client.getTipHeader();
  const genesisAR = BigInt("10000000000000000"); // 10^16
  const currentAR = BigInt(tip.dao.slice(8, 24), 16); // Extract AR from DAO field
  
  // Calculate exchange rate
  const exchangeRate = currentAR * BigInt(1e18) / genesisAR;
  
  // Verify against protocol calculation
  const protocolRate = ickbExchangeRatio(tip);
  
  return {
    tipBlockNumber: tip.number,
    tipEpoch: tip.epoch,
    genesisAR: genesisAR.toString(),
    currentAR: currentAR.toString(),
    calculatedRate: exchangeRate.toString(),
    protocolRate: protocolRate.toString(),
    rateMatch: exchangeRate === BigInt(protocolRate.numerator) / BigInt(protocolRate.denominator),
    expectedIckbValue: (BigInt("100000000000") * exchangeRate / BigInt(1e18)).toString() // 1000 CKB worth
  };
}
```

#### Standard Deposit Size Calculation
**Symptoms:**
- Deposits rejected as oversized or undersized
- Fee calculations incorrect
- Receipt data inconsistent

**Validate Deposit Size:**
```typescript
function validateDepositSize(
  ckbAmount: string,
  tipHeader: ccc.ClientBlockHeader
): DepositValidation {
  const amount = BigInt(ckbAmount);
  const occupiedCapacity = BigInt("8200000000"); // 82 CKB
  const unoccupiedCapacity = amount - occupiedCapacity;
  
  const standardSize = BigInt("100000000000000"); // 100k CKB in iCKB terms
  const minSize = BigInt("100000000000"); // 1k CKB
  const maxSize = BigInt("100000000000000000"); // 1M CKB
  
  // Convert to iCKB equivalent
  const currentAR = BigInt(tipHeader.dao.slice(8, 24), 16);
  const genesisAR = BigInt("10000000000000000");
  const ickbEquivalent = unoccupiedCapacity * genesisAR / currentAR;
  
  return {
    isValid: unoccupiedCapacity >= minSize && unoccupiedCapacity <= maxSize,
    ckbAmount: ckbAmount,
    unoccupiedCapacity: unoccupiedCapacity.toString(),
    ickbEquivalent: ickbEquivalent.toString(),
    isStandardSize: ickbEquivalent === standardSize,
    oversizePenalty: ickbEquivalent > standardSize 
      ? ((ickbEquivalent - standardSize) / BigInt(10)).toString() // 10% penalty
      : "0",
    effectiveIckbAmount: ickbEquivalent > standardSize
      ? (ickbEquivalent - (ickbEquivalent - standardSize) / BigInt(10)).toString()
      : ickbEquivalent.toString(),
    recommendation: ickbEquivalent < standardSize
      ? "Consider depositing more CKB to reach standard size for optimal efficiency"
      : ickbEquivalent > standardSize
      ? "Consider splitting into multiple standard deposits to avoid penalty"
      : "Optimal deposit size"
  };
}
```

### 3. Order Management Issues

#### Order Status Confusion
**Symptoms:**
- Orders appear stuck in pending state
- Maturity calculations incorrect
- Order claiming fails

**Debug Order State:**
```typescript
async function debugOrderState(orderTxHash: string): Promise<OrderDebug> {
  try {
    // Get transaction details
    const tx = await client.getTransaction(orderTxHash);
    if (!tx) {
      return { status: 'not_found', error: 'Transaction not found' };
    }
    
    // Find order cell in outputs
    const orderOutputs = tx.transaction.outputs.filter((output, index) => {
      return output.type && isLimitOrderType(output.type);
    });
    
    if (orderOutputs.length === 0) {
      return { status: 'invalid', error: 'No limit order outputs found' };
    }
    
    // Check if order still exists (not consumed)
    const currentOrders = [];
    for (let i = 0; i < orderOutputs.length; i++) {
      const outPoint = { txHash: orderTxHash, index: `0x${i.toString(16)}` };
      try {
        const cell = await client.getCell(outPoint);
        if (cell) {
          currentOrders.push({
            outPoint,
            cell,
            orderData: parseOrderData(cell.outputData)
          });
        }
      } catch (error) {
        // Cell consumed (order completed/claimed)
      }
    }
    
    // Analyze order data
    const analysis = currentOrders.map(order => {
      const data = order.orderData;
      return {
        outPoint: order.outPoint,
        type: data.variant,
        ckbAmount: data.ckbAmount,
        udtAmount: data.udtAmount,
        ckbToUdtRatio: data.info.ckbToUdt,
        udtToCkbRatio: data.info.udtToCkb,
        minMatchSize: Math.pow(2, data.info.ckbMinMatchLog),
        isMatchable: this.isOrderMatchable(order, await sdk.getL1State(client, [])),
      };
    });
    
    return {
      status: currentOrders.length > 0 ? 'active' : 'completed',
      originalOutputs: orderOutputs.length,
      activeOrders: currentOrders.length,
      analysis
    };
    
  } catch (error) {
    return { status: 'error', error: error.message };
  }
}
```

#### Maturity Calculation Errors
**Symptoms:**
- Orders show incorrect maturity times
- Orders claimed before expected maturity
- Maturity never arrives

**Debug Maturity Calculation:**
```typescript
async function debugMaturity(orderCell: OrderCell): Promise<MaturityDebug> {
  const { system } = await sdk.getL1State(client, []);
  
  // Manual maturity calculation
  const orderInfo = parseOrderData(orderCell.outputData).info;
  const amounts = {
    ckbValue: orderCell.ckbUnoccupied,
    udtValue: orderCell.udtValue
  };
  
  // Check if dual-ratio (no fixed maturity)
  if (orderInfo.isDualRatio()) {
    return {
      hasDualRatio: true,
      maturity: undefined,
      reason: "Dual-ratio orders have no fixed maturity"
    };
  }
  
  const isCkb2Udt = orderInfo.isCkb2Udt();
  const amount = isCkb2Udt ? amounts.ckbValue : amounts.udtValue;
  
  if (amount === 0n) {
    return {
      isCompleted: true,
      maturity: 0n,
      reason: "Order already fulfilled"
    };
  }
  
  // Calculate required liquidity from system
  let requiredCkb = isCkb2Udt ? amount : 0n;
  let requiredUdt = isCkb2Udt ? 0n : amount;
  
  // Add competing orders
  for (const competingOrder of system.orderPool) {
    const competingInfo = competingOrder.data.info;
    if (competingInfo.isCkb2Udt()) {
      if (!isCkb2Udt || competingInfo.ckb2UdtCompare(orderInfo) < 0) {
        requiredCkb += competingOrder.ckbUnoccupied;
      }
    } else {
      if (isCkb2Udt || competingInfo.udt2CkbCompare(orderInfo) < 0) {
        requiredUdt += competingOrder.udtValue;
      }
    }
  }
  
  // Convert UDT requirement to CKB
  requiredCkb -= convert(false, requiredUdt, system.exchangeRatio);
  
  // Check immediate availability
  if (isCkb2Udt) {
    const availableCkb = system.ckbAvailable;
    if (availableCkb >= requiredCkb) {
      return {
        maturity: BigInt(Date.now()) + BigInt(600000), // 10 minutes minimum
        availableLiquidity: availableCkb.toString(),
        requiredLiquidity: requiredCkb.toString(),
        reason: "Sufficient immediate liquidity available"
      };
    }
  }
  
  // Find maturity in maturing deposits
  const deficit = requiredCkb - system.ckbAvailable;
  const maturityEntry = system.ckbMaturing.find(entry => entry.ckbCumulative >= deficit);
  
  return {
    maturity: maturityEntry?.maturity,
    deficit: deficit.toString(),
    availableLiquidity: system.ckbAvailable.toString(),
    requiredLiquidity: requiredCkb.toString(),
    maturingDeposits: system.ckbMaturing.length,
    reason: maturityEntry 
      ? `Maturity found in deposit schedule`
      : `Insufficient deposits in pipeline`
  };
}
```

### 4. Transaction Construction Errors

#### Cell Dependency Issues
**Symptoms:**
- Transactions fail validation with dependency errors
- "Type script not found" errors
- Cell dep loading failures

**Debug Cell Dependencies:**
```typescript
async function debugCellDeps(tx: ccc.Transaction): Promise<CellDepDebug> {
  const debug: CellDepDebug = {
    requiredDeps: [],
    missingDeps: [],
    conflictingDeps: [],
    recommendations: []
  };
  
  // Check for iCKB Logic dependency
  const hasIckbLogic = tx.cellDeps.some(dep => {
    return dep.outPoint.txHash === ICKB_LOGIC_TX_HASH; // From deployment
  });
  
  if (!hasIckbLogic) {
    debug.missingDeps.push({
      name: "iCKB Logic",
      outPoint: { txHash: ICKB_LOGIC_TX_HASH, index: "0x0" },
      depType: "depGroup"
    });
  }
  
  // Check for xUDT dependency
  const hasXUDT = tx.cellDeps.some(dep => {
    return dep.outPoint.txHash === XUDT_TX_HASH;
  });
  
  if (!hasXUDT) {
    debug.missingDeps.push({
      name: "xUDT Script",
      outPoint: { txHash: XUDT_TX_HASH, index: "0x0" },
      depType: "code"
    });
  }
  
  // Check for NervosDAO dependency
  const hasNervosDAO = tx.cellDeps.some(dep => {
    return dep.outPoint.txHash === NERVOS_DAO_TX_HASH;
  });
  
  if (!hasNervosDAO) {
    debug.missingDeps.push({
      name: "NervosDAO Script",
      outPoint: { txHash: NERVOS_DAO_TX_HASH, index: "0x0" },
      depType: "code"
    });
  }
  
  // Validate header dependencies
  const headerDeps = tx.headerDeps || [];
  debug.headerDeps = {
    count: headerDeps.length,
    required: this.getRequiredHeaderDeps(tx),
    valid: await this.validateHeaderDeps(headerDeps)
  };
  
  return debug;
}

function fixCellDeps(tx: ccc.Transaction): ccc.Transaction {
  const requiredDeps = [
    {
      outPoint: { txHash: ICKB_LOGIC_TX_HASH, index: "0x0" },
      depType: "depGroup" as const
    },
    {
      outPoint: { txHash: XUDT_TX_HASH, index: "0x0" },
      depType: "code" as const
    },
    {
      outPoint: { txHash: NERVOS_DAO_TX_HASH, index: "0x0" },
      depType: "code" as const
    }
  ];
  
  const existingDeps = new Set(
    tx.cellDeps.map(dep => `${dep.outPoint.txHash}-${dep.outPoint.index}`)
  );
  
  for (const requiredDep of requiredDeps) {
    const key = `${requiredDep.outPoint.txHash}-${requiredDep.outPoint.index}`;
    if (!existingDeps.has(key)) {
      tx.cellDeps.push(requiredDep);
    }
  }
  
  return tx;
}
```

### 5. Performance and Optimization

#### Slow Order Resolution
**Symptoms:**
- Orders take longer than expected to fulfill
- High gas costs
- Multiple transaction rounds needed

**Performance Analysis:**
```typescript
async function analyzePerformance(): Promise<PerformanceAnalysis> {
  const startTime = Date.now();
  
  // Measure system state query time
  const stateStart = Date.now();
  const { system } = await sdk.getL1State(client, []);
  const stateTime = Date.now() - stateStart;
  
  // Measure order pool analysis
  const poolStart = Date.now();
  const poolAnalysis = await analyzeOrderPool(system.orderPool);
  const poolTime = Date.now() - poolStart;
  
  // Measure bot response time
  const botStart = Date.now();
  const botStatus = await checkBotResponseTime();
  const botTime = Date.now() - botStart;
  
  return {
    totalTime: Date.now() - startTime,
    stateQueryTime: stateTime,
    poolAnalysisTime: poolTime,
    botResponseTime: botTime,
    systemHealth: {
      orderPoolSize: system.orderPool.length,
      poolLiquidity: system.ckbAvailable,
      activeBot: botStatus.isResponsive,
      avgFulfillmentTime: botStatus.avgFulfillmentTime
    },
    recommendations: this.generatePerformanceRecommendations({
      stateTime,
      poolTime,
      botTime,
      poolSize: system.orderPool.length
    })
  };
}
```

## Debugging Tools

### 1. iCKB Inspector
```typescript
class IckbInspector {
  async inspectProtocolState(): Promise<ProtocolInspection> {
    const { system, user } = await sdk.getL1State(client, this.userLocks);
    
    return {
      timestamp: Date.now(),
      blockNumber: system.tip.number,
      epoch: system.tip.epoch,
      poolState: {
        ckbAmount: system.ckbAvailable,
        udtAmount: system.ckbMaturing.reduce((sum, m) => sum + m.ckbCumulative, 0n),
        exchangeRate: system.exchangeRatio,
        utilization: this.calculateUtilization(system)
      },
      userState: {
        activeOrders: user.orders.length,
        totalValue: this.calculateTotalValue(user.orders, system),
        pendingMaturity: user.orders.filter(o => o.maturity && o.maturity > Date.now()).length
      },
      systemHealth: await this.assessSystemHealth(system)
    };
  }
  
  async generateReport(): Promise<string> {
    const inspection = await this.inspectProtocolState();
    
    return `
# iCKB Protocol Inspection Report

**Timestamp:** ${new Date(inspection.timestamp).toISOString()}
**Block:** ${inspection.blockNumber}
**Epoch:** ${inspection.epoch[0]}:${inspection.epoch[1]}:${inspection.epoch[2]}

## Pool State
- **CKB Available:** ${(Number(inspection.poolState.ckbAmount) / 1e8).toFixed(2)} CKB
- **Total Maturing:** ${(Number(inspection.poolState.udtAmount) / 1e8).toFixed(2)} CKB
- **Exchange Rate:** ${inspection.poolState.exchangeRate.numerator}/${inspection.poolState.exchangeRate.denominator}
- **Utilization:** ${(inspection.poolState.utilization * 100).toFixed(1)}%

## User State
- **Active Orders:** ${inspection.userState.activeOrders}
- **Total Value:** ${(Number(inspection.userState.totalValue) / 1e8).toFixed(2)} CKB
- **Pending Maturity:** ${inspection.userState.pendingMaturity}

## System Health
- **Overall Status:** ${inspection.systemHealth.status}
- **Bot Response:** ${inspection.systemHealth.botResponsive ? 'Active' : 'Inactive'}
- **Pool Balance:** ${inspection.systemHealth.poolBalanced ? 'Balanced' : 'Imbalanced'}
- **Recommendations:** ${inspection.systemHealth.recommendations.join(', ')}
    `;
  }
}
```

### 2. Transaction Simulator
```typescript
class TransactionSimulator {
  async simulateConversion(
    amount: string,
    isCkb2Udt: boolean
  ): Promise<SimulationResult> {
    try {
      // Get current state
      const { system } = await sdk.getL1State(client, []);
      
      // Estimate the conversion
      const estimate = IckbSdk.estimate(
        isCkb2Udt,
        isCkb2Udt ? { ckb: amount } : { udt: amount },
        system
      );
      
      // Simulate transaction construction
      const tx = ccc.Transaction.from({});
      const smartTx = new ccc.SmartTransaction(tx);
      
      // Add inputs (simulation only)
      const mockSigner = this.createMockSigner();
      await sdk.request(smartTx, mockSigner, estimate.info, estimate.amounts);
      
      // Calculate fees
      await smartTx.completeFeeBy(mockSigner, { feeRate: system.feeRate });
      
      return {
        success: true,
        estimate,
        transaction: smartTx.tx,
        totalFee: smartTx.getFee(),
        gasUsed: this.estimateGasUsage(smartTx.tx),
        maturityTime: estimate.maturity
      };
      
    } catch (error) {
      return {
        success: false,
        error: error.message,
        recommendation: this.getErrorRecommendation(error)
      };
    }
  }
}
```

This debugging guide provides comprehensive tools and techniques for identifying and resolving issues in iCKB protocol implementations, ensuring smooth operation and optimal performance.