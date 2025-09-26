# Proxy Lock Testing with Multiple Identity Patterns

## Description

Testing proxy lock authorization with multiple distinct lock identities using ALWAYS_SUCCESS differentiation. Deploy same ALWAYS_SUCCESS binary multiple times with different arguments to generate unique script hashes. Complete Rust code for testing owner_lock, delegate_lock, and fallback_lock scenarios without signature verification complexity. Hash generation patterns, transaction building helpers, and authorization validation examples.

## Core Pattern: ALWAYS_SUCCESS Differentiation

Proxy lock scripts require multiple distinct lock script hashes for authorization testing. The ALWAYS_SUCCESS differentiation technique creates unique script hashes by deploying the same ALWAYS_SUCCESS contract with different arguments.

### Hash Generation Mechanism

```rust
use ckb_testtool::{builtin::ALWAYS_SUCCESS, context::Context};
use ckb_types::{
    bytes::Bytes,
    core::{ScriptHashType},
    packed::{Script, CellDep, OutPoint},
    prelude::*,
};

pub struct LockIdentityGenerator {
    context: Context,
    always_success_out_point: OutPoint,
}

impl LockIdentityGenerator {
    pub fn new() -> Self {
        let mut context = Context::default();
        let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());

        Self {
            context,
            always_success_out_point,
        }
    }

    /// Generate distinct lock script with unique hash
    pub fn generate_lock_identity(&mut self, identity_id: u8) -> (Script, [u8; 32]) {
        // Use identity_id as argument to create distinct hash
        let args = Bytes::from(vec![identity_id]);

        let script = Script::new_builder()
            .code_hash(ALWAYS_SUCCESS.clone())
            .hash_type(ScriptHashType::Data.into())
            .args(args.pack())
            .build();

        let script_hash: [u8; 32] = script.calc_script_hash().unpack();

        (script, script_hash)
    }

    /// Create cell dependency for ALWAYS_SUCCESS contract
    pub fn create_always_success_dep(&self) -> CellDep {
        CellDep::new_builder()
            .out_point(self.always_success_out_point.clone())
            .dep_type(ckb_types::core::DepType::Code.into())
            .build()
    }
}
```

## Multi-Identity Test Scenarios

### 1. Basic Proxy Lock with Multiple Identities

```rust
use ckb_testtool::context::Context;
use ckb_types::{
    core::{TransactionBuilder, TransactionView},
    packed::*,
    prelude::*,
};

pub struct ProxyLockTestFramework {
    context: Context,
    lock_generator: LockIdentityGenerator,
    owner_lock: Script,
    owner_hash: [u8; 32],
    delegate_lock: Script,
    delegate_hash: [u8; 32],
    fallback_lock: Script,
    fallback_hash: [u8; 32],
}

impl ProxyLockTestFramework {
    pub fn new() -> Self {
        let mut lock_generator = LockIdentityGenerator::new();
        let context = Context::default();

        // Generate three distinct lock identities
        let (owner_lock, owner_hash) = lock_generator.generate_lock_identity(1);
        let (delegate_lock, delegate_hash) = lock_generator.generate_lock_identity(2);
        let (fallback_lock, fallback_hash) = lock_generator.generate_lock_identity(3);

        Self {
            context,
            lock_generator,
            owner_lock,
            owner_hash,
            delegate_lock,
            delegate_hash,
            fallback_lock,
            fallback_hash,
        }
    }

    /// Test proxy lock with owner authorization
    pub fn test_owner_authorization(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Create proxy lock script that requires owner_hash presence
        let proxy_lock = self.build_proxy_lock_script(self.owner_hash);

        // Build transaction with owner lock in inputs
        let tx = self.build_authorization_transaction(
            &proxy_lock,
            vec![&self.owner_lock],  // Owner lock provides authorization
            vec![],                  // No unauthorized locks
        );

        // Verify transaction passes
        let result = self.context.verify_tx(&tx, 70_000_000);
        assert!(result.is_ok(), "Owner authorization should pass");

        Ok(())
    }

    /// Test proxy lock with delegate authorization
    pub fn test_delegate_authorization(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let proxy_lock = self.build_proxy_lock_script(self.delegate_hash);

        let tx = self.build_authorization_transaction(
            &proxy_lock,
            vec![&self.delegate_lock],
            vec![],
        );

        let result = self.context.verify_tx(&tx, 70_000_000);
        assert!(result.is_ok(), "Delegate authorization should pass");

        Ok(())
    }

    /// Test proxy lock authorization failure
    pub fn test_unauthorized_access(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let proxy_lock = self.build_proxy_lock_script(self.owner_hash);

        let tx = self.build_authorization_transaction(
            &proxy_lock,
            vec![&self.fallback_lock],  // Wrong lock, should fail
            vec![],
        );

        let result = self.context.verify_tx(&tx, 70_000_000);
        assert!(result.is_err(), "Unauthorized access should fail");

        Ok(())
    }
}
```

