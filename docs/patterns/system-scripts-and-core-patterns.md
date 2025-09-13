## Description

Comprehensive guide to CKB's core system scripts and fundamental blockchain development patterns. Covers signature verification (secp256k1, multisig), Type ID implementation, advanced lock patterns, gas optimization, security best practices, and testing frameworks, with modern Rust implementations for reference and development.

**While system scripts are implemented in C for performance, modern smart contract development should use Rust** with these patterns as reference implementations.

## Core System Scripts Overview

CKB includes several essential system scripts in the genesis block that provide fundamental blockchain functionality.

**System Scripts:**
- **secp256k1_blake160_sighash_all**: Bitcoin-compatible signature verification
- **secp256k1_blake160_multisig_all**: Multi-signature support with threshold requirements  
- **dao**: Nervos DAO implementation for token staking and rewards
- **type_id**: Unique type script identification system

**Implementation Language:** C (for maximum performance and deterministic execution)
**Modern Development:** Use Rust for new contracts, referencing these C implementations

**Reference:** `resources/ckb-system-scripts/c/`

## 1. Signature Verification Patterns (Rust Implementation)

### Bitcoin-Compatible Signature Script

```rust
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{bytes::Bytes, prelude::*},
    high_level::{load_script, load_script_hash, load_witness_args, QueryIter},
    debug, error,
};
use secp256k1::{PublicKey, Signature, Message, Secp256k1, ecdsa::RecoveryId};

// Modern Rust implementation of secp256k1 signature verification
pub fn main() -> Result<(), Error> {
    // Load script arguments (should contain pubkey hash)
    let script = load_script()?;
    let args: Bytes = script.args().unpack();
    
    if args.len() != 20 {
        return Err(Error::InvalidArgs);
    }
    
    let expected_pubkey_hash: [u8; 20] = args.as_ref().try_into()
        .map_err(|_| Error::InvalidArgs)?;
    
    // Verify signature for all inputs in the same script group
    verify_signature_group(&expected_pubkey_hash)?;
    
    Ok(())
}

fn verify_signature_group(expected_pubkey_hash: &[u8; 20]) -> Result<(), Error> {
    // Load transaction hash for signing
    let tx_hash = load_tx_hash()?;
    let message = Message::from_slice(&tx_hash)
        .map_err(|_| Error::InvalidSignature)?;
    
    // Process each input in the script group
    let script_hash = load_script_hash()?;
    
    for (i, input_type) in QueryIter::new(load_input, Source::Input).enumerate() {
        let input_lock_hash = load_cell_lock_hash(i, Source::Input)?;
        
        // Only process inputs with matching lock script
        if input_lock_hash != script_hash {
            continue;
        }
        
        // Load witness for this input
        let witness_args = load_witness_args(i, Source::Input)?;
        let signature_data = witness_args
            .lock()
            .to_opt()
            .ok_or(Error::InvalidWitness)?
            .unpack();
        
        if signature_data.len() != 65 {
            return Err(Error::InvalidSignature);
        }
        
        // Parse signature components
        let recovery_id = RecoveryId::from_i32(signature_data[64] as i32)
            .map_err(|_| Error::InvalidSignature)?;
        
        let signature = Signature::from_compact(&signature_data[0..64])
            .map_err(|_| Error::InvalidSignature)?;
        
        // Recover public key from signature
        let secp = Secp256k1::new();
        let pubkey = secp.recover_ecdsa(&message, &signature, &recovery_id)
            .map_err(|_| Error::InvalidSignature)?;
        
        // Verify public key hash matches script args
        let pubkey_hash = blake160(&pubkey.serialize());
        if pubkey_hash != *expected_pubkey_hash {
            return Err(Error::InvalidSignature);
        }
    }
    
    Ok(())
}

// Blake160 hash function (Blake2b truncated to 160 bits)
fn blake160(data: &[u8]) -> [u8; 20] {
    use blake2b_ref::{Blake2bBuilder, Blake2b};
    
    let mut hasher = Blake2bBuilder::new(20).build();
    hasher.update(data);
    
    let mut result = [0u8; 20];
    hasher.finalize(&mut result);
    result
}
```

