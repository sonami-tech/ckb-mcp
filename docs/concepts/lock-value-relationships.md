## Description

Cryptographic transformation chain in CKB blockchain development: Private Key → Public Key → Lock Arg → Lock Script → Lock Hash → Address. Practical Rust examples using ckb-sdk-rust, step-by-step demonstrations, and comparison with CCC patterns for developers building CKB applications and smart contracts. Detailed code examples, cryptographic foundations, and modern development patterns.

## Overview

Understanding lock value relationships is fundamental to CKB development. The complete transformation chain that converts a private key into a CKB address, showing how each component is derived and used in the CKB ecosystem.

## Transformation Chain

```
Private Key (32 bytes)
    ↓ secp256k1 elliptic curve
Public Key (33 bytes, compressed)
    ↓ blake2b hash with "ckb-default-hash", truncated to 20 bytes
Lock Arg (20 bytes)
    ↓ combine with code_hash + hash_type
Lock Script (structure)
    ↓ blake2b hash of molecule-serialized script
Lock Hash (32 bytes)
    ↓ bech32m encoding with network prefix
Address (human-readable string)
```

## Rust Implementation Examples

### Prerequisites

Add these dependencies to your `Cargo.toml`:

```toml
[dependencies]
ckb-sdk = "3.0"
ckb-types = "0.118"
secp256k1 = { version = "0.28", features = ["rand", "hashes"] }
blake2b-rs = "0.2"
hex = "0.4"
```

### Step 1: Private Key to Public Key

```rust
use secp256k1::{Secp256k1, SecretKey, PublicKey};
use ckb_types::H256;
use hex;

fn private_to_public_key(private_key_hex: &str) -> Result<PublicKey, Box<dyn std::error::Error>> {
    // Remove 0x prefix if present
    let private_key_hex = private_key_hex.strip_prefix("0x").unwrap_or(private_key_hex);
    
    // Parse private key
    let private_key_bytes = hex::decode(private_key_hex)?;
    let secret_key = SecretKey::from_slice(&private_key_bytes)?;
    
    // Generate public key
    let secp = Secp256k1::new();
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);
    
    Ok(public_key)
}

// Example usage
let private_key = "0xd00c06bfd800d27397002dca6fb0993d5ba6399b4238b2f29ee9deb97593d2bc";
let public_key = private_to_public_key(private_key)?;
let public_key_bytes = public_key.serialize();
println!("Public Key: 0x{}", hex::encode(public_key_bytes));
```

### Step 2: Public Key to Lock Arg

```rust
use blake2b_rs::{Blake2b, Blake2bBuilder};
use ckb_types::packed::Bytes;

fn public_key_to_lock_arg(public_key: &PublicKey) -> Result<[u8; 20], Box<dyn std::error::Error>> {
    let public_key_bytes = public_key.serialize();
    
    // Blake2b hash with CKB's personalization
    let mut hasher = Blake2bBuilder::new(32)
        .personal(b"ckb-default-hash")
        .build();
    
    hasher.update(&public_key_bytes);
    let mut hash = [0u8; 32];
    hasher.finalize(&mut hash);
    
    // Take first 20 bytes (blake160)
    let mut lock_arg = [0u8; 20];
    lock_arg.copy_from_slice(&hash[..20]);
    
    Ok(lock_arg)
}

// Example usage
let lock_arg = public_key_to_lock_arg(&public_key)?;
println!("Lock Arg: 0x{}", hex::encode(lock_arg));
```

### Step 3: Lock Arg to Lock Script

```rust
use ckb_types::{
    packed::{Script, ScriptBuilder},
    prelude::*,
    core::ScriptHashType,
    H256,
};

fn create_secp256k1_lock_script(lock_arg: &[u8; 20]) -> Script {
    // Secp256k1 Blake160 lock script code hash (mainnet)
    let code_hash = H256::from_slice(
        &hex::decode("9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8")
            .expect("Invalid code hash hex")
    ).expect("Invalid code hash");
    
    ScriptBuilder::default()
        .code_hash(code_hash.pack())
        .hash_type(ScriptHashType::Type.into())
        .args(Bytes::from(lock_arg.to_vec()).pack())
        .build()
}

// Example usage
let lock_script = create_secp256k1_lock_script(&lock_arg);
println!("Lock Script:");
println!("  Code Hash: 0x{}", hex::encode(lock_script.code_hash().raw_data()));
println!("  Hash Type: {:?}", lock_script.hash_type());
println!("  Args: 0x{}", hex::encode(lock_script.args().raw_data()));
```

