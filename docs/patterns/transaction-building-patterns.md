# CKB Transaction Building Patterns

## Description

Advanced patterns for building CKB transactions using the Rust SDK. Covers basic transfers, multi-signature transactions, UDT operations, Anyone-Can-Pay patterns, DAO operations, batch processing, fee estimation, transaction templates, capacity management, and security validation with production-ready code examples.

This guide covers advanced patterns for building CKB transactions using the Rust SDK, based on production implementations from ckb-sdk-rust.

## Core Transaction Building Patterns

### 1. Basic Transaction Builder

Foundation pattern for constructing CKB transactions.

```rust
use ckb_sdk::{
    traits::{DefaultCellCollector, DefaultHeaderDepResolver, DefaultTransactionDependencyProvider},
    tx_builder::*,
    unlock::*,
    Address, AddressPayload, NetworkType,
};
use ckb_types::{
    core::{TransactionBuilder, TransactionView},
    packed::*,
    prelude::*,
};

pub struct TransactionHelper {
    network: NetworkType,
    cell_collector: DefaultCellCollector,
    header_dep_resolver: DefaultHeaderDepResolver,
    tx_dep_provider: DefaultTransactionDependencyProvider,
}

impl TransactionHelper {
    pub fn new(network: NetworkType, ckb_uri: &str) -> Self {
        let mut cell_collector = DefaultCellCollector::new(ckb_uri);
        let header_dep_resolver = DefaultHeaderDepResolver::new(ckb_uri);
        let tx_dep_provider = DefaultTransactionDependencyProvider::new(ckb_uri, 10);
        
        Self {
            network,
            cell_collector,
            header_dep_resolver,
            tx_dep_provider,
        }
    }
    
    pub fn build_transfer_tx(
        &mut self,
        from_address: &Address,
        to_address: &Address,
        amount_ckb: u64,
        fee_rate: u64,
        private_key: &[u8; 32],
    ) -> Result<TransactionView, Box<dyn std::error::Error>> {
        // Build initial transaction
        let mut builder = CapacityBalancer::new_simple(
            &mut self.cell_collector,
            &self.header_dep_resolver,
            &self.tx_dep_provider,
        );
        
        let to_lock_script = to_address.payload().into();
        let placeholder_witness = WitnessArgs::new_builder()
            .lock(Some(Bytes::from(vec![0u8; 65])).pack())
            .build();
            
        builder.add_output_and_data(
            CellOutput::new_builder()
                .capacity(amount_ckb.pack())
                .lock(to_lock_script)
                .build(),
            Bytes::new(),
        );
        
        // Balance capacity and add change output
        let balancer = builder.build(&mut PlaceholderWitnessGenerator::new(placeholder_witness))?;
        let tx = balancer.finalize(from_address.payload(), fee_rate)?;
        
        // Sign transaction
        let signer = SecpCkbRawKeySigner::new_with_secret_keys(vec![*private_key]);
        let tx_with_groups = TransactionWithScriptGroups::get_script_groups(&tx)?;
        let signed_tx = signer.sign_transaction(&tx_with_groups)?;
        
        Ok(signed_tx)  
    }
}
```

**Reference:** `resources/ckb-sdk-rust/examples/transfer_from_secp256k1.rs`

### 2. Multi-Signature Transaction Pattern

Build transactions requiring multiple signatures.

```rust
use ckb_sdk::unlock::MultisigConfig;

pub fn build_multisig_transaction(
    &mut self,
    multisig_config: &MultisigConfig,
    from_address: &Address,
    to_address: &Address,
    amount_ckb: u64,
    signers: &[Secp256k1PrivateKey],
) -> Result<TransactionView, Box<dyn std::error::Error>> {
    // Create multisig script
    let multisig_script = multisig_config.to_script();
    
    // Build transaction with multisig lock
    let mut builder = CapacityBalancer::new_simple(
        &mut self.cell_collector,
        &self.header_dep_resolver,
        &self.tx_dep_provider,
    );
    
    // Add multisig witness placeholder
    let placeholder_witness = WitnessArgs::new_builder()
        .lock(Some(multisig_config.placeholder_witness()).pack())
        .build();
        
    let to_lock_script = to_address.payload().into();
    builder.add_output_and_data(
        CellOutput::new_builder()
            .capacity(amount_ckb.pack())
            .lock(to_lock_script)
            .build(),
        Bytes::new(),
    );
    
    // Balance and finalize
    let balancer = builder.build(&mut PlaceholderWitnessGenerator::new(placeholder_witness))?;
    let tx = balancer.finalize(from_address.payload(), fee_rate)?;
    
    // Sign with multiple keys
    let mut multisig_signer = MultisigSigner::new(multisig_config.clone());
    for signer in signers {
        multisig_signer.add_signature(*signer, &tx)?;
    }
    
    let signed_tx = multisig_signer.finalize_transaction(&tx)?;
    Ok(signed_tx)
}
```

