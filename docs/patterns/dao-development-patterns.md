# CKB DAO Development Patterns

## Description

Develop applications using the Nervos DAO system for CKB staking and rewards. Learn Rust transaction building, TypeScript frontend integration with CCC, multi-wallet support, and advanced patterns including iCKB integration and analytics. Covers deposit/withdrawal workflows, compensation calculations, testing strategies, and production-ready DAO application development.

This guide covers comprehensive patterns for developing with the Nervos DAO, based on production implementations from nervdao and ckb-system-scripts. **Modern DAO development should use Rust** for smart contracts and TypeScript/JavaScript for frontend applications.

## Core DAO Concepts

### 1. DAO Script Overview

The Nervos DAO is a core system script that allows CKB holders to lock tokens and earn interest based on the issuance rate.

**Key Features:**
- **Two-phase withdrawal**: Deposit → Request withdrawal → Complete withdrawal
- **Interest calculation**: Based on epoch rewards and inflation
- **Lock period**: Minimum lock time from deposit to withdrawal completion
- **Compensation formula**: Uses accumulation rate (AR) from DAO field in block headers

**System Script Reference:** `resources/ckb-system-scripts/c/dao.c`

## Modern DAO Development Patterns (Rust + TypeScript)

### 1. Rust DAO Transaction Builder

```rust
use ckb_sdk::{
    CkbRpcClient, HttpRpcClient,
    traits::{DefaultCellCollector, DefaultHeaderDepResolver, DefaultTransactionDependencyProvider},
    tx_builder::*,
    unlock::*,
    constants::DAO_TYPE_HASH,
};
use ckb_types::{
    core::{TransactionBuilder, TransactionView, EpochNumber},
    packed::*,
    prelude::*,
    H256, U256,
};

pub struct DaoTransactionBuilder {
    rpc_client: HttpRpcClient,
    cell_collector: DefaultCellCollector,
    header_dep_resolver: DefaultHeaderDepResolver,
    tx_dep_provider: DefaultTransactionDependencyProvider,
}

impl DaoTransactionBuilder {
    pub fn new(ckb_uri: &str) -> Self {
        Self {
            rpc_client: HttpRpcClient::new(ckb_uri.to_string()),
            cell_collector: DefaultCellCollector::new(ckb_uri),
            header_dep_resolver: DefaultHeaderDepResolver::new(ckb_uri),
            tx_dep_provider: DefaultTransactionDependencyProvider::new(ckb_uri, 10),
        }
    }
    
    // Deposit CKB into DAO
    pub async fn create_dao_deposit(
        &mut self,
        from_address: &Address,
        deposit_amount: u64,
        fee_rate: u64,
        private_key: [u8; 32],
    ) -> Result<TransactionView, Box<dyn std::error::Error>> {
        // Create DAO type script
        let dao_type_script = Script::new_builder()
            .code_hash(DAO_TYPE_HASH.pack())
            .hash_type(ScriptHashType::Type.into())
            .build();
        
        let mut builder = CapacityBalancer::new_simple(
            &mut self.cell_collector,
            &self.header_dep_resolver,
            &self.tx_dep_provider,
        );
        
        // Add DAO deposit output
        builder.add_output_and_data(
            CellOutput::new_builder()
                .capacity(deposit_amount.pack())
                .lock(from_address.payload().into())
                .type_(Some(dao_type_script).pack())
                .build(),
            Bytes::from(vec![0u8; 8]), // DAO deposit data: 8 zero bytes
        );
        
        let placeholder_witness = WitnessArgs::new_builder()
            .lock(Some(Bytes::from(vec![0u8; 65])).pack())
            .build();
            
        let balancer = builder.build(&mut PlaceholderWitnessGenerator::new(placeholder_witness))?;
        let tx = balancer.finalize(from_address.payload(), fee_rate)?;
        
        // Sign transaction
        let signer = SecpCkbRawKeySigner::new_with_secret_keys(vec![private_key]);
        let tx_with_groups = TransactionWithScriptGroups::get_script_groups(&tx)?;
        let signed_tx = signer.sign_transaction(&tx_with_groups)?;
        
        Ok(signed_tx)
    }
    
    // Start DAO withdrawal (Phase 1)
    pub async fn start_dao_withdrawal(
        &mut self,
        dao_deposit_outpoint: &OutPoint,
        from_address: &Address,
        fee_rate: u64,
        private_key: [u8; 32],
    ) -> Result<TransactionView, Box<dyn std::error::Error>> {
        // Get original DAO deposit cell
        let (deposit_output, deposit_data) = self.get_cell_details(&dao_deposit_outpoint)
            .await?
            .ok_or("DAO deposit cell not found")?;
        
        let mut builder = TransactionBuilder::default();
        
        // Add DAO input
        builder = builder.input(CellInput::new(dao_deposit_outpoint.clone(), 0));
        
        // Add withdrawal output (same capacity, updated data)
        builder = builder.output(deposit_output.clone());
        
        // Get current tip header for withdrawal marker
        let tip_header = self.rpc_client.get_tip_header()?;
        let current_epoch = tip_header.epoch();
        
        // Create withdrawal data: original_data + current_epoch_number
        let mut withdrawal_data = deposit_data.to_vec();
        withdrawal_data.extend_from_slice(&current_epoch.number().to_le_bytes());
        
        builder = builder.output_data(Bytes::from(withdrawal_data).pack());
        
        // Add header dependency
        builder = builder.header_dep(tip_header.hash());
        
        // Balance and sign
        let tx = builder.build();
        let balanced_tx = self.balance_transaction_fee(&tx, fee_rate).await?;
        
        let signer = SecpCkbRawKeySigner::new_with_secret_keys(vec![private_key]);
        let tx_with_groups = TransactionWithScriptGroups::get_script_groups(&balanced_tx)?;
        let signed_tx = signer.sign_transaction(&tx_with_groups)?;
        
        Ok(signed_tx)
    }
    
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

**Reference:** `resources/nervdao/src/cores/transaction.ts` (TypeScript implementation)

### 2. TypeScript Frontend Integration

Modern DAO applications use TypeScript with CCC (Common Chain Connector) for wallet integration:

```typescript
// Based on nervdao production implementation
import { ccc } from "@ckb-ccc/connector-react";
import { Script, Transaction } from "@ckb-lumos/base";