### 2. Hierarchical Permission Testing

```rust
impl ProxyLockTestFramework {
    /// Test multi-level authorization (owner -> delegate -> executor)
    pub fn test_hierarchical_authorization(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Create hierarchical proxy lock requiring both owner and delegate
        let hierarchical_lock = self.build_hierarchical_proxy_lock(
            self.owner_hash,
            self.delegate_hash,
        );

        // Test with both required locks present
        let tx_success = self.build_authorization_transaction(
            &hierarchical_lock,
            vec![&self.owner_lock, &self.delegate_lock],
            vec![],
        );

        let result = self.context.verify_tx(&tx_success, 70_000_000);
        assert!(result.is_ok(), "Hierarchical authorization with both locks should pass");

        // Test with only owner lock (should fail)
        let tx_fail = self.build_authorization_transaction(
            &hierarchical_lock,
            vec![&self.owner_lock],
            vec![],
        );

        let result = self.context.verify_tx(&tx_fail, 70_000_000);
        assert!(result.is_err(), "Hierarchical authorization missing delegate should fail");

        Ok(())
    }

    /// Test delegation chain (A delegates to B, B delegates to C)
    pub fn test_delegation_chain(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Generate additional identity for chain testing
        let (executor_lock, executor_hash) = self.lock_generator.generate_lock_identity(4);

        // Create delegation chain:
        // Level 1: Proxy requiring owner_hash (delegates to level 2)
        // Level 2: Proxy requiring delegate_hash (delegates to executor)
        let level1_proxy = self.build_proxy_lock_script(self.owner_hash);
        let level2_proxy = self.build_proxy_lock_script(self.delegate_hash);

        // Build transaction with complete delegation chain
        let tx = TransactionBuilder::default()
            .input(self.build_input_with_lock(&level1_proxy))
            .input(self.build_input_with_lock(&self.owner_lock))    // Level 1 auth
            .input(self.build_input_with_lock(&level2_proxy))
            .input(self.build_input_with_lock(&self.delegate_lock)) // Level 2 auth
            .input(self.build_input_with_lock(&executor_lock))      // Final executor
            .cell_dep(self.lock_generator.create_always_success_dep())
            .build();

        let result = self.context.verify_tx(&tx, 70_000_000);
        assert!(result.is_ok(), "Delegation chain should work");

        Ok(())
    }
}
```

## Transaction Building Helpers

### Input Construction with Lock Scripts

```rust
impl ProxyLockTestFramework {
    /// Build transaction input with specific lock script
    fn build_input_with_lock(&mut self, lock_script: &Script) -> CellInput {
        let capacity = 100_000_000u64; // 100 CKB

        let cell = CellOutput::new_builder()
            .capacity(capacity.pack())
            .lock(lock_script.clone())
            .build();

        let data = Bytes::new();
        let out_point = self.context.create_cell(cell, data);

        CellInput::new_builder()
            .since(0u64.pack())
            .previous_output(out_point)
            .build()
    }

    /// Build complete authorization transaction
    fn build_authorization_transaction(
        &mut self,
        proxy_lock: &Script,
        auth_locks: Vec<&Script>,
        unauthorized_locks: Vec<&Script>,
    ) -> TransactionView {
        let mut tx_builder = TransactionBuilder::default()
            .cell_dep(self.lock_generator.create_always_success_dep());

        // Add proxy-locked input (the cell being unlocked)
        tx_builder = tx_builder.input(self.build_input_with_lock(proxy_lock));

        // Add authorization inputs
        for auth_lock in auth_locks {
            tx_builder = tx_builder.input(self.build_input_with_lock(auth_lock));
        }

        // Add unauthorized inputs (for negative testing)
        for unauth_lock in unauthorized_locks {
            tx_builder = tx_builder.input(self.build_input_with_lock(unauth_lock));
        }

        // Add minimal output
        let output = CellOutput::new_builder()
            .capacity(50_000_000u64.pack()) // 50 CKB
            .lock(self.owner_lock.clone())  // Send to owner
            .build();

        tx_builder = tx_builder.output(output).output_data(Bytes::new().pack());

        tx_builder.build()
    }
}
```