**Reference:** `resources/ckb-sdk-rust/examples/transfer_from_multisig.rs`

### 3. UDT (User Defined Token) Transfer Pattern

Transfer custom tokens with proper balance validation.

```rust
use ckb_sdk::traits::CellQueryOptions;

pub fn build_udt_transfer(
    &mut self,
    udt_type_script: &Script,
    from_address: &Address,
    to_address: &Address,
    udt_amount: u128,
    fee_rate: u64,
    private_key: &[u8; 32],
) -> Result<TransactionView, Box<dyn std::error::Error>> {
    // Query UDT cells
    let udt_query = CellQueryOptions::new_type(udt_type_script.clone());
    let udt_cells = self.cell_collector.collect_live_cells(&udt_query, true)?;
    
    let mut input_udt_amount = 0u128;
    let mut builder = TransactionBuilder::default();
    
    // Add UDT inputs
    for (out_point, cell_output, cell_data) in udt_cells {
        builder = builder.input(CellInput::new(out_point, 0));
        
        // Parse UDT amount from cell data (little-endian u128)
        if cell_data.len() >= 16 {
            let amount_bytes: [u8; 16] = cell_data[0..16].try_into()?;
            input_udt_amount += u128::from_le_bytes(amount_bytes);
        }
        
        if input_udt_amount >= udt_amount {
            break;
        }
    }
    
    if input_udt_amount < udt_amount {
        return Err("Insufficient UDT balance".into());
    }
    
    // Create UDT output
    let to_lock_script = to_address.payload().into();
    builder = builder.output(
        CellOutput::new_builder()
            .capacity(142u64.pack()) // Minimum capacity for UDT cell
            .lock(to_lock_script)
            .type_(Some(udt_type_script.clone()).pack())
            .build()
    );
    builder = builder.output_data(Bytes::from(udt_amount.to_le_bytes().to_vec()));
    
    // Create UDT change output if needed
    let change_amount = input_udt_amount - udt_amount;
    if change_amount > 0 {
        let from_lock_script = from_address.payload().into();
        builder = builder.output(
            CellOutput::new_builder()
                .capacity(142u64.pack())
                .lock(from_lock_script)
                .type_(Some(udt_type_script.clone()).pack())
                .build()
        );
        builder = builder.output_data(Bytes::from(change_amount.to_le_bytes().to_vec()));
    }
    
    // Add CKB capacity inputs and change
    let mut capacity_builder = CapacityBalancer::new_simple(
        &mut self.cell_collector,
        &self.header_dep_resolver,
        &self.tx_dep_provider,
    );
    
    let tx = builder.build();
    let balanced_tx = capacity_builder.balance_transaction_with_fee(&tx, fee_rate)?;
    
    // Sign transaction
    let signer = SecpCkbRawKeySigner::new_with_secret_keys(vec![*private_key]);
    let tx_with_groups = TransactionWithScriptGroups::get_script_groups(&balanced_tx)?;
    let signed_tx = signer.sign_transaction(&tx_with_groups)?;
    
    Ok(signed_tx)
}
```

**Reference:** `resources/ckb-sdk-rust/examples/transfer_from_udt.rs`

### 4. Anyone-Can-Pay Transaction Pattern

Enable partial transaction construction that others can complete.

```rust
use ckb_sdk::unlock::AnyoneCanPayUnlocker;

pub fn build_anyone_can_pay_transaction(
    &mut self,
    acp_address: &Address,
    contribution_amount: u64,
    private_key: &[u8; 32],
) -> Result<TransactionView, Box<dyn std::error::Error>> {
    // Load Anyone-Can-Pay script
    let acp_script_dep = self.tx_dep_provider
        .get_script_dep(&AnyoneCanPayUnlocker::TYPE_ID.into())?
        .ok_or("ACP script not found")?;
    
    let mut builder = TransactionBuilder::default();
    builder = builder.cell_dep(acp_script_dep);
    
    // Add contribution input
    let contribution_query = CellQueryOptions::new_lock(acp_address.payload().into());
    let contribution_cells = self.cell_collector.collect_live_cells(&contribution_query, true)?;
    
    for (out_point, cell_output, _) in contribution_cells.into_iter().take(1) {
        builder = builder.input(CellInput::new(out_point, 0));
        
        // Create ACP output with increased capacity
        let new_capacity = cell_output.capacity().unpack() + contribution_amount;
        builder = builder.output(
            CellOutput::new_builder()
                .capacity(new_capacity.pack())
                .lock(cell_output.lock())
                .type_(cell_output.type_())
                .build()
        );
        builder = builder.output_data(Bytes::new());
        break;
    }
    
    // Add witness for ACP unlock
    let witness = WitnessArgs::new_builder()
        .lock(Some(Bytes::from(vec![0u8; 65])).pack())
        .build();
    builder = builder.witness(witness.as_bytes().pack());
    
    let tx = builder.build();
    
    // Sign with ACP unlocker
    let unlocker = AnyoneCanPayUnlocker::new();
    let signed_tx = unlocker.sign_transaction(&tx, private_key)?;
    
    Ok(signed_tx)
}
```