export class DaoManager {
    private signer: ccc.Signer;
    
    constructor(signer: ccc.Signer) {
        this.signer = signer;
    }
    
    // Create DAO deposit transaction
    async createDaoDeposit(amount: bigint): Promise<ccc.Transaction> {
        const tx = ccc.Transaction.from({
            inputs: [],
            outputs: [],
            outputsData: [],
            cellDeps: [],
            headerDeps: [],
            witnesses: [],
        });
        
        // Add DAO type script
        const daoTypeScript = {
            codeHash: "0x82d76d1b75fe2fd9a27dfbaa65a039221a380d76c926f378d3f81cf3e7e13f2e",
            hashType: "type" as const,
            args: "0x",
        };
        
        // Build DAO deposit output
        const daoOutput = {
            capacity: ccc.fixedPointFrom(amount),
            lock: await this.signer.getRecommendedAddressObj(),
            type: daoTypeScript,
        };
        
        tx.outputs.push(daoOutput);
        tx.outputsData.push("0x0000000000000000"); // 8 zero bytes for deposit
        
        // Add inputs and balance capacity
        await tx.addCellDepsOfKnownScripts(this.signer.client, ccc.KnownScript.NervosDao);
        await tx.completeInputsByCapacity(this.signer);
        await tx.completeFeeBy(this.signer, 1000n);
        
        return tx;
    }
    
