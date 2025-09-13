## Description

Integrate CoBuild protocol into CKB applications for enhanced user experience and transaction composability. Learn to define action schemas, implement message validation, build transactions with BuildingPacket, handle multi-party signatures, and support wallet integration. Covers migration strategies, cross-script interactions, and best practices for building user-friendly blockchain applications.

## Overview

CoBuild integration involves:
1. Defining action schemas for your scripts
2. Implementing message validation in scripts
3. Building transactions with BuildingPacket
4. Handling multi-party signatures

## Defining Action Schemas

### Basic Action Structure

```rust
// Define your application's actions using Molecule
table Transfer {
    from: Address,
    to: Address,
    amount: Uint128,
    token_id: Byte32,
}

table Mint {
    to: Address,
    amount: Uint128,
    metadata: Bytes,
}

union MyAppAction {
    Transfer,
    Mint,
}
```

### ScriptInfo Declaration

```rust
const SCRIPT_INFO: ScriptInfo = ScriptInfo {
    name: "MyDeFiApp",
    url: "https://myapp.example.com",
    script_hash: MY_SCRIPT_HASH,
    schema: include_str!("./schemas/actions.mol"),
    message_type: "MyAppAction",
};
```

## Script Implementation

### Message Validation

```rust
use ckb_std::high_level::load_witness;

pub fn validate_message() -> Result<(), Error> {
    // Load witness with CoBuild format
    let witness = load_witness(0, Source::GroupInput)?;
    
    // Parse WitnessLayout
    let witness_layout = WitnessLayout::from_slice(&witness)
        .map_err(|_| Error::InvalidWitness)?;
    
    match witness_layout {
        WitnessLayout::SighashAll { message, .. } => {
            validate_message_actions(message)?;
        }
        WitnessLayout::OtxStart { .. } => {
            // Handle OTX validation
        }
        _ => return Err(Error::UnsupportedWitnessFormat),
    }
    
    Ok(())
}

fn validate_message_actions(message: Message) -> Result<(), Error> {
    for action in message.actions() {
        // Verify action matches our script
        if action.script_hash() != MY_SCRIPT_HASH {
            continue;
        }
        
        // Parse and validate action data
        let app_action = MyAppAction::from_slice(action.data())
            .map_err(|_| Error::InvalidAction)?;
            
        match app_action {
            MyAppAction::Transfer(transfer) => {
                validate_transfer(transfer)?;
            }
            MyAppAction::Mint(mint) => {
                validate_mint(mint)?;
            }
        }
    }
    
    Ok(())
}
```

### State Transition Validation

```rust
fn validate_transfer(transfer: Transfer) -> Result<(), Error> {
    // Load current state
    let input_data = load_cell_data(0, Source::GroupInput)?;
    let output_data = load_cell_data(0, Source::GroupOutput)?;
    
    // Verify transfer matches state changes
    let input_balance = parse_balance(&input_data)?;
    let output_balance = parse_balance(&output_data)?;
    
    if input_balance - output_balance != transfer.amount() {
        return Err(Error::AmountMismatch);
    }
    
    // Additional validation...
    Ok(())
}
```

## Building Transactions

### Creating BuildingPacket

```rust
use ckb_types::packed::{Transaction, BuildingPacketV1};

pub fn create_building_packet(
    user_action: MyAppAction,
    tx: Transaction,
) -> Result<BuildingPacket, Error> {
    // Create action
    let action = Action::new_builder()
        .script_info_hash(calculate_script_info_hash(&SCRIPT_INFO))
        .script_hash(MY_SCRIPT_HASH.pack())
        .data(user_action.as_slice().pack())
        .build();
    
    // Create message
    let message = Message::new_builder()
        .actions(vec![action].pack())
        .build();
    
    // Resolve inputs for fee calculation
    let resolved_inputs = resolve_transaction_inputs(&tx)?;
    
    // Build packet
    let packet = BuildingPacketV1::new_builder()
        .message(message)
        .payload(tx)
        .resolved_inputs(resolved_inputs)
        .change_output(Some(1).pack()) // Output 1 is change
        .script_infos(vec![SCRIPT_INFO].pack())
        .build();
    
    Ok(BuildingPacket::new_builder()
        .set(packet)
        .build())
}
```

### Multi-Lock Transaction

```rust
pub fn build_multi_lock_transaction() -> Result<Transaction, Error> {
    let tx = TransactionBuilder::default()
        .inputs(vec![
            // Inputs from different locks
            input_lock_a.clone(),
            input_lock_b.clone(),
        ])
        .outputs(vec![
            output_new_state,
            change_output,
        ])
        .witnesses(vec![
            // First lock uses SighashAll with message
            WitnessLayout::SighashAll {
                seal: vec![],
                message: message.clone(),
            }.as_bytes(),
            // Other locks use SighashAllOnly
            WitnessLayout::SighashAllOnly {
                seal: vec![],
            }.as_bytes(),
        ])
        .build();
    
    Ok(tx)
}
```