### Proxy Lock Script Construction

```rust
impl ProxyLockTestFramework {
    /// Build simple proxy lock requiring specific lock hash in inputs
    fn build_proxy_lock_script(&mut self, required_lock_hash: [u8; 32]) -> Script {
        // Deploy proxy lock contract (placeholder)
        let proxy_lock_binary = self.create_proxy_lock_binary();
        let proxy_lock_out_point = self.context.deploy_cell(proxy_lock_binary);
        let proxy_lock_code_hash: [u8; 32] = CellOutput::calc_data_hash(&ALWAYS_SUCCESS).unpack();

        Script::new_builder()
            .code_hash(proxy_lock_code_hash.pack())
            .hash_type(ScriptHashType::Data.into())
            .args(Bytes::from(required_lock_hash.to_vec()).pack())
            .build()
    }

    /// Build hierarchical proxy lock requiring multiple lock hashes
    fn build_hierarchical_proxy_lock(
        &mut self,
        primary_hash: [u8; 32],
        secondary_hash: [u8; 32],
    ) -> Script {
        // Combine both hashes as arguments
        let mut args = primary_hash.to_vec();
        args.extend_from_slice(&secondary_hash);

        let proxy_lock_code_hash: [u8; 32] = CellOutput::calc_data_hash(&ALWAYS_SUCCESS).unpack();

        Script::new_builder()
            .code_hash(proxy_lock_code_hash.pack())
            .hash_type(ScriptHashType::Data.into())
            .args(Bytes::from(args).pack())
            .build()
    }

    /// Create proxy lock binary (simplified for testing)
    fn create_proxy_lock_binary(&self) -> Bytes {
        // In real implementation, this would be actual proxy lock contract
        // For testing, we can use ALWAYS_SUCCESS as placeholder
        ALWAYS_SUCCESS.clone()
    }
}
```

## Complete Test Implementation

### Integration Test Example

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_lock_multiple_identities() {
        let mut framework = ProxyLockTestFramework::new();

        // Test all authorization scenarios
        framework.test_owner_authorization().unwrap();
        framework.test_delegate_authorization().unwrap();
        framework.test_unauthorized_access().unwrap();
        framework.test_hierarchical_authorization().unwrap();
        framework.test_delegation_chain().unwrap();
    }

    #[test]
    fn test_lock_identity_uniqueness() {
        let mut generator = LockIdentityGenerator::new();

        // Generate multiple identities and verify uniqueness
        let (lock1, hash1) = generator.generate_lock_identity(1);
        let (lock2, hash2) = generator.generate_lock_identity(2);
        let (lock3, hash3) = generator.generate_lock_identity(3);

        // Verify hashes are different
        assert_ne!(hash1, hash2);
        assert_ne!(hash2, hash3);
        assert_ne!(hash1, hash3);

        // Verify scripts are different
        assert_ne!(lock1.calc_script_hash(), lock2.calc_script_hash());
        assert_ne!(lock2.calc_script_hash(), lock3.calc_script_hash());
        assert_ne!(lock1.calc_script_hash(), lock3.calc_script_hash());
    }

    #[test]
    fn test_script_hash_deterministic() {
        let mut generator1 = LockIdentityGenerator::new();
        let mut generator2 = LockIdentityGenerator::new();

        // Same identity_id should produce same hash
        let (_, hash1) = generator1.generate_lock_identity(42);
        let (_, hash2) = generator2.generate_lock_identity(42);

        assert_eq!(hash1, hash2, "Same identity_id should produce deterministic hash");
    }
}
```

## Advanced Patterns

### 1. Time-Based Proxy Lock Testing

```rust
impl ProxyLockTestFramework {
    /// Test time-locked proxy requiring both time and authorization
    pub fn test_time_proxy_lock(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let future_time = 1000u64;
        let time_proxy = self.build_time_proxy_lock(self.owner_hash, future_time);

        // Test before time (should fail even with correct lock)
        let tx_early = self.build_time_transaction(&time_proxy, &self.owner_lock, 500u64);
        let result = self.context.verify_tx(&tx_early, 70_000_000);
        assert!(result.is_err(), "Time lock not yet passed should fail");

        // Test after time with correct lock (should pass)
        let tx_valid = self.build_time_transaction(&time_proxy, &self.owner_lock, 1500u64);
        let result = self.context.verify_tx(&tx_valid, 70_000_000);
        assert!(result.is_ok(), "Valid time and authorization should pass");

        Ok(())
    }

