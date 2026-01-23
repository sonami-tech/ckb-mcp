# DAO Advanced Patterns

## Description

Nervos DAO withdrawal phase 2 completion, interest calculation, and SDK integration. Compensation formula using accumulation rate (AR) from block headers. iCKB liquidity integration, DAO analytics and monitoring, APY tracking across epochs. Testing patterns for full DAO lifecycle verification.

## Withdraw Phase 2: Completion with Compensation

### Rust Implementation

```rust
impl DaoTransactionBuilder {
    // Complete DAO withdrawal (Phase 2) with compensation calculation
    pub async fn complete_dao_withdrawal(
        &mut self,
        withdrawal_outpoint: &OutPoint,
        to_address: &Address,
        private_key: [u8; 32],
    ) -> Result<(TransactionView, u64), Box<dyn std::error::Error>> {
        // Get withdrawal cell
        let (withdrawal_output, withdrawal_data) = self.get_cell_details(&withdrawal_outpoint)
            .await?
            .ok_or("Withdrawal cell not found")?;

        // Parse withdrawal data
        if withdrawal_data.len() < 16 {
            return Err("Invalid withdrawal data".into());
        }

        let deposit_epoch = u64::from_le_bytes(
            withdrawal_data[0..8].try_into().unwrap()
        );
        let withdrawal_start_epoch = u64::from_le_bytes(
            withdrawal_data[8..16].try_into().unwrap()
        );

        // Calculate compensation
        let original_capacity = withdrawal_output.capacity().unpack();
        let compensation = self.calculate_dao_compensation(
            original_capacity,
            deposit_epoch,
            withdrawal_start_epoch,
        ).await?;

        let mut builder = TransactionBuilder::default();

        // Add withdrawal input
        builder = builder.input(CellInput::new(withdrawal_outpoint.clone(), 0));

        // Add compensation output
        builder = builder.output(
            CellOutput::new_builder()
                .capacity(compensation.pack())
                .lock(to_address.payload().into())
                .build()
        );
        builder = builder.output_data(Bytes::new().pack());

        // Add required header dependencies
        let deposit_block_hash = self.rpc_client.get_block_hash(deposit_epoch)?;
        let withdrawal_block_hash = self.rpc_client.get_block_hash(withdrawal_start_epoch)?;

        builder = builder.header_dep(deposit_block_hash);
        builder = builder.header_dep(withdrawal_block_hash);

        let tx = builder.build();

        // Sign transaction
        let signer = SecpCkbRawKeySigner::new_with_secret_keys(vec![private_key]);
        let tx_with_groups = TransactionWithScriptGroups::get_script_groups(&tx)?;
        let signed_tx = signer.sign_transaction(&tx_with_groups)?;

        Ok((signed_tx, compensation))
    }
}
```

### TypeScript Implementation

```typescript
export class DaoManager {
    // Complete withdrawal with compensation
    async completeDaoWithdrawal(
        withdrawalOutPoint: ccc.OutPoint
    ): Promise<{ tx: ccc.Transaction; compensation: bigint }> {
        const tx = ccc.Transaction.from({
            inputs: [],
            outputs: [],
            outputsData: [],
            cellDeps: [],
            headerDeps: [],
            witnesses: [],
        });

        // Get withdrawal cell
        const withdrawalCell = await this.signer.client.getCell(withdrawalOutPoint);
        if (!withdrawalCell) {
            throw new Error("Withdrawal cell not found");
        }

        // Parse epochs from data
        const data = withdrawalCell.data;
        const depositEpoch = ccc.bytesLeToNum(data.slice(2, 18));
        const withdrawalEpoch = ccc.bytesLeToNum(data.slice(18, 34));

        // Calculate compensation
        const compensation = await this.calculateCompensation(
            BigInt(withdrawalCell.cellOutput.capacity),
            depositEpoch,
            withdrawalEpoch
        );

        // Add input
        tx.inputs.push({ previousOutput: withdrawalOutPoint, since: "0x0" });

        // Add compensation output
        const outputLock = await this.signer.getRecommendedAddressObj();
        tx.outputs.push({
            capacity: ccc.fixedPointFrom(compensation),
            lock: outputLock,
        });
        tx.outputsData.push("0x");

        // Add header dependencies
        const depositHeader = await this.signer.client.getHeaderByNumber(depositEpoch);
        const withdrawalHeader = await this.signer.client.getHeaderByNumber(withdrawalEpoch);

        tx.headerDeps.push(depositHeader.hash);
        tx.headerDeps.push(withdrawalHeader.hash);

        await tx.addCellDepsOfKnownScripts(this.signer.client, ccc.KnownScript.NervosDao);

        return { tx, compensation };
    }
}
```

## Interest Calculation

### Compensation Formula

The DAO compensation is calculated using the Accumulation Rate (AR) from block headers:

```
compensation = original_capacity * withdrawal_ar / deposit_ar
```

### Rust AR Extraction

