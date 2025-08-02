# Omnilock: Universal Interoperability Lock

## Description

A comprehensive guide to implementing cross-chain interoperability with Omnilock, CKB's universal lock script that enables native support for Bitcoin, Ethereum, and other blockchain signature schemes. This guide covers authentication methods, multi-chain wallet integration, regulatory compliance features, and advanced modes like time-locks and supply control.

## Overview

Omnilock is a universal lock script designed for cross-chain interoperability. It provides built-in support for transaction signing methods from Bitcoin, Ethereum, EOS, Dogecoin, and other blockchains, enabling users to control CKB assets using their existing wallets and private keys.

## Key Features

- **Multi-Chain Support**: Native verification for Bitcoin, Ethereum, Dogecoin, Tron signatures
- **Extensible Authentication**: Support for custom signature schemes via dynamic linking
- **Regulatory Compliance**: Optional administrator mode for compliance requirements
- **Advanced Modes**: Anyone-can-pay, time-lock, and supply control features
- **Delegation Support**: P2SH-style script delegation and exec-based validation

## Lock Script Structure

```rust
// Omnilock Script
code_hash: omnilock_script_hash
hash_type: type
args: <21-byte auth> + <omnilock_args>
```

### Authentication (21 bytes)

```rust
struct Auth {
    flag: u8,        // Authentication method identifier
    content: [u8; 20] // Authentication-specific data (usually pubkey hash)
}
```

### Authentication Methods

| Flag | Method | Content | Description |
|------|--------|---------|-------------|
| `0x00` | CKB Native | Blake160(secp256k1_pubkey) | Standard CKB signature |
| `0x01` | Ethereum | Blake160(eth_pubkey) | Ethereum-style signing |
| `0x03` | Tron | Blake160(tron_pubkey) | Tron-compatible signatures |
| `0x04` | Bitcoin | BTC address hash | Bitcoin P2WPKH/P2SH-P2WPKH/P2PKH |
| `0x05` | Dogecoin | DOGE address hash | Dogecoin-compatible signatures |
| `0x12` | Ethereum Display | Blake160(eth_pubkey) | Wallet-friendly message display |
| `0x06` | MultiSig | Blake160(multisig_script) | CKB multisig compatibility |
| `0xFC` | Script Delegation | Blake160(lock_script) | P2SH-style delegation |
| `0xFD` | Exec Delegation | Blake160(exec_preimage) | Dynamic execution |
| `0xFE` | Dynamic Linking | Blake160(dl_preimage) | Dynamic library loading |

## Omnilock Args Structure

```rust
struct OmnilockArgs {
    flags: u8,                    // Feature flags
    admin_list_type_id: [u8; 32], // Optional: Administrator mode
    acp_minimum: [u8; 2],         // Optional: Anyone-can-pay minimum
    since: [u8; 8],               // Optional: Time-lock
    supply_type_hash: [u8; 32],   // Optional: Supply control
}
```

### Mode Flags

| Flag | Mode | Description | Args Size |
|------|------|-------------|-----------|
| `0x01` | Administrator | Regulatory compliance mode | 32 bytes |
| `0x02` | Anyone-Can-Pay | Allow partial spending | 2 bytes |
| `0x04` | Time-Lock | Time-based restrictions | 8 bytes |
| `0x08` | Supply | Token supply management | 32 bytes |

## Authentication Examples

### Ethereum Wallet Integration

```typescript
// Using MetaMask to sign CKB transaction
import { ethers } from 'ethers';

async function signWithEthereum(messageHash: string, privateKey: string) {
    const wallet = new ethers.Wallet(privateKey);
    
    // Format message for Ethereum signing
    const message = `CKB transaction: 0x${messageHash}`;
    const signature = await wallet.signMessage(message);
    
    return {
        auth: {
            flag: 0x12, // Ethereum with display
            content: blake160(wallet.publicKey) // 20-byte pubkey hash
        },
        signature: signature
    };
}
```

### Bitcoin Address Support

```rust
// Bitcoin address verification
pub fn verify_bitcoin_signature(
    address_hash: &[u8; 20],
    signature: &[u8],
    message_hash: &[u8; 32]
) -> Result<(), Error> {
    // Bitcoin message format
    let bitcoin_message = format!(
        "CKB (Bitcoin Layer) transaction: 0x{}", 
        hex::encode(message_hash)
    );
    
    // Verify signature against formatted message
    let recovered_pubkey = recover_bitcoin_pubkey(signature, &bitcoin_message)?;
    let recovered_hash = bitcoin_address_hash(&recovered_pubkey)?;
    
    if recovered_hash == *address_hash {
        Ok(())
    } else {
        Err(Error::InvalidSignature)
    }
}

// Supported Bitcoin address types
enum BitcoinAddressType {
    P2WPKH,      // Native SegWit (bech32)
    P2SH_P2WPKH, // Nested SegWit 
    P2PKH,       // Legacy
    // Note: P2TR (Taproot) not supported
}
```