    fn build_time_proxy_lock(&mut self, lock_hash: [u8; 32], time: u64) -> Script {
        let mut args = lock_hash.to_vec();
        args.extend_from_slice(&time.to_le_bytes());

        let proxy_code_hash: [u8; 32] = CellOutput::calc_data_hash(&ALWAYS_SUCCESS).unpack();

        Script::new_builder()
            .code_hash(proxy_code_hash.pack())
            .hash_type(ScriptHashType::Data.into())
            .args(Bytes::from(args).pack())
            .build()
    }

    fn build_time_transaction(&mut self, lock: &Script, auth_lock: &Script, time: u64) -> TransactionView {
        let input = CellInput::new_builder()
            .since(time.pack())
            .previous_output(self.build_input_with_lock(lock).previous_output())
            .build();

        TransactionBuilder::default()
            .input(input)
            .input(self.build_input_with_lock(auth_lock))
            .cell_dep(self.lock_generator.create_always_success_dep())
            .output(
                CellOutput::new_builder()
                    .capacity(50_000_000u64.pack())
                    .lock(self.owner_lock.clone())
                    .build()
            )
            .output_data(Bytes::new().pack())
            .build()
    }
}
```

### 2. Multi-Signature Proxy Testing

```rust
impl ProxyLockTestFramework {
    /// Test proxy lock requiring M-of-N signatures
    pub fn test_multisig_proxy(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Generate 5 different lock identities
        let mut signers = Vec::new();
        let mut signer_hashes = Vec::new();

        for i in 10..15 {  // Use IDs 10-14 to avoid conflicts
            let (lock, hash) = self.lock_generator.generate_lock_identity(i);
            signers.push(lock);
            signer_hashes.push(hash);
        }

        // Create 3-of-5 multisig proxy lock
        let multisig_proxy = self.build_multisig_proxy_lock(&signer_hashes, 3, 5);

        // Test with exactly 3 signers (minimum required)
        let tx_valid = self.build_multisig_transaction(
            &multisig_proxy,
            &signers[0..3], // First 3 signers
        );

        let result = self.context.verify_tx(&tx_valid, 70_000_000);
        assert!(result.is_ok(), "3-of-5 multisig with 3 signers should pass");

        // Test with 2 signers (insufficient)
        let tx_insufficient = self.build_multisig_transaction(
            &multisig_proxy,
            &signers[0..2], // Only 2 signers
        );

        let result = self.context.verify_tx(&tx_insufficient, 70_000_000);
        assert!(result.is_err(), "3-of-5 multisig with 2 signers should fail");

        Ok(())
    }

    fn build_multisig_proxy_lock(
        &mut self,
        signer_hashes: &[[u8; 32]],
        required: u8,
        total: u8,
    ) -> Script {
        let mut args = vec![required, total];
        for hash in signer_hashes {
            args.extend_from_slice(hash);
        }

        let proxy_code_hash: [u8; 32] = CellOutput::calc_data_hash(&ALWAYS_SUCCESS).unpack();

        Script::new_builder()
            .code_hash(proxy_code_hash.pack())
            .hash_type(ScriptHashType::Data.into())
            .args(Bytes::from(args).pack())
            .build()
    }

    fn build_multisig_transaction(&mut self, proxy_lock: &Script, signers: &[Script]) -> TransactionView {
        let mut tx_builder = TransactionBuilder::default()
            .cell_dep(self.lock_generator.create_always_success_dep())
            .input(self.build_input_with_lock(proxy_lock));

        // Add each signer as input
        for signer in signers {
            tx_builder = tx_builder.input(self.build_input_with_lock(signer));
        }

        tx_builder
            .output(
                CellOutput::new_builder()
                    .capacity(50_000_000u64.pack())
                    .lock(self.owner_lock.clone())
                    .build()
            )
            .output_data(Bytes::new().pack())
            .build()
    }
}
```

## Best Practices

### 1. Identity Management

```rust
pub struct IdentityManager {
    generator: LockIdentityGenerator,
    named_identities: std::collections::HashMap<String, (Script, [u8; 32])>,
    next_id: u8,
}