**Reference:** `resources/ckb-sdk-rust/examples/transfer_from_acp.rs`

### 5. DAO (Nervos DAO) Transaction Patterns

Handle DAO deposits and withdrawals.

```rust
use ckb_sdk::constants::DAO_TYPE_HASH;

pub fn build_dao_deposit(
    &mut self,
    from_address: &Address,
    deposit_amount: u64,
    fee_rate: u64,
    private_key: &[u8; 32],
) -> Result<TransactionView, Box<dyn std::error::Error>> {
    // Load DAO type script
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
        Bytes::from(vec![0u8; 8]), // DAO data is 8 zero bytes for deposit
    );
    
    let placeholder_witness = WitnessArgs::new_builder()
        .lock(Some(Bytes::from(vec![0u8; 65])).pack())
        .build();
        
    let balancer = builder.build(&mut PlaceholderWitnessGenerator::new(placeholder_witness))?;
    let tx = balancer.finalize(from_address.payload(), fee_rate)?;
    
    // Sign transaction
    let signer = SecpCkbRawKeySigner::new_with_secret_keys(vec![*private_key]);
    let tx_with_groups = TransactionWithScriptGroups::get_script_groups(&tx)?;
    let signed_tx = signer.sign_transaction(&tx_with_groups)?;
    
    Ok(signed_tx)
}

pub fn build_dao_withdraw_phase1(
    &mut self,
    dao_cell_outpoint: &OutPoint,
    from_address: &Address,
    fee_rate: u64,
    private_key: &[u8; 32],
) -> Result<TransactionView, Box<dyn std::error::Error>> {
    // Load original DAO cell
    let (dao_output, dao_data) = self.cell_collector.get_live_cell(dao_cell_outpoint)?;
    
    let mut builder = TransactionBuilder::default();
    
    // Add DAO input
    builder = builder.input(CellInput::new(dao_cell_outpoint.clone(), 0));
    
    // Add withdrawal output (same capacity, different data)
    builder = builder.output(dao_output.clone());
    
    // Calculate withdrawal epoch
    let tip_header = self.header_dep_resolver.get_tip_header()?;
    let withdraw_block_hash = tip_header.hash();
    let withdraw_epoch = tip_header.epoch();
    
    // DAO withdrawal data: deposit_block_number (8 bytes) + withdraw_block_number (8 bytes)
    let mut withdraw_data = dao_data.to_vec();
    withdraw_data.extend_from_slice(&withdraw_epoch.number().to_le_bytes());
    
    builder = builder.output_data(Bytes::from(withdraw_data));
    
    // Add header dep
    builder = builder.header_dep(withdraw_block_hash);
    
    // Balance capacity for fee
    let mut capacity_builder = CapacityBalancer::new_simple(
        &mut self.cell_collector,
        &self.header_dep_resolver,
        &self.tx_dep_provider,
    );
    
    let tx = builder.build();
    let balanced_tx = capacity_builder.balance_transaction_with_fee(&tx, fee_rate)?;
    
    // Sign transaction
    let signer = SecpCkbRawKeySigner::new_with_secret_keys(vec![*private_key]);
    let tx_with_groups = TransactionWithScriptGroups::get_script_groups(&balanced_tx)?;
    let signed_tx = signer.sign_transaction(&tx_with_groups)?;
    
    Ok(signed_tx)
}
```

**Reference:** `resources/ckb-sdk-rust/examples/dao_operations.rs`

## Advanced Transaction Patterns

### 6. Batch Operations Pattern

Efficiently handle multiple operations in a single transaction.