### Dynamic Script Loading

```rust
// Dynamic linking for custom signature schemes
struct DynamicLinkingPreimage {
    code_hash: [u8; 32],    // Dynamic library hash
    hash_type: u8,          // Library hash type
    pubkey_hash: [u8; 20],  // Expected pubkey hash
}

// Interface for dynamic signature verification
extern "C" {
    fn validate_signature(
        prefilled_data: *mut c_void,
        signature_buffer: *const u8,
        signature_size: usize,
        message_buffer: *const u8,
        message_size: usize,
        pubkey_hash: *mut u8,
        pubkey_hash_len: *mut usize
    ) -> i32;
}
```

## Advanced Modes

### Administrator Mode

Enables regulatory compliance and asset recovery.

```rust
// Administrator unlock example
CellDeps: [omnilock_script, admin_list_cell]
Inputs: [
    Cell {
        lock: Script {
            code_hash: omnilock_hash,
            args: [
                [0x00, user_pubkey_hash], // Regular user auth
                [0x01],                   // Admin mode flag
                admin_list_type_id        // Admin list reference
            ].concat()
        }
    }
]
Witnesses: [
    WitnessArgs {
        lock: OmniLockWitnessLock {
            signature: admin_signature,
            omni_identity: Identity {
                identity: [0x00, admin_pubkey_hash],
                proofs: smt_proofs // Sparse Merkle Tree proofs
            }
        }
    }
]
```

### Anyone-Can-Pay Mode

Allows partial spending with minimum requirements.

```rust
// ACP transaction (adding funds without full control)
Transaction {
    inputs: [
        Cell {
            capacity: 200_000_000, // 200 CKB
            lock: omnilock_with_acp,
            // data can contain UDT tokens
        }
    ],
    outputs: [
        Cell {
            capacity: 300_000_000, // 300 CKB (increased)
            lock: omnilock_with_acp,
            // ACP rules ensure minimum amounts maintained
        }
    ]
}

// ACP args: [ckb_minimum: u8, udt_minimum: u8]
// Ensures output has at least minimum CKB and UDT amounts
```

### Time-Lock Mode

Implements time-based spending restrictions.

```rust
// Time-locked cell
let timelock_args = OmnilockArgs {
    flags: 0x04,
    since: timestamp_or_block_number.to_le_bytes(),
    // Other fields...
};

// Can only be spent after specified time/block
```

### Supply Mode

Controls token minting with supply limits.

```rust
// Supply control cell data
struct SupplyInfo {
    version: u8,           // Currently 0
    current_supply: u128,  // Current circulating supply
    max_supply: u128,      // Maximum allowed supply
    sudt_script_hash: [u8; 32], // Token type script hash
}

// Minting validation
let issued_amount = output_amount - input_amount;
let new_supply = current_supply + issued_amount;

if new_supply > max_supply {
    return Err(Error::ExceedsMaxSupply);
}
```

## Witness Structure

```rust
table OmniLockWitnessLock {
    signature: BytesOpt,      // Signature data
    omni_identity: IdentityOpt, // Administrator identity
    preimage: BytesOpt,       // Delegation preimage
}

table Identity {
    identity: Auth,           // Administrator auth
    proofs: SmtProofEntryVec, // SMT compliance proofs
}
```

## Integration Patterns

### Multi-Chain Asset Management

```typescript
// Universal asset controller supporting multiple chains
class UniversalAssetController {
    async unlockWithChain(
        chain: 'ethereum' | 'bitcoin' | 'dogecoin',
        privateKey: string,
        transaction: CKBTransaction
    ) {
        const messageHash = transaction.hash();
        
        switch (chain) {
            case 'ethereum':
                return this.unlockWithEthereum(privateKey, messageHash);
            case 'bitcoin':
                return this.unlockWithBitcoin(privateKey, messageHash);
            case 'dogecoin':
                return this.unlockWithDogecoin(privateKey, messageHash);
        }
    }
    
    private async unlockWithEthereum(
        privateKey: string, 
        messageHash: string
    ) {
        // Ethereum signing with MetaMask compatibility
        const message = `CKB transaction: 0x${messageHash}`;
        const signature = await this.signEthereum(message, privateKey);
        
        return {
            auth: { flag: 0x12, content: this.ethPubkeyHash(privateKey) },
            signature
        };
    }
}
```