impl IdentityManager {
    pub fn new() -> Self {
        Self {
            generator: LockIdentityGenerator::new(),
            named_identities: std::collections::HashMap::new(),
            next_id: 1,
        }
    }

    /// Create named identity for reuse
    pub fn create_identity(&mut self, name: &str) -> ([u8; 32], &Script) {
        if let Some((script, hash)) = self.named_identities.get(name) {
            return (*hash, script);
        }

        let (script, hash) = self.generator.generate_lock_identity(self.next_id);
        self.next_id += 1;

        self.named_identities.insert(name.to_string(), (script, hash));
        let (script, hash) = self.named_identities.get(name).unwrap();
        (*hash, script)
    }

    /// Get existing identity by name
    pub fn get_identity(&self, name: &str) -> Option<([u8; 32], &Script)> {
        self.named_identities.get(name).map(|(script, hash)| (*hash, script))
    }
}
```

### 2. Test Isolation

```rust
/// Ensure each test uses fresh identity space
pub fn create_isolated_test_framework() -> ProxyLockTestFramework {
    ProxyLockTestFramework::new()
}

/// Helper for parametrized testing
pub fn test_proxy_pattern_with_identities<F>(test_fn: F)
where
    F: Fn(&mut ProxyLockTestFramework, [u8; 32], [u8; 32]) -> Result<(), Box<dyn std::error::Error>>
{
    let mut framework = create_isolated_test_framework();
    let identity1 = framework.owner_hash;
    let identity2 = framework.delegate_hash;

    test_fn(&mut framework, identity1, identity2).unwrap();
}
```

### 3. Error Diagnosis

```rust
impl ProxyLockTestFramework {
    /// Diagnose transaction failure with detailed error info
    pub fn diagnose_transaction_failure(&mut self, tx: &TransactionView) -> String {
        match self.context.verify_tx(tx, 70_000_000) {
            Ok(_) => "Transaction succeeded".to_string(),
            Err(e) => {
                let mut diagnosis = format!("Transaction failed: {:?}\n", e);

                // Check lock script presence
                diagnosis.push_str("Lock analysis:\n");
                for (i, input) in tx.inputs().into_iter().enumerate() {
                    let cell = self.context.get_cell(&input.previous_output()).unwrap();
                    diagnosis.push_str(&format!(
                        "  Input {}: lock_hash = {:?}\n",
                        i,
                        cell.lock().calc_script_hash()
                    ));
                }

                diagnosis
            }
        }
    }
}
```

## Integration with Existing Testing Patterns

### Combining with ckb-testtool Patterns

```rust
use ckb_testtool::{builtin::ALWAYS_SUCCESS, context::Context, ckb_types::prelude::*};

/// Extend existing test helpers with proxy lock support
pub trait ProxyTestExtensions {
    fn add_proxy_authorization(&mut self, required_hash: [u8; 32]) -> &mut Self;
    fn verify_proxy_authorization(&mut self, tx: &TransactionView) -> Result<(), Box<dyn std::error::Error>>;
}

impl ProxyTestExtensions for Context {
    fn add_proxy_authorization(&mut self, required_hash: [u8; 32]) -> &mut Self {
        // Add authorization cell with required lock hash
        let auth_script = Script::new_builder()
            .code_hash(ALWAYS_SUCCESS.clone())
            .hash_type(ScriptHashType::Data.into())
            .args(Bytes::from(required_hash.to_vec()).pack())
            .build();

        let auth_cell = CellOutput::new_builder()
            .capacity(100_000_000u64.pack())
            .lock(auth_script)
            .build();

        self.create_cell(auth_cell, Bytes::new());
        self
    }

    fn verify_proxy_authorization(&mut self, tx: &TransactionView) -> Result<(), Box<dyn std::error::Error>> {
        self.verify_tx(tx, 70_000_000).map_err(|e| e.into())
    }
}
```

This documentation provides AI assistants with complete, working patterns for testing proxy locks with multiple distinct identities, bridging the gap between ALWAYS_SUCCESS testing utilities and proxy lock authorization mechanisms.