**C Reference:** `resources/ckb-system-scripts/c/secp256k1_blake160_sighash_all.c`

### Multi-Signature Implementation

```rust
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{bytes::Bytes, prelude::*},
    high_level::{load_script, load_witness_args},
};

pub struct MultisigConfig {
    pub threshold: u8,
    pub public_keys: Vec<[u8; 33]>, // Compressed public keys
}

impl MultisigConfig {
    pub fn from_script_args(args: &Bytes) -> Result<Self, Error> {
        if args.len() < 21 {
            return Err(Error::InvalidArgs);
        }
        
        // First 20 bytes: multisig script hash
        // 21st byte: S | R | M (S=0, R=require_first_n, M=threshold)
        let config_byte = args[20];
        let threshold = config_byte & 0x1F; // Last 5 bits
        let require_first_n = (config_byte >> 5) & 0x03; // Bits 5-6
        
        // Remaining bytes: compressed public keys (33 bytes each)
        let pubkey_bytes = &args[21..];
        if pubkey_bytes.len() % 33 != 0 {
            return Err(Error::InvalidArgs);
        }
        
        let mut public_keys = Vec::new();
        for chunk in pubkey_bytes.chunks(33) {
            let pubkey: [u8; 33] = chunk.try_into()
                .map_err(|_| Error::InvalidArgs)?;
            public_keys.push(pubkey);
        }
        
        Ok(MultisigConfig {
            threshold,
            public_keys,
        })
    }
}

pub fn verify_multisig() -> Result<(), Error> {
    let script = load_script()?;
    let args: Bytes = script.args().unpack();
    
    let config = MultisigConfig::from_script_args(&args)?;
    
    // Load transaction hash
    let tx_hash = load_tx_hash()?;
    let message = Message::from_slice(&tx_hash)
        .map_err(|_| Error::InvalidSignature)?;
    
    // Load multisig witness
    let witness_args = load_witness_args(0, Source::GroupInput)?;
    let witness_data = witness_args
        .lock()
        .to_opt()
        .ok_or(Error::InvalidWitness)?
        .unpack();
    
    // Parse signatures from witness
    let signatures = parse_multisig_witness(&witness_data, &config)?;
    
    // Verify threshold met
    if signatures.len() < config.threshold as usize {
        return Err(Error::InsufficientSignatures);
    }
    
    // Verify each signature
    let secp = Secp256k1::new();
    let mut valid_signatures = 0;
    
    for (pubkey_index, signature) in signatures {
        if pubkey_index >= config.public_keys.len() {
            return Err(Error::InvalidSignature);
        }
        
        let pubkey = PublicKey::from_slice(&config.public_keys[pubkey_index])
            .map_err(|_| Error::InvalidSignature)?;
        
        if secp.verify_ecdsa(&message, &signature, &pubkey).is_ok() {
            valid_signatures += 1;
        }
    }
    
    if valid_signatures >= config.threshold as usize {
        Ok(())
    } else {
        Err(Error::InsufficientSignatures)
    }
}

fn parse_multisig_witness(
    witness_data: &[u8],
    config: &MultisigConfig,
) -> Result<Vec<(usize, Signature)>, Error> {
    // Multisig witness format:
    // - First bytes: signature bitmap (indicates which pubkeys signed)
    // - Following: actual signatures in order
    
    let bitmap_size = (config.public_keys.len() + 7) / 8; // Round up to byte boundary
    if witness_data.len() < bitmap_size {
        return Err(Error::InvalidWitness);
    }
    
    let bitmap = &witness_data[0..bitmap_size];
    let signature_data = &witness_data[bitmap_size..];
    
    let mut signatures = Vec::new();
    let mut signature_offset = 0;
    
    for (i, pubkey) in config.public_keys.iter().enumerate() {
        let byte_index = i / 8;
        let bit_index = i % 8;
        
        // Check if this pubkey signed (bit set in bitmap)
        if bitmap[byte_index] & (1 << bit_index) != 0 {
            if signature_offset + 64 > signature_data.len() {
                return Err(Error::InvalidWitness);
            }
            
            let signature = Signature::from_compact(&signature_data[signature_offset..signature_offset + 64])
                .map_err(|_| Error::InvalidSignature)?;
            
            signatures.push((i, signature));
            signature_offset += 64;
        }
    }
    
    Ok(signatures)
}
```