```rust
impl DaoTransactionBuilder {
    // Calculate DAO compensation using proper formula
    async fn calculate_dao_compensation(
        &self,
        original_capacity: u64,
        deposit_epoch: u64,
        withdrawal_epoch: u64,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        // Get DAO AR (Accumulation Rate) from headers
        let deposit_header = self.rpc_client.get_header_by_number(deposit_epoch)?;
        let withdrawal_header = self.rpc_client.get_header_by_number(withdrawal_epoch)?;

        let deposit_ar = extract_dao_ar(&deposit_header.inner.dao);
        let withdrawal_ar = extract_dao_ar(&withdrawal_header.inner.dao);

        // Calculate compensation: original_capacity * withdrawal_ar / deposit_ar
        let compensation = (U256::from(original_capacity) * U256::from(withdrawal_ar)
            / U256::from(deposit_ar)).as_u64();

        Ok(compensation)
    }
}

// Helper function to extract AR from DAO field
fn extract_dao_ar(dao: &Byte32) -> u64 {
    let dao_bytes = dao.as_slice();
    u64::from_le_bytes(dao_bytes[8..16].try_into().unwrap())
}
```

### TypeScript AR Extraction

```typescript
private async calculateCompensation(
    originalCapacity: bigint,
    depositEpoch: bigint,
    withdrawalEpoch: bigint
): Promise<bigint> {
    const depositHeader = await this.signer.client.getHeaderByNumber(depositEpoch);
    const withdrawalHeader = await this.signer.client.getHeaderByNumber(withdrawalEpoch);

    // Extract AR from DAO field (bytes 8-15)
    const depositAR = ccc.bytesLeToNum(depositHeader.dao.slice(18, 34));
    const withdrawalAR = ccc.bytesLeToNum(withdrawalHeader.dao.slice(18, 34));

    // Calculate: original_capacity * withdrawal_ar / deposit_ar
    const compensation = (originalCapacity * withdrawalAR) / depositAR;

    return compensation;
}
```

## Advanced DAO Patterns

### iCKB Integration Pattern

Modern DAO applications integrate with iCKB for enhanced liquidity:

```rust
use ickb_sdk::{IckbPoolManager, LiquidityPosition};

pub struct AdvancedDaoManager {
    dao_builder: DaoTransactionBuilder,
    ickb_pool: IckbPoolManager,
}

impl AdvancedDaoManager {
    // Deposit into DAO and provide iCKB liquidity
    pub async fn deposit_with_ickb_liquidity(
        &mut self,
        deposit_amount: u64,
        liquidity_amount: u64,
        user_address: &Address,
        private_key: [u8; 32],
    ) -> Result<(TransactionView, LiquidityPosition), Box<dyn std::error::Error>> {
        // Create DAO deposit
        let dao_tx = self.dao_builder.create_dao_deposit(
            user_address,
            deposit_amount,
            1000, // fee rate
            private_key,
        ).await?;

        // Create iCKB liquidity position
        let liquidity_position = self.ickb_pool.add_liquidity(
            user_address,
            liquidity_amount,
            private_key,
        ).await?;

        // Combine transactions if needed or return separately
        Ok((dao_tx, liquidity_position))
    }

    // Calculate optimal DAO vs iCKB allocation
    pub async fn calculate_optimal_allocation(
        &self,
        total_amount: u64,
    ) -> Result<(u64, u64), Box<dyn std::error::Error>> {
        // Get current DAO APY
        let dao_apy = self.calculate_dao_apy().await?;

        // Get current iCKB yield
        let ickb_apy = self.ickb_pool.get_current_apy().await?;

        // Simple allocation based on yields
        let dao_allocation = if dao_apy > ickb_apy {
            total_amount * 70 / 100 // 70% to DAO if better yield
        } else {
            total_amount * 30 / 100 // 30% to DAO otherwise
        };

        let ickb_allocation = total_amount - dao_allocation;

        Ok((dao_allocation, ickb_allocation))
    }

    async fn calculate_dao_apy(&self) -> Result<f64, Box<dyn std::error::Error>> {
        // Implementation based on recent epoch rewards
        // This would calculate APY from blockchain data
        Ok(2.8) // Approximate current DAO APY
    }
}
```

### DAO Analytics and Monitoring