```rust
pub struct BatchOperationBuilder {
    builder: TransactionBuilder,
    witness_count: usize,
}

impl BatchOperationBuilder {
    pub fn new() -> Self {
        Self {
            builder: TransactionBuilder::default(),
            witness_count: 0,
        }
    }
    
    pub fn add_transfer(
        &mut self,
        from_outpoint: &OutPoint,
        from_output: &CellOutput,
        to_address: &Address,
        amount: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Add input
        self.builder = self.builder.input(CellInput::new(from_outpoint.clone(), 0));
        
        // Add transfer output
        let to_lock_script = to_address.payload().into();
        self.builder = self.builder.output(
            CellOutput::new_builder()
                .capacity(amount.pack())
                .lock(to_lock_script)
                .build()
        );
        self.builder = self.builder.output_data(Bytes::new());
        
        // Add change output if needed
        let input_capacity = from_output.capacity().unpack();
        if input_capacity > amount {
            let change_amount = input_capacity - amount;
            self.builder = self.builder.output(
                CellOutput::new_builder()
                    .capacity(change_amount.pack())
                    .lock(from_output.lock())
                    .build()
            );
            self.builder = self.builder.output_data(Bytes::new());
        }
        
        // Add witness placeholder
        let witness = WitnessArgs::new_builder()
            .lock(Some(Bytes::from(vec![0u8; 65])).pack())
            .build();
        self.builder = self.builder.witness(witness.as_bytes().pack());
        self.witness_count += 1;
        
        Ok(())
    }
    
    pub fn finalize(self) -> TransactionView {
        self.builder.build()
    }
}
```

### 7. Fee Estimation Pattern

Advanced fee calculation for complex transactions.

```rust
use ckb_sdk::tx_builder::FeeCalculator;

pub fn estimate_transaction_fee(
    tx: &TransactionView,
    fee_rate: u64, // shannons per KB
) -> Result<u64, Box<dyn std::error::Error>> {
    // Calculate transaction size
    let tx_size = tx.data().serialized_size_in_block() as u64;
    
    // Base fee calculation: size * fee_rate / 1000
    let base_fee = tx_size * fee_rate / 1000;
    
    // Add complexity adjustments
    let input_count = tx.inputs().len() as u64;
    let output_count = tx.outputs().len() as u64;
    let witness_count = tx.witnesses().len() as u64;
    
    // Complexity factor for large transactions
    let complexity_factor = if input_count > 10 || output_count > 10 {
        1.5
    } else {
        1.0
    };
    
    let adjusted_fee = (base_fee as f64 * complexity_factor) as u64;
    
    // Minimum fee
    let min_fee = 1000u64; // 0.00001 CKB
    Ok(adjusted_fee.max(min_fee))
}

pub fn optimize_transaction_fee(
    &mut self,
    mut tx: TransactionView,
    target_fee_rate: u64,
    max_iterations: usize,
) -> Result<TransactionView, Box<dyn std::error::Error>> {
    for iteration in 0..max_iterations {
        let estimated_fee = estimate_transaction_fee(&tx, target_fee_rate)?;
        let current_fee = calculate_current_fee(&tx)?;
        
        if current_fee >= estimated_fee {
            break; // Fee is sufficient
        }
        
        // Adjust fee by modifying change output
        tx = adjust_change_output_for_fee(&tx, estimated_fee - current_fee)?;
        
        if iteration == max_iterations - 1 {
            return Err("Failed to optimize fee within iteration limit".into());
        }
    }
    
    Ok(tx)
}

fn calculate_current_fee(tx: &TransactionView) -> Result<u64, Box<dyn std::error::Error>> {
    let input_capacity: u64 = tx.inputs().into_iter()
        .map(|input| get_cell_capacity(&input.previous_output()))
        .sum::<Result<u64, _>>()?;
        
    let output_capacity: u64 = tx.outputs().into_iter()
        .map(|output| output.capacity().unpack())
        .sum();
        
    Ok(input_capacity - output_capacity)
}
```

### 8. Transaction Template Pattern

Create reusable transaction templates.