**C Reference:** `resources/ckb-system-scripts/c/secp256k1_blake160_multisig_all.c`

## 2. Type ID Pattern Implementation

The Type ID pattern provides unique identification for type scripts across the blockchain.

```rust
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{bytes::Bytes, prelude::*},
    high_level::{load_input, load_script, QueryIter},
};

pub fn main() -> Result<(), Error> {
    let script = load_script()?;
    let args: Bytes = script.args().unpack();
    
    if args.len() != 32 {
        return Err(Error::InvalidArgs);
    }
    
    let expected_type_id: [u8; 32] = args.as_ref().try_into()
        .map_err(|_| Error::InvalidArgs)?;
    
    // Verify Type ID is correctly calculated
    verify_type_id(&expected_type_id)?;
    
    Ok(())
}

fn verify_type_id(expected_type_id: &[u8; 32]) -> Result<(), Error> {
    // Type ID calculation:
    // 1. Find the first input in the transaction
    // 2. Calculate: blake2b(first_input_outpoint | first_output_index_with_this_type)
    
    let first_input = load_input(0, Source::Input)?;
    let first_outpoint = first_input.previous_output();
    
    // Find first output with this type script
    let script_hash = load_script_hash()?;
    let mut first_output_index: Option<u64> = None;
    
    for (i, output) in QueryIter::new(load_cell_output, Source::Output).enumerate() {
        if let Some(type_script) = output.type_().to_opt() {
            let type_hash = type_script.calc_script_hash();
            if type_hash == script_hash {
                first_output_index = Some(i as u64);
                break;
            }
        }
    }
    
    let output_index = first_output_index.ok_or(Error::InvalidTransaction)?;
    
    // Calculate Type ID
    let calculated_type_id = calculate_type_id(&first_outpoint, output_index);
    
    if calculated_type_id != *expected_type_id {
        return Err(Error::InvalidTypeId);
    }
    
    Ok(())
}

fn calculate_type_id(outpoint: &OutPoint, output_index: u64) -> [u8; 32] {
    use blake2b_ref::{Blake2bBuilder, Blake2b};
    
    let mut hasher = Blake2bBuilder::new(32).build();
    
    // Hash outpoint (tx_hash + index)
    hasher.update(outpoint.tx_hash().as_slice());
    hasher.update(&outpoint.index().unpack().to_le_bytes());
    
    // Hash output index
    hasher.update(&output_index.to_le_bytes());
    
    let mut result = [0u8; 32];
    hasher.finalize(&mut result);
    result
}
```

**Reference Pattern:** Used throughout CKB ecosystem for unique type identification

## 3. Advanced System Script Patterns

### Lock Script Template with Upgradability

```rust
// Modern lock script pattern with upgrade support
pub struct UpgradeableLockScript {
    pub code_hash: [u8; 32],
    pub hash_type: ScriptHashType,
    pub args: Bytes,
    pub version: u8,
}

impl UpgradeableLockScript {
    pub fn verify(&self) -> Result<(), Error> {
        match self.version {
            1 => self.verify_v1(),
            2 => self.verify_v2(),
            _ => Err(Error::UnsupportedVersion),
        }
    }
    
    fn verify_v1(&self) -> Result<(), Error> {
        // Legacy verification logic
        verify_secp256k1_signature(&self.args)
    }
    
    fn verify_v2(&self) -> Result<(), Error> {
        // Enhanced verification with additional features
        let config = parse_v2_config(&self.args)?;
        
        if config.multi_sig_enabled {
            verify_multisig(&config.multisig_config)
        } else {
            verify_secp256k1_signature(&config.single_sig_config)
        }
    }
}

// Configuration parsing for v2 locks
struct LockConfigV2 {
    pub multi_sig_enabled: bool,
    pub single_sig_config: SingleSigConfig,
    pub multisig_config: MultisigConfig,
}

fn parse_v2_config(args: &Bytes) -> Result<LockConfigV2, Error> {
    if args.len() < 21 {
        return Err(Error::InvalidArgs);
    }
    
    let flags = args[20];
    let multi_sig_enabled = flags & 0x01 != 0;
    
    if multi_sig_enabled {
        let multisig_config = MultisigConfig::from_script_args(args)?;
        Ok(LockConfigV2 {
            multi_sig_enabled: true,
            single_sig_config: SingleSigConfig::default(),
            multisig_config,
        })
    } else {
        let pubkey_hash: [u8; 20] = args[0..20].try_into()
            .map_err(|_| Error::InvalidArgs)?;
        
        Ok(LockConfigV2 {
            multi_sig_enabled: false,
            single_sig_config: SingleSigConfig { pubkey_hash },
            multisig_config: MultisigConfig::default(),
        })
    }
}
```