### Step 4: Lock Script to Lock Hash

```rust
use ckb_types::{core::ScriptHashType, packed::Script};

fn script_to_hash(script: &Script) -> H256 {
    script.calc_script_hash()
}

// Example usage
let lock_hash = script_to_hash(&lock_script);
println!("Lock Hash: 0x{}", hex::encode(lock_hash.as_bytes()));
```

### Step 5: Lock Script to Address

```rust
use ckb_sdk::Address;
use ckb_types::core::NetworkType;

fn script_to_address(script: &Script, network: NetworkType) -> Address {
    Address::new(network, script.clone(), true)
}

// Example usage
let address = script_to_address(&lock_script, NetworkType::Testnet);
println!("Address: {}", address);
```

## Complete Example

Here's a complete example that demonstrates the entire transformation chain:

```rust
use secp256k1::{Secp256k1, SecretKey, PublicKey};
use blake2b_rs::Blake2bBuilder;
use ckb_types::{
    packed::{Script, ScriptBuilder, Bytes},
    prelude::*,
    core::{ScriptHashType, NetworkType},
    H256,
};
use ckb_sdk::Address;
use hex;

fn demonstrate_lock_value_relationships(
    private_key_hex: &str,
    network: NetworkType,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("=".repeat(80));
    println!("CKB Lock Value Relationships Demo - Rust Implementation");
    println!("=".repeat(80));
    
    // Step 1: Private Key
    println!("\n1. Private Key: {}", private_key_hex);
    println!("   Length: {} hex chars (32 bytes)", private_key_hex.len() - 2);
    println!("   Purpose: Secret key that is the basis of all derived values.");
    
    // Step 2: Generate Public Key
    let private_key_hex = private_key_hex.strip_prefix("0x").unwrap_or(private_key_hex);
    let private_key_bytes = hex::decode(private_key_hex)?;
    let secret_key = SecretKey::from_slice(&private_key_bytes)?;
    
    let secp = Secp256k1::new();
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);
    let public_key_hex = format!("0x{}", hex::encode(public_key.serialize()));
    
    println!("\n2. Public Key: {}", public_key_hex);
    println!("   Length: {} hex chars (33 bytes)", public_key_hex.len() - 2);
    println!("   Generation: secp256k1 elliptic curve from private key.");
    println!("   Purpose: Public component of cryptographic key pair for digital signatures.");
    
    // Step 3: Generate Lock Arg
    let mut hasher = Blake2bBuilder::new(32)
        .personal(b"ckb-default-hash")
        .build();
    hasher.update(&public_key.serialize());
    let mut hash = [0u8; 32];
    hasher.finalize(&mut hash);
    
    let mut lock_arg = [0u8; 20];
    lock_arg.copy_from_slice(&hash[..20]);
    let lock_arg_hex = format!("0x{}", hex::encode(lock_arg));
    
    println!("\n3. Lock Arg: {}", lock_arg_hex);
    println!("   Length: {} hex chars (20 bytes)", lock_arg_hex.len() - 2);
    println!("   Generation: Blake2b hash of public key, truncated to 20 bytes.");
    println!("   Purpose: Unique identifier derived from public key to specify ownership.");
    
    // Step 4: Create Lock Script
    let code_hash = H256::from_slice(
        &hex::decode("9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8")?
    )?;
    
    let lock_script = ScriptBuilder::default()
        .code_hash(code_hash.pack())
        .hash_type(ScriptHashType::Type.into())
        .args(Bytes::from(lock_arg.to_vec()).pack())
        .build();
    
    println!("\n4. Lock Script:");
    println!("   Code Hash: 0x{}", hex::encode(lock_script.code_hash().raw_data()));
    println!("   Hash Type: {:?}", lock_script.hash_type());
    println!("   Args: 0x{}", hex::encode(lock_script.args().raw_data()));
    println!("   Purpose: Complete specification of lock ownership rules.");
    
    // Step 5: Generate Lock Hash
    let lock_hash = lock_script.calc_script_hash();
    let lock_hash_hex = format!("0x{}", hex::encode(lock_hash.as_bytes()));
    
    println!("\n5. Lock Hash: {}", lock_hash_hex);
    println!("   Length: {} hex chars (32 bytes)", lock_hash_hex.len() - 2);
    println!("   Generation: Blake2b hash of molecule-serialized lock script.");
    println!("   Purpose: Unique fingerprint of the complete lock script.");
    
    // Step 6: Generate Address
    let address = Address::new(network, lock_script.clone(), true);
    let address_str = address.to_string();
    
    println!("\n6. Address: {}", address_str);
    println!("   Length: {} characters", address_str.len());
    println!("   Generation: Bech32m encoding of lock script with network prefix.");
    println!("   Purpose: Human-readable encoding of the lock script for transactions.");
    
    // Summary
    println!("\n{}", "=".repeat(80));
    println!("TRANSFORMATION SUMMARY:");
    println!("{}", "=".repeat(80));
    println!("Private Key (32B) → secp256k1 → Public Key (33B)");
    println!("Public Key (33B) → blake2b (ckb-default-hash) → Lock Arg (20B)");
    println!("Lock Arg (20B) + Code Hash + Hash Type → Lock Script");
    println!("Lock Script → blake2b(molecule_encode) → Lock Hash (32B)");
    println!("Lock Script → bech32m encode → Address");
    
    println!("\n{}", "=".repeat(80));
    println!("Demo completed successfully!");
    println!("{}", "=".repeat(80));
    
    Ok(())
}

// Example usage
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let private_key = "0xd00c06bfd800d27397002dca6fb0993d5ba6399b4238b2f29ee9deb97593d2bc";
    demonstrate_lock_value_relationships(private_key, NetworkType::Testnet)?;
    Ok(())
}
```

