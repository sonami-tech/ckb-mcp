## Description

Nervos DAO deposit and withdrawal phase 1 patterns. Rust transaction building for CKB staking with ckb-sdk. TypeScript frontend integration using CCC connector. DAO type script setup, deposit cell creation with 8 zero bytes data, withdrawal request with epoch markers, and header dependency management.

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