    // Start withdrawal process
    async startDaoWithdrawal(depositOutPoint: ccc.OutPoint): Promise<ccc.Transaction> {
        const tx = ccc.Transaction.from({
            inputs: [],
            outputs: [],
            outputsData: [],
            cellDeps: [],
            headerDeps: [],
            witnesses: [],
        });
        
        // Get deposit cell
        const depositCell = await this.signer.client.getCell(depositOutPoint);
        if (!depositCell) {
            throw new Error("Deposit cell not found");
        }
        
        // Add input
        tx.inputs.push({ previousOutput: depositOutPoint, since: "0x0" });
        
        // Add output (same as input)
        tx.outputs.push(depositCell.cellOutput);
        
        // Update data with current epoch
        const tipHeader = await this.signer.client.getTipHeader();
        const currentEpoch = tipHeader.epoch;
        
        const withdrawalData = depositCell.data + ccc.numLeToBytes(currentEpoch, 8).slice(2);
        tx.outputsData.push(withdrawalData);
        
        // Add header dependency
        tx.headerDeps.push(tipHeader.hash);
        
        await tx.addCellDepsOfKnownScripts(this.signer.client, ccc.KnownScript.NervosDao);
        await tx.completeFeeBy(this.signer, 1000n);
        
        return tx;
    }
    
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
}
```

**Reference:** `resources/nervdao/src/cores/transaction.ts`

### 3. Multi-Wallet Integration Pattern

Modern DAO applications support multiple wallets using CCC:

```typescript
import { ccc } from "@ckb-ccc/connector-react";

// Wallet configuration for DAO applications
export const walletConfig = {
    wallets: [
        ccc.WalletWithSigners.new(
            new ccc.wallets.MetaMask(),
            new ccc.SignerCkbScriptOmniLock()
        ),
        ccc.WalletWithSigners.new(
            new ccc.wallets.Unisat(),
            new ccc.SignerCkbScriptOmniLock()
        ),
        ccc.WalletWithSigners.new(
            new ccc.wallets.OkxWallet(),
            new ccc.SignerCkbScriptOmniLock()
        ),
        ccc.WalletWithSigners.new(
            new ccc.wallets.JoyId(),
            new ccc.SignerCkbScriptOmniLock()
        ),
    ],
};

// React hook for DAO operations
export function useDaoOperations() {
    const { signer } = ccc.useCcc();
    const [daoManager, setDaoManager] = useState<DaoManager | null>(null);
    
    useEffect(() => {
        if (signer) {
            setDaoManager(new DaoManager(signer));
        }
    }, [signer]);
    
    const deposit = useCallback(async (amount: bigint) => {
        if (!daoManager) throw new Error("No signer available");
        
        const tx = await daoManager.createDaoDeposit(amount);
        const txHash = await signer.sendTransaction(tx);
        
        return txHash;
    }, [daoManager, signer]);
    
    const startWithdrawal = useCallback(async (outPoint: ccc.OutPoint) => {
        if (!daoManager) throw new Error("No signer available");
        
        const tx = await daoManager.startDaoWithdrawal(outPoint);
        const txHash = await signer.sendTransaction(tx);
        
        return txHash;
    }, [daoManager, signer]);
    
    return { deposit, startWithdrawal, daoManager };
}
```

**Reference:** `resources/nervdao/src/app/` (React components)

## Advanced DAO Patterns

### 4. iCKB Integration Pattern

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

### 5. DAO Analytics and Monitoring

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

### 6. Comprehensive DAO Testing

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

### 1. **Use Rust for Smart Contracts**
- System scripts are implemented in C for performance, but modern development should use Rust
- Rust provides better safety guarantees and development experience
- Use ckb-std crate for Rust contract development

### 2. **TypeScript for Frontend**
- Use CCC (Common Chain Connector) for wallet integration
- Support multiple wallets (MetaMask, Unisat, OKX, JoyID)
- Implement proper error handling and user feedback

### 3. **Proper Testing**
- Test entire DAO lifecycle: deposit → withdrawal start → withdrawal completion
- Test edge cases: minimum lock periods, compensation calculations
- Use ckb-tool for comprehensive transaction testing

### 4. **Security Considerations**
- Always validate epoch numbers and header dependencies
- Verify DAO data format (8 bytes for deposit, 16 bytes for withdrawal)
- Implement proper fee estimation and slippage protection

### 5. **User Experience**
- Provide clear feedback on lock periods and estimated rewards
- Show real-time compensation calculations
- Support both individual operations and batch processing

This comprehensive guide provides patterns for building modern DAO applications that leverage both the security of Rust smart contracts and the flexibility of TypeScript frontend development.