### Gas-Optimized Verification Pattern

```rust
// Optimized verification pattern for minimal cycle usage
pub fn optimized_main() -> Result<(), Error> {
    // Early exit checks
    let script = load_script()?;
    let args: Bytes = script.args().unpack();
    
    // Fast path: single signature verification
    if args.len() == 20 {
        return verify_single_signature_fast(&args);
    }
    
    // Complex path: multisig or advanced features
    verify_complex_logic(&args)
}

fn verify_single_signature_fast(pubkey_hash: &Bytes) -> Result<(), Error> {
    // Optimized single signature verification
    // - Minimize syscalls
    // - Cache commonly used data
    // - Early validation of witness format
    
    let witness_args = load_witness_args(0, Source::GroupInput)?;
    let signature_data = witness_args
        .lock()
        .to_opt()
        .ok_or(Error::InvalidWitness)?
        .unpack();
    
    // Quick format validation
    if signature_data.len() != 65 {
        return Err(Error::InvalidSignature);
    }
    
    // Batch load transaction data
    let tx_hash = load_tx_hash()?;
    
    // Single verification call
    verify_ecdsa_signature(&tx_hash, &signature_data, pubkey_hash.as_ref())
}

fn verify_ecdsa_signature(
    message_hash: &[u8; 32],
    signature_data: &[u8],
    expected_pubkey_hash: &[u8],
) -> Result<(), Error> {
    use secp256k1::{PublicKey, Signature, Message, Secp256k1, ecdsa::RecoveryId};
    
    let message = Message::from_slice(message_hash)
        .map_err(|_| Error::InvalidSignature)?;
    
    let recovery_id = RecoveryId::from_i32(signature_data[64] as i32)
        .map_err(|_| Error::InvalidSignature)?;
    
    let signature = Signature::from_compact(&signature_data[0..64])
        .map_err(|_| Error::InvalidSignature)?;
    
    let secp = Secp256k1::new();
    let pubkey = secp.recover_ecdsa(&message, &signature, &recovery_id)
        .map_err(|_| Error::InvalidSignature)?;
    
    let pubkey_hash = blake160(&pubkey.serialize());
    
    if pubkey_hash == expected_pubkey_hash {
        Ok(())
    } else {
        Err(Error::InvalidSignature)
    }
}
```

## 4. Testing System Script Patterns

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

## 5. Error Handling and Security Patterns

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

### 1. **Use Rust for New Development**
- System scripts are in C for maximum performance and deterministic execution
- New smart contracts should use Rust with ckb-std for better safety and developer experience
- Reference C implementations for understanding core patterns

### 2. **Optimize for Gas Efficiency**
- Minimize syscalls by batching operations
- Use early exit patterns for common cases
- Cache frequently accessed data

### 3. **Comprehensive Testing**
- Test all error conditions and edge cases
- Use property-based testing for complex logic
- Validate against system script behavior

### 4. **Security First**
- Always validate input parameters
- Check for integer overflows
- Implement proper error handling
- Use constant-time operations for cryptography

### 5. **Follow System Script Patterns**
- Use established patterns for signature verification
- Implement proper Type ID calculation
- Follow multisig configuration standards