### Regulated Token Implementation

```rust
// Combine Omnilock with xUDT for compliant tokens
pub struct RegulatedToken {
    omnilock_with_admin: Script,
    xudt_with_rce: Script,
}

impl RegulatedToken {
    pub fn transfer_with_compliance(
        &self,
        from: Address,
        to: Address,
        amount: u128,
        compliance_proof: ComplianceProof
    ) -> Result<Transaction, Error> {
        // Verify both parties are compliant
        self.verify_kyc_status(&from, &compliance_proof)?;
        self.verify_kyc_status(&to, &compliance_proof)?;
        
        // Build transaction with admin oversight
        let tx = TransactionBuilder::default()
            .input(self.build_input(from, amount)?)
            .output(self.build_output(to, amount)?)
            .witness(self.build_compliance_witness(compliance_proof)?)
            .build();
            
        Ok(tx)
    }
}
```

## Deployment Information

**Mainnet (Mirana)**
- Code Hash: `0x9b819793a64463aed77c615d6cb226eea5487ccfc0783043a587254cda2b6f26`
- Hash Type: `type`
- TX Hash: `0xc76edf469816aa22f416503c38d0b533d2a018e253e379f134c3985b3472c842`
- Index: `0x0`
- Dep Type: `code`

**Testnet (Pudge)**
- Code Hash: `0xf329effd1c475a2978453c8600e1eaf0bc2087ee093c3ee64cc96ec6847752cb`
- Hash Type: `type`
- TX Hash: `0x3d4296df1bd2cc2bd3f483f61ab7ebeac462a2f336f2b944168fe6ba5d81c014`
- Index: `0x0`
- Dep Type: `code`

## Best Practices

### Security Considerations

1. **Signature Verification**: Always validate signatures against formatted messages
2. **Admin Mode**: Carefully manage administrator keys and SMT proofs
3. **Chain-Specific**: Understand signing differences between blockchains
4. **Delegation**: Verify preimages and dynamic libraries thoroughly

### Development Guidelines

```rust
// Safe Omnilock integration
pub fn create_omnilock_script(
    auth_method: AuthMethod,
    pubkey_hash: [u8; 20],
    modes: Vec<OmnilockMode>
) -> Script {
    let mut args = Vec::new();
    
    // Add authentication
    args.push(auth_method as u8);
    args.extend_from_slice(&pubkey_hash);
    
    // Add mode flags and data
    let flags = modes.iter().fold(0u8, |acc, mode| acc | mode.flag());
    args.push(flags);
    
    for mode in modes {
        args.extend_from_slice(&mode.args_data());
    }
    
    Script::new_builder()
        .code_hash(OMNILOCK_CODE_HASH.pack())
        .hash_type(ScriptHashType::Type.into())
        .args(Bytes::from(args).pack())
        .build()
}

// Multi-chain address derivation
pub fn derive_ckb_address(
    chain: SupportedChain,
    external_address: &str
) -> Result<Address, Error> {
    let pubkey_hash = match chain {
        SupportedChain::Ethereum => {
            let eth_addr = parse_ethereum_address(external_address)?;
            blake160(&eth_addr.to_bytes())
        },
        SupportedChain::Bitcoin => {
            let btc_addr = parse_bitcoin_address(external_address)?;
            btc_addr.script_pubkey_hash()
        },
        // Handle other chains...
    };
    
    let omnilock = create_omnilock_script(
        AuthMethod::from_chain(chain),
        pubkey_hash,
        vec![] // No additional modes
    );
    
    Address::from_script(&omnilock)
}
```

### Testing Strategies

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ethereum_signature_verification() {
        let private_key = "0x...";
        let message_hash = "0x...";
        
        let signature = sign_ethereum_message(private_key, message_hash);
        let omnilock = create_ethereum_omnilock(private_key);
        
        assert!(verify_omnilock_signature(&omnilock, &signature, message_hash));
    }
    
    #[test] 
    fn test_bitcoin_address_types() {
        let test_cases = vec![
            ("bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4", AddressType::P2WPKH),
            ("3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy", AddressType::P2SH_P2WPKH),
            ("1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2", AddressType::P2PKH),
        ];
        
        for (address, expected_type) in test_cases {
            let addr_type = detect_bitcoin_address_type(address).unwrap();
            assert_eq!(addr_type, expected_type);
        }
    }
}
```

Omnilock provides a universal solution for cross-chain interoperability on CKB, enabling users to control assets using familiar wallets while providing advanced features for regulatory compliance and sophisticated access control patterns.