```rust
pub struct DaoAnalytics {
    rpc_client: HttpRpcClient,
}

impl DaoAnalytics {
    // Track DAO deposits across epochs
    pub async fn get_dao_statistics(
        &self,
        from_epoch: u64,
        to_epoch: u64,
    ) -> Result<DaoStats, Box<dyn std::error::Error>> {
        let mut total_deposits = 0u64;
        let mut total_withdrawals = 0u64;
        let mut active_deposits = 0u64;

        for epoch in from_epoch..=to_epoch {
            let block_hash = self.rpc_client.get_block_hash(epoch)?;
            let block = self.rpc_client.get_block(block_hash)?;

            for tx in block.transactions {
                // Analyze DAO operations in each transaction
                let (deposits, withdrawals) = self.analyze_dao_operations(&tx)?;
                total_deposits += deposits;
                total_withdrawals += withdrawals;
            }
        }

        Ok(DaoStats {
            total_deposits,
            total_withdrawals,
            active_deposits: total_deposits - total_withdrawals,
            average_lock_period: self.calculate_average_lock_period().await?,
        })
    }

    // Monitor DAO compensation rates
    pub async fn track_compensation_rates(&self) -> Result<Vec<CompensationRate>, Box<dyn std::error::Error>> {
        let mut rates = Vec::new();
        let tip_header = self.rpc_client.get_tip_header()?;
        let current_epoch = tip_header.epoch().number();

        // Calculate rates for last 100 epochs
        for epoch in (current_epoch - 100)..current_epoch {
            let header = self.rpc_client.get_header_by_number(epoch)?;
            let ar = extract_dao_ar(&header.inner.dao);

            if epoch > 0 {
                let prev_header = self.rpc_client.get_header_by_number(epoch - 1)?;
                let prev_ar = extract_dao_ar(&prev_header.inner.dao);

                let rate = ((ar as f64 / prev_ar as f64) - 1.0) * 100.0;
                rates.push(CompensationRate { epoch, rate });
            }
        }

        Ok(rates)
    }
}

pub struct DaoStats {
    pub total_deposits: u64,
    pub total_withdrawals: u64,
    pub active_deposits: u64,
    pub average_lock_period: u64,
}

pub struct CompensationRate {
    pub epoch: u64,
    pub rate: f64, // Percentage growth rate
}
```

## Testing Patterns

### Comprehensive DAO Testing

```rust
#[cfg(test)]
mod dao_tests {
    use super::*;
    use ckb_tool::{ckb_error::assert_error_eq, ckb_script::ScriptError};

    #[test]
    fn test_dao_deposit_lifecycle() {
        let mut context = Context::default();

        // Deploy DAO script
        let dao_bin = include_bytes!("../../../resources/ckb-system-scripts/build/dao");
        let dao_out_point = context.deploy_cell(dao_bin.to_vec().into());

        // Create test account
        let private_key = [1u8; 32];
        let public_key = secp256k1_pubkey(&private_key);
        let lock_script = build_secp256k1_lock_script(&public_key);

        // Test deposit
        let deposit_tx = build_dao_deposit_tx(&context, &lock_script, 10000000000u64);
        let cycles = context.verify_tx(&deposit_tx, MAX_CYCLES).expect("deposit should succeed");
        println!("DAO deposit cycles: {}", cycles);

        // Test withdrawal start
        let withdrawal_tx = build_dao_withdrawal_start_tx(&context, &deposit_tx, 0);
        let cycles = context.verify_tx(&withdrawal_tx, MAX_CYCLES).expect("withdrawal start should succeed");
        println!("DAO withdrawal start cycles: {}", cycles);

        // Test withdrawal completion (with proper epoch gap)
        let completion_tx = build_dao_withdrawal_completion_tx(&context, &withdrawal_tx, 0);
        let cycles = context.verify_tx(&completion_tx, MAX_CYCLES).expect("withdrawal completion should succeed");
        println!("DAO withdrawal completion cycles: {}", cycles);
    }

    #[test]
    fn test_dao_compensation_calculation() {
        // Test the DAO compensation formula
        let original_capacity = 10000000000u64; // 100 CKB
        let deposit_ar = 1000000000000000000u64; // Initial AR
        let withdrawal_ar = 1050000000000000000u64; // 5% increase

        let expected_compensation = (original_capacity as u128 * withdrawal_ar as u128 / deposit_ar as u128) as u64;
        let calculated_compensation = calculate_dao_compensation(original_capacity, deposit_ar, withdrawal_ar);

        assert_eq!(expected_compensation, calculated_compensation);
        assert!(calculated_compensation > original_capacity); // Should be more than original
    }
}
```

**Reference:** `resources/ckb-system-scripts/src/tests/dao_tests.rs`

## Best Practices

### 1. Use Rust for Smart Contracts
- System scripts are implemented in C for performance, but modern development should use Rust
- Rust provides better safety guarantees and development experience
- Use ckb-std crate for Rust contract development

### 2. TypeScript for Frontend
- Use CCC (Common Chain Connector) for wallet integration
- Support multiple wallets (MetaMask, Unisat, OKX, JoyID)
- Implement proper error handling and user feedback

### 3. Proper Testing
- Test entire DAO lifecycle: deposit -> withdrawal start -> withdrawal completion
- Test edge cases: minimum lock periods, compensation calculations
- Use ckb-tool for comprehensive transaction testing

### 4. Security Considerations
- Always validate epoch numbers and header dependencies
- Verify DAO data format (8 bytes for deposit, 16 bytes for withdrawal)
- Implement proper fee estimation and slippage protection

### 5. User Experience
- Provide clear feedback on lock periods and estimated rewards
- Show real-time compensation calculations
- Support both individual operations and batch processing