```rust
pub struct TransactionTemplate {
    pub inputs: Vec<CellInput>,
    pub outputs: Vec<CellOutput>,
    pub outputs_data: Vec<Bytes>,
    pub cell_deps: Vec<CellDep>,
    pub header_deps: Vec<Byte32>,
    pub witnesses: Vec<Bytes>,
}

impl TransactionTemplate {
    pub fn simple_transfer_template() -> Self {
        Self {
            inputs: vec![],
            outputs: vec![],
            outputs_data: vec![],
            cell_deps: vec![],
            header_deps: vec![],
            witnesses: vec![],
        }
    }
    
    pub fn udt_transfer_template(udt_type_script: Script) -> Self {
        let mut template = Self::simple_transfer_template();
        
        // Add UDT type script dependency
        template.cell_deps.push(
            CellDep::new_builder()
                .out_point(udt_type_script.into())
                .dep_type(DepType::Code.into())
                .build()
        );
        
        template
    }
    
    pub fn apply_template(
        &self,
        parameters: &TransactionParameters,
    ) -> Result<TransactionView, Box<dyn std::error::Error>> {
        let mut builder = TransactionBuilder::default();
        
        // Apply template structure
        for input in &self.inputs {
            builder = builder.input(input.clone());
        }
        
        for output in &self.outputs {
            builder = builder.output(output.clone());
        }
        
        for data in &self.outputs_data {
            builder = builder.output_data(data.clone());
        }
        
        for cell_dep in &self.cell_deps {
            builder = builder.cell_dep(cell_dep.clone());
        }
        
        for header_dep in &self.header_deps {
            builder = builder.header_dep(header_dep.clone());
        }
        
        for witness in &self.witnesses {
            builder = builder.witness(witness.clone().pack());
        }
        
        // Apply parameters
        builder = parameters.apply_to_builder(builder)?;
        
        Ok(builder.build())
    }
}

pub struct TransactionParameters {
    pub from_address: Address,
    pub to_address: Address,
    pub amount: u64,
    pub fee_rate: u64,
    pub private_keys: Vec<[u8; 32]>,
}

impl TransactionParameters {
    fn apply_to_builder(
        &self,
        mut builder: TransactionBuilder,
    ) -> Result<TransactionBuilder, Box<dyn std::error::Error>> {
        // Apply parameter-specific modifications
        // This is template-specific implementation
        Ok(builder)
    }
}
```

## Best Practices

### 1. Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum TransactionError {
    #[error("Insufficient capacity: required {required}, available {available}")]
    InsufficientCapacity { required: u64, available: u64 },
    
    #[error("Invalid address format: {0}")]
    InvalidAddress(String),
    
    #[error("Cell collection failed: {0}")]
    CellCollectionFailed(String),
    
    #[error("Transaction signing failed: {0}")]
    SigningFailed(String),
    
    #[error("Fee calculation failed: {0}")]
    FeeCalculationFailed(String),
}

pub type TransactionResult<T> = Result<T, TransactionError>;
```

### 2. Capacity Management

```rust
pub fn validate_transaction_capacity(tx: &TransactionView) -> TransactionResult<()> {
    let input_capacity: u64 = tx.inputs().into_iter()
        .map(|input| get_input_capacity(&input.previous_output()))
        .sum::<Result<u64, _>>()
        .map_err(|e| TransactionError::CellCollectionFailed(e.to_string()))?;
    
    let output_capacity: u64 = tx.outputs().into_iter()
        .map(|output| output.capacity().unpack())
        .sum();
    
    if input_capacity < output_capacity {
        return Err(TransactionError::InsufficientCapacity {
            required: output_capacity,
            available: input_capacity,
        });
    }
    
    // Validate minimum capacity for each output
    for (i, output) in tx.outputs().into_iter().enumerate() {
        let min_capacity = output.occupied_capacity(tx.outputs_data().get(i).unwrap_or(&Bytes::new()))?;
        let actual_capacity = output.capacity().unpack();
        
        if actual_capacity < min_capacity {
            return Err(TransactionError::InsufficientCapacity {
                required: min_capacity,
                available: actual_capacity,
            });
        }
    }
    
    Ok(())
}
```

### 3. Security Considerations

```rust
pub fn validate_transaction_security(tx: &TransactionView) -> TransactionResult<()> {
    // Check for double spending
    let mut seen_outpoints = std::collections::HashSet::new();
    for input in tx.inputs() {
        let outpoint = input.previous_output();
        if !seen_outpoints.insert(outpoint.clone()) {
            return Err(TransactionError::SigningFailed(
                "Transaction attempts to double spend".to_string()
            ));
        }
    }
    
    // Validate witness count matches input count
    let input_count = tx.inputs().len();
    let witness_count = tx.witnesses().len();
    
    if witness_count < input_count {
        return Err(TransactionError::SigningFailed(
            "Insufficient witnesses for inputs".to_string()
        ));
    }
    
    // Check for reasonable fee (not too high)
    let fee = calculate_current_fee(tx)?;
    let max_reasonable_fee = 1_000_000u64; // 0.01 CKB
    
    if fee > max_reasonable_fee {
        return Err(TransactionError::FeeCalculationFailed(
            format!("Fee too high: {} shannons", fee)
        ));
    }
    
    Ok(())
}
```

These transaction building patterns provide robust foundations for creating complex CKB transactions with proper error handling, security validation, and optimization strategies.