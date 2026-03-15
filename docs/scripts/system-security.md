## Description

Security patterns, testing frameworks, and best practices for CKB system scripts. Error handling with comprehensive error types, input/output balance validation, witness format verification, replay attack prevention. Includes testing framework with cycle measurement, signature verification tests, and multisig test patterns.

## Testing System Script Patterns

### Comprehensive Testing Framework

```rust
#[cfg(test)]
mod system_script_tests {
    use super::*;
    use ckb_tool::{
        ckb_error::assert_error_eq,
        ckb_script::ScriptError,
        ckb_types::{
            bytes::Bytes,
            core::{TransactionBuilder, TransactionView},
            packed::*,
            prelude::*,
        },
    };

    const MAX_CYCLES: u64 = 10_000_000;

    #[test]
    fn test_secp256k1_signature_verification() {
        let mut context = Context::default();

        // Deploy secp256k1 script
        let secp256k1_bin = include_bytes!("../../../resources/ckb-system-scripts/build/secp256k1_blake160_sighash_all");
        let secp256k1_out_point = context.deploy_cell(secp256k1_bin.to_vec().into());

        // Generate test key pair
        let private_key = [1u8; 32];
        let public_key = secp256k1_pubkey(&private_key);
        let pubkey_hash = blake160(&public_key);

        // Create lock script
        let lock_script = Script::new_builder()
            .code_hash(secp256k1_out_point.tx_hash())
            .hash_type(ScriptHashType::Data.into())
            .args(Bytes::from(pubkey_hash.to_vec()).pack())
            .build();

        // Create test transaction
        let tx = build_test_transaction(&context, &lock_script, &private_key);

        // Verify transaction succeeds
        let cycles = context.verify_tx(&tx, MAX_CYCLES).expect("secp256k1 verification should succeed");
        println!("secp256k1 verification cycles: {}", cycles);

        // Test invalid signature
        let invalid_tx = build_test_transaction(&context, &lock_script, &[2u8; 32]);
        let err = context.verify_tx(&invalid_tx, MAX_CYCLES).unwrap_err();
        assert_error_eq!(err, ScriptError::ValidationFailure(-2)); // Invalid signature
    }

    #[test]
    fn test_multisig_verification() {
        let mut context = Context::default();

        // Deploy multisig script
        let multisig_bin = include_bytes!("../../../resources/ckb-system-scripts/build/secp256k1_blake160_multisig_all");
        let multisig_out_point = context.deploy_cell(multisig_bin.to_vec().into());

        // Generate 3 key pairs for 2-of-3 multisig
        let private_keys = [
            [1u8; 32],
            [2u8; 32],
            [3u8; 32],
        ];

        let public_keys: Vec<[u8; 33]> = private_keys.iter()
            .map(|key| secp256k1_pubkey(key))
            .collect();

        // Create multisig configuration (2-of-3)
        let multisig_config = create_multisig_config(&public_keys, 2);
        let multisig_hash = blake160(&multisig_config);

        // Create multisig lock script
        let lock_script = Script::new_builder()
            .code_hash(multisig_out_point.tx_hash())
            .hash_type(ScriptHashType::Data.into())
            .args(Bytes::from(multisig_hash.to_vec()).pack())
            .build();

        // Test valid 2-of-3 signature
        let tx = build_multisig_transaction(&context, &lock_script, &private_keys[0..2]);
        let cycles = context.verify_tx(&tx, MAX_CYCLES).expect("2-of-3 multisig should succeed");
        println!("2-of-3 multisig cycles: {}", cycles);

        // Test insufficient signatures (1-of-3)
        let invalid_tx = build_multisig_transaction(&context, &lock_script, &private_keys[0..1]);
        let err = context.verify_tx(&invalid_tx, MAX_CYCLES).unwrap_err();
        assert_error_eq!(err, ScriptError::ValidationFailure(-3)); // Insufficient signatures
    }

    #[test]
    fn test_type_id_generation() {
        let mut context = Context::default();

        // Create first input
        let input_outpoint = OutPoint::new_builder()
            .tx_hash([1u8; 32].pack())
            .index(0u32.pack())
            .build();

        // Calculate expected Type ID
        let expected_type_id = calculate_type_id(&input_outpoint, 0);

        // Create Type ID script
        let type_id_script = Script::new_builder()
            .code_hash([0u8; 32].pack()) // Placeholder
            .hash_type(ScriptHashType::Type.into())
            .args(Bytes::from(expected_type_id.to_vec()).pack())
            .build();

        // Verify Type ID calculation
        assert_eq!(
            calculate_type_id(&input_outpoint, 0),
            expected_type_id
        );
    }

    fn build_test_transaction(
        context: &Context,
        lock_script: &Script,
        private_key: &[u8; 32],
    ) -> TransactionView {
        // Implementation details for building test transactions
        // Include proper witness generation and signing
        unimplemented!()
    }
}
```

**Reference:** `resources/ckb-system-scripts/src/tests/`

