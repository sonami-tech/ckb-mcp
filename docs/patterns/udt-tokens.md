## Description

Comprehensive guide to User Defined Tokens on CKB, covering Simple UDT (sUDT) and Extensible UDT (xUDT) standards. Includes cell structures, transfer/minting operations, extension scripts, regulatory compliance patterns, multi-signature tokens, deployment information, and development guidelines for choosing between UDT standards.

## Overview

User Defined Tokens (UDTs) are custom tokens built on the Nervos CKB blockchain. CKB provides multiple UDT standards with different capabilities, from simple token functionality to advanced regulatory compliance and extensibility features.

## UDT Standards Comparison

| Feature | Simple UDT (sUDT) | Extensible UDT (xUDT) |
|---------|-------------------|----------------------|
| **Token Storage** | 16-byte amount | 16-byte amount + extensible data |
| **Governance** | Owner lock only | Owner lock + extension scripts |
| **Compliance** | Basic transfer rules | Advanced compliance & regulations |
| **Upgradeability** | Fixed behavior | Extensible via scripts |
| **Use Cases** | Basic tokens, utility coins | Regulated assets, complex DeFi |

## Simple UDT (sUDT)

Simple UDT provides the minimal functionality needed for custom tokens on CKB.

### Cell Structure

```rust
// sUDT Cell
data: <16-byte amount: uint128>
type: {
    code_hash: simple_udt_script_hash,
    args: <32-byte owner_lock_hash> + <additional_args>
}
lock: <user_defined_lock>
```

### Key Rules

1. **Amount Storage**: First 16 bytes store token amount in little-endian uint128 format
2. **Owner Lock**: First 32 bytes of type args contain owner lock script hash
3. **Unique Type**: Each sUDT has unique type script for identification
4. **Conservation**: Input amounts ≥ output amounts (allows burning)
5. **Governance**: Minting requires owner lock authorization

### Operations

#### Transfer Operation

```rust
// Transfer sUDT tokens between users
Transaction {
    inputs: vec![
        Cell {
            data: amount_in_1,
            type: sudt_type_script,
            lock: sender_lock_1
        },
        Cell {
            data: amount_in_2, 
            type: sudt_type_script,
            lock: sender_lock_2
        }
    ],
    outputs: vec![
        Cell {
            data: amount_out_1,
            type: sudt_type_script,
            lock: recipient_lock_1
        },
        Cell {
            data: amount_out_2,
            type: sudt_type_script,
            lock: recipient_lock_2
        }
    ]
}

// Rule: amount_in_1 + amount_in_2 ≥ amount_out_1 + amount_out_2
```

#### Minting Operation

```rust
// Issue new sUDT tokens (requires owner authorization)
Transaction {
    inputs: vec![
        Cell {
            lock: owner_lock, // Must match owner_lock_hash in sUDT type args
            // ... other fields
        }
    ],
    outputs: vec![
        Cell {
            data: new_token_amount,
            type: sudt_type_script,
            lock: recipient_lock
        }
    ]
}
```

### Implementation Example

```rust
use ckb_std::{
    ckb_constants::Source,
    high_level::{load_cell_data, load_cell_type_hash, load_script},
    syscalls::load_cell,
};

// Validate sUDT transfer rules
pub fn validate_sudt_transfer() -> Result<(), Error> {
    let script = load_script()?;
    let args = script.args().raw_data();
    
    // Extract owner lock hash (first 32 bytes)
    let owner_lock_hash = &args[0..32];
    
    // Check if owner is authorizing (for minting)
    let is_owner_mode = check_owner_mode(owner_lock_hash)?;
    
    if !is_owner_mode {
        // Regular transfer - enforce conservation rule
        let input_amount = sum_input_amounts()?;
        let output_amount = sum_output_amounts()?;
        
        if input_amount < output_amount {
            return Err(Error::InsufficientInputAmount);
        }
    }
    
    Ok(())
}

fn sum_input_amounts() -> Result<u128, Error> {
    let mut total = 0u128;
    let mut i = 0;
    
    loop {
        match load_cell_data(i, Source::GroupInput) {
            Ok(data) => {
                if data.len() >= 16 {
                    let amount = u128::from_le_bytes(
                        data[0..16].try_into().unwrap()
                    );
                    total = total.checked_add(amount)
                        .ok_or(Error::AmountOverflow)?;
                }
                i += 1;
            }
            Err(_) => break,
        }
    }
    
    Ok(total)
}
```

## Extensible UDT (xUDT)

Extensible UDT builds on Simple UDT with advanced features for complex use cases.

### Cell Structure

```rust
// xUDT Cell
data: <16-byte amount> + <xUDT_data>
type: {
    code_hash: xudt_script_hash,
    args: <32-byte owner_lock_hash> + <xudt_args>
}
lock: <user_defined_lock>
```

### xUDT Args Structure

```rust
// xUDT args format
struct XudtArgs {
    flags: u32,           // 4-byte feature flags
    extension_data: Vec<u8> // Variable length extension data
}

// Flags determine extension data format:
// flags & 0x1FFFFFFF == 0x0: No extensions
// flags & 0x1FFFFFFF == 0x1: Raw extension scripts  
// flags & 0x1FFFFFFF == 0x2: Hash of extension scripts (P2SH style)
```

### Extension Scripts

Extension scripts add custom validation logic to xUDT tokens.

```rust
// Extension script interface
extern "C" {
    fn validate(
        is_owner_mode: bool,
        extension_index: usize,
        args: *const u8,
        args_length: usize
    ) -> i32;
}

// Extension script examples:
// - Regulatory compliance (KYC/AML)
// - Transfer limits and time locks
// - Multi-signature requirements
// - Custom business logic
```