## Wallet Integration

### Processing BuildingPacket

```rust
pub async fn process_building_packet(
    packet: BuildingPacket,
) -> Result<Transaction, Error> {
    // Extract and display message
    let message = packet.message();
    display_actions_to_user(&message)?;
    
    // Calculate fees
    let fee = calculate_transaction_fee(&packet)?;
    display_fee_to_user(fee)?;
    
    // Get user confirmation
    if !get_user_confirmation().await? {
        return Err(Error::UserRejected);
    }
    
    // Sign transaction
    let signed_tx = sign_transaction(packet).await?;
    
    Ok(signed_tx)
}
```

### Signature Generation

```rust
pub fn sign_with_cobuild(
    packet: BuildingPacket,
    key: &PrivateKey,
) -> Result<Vec<u8>, Error> {
    // Calculate CoBuild hash
    let tx_hash = packet.payload().calc_tx_hash();
    let message_hash = calculate_message_hash(&packet.message())?;
    
    // Combine hashes according to CoBuild spec
    let signing_hash = combine_hashes(tx_hash, message_hash)?;
    
    // Generate signature
    let signature = key.sign(&signing_hash)?;
    
    Ok(signature.serialize())
}
```

## Best Practices

### Action Design

1. **Atomic Actions**: Each action should represent one logical operation
2. **Clear Parameters**: Use descriptive field names
3. **Validation**: Thoroughly validate action parameters
4. **Compatibility**: Support both CoBuild and legacy formats during transition

### Error Handling

```rust
pub enum CoBuildError {
    InvalidMessage,
    ActionMismatch,
    UnsupportedVersion,
    SignatureMissing,
}

impl From<CoBuildError> for Error {
    fn from(err: CoBuildError) -> Self {
        match err {
            CoBuildError::InvalidMessage => Error::InvalidWitness,
            CoBuildError::ActionMismatch => Error::ValidationFailure,
            // ... other mappings
        }
    }
}
```

### Testing

```rust
#[test]
fn test_cobuild_validation() {
    let action = MyAppAction::Transfer(Transfer {
        from: alice_address(),
        to: bob_address(),
        amount: 1000,
        token_id: token_id(),
    });
    
    let message = build_message(vec![action]);
    let witness = build_witness_sighash_all(vec![], message);
    
    // Test script validation
    assert!(validate_message_from_witness(&witness).is_ok());
}
```

## Migration Strategy

### Supporting Both Formats

```rust
pub fn load_message() -> Result<Message, Error> {
    let witness = load_witness(0, Source::GroupInput)?;
    
    // Try CoBuild format first
    if let Ok(layout) = WitnessLayout::from_slice(&witness) {
        match layout {
            WitnessLayout::SighashAll { message, .. } => {
                return Ok(message);
            }
            _ => {}
        }
    }
    
    // Fall back to legacy format
    if let Ok(witness_args) = WitnessArgs::from_slice(&witness) {
        // Extract message from output_type or other field
        if let Some(data) = witness_args.output_type() {
            if let Ok(message) = Message::from_slice(&data.raw_data()) {
                return Ok(message);
            }
        }
    }
    
    Err(Error::NoMessage)
}
```

### Gradual Migration

1. **Phase 1**: Add CoBuild support while maintaining legacy
2. **Phase 2**: Encourage CoBuild usage with better UX
3. **Phase 3**: Deprecate legacy format
4. **Phase 4**: Remove legacy support

## Common Patterns

### Batch Operations

```rust
let actions = vec![
    Action::new_transfer(from1, to1, amount1),
    Action::new_transfer(from2, to2, amount2),
    Action::new_transfer(from3, to3, amount3),
];

let message = Message::new_builder()
    .actions(actions.pack())
    .build();
```

### Cross-Script Interaction

```rust
let actions = vec![
    // Action for Script A
    Action::new_builder()
        .script_hash(SCRIPT_A_HASH.pack())
        .data(script_a_action.as_slice().pack())
        .build(),
    // Action for Script B
    Action::new_builder()
        .script_hash(SCRIPT_B_HASH.pack())
        .data(script_b_action.as_slice().pack())
        .build(),
];
```

### Fee Adjustment

```rust
pub fn adjust_fee_with_change_output(
    packet: &mut BuildingPacket,
    new_fee: u64,
) -> Result<(), Error> {
    if let Some(change_index) = packet.change_output() {
        let mut tx = packet.payload();
        let mut outputs = tx.outputs().into_iter().collect::<Vec<_>>();
        
        // Adjust change output capacity
        let change_output = outputs[change_index as usize];
        let new_capacity = change_output.capacity().unpack() - new_fee;
        
        outputs[change_index as usize] = change_output
            .as_builder()
            .capacity(new_capacity.pack())
            .build();
        
        // Update transaction
        packet.payload = tx.as_builder()
            .outputs(outputs.pack())
            .build();
    }
    
    Ok(())
}
```