## Error Handling and Security Patterns

### Comprehensive Error Types

```rust
#[repr(i8)]
pub enum SystemScriptError {
    // Standard CKB errors
    IndexOutOfBound = 1,
    ItemMissing = 2,
    LengthNotEnough = 3,
    Encoding = 4,

    // Script-specific errors
    InvalidArgs = 5,
    InvalidWitness = 6,
    InvalidTransaction = 7,

    // Signature errors
    InvalidSignature = -1,
    InvalidPubkey = -2,
    InvalidRecoveryId = -3,

    // Multisig errors
    InsufficientSignatures = -11,
    InvalidMultisigConfig = -12,
    DuplicateSignature = -13,

    // Type ID errors
    InvalidTypeId = -21,
    TypeIdMismatch = -22,

    // DAO errors
    InvalidDaoData = -31,
    InvalidEpoch = -32,
    InsufficientLockPeriod = -33,
}

impl From<SysError> for SystemScriptError {
    fn from(err: SysError) -> Self {
        match err {
            SysError::IndexOutOfBound => SystemScriptError::IndexOutOfBound,
            SysError::ItemMissing => SystemScriptError::ItemMissing,
            SysError::LengthNotEnough(_) => SystemScriptError::LengthNotEnough,
            SysError::Encoding => SystemScriptError::Encoding,
            SysError::Unknown(err_code) => {
                debug!("Unknown system error: {}", err_code);
                SystemScriptError::InvalidTransaction
            }
        }
    }
}
```

### Security Best Practices

```rust
// Security validation patterns
pub fn validate_transaction_security() -> Result<(), Error> {
    // 1. Validate input/output consistency
    validate_input_output_balance()?;

    // 2. Check for replay attacks
    validate_transaction_uniqueness()?;

    // 3. Verify witness integrity
    validate_witness_format()?;

    // 4. Check signature timing attacks
    validate_signature_timing()?;

    Ok(())
}

fn validate_input_output_balance() -> Result<(), Error> {
    let mut input_capacity = 0u64;
    let mut output_capacity = 0u64;

    // Sum input capacities
    for (i, _) in QueryIter::new(load_input, Source::Input).enumerate() {
        let cell_output = load_cell_output(i, Source::Input)?;
        input_capacity = input_capacity
            .checked_add(cell_output.capacity().unpack())
            .ok_or(Error::CapacityOverflow)?;
    }

    // Sum output capacities
    for (i, _) in QueryIter::new(load_cell_output, Source::Output).enumerate() {
        let cell_output = load_cell_output(i, Source::Output)?;
        output_capacity = output_capacity
            .checked_add(cell_output.capacity().unpack())
            .ok_or(Error::CapacityOverflow)?;
    }

    // Ensure inputs >= outputs (allowing for fees)
    if input_capacity < output_capacity {
        return Err(Error::InsufficientCapacity);
    }

    Ok(())
}

fn validate_witness_format() -> Result<(), Error> {
    // Ensure witness data is properly formatted
    for (i, _) in QueryIter::new(load_witness_args, Source::Input).enumerate() {
        let witness_args = load_witness_args(i, Source::Input)?;

        // Validate lock witness if present
        if let Some(lock_witness) = witness_args.lock().to_opt() {
            let lock_data = lock_witness.unpack();
            validate_lock_witness_format(&lock_data)?;
        }

        // Validate type witness if present
        if let Some(type_witness) = witness_args.type_().to_opt() {
            let type_data = type_witness.unpack();
            validate_type_witness_format(&type_data)?;
        }
    }

    Ok(())
}

fn validate_lock_witness_format(witness_data: &[u8]) -> Result<(), Error> {
    // Standard signature witness should be 65 bytes
    if witness_data.len() == 65 {
        // Validate signature format
        let recovery_id = witness_data[64];
        if recovery_id > 3 {
            return Err(Error::InvalidSignature);
        }
        return Ok(());
    }

    // For multisig, validate bitmap + signatures format
    if witness_data.len() >= 1 {
        // Complex validation for multisig format
        return validate_multisig_witness_format(witness_data);
    }

    Err(Error::InvalidWitness)
}
```

## Best Practices for Modern Development

### 1. Use Rust for New Development
- System scripts are in C for maximum performance and deterministic execution.
- New smart contracts should use Rust with ckb-std for better safety and developer experience.
- Reference C implementations for understanding core patterns.

### 2. Optimize for Gas Efficiency
- Minimize syscalls by batching operations.
- Use early exit patterns for common cases.
- Cache frequently accessed data.

### 3. Comprehensive Testing
- Test all error conditions and edge cases.
- Use property-based testing for complex logic.
- Validate against system script behavior.

### 4. Security First
- Always validate input parameters.
- Check for integer overflows.
- Implement proper error handling.
- Use constant-time operations for cryptography.

### 5. Follow System Script Patterns
- Use established patterns for signature verification.
- Implement proper Type ID calculation.
- Follow multisig configuration standards.