### Owner Mode Extensions

xUDT extends owner mode detection:

```rust
// Original sUDT: Owner mode if input uses owner lock
// xUDT additions based on flags:

if flags & 0x20000000 == 0 {
    // Default: Check input lock scripts (sUDT behavior)
}

if flags & 0x40000000 != 0 {
    // Also check output type scripts
}

if flags & 0x80000000 != 0 {
    // Also check input type scripts  
}

// Witness-based owner mode (no cell consumption required)
if witness.owner_script.is_some() {
    // Validate owner script in witness
    // Set owner mode if validation passes
}
```

### Witness Structure

```rust
table XudtWitness {
    owner_script: ScriptOpt,        // For witness-based owner mode
    owner_signature: BytesOpt,      // Owner signature data
    extension_scripts: ScriptVecOpt, // Extension scripts (P2SH mode)
    extension_data: BytesVec,       // Per-extension transaction data
}
```

### Advanced Use Cases

#### Regulatory Compliance Token

```rust
// Regulated token with KYC requirements
Transaction {
    cell_deps: vec![
        xudt_script_cell,
        kyc_extension_script_cell,
        admin_whitelist_cell
    ],
    inputs: vec![
        Cell {
            data: [amount_bytes, user_kyc_data].concat(),
            type: Script {
                code_hash: xudt_code_hash,
                args: [
                    owner_lock_hash,
                    flags_with_extensions,
                    kyc_extension_hash
                ].concat()
            },
            lock: sender_lock
        }
    ],
    outputs: vec![
        Cell {
            data: [amount_bytes, recipient_kyc_data].concat(),
            type: xudt_type_script, // Same type script
            lock: recipient_lock
        }
    ],
    witnesses: vec![
        WitnessArgs {
            input_type: XudtWitness {
                extension_data: vec![kyc_proof_data]
            }
        }
    ]
}
```

#### Multi-Signature Token

```rust
// Token requiring multiple signatures for large transfers
let multisig_extension = Script {
    code_hash: multisig_extension_hash,
    args: [
        threshold: 2u8,
        pubkey_1,
        pubkey_2, 
        pubkey_3,
        min_amount_for_multisig
    ].concat()
};

// Large transfer triggers multisig validation
if transfer_amount > min_amount_for_multisig {
    // Extension validates multiple signatures
    // All signatures must be present in witness
}
```

## Deployment Information

### Simple UDT

**Mainnet (Lina)**
- Code Hash: `0x5e7a36a77e68eecc013dfa2fe6a23f3b6c344b04005808694ae6dd45eea4cfd5`
- Hash Type: `type`
- TX Hash: `0xc7813f6a415144643970c2e88e0bb6ca6a8edc5dd7c1022746f628284a9936d5`

**Testnet (Aggron)**  
- Code Hash: `0xc5e5dcf215925f7ef4dfaf5f4b4f105bc321c02776d6e7d52a1db3fcd9d011a4`
- Hash Type: `type`
- TX Hash: `0xe12877ebd2c3c364dc46c5c992bcfaf4fee33fa13eebdf82c591fc9825aab769`

### Extensible UDT

**Mainnet (Mirana)**
- Code Hash: `0x50bd8d6680b8b9cf98b73f3c08faf8b2a21914311954118ad6609be6e78a1b95`
- Hash Type: `data1`
- TX Hash: `0xc07844ce21b38e4b071dd0e1ee3b0e27afd8d7532491327f39b786343f558ab7`

**Testnet (Pudge)**
- Code Hash: `0x50bd8d6680b8b9cf98b73f3c08faf8b2a21914311954118ad6609be6e78a1b95`
- Hash Type: `data1`
- TX Hash: `0xbf6fb538763efec2a70a6a3dcb7242787087e1030c4e7d86585bc63a9d337f5f`

## Development Guidelines

### Choosing Between sUDT and xUDT

**Use Simple UDT when:**
- Building basic utility tokens
- Need simple transfer and minting functionality
- Minimal complexity requirements
- Lower transaction costs are important

**Use Extensible UDT when:**
- Regulatory compliance is required
- Need complex transfer rules
- Building sophisticated DeFi applications
- Extensibility for future features

### Best Practices

1. **Amount Handling**: Always use little-endian uint128 for amounts
2. **Owner Security**: Secure owner lock scripts carefully
3. **Extension Design**: Keep extension scripts simple and focused
4. **Testing**: Thoroughly test all transfer and minting scenarios
5. **Upgradeability**: Consider using Type ID for upgradeable extensions

### Common Patterns

```rust
// Check if current script is sUDT compatible
fn is_sudt_compatible(type_script: &Script) -> bool {
    type_script.args().len() >= 32 && // Has owner lock hash
    type_script.code_hash() == SUDT_CODE_HASH
}

// Parse sUDT amount from cell data
fn parse_sudt_amount(data: &[u8]) -> Result<u128, Error> {
    if data.len() < 16 {
        return Err(Error::InvalidDataLength);
    }
    
    Ok(u128::from_le_bytes(data[0..16].try_into().unwrap()))
}

// Create sUDT type script
fn create_sudt_type_script(owner_lock_hash: [u8; 32]) -> Script {
    Script::new_builder()
        .code_hash(SUDT_CODE_HASH.pack())
        .hash_type(ScriptHashType::Type.into())
        .args(Bytes::from(owner_lock_hash.to_vec()).pack())
        .build()
}
```

User Defined Tokens provide a powerful foundation for creating custom digital assets on CKB, from simple utility tokens to complex regulated financial instruments. The choice between Simple UDT and Extensible UDT depends on your specific requirements for functionality, compliance, and future extensibility.