## Key Concepts

### Blake2b Personalization

CKB uses Blake2b with a specific personalization parameter:
- **Personalization**: `"ckb-default-hash"`
- **Output size**: 32 bytes (256 bits)
- **Lock arg**: First 20 bytes (160 bits) of the hash

### Secp256k1 Lock Script

The default lock script uses:
- **Code Hash**: `0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8`
- **Hash Type**: `Type` (references type script hash)
- **Args**: 20-byte blake160 hash of public key

### Network Types

CKB addresses use different prefixes:
- **Mainnet**: `ckb1` prefix
- **Testnet**: `ckt1` prefix

## Integration with CCC SDK

For modern applications, consider using CCC SDK which provides higher-level abstractions:

```rust
// This is conceptual - actual CCC Rust bindings may vary
use ccc::Signer;

let signer = Signer::from_private_key(private_key)?;
let address = signer.get_secp256k1_address();
let lock_script = address.script();
```

## Common Use Cases

### 1. Wallet Address Generation
Generate addresses from private keys for wallet applications.

### 2. Transaction Authorization
Use lock scripts to specify transaction authorization rules.

### 3. Multi-signature Wallets
Combine multiple public keys in custom lock script arguments.

### 4. Cross-chain Integration
Derive CKB addresses from other blockchain key pairs.

## Best Practices

1. **Use secure random number generation** for private keys
2. **Validate all inputs** before processing
3. **Use established libraries** for cryptographic operations
4. **Test on testnet first** before deploying to mainnet
5. **Store private keys securely** and never log them

## Related Documentation

- [Cell Model](ckb-dev-context://concepts/cell-model) - Understanding cells and scripts
- [Minimal Lock Script](ckb-dev-context://patterns/minimal-lock-script) - Building custom locks
- [CKB Rust SDK Examples](ckb-dev-context://api-reference/ckb-rust-sdk-practical-examples) - SDK usage patterns
- [Transaction Structure](ckb-dev-context://concepts/transaction-structure) - Transaction composition

## Security Considerations

### Private Key Management
- Never hardcode private keys in production code
- Use secure random number generators
- Implement proper key derivation for HD wallets

### Validation
- Always validate public key format and curve membership
- Verify script hash computations match expected values
- Test address generation with known test vectors

### Network Configuration
- Ensure correct network type for address generation
- Use appropriate code hashes for target network
- Validate against known system script configurations