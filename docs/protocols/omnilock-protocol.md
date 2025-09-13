## Description

CKB's universal lock script enabling cross-blockchain interoperability with native support for Bitcoin, Ethereum, Tron, Dogecoin signatures and wallets. Covers authentication methods, administrator compliance modes, anyone-can-pay features, time locks, supply management, witness structures, deployment information, and multi-chain wallet integration patterns.

## Related Resources

- Development Guide: ckb-dev-context://patterns/omnilock-development
- API Examples: ckb-dev-context://api-reference/omnilock-api-examples
- Interoperability Patterns: ckb-dev-context://patterns/omnilock-interoperability

## Overview

Omnilock is CKB's universal lock script designed for maximum interoperability across blockchain ecosystems. It enables users from Bitcoin, Ethereum, EOS, Tron, Dogecoin, and other blockchains to interact with CKB using their native wallets and signature methods, while providing advanced features like regulatory compliance, time locks, and supply management.

## Core Features

### Multi-Blockchain Signature Support

Omnilock natively supports signature verification from multiple blockchain ecosystems:

- **Bitcoin**: P2WPKH, P2SH-P2WPKH, P2PKH address formats
- **Ethereum**: Standard ECDSA signatures with recovery ID
- **Tron**: TronLink wallet compatibility
- **EOS**: EOS signature format
- **Dogecoin**: Dogecoin-style transaction signing
- **CKB**: Native secp256k1 and multisig support

### Advanced Lock Modes

1. **Administrator Mode**: Whitelist/blacklist functionality for regulatory compliance
2. **Anyone-Can-Pay (ACP)**: Allows partial spending with minimum balance requirements
3. **Time Lock**: Built-in time-based restrictions
4. **Supply Mode**: Token supply management capabilities

## Authentication Flags

Omnilock uses a single-byte flag to determine the authentication method:

| Flag | Method | Description |
|------|--------|-------------|
| 0x0 | CKB Native | Standard secp256k1 verification |
| 0x01 | Ethereum | Ethereum-style ECDSA with recovery |
| 0x03 | Tron | TronLink wallet signatures |
| 0x04 | Bitcoin | Bitcoin address formats (P2WPKH, P2SH-P2WPKH, P2PKH) |
| 0x05 | Dogecoin | Dogecoin transaction signatures |
| 0x06 | CKB MultiSig | Native CKB multisignature |
| 0x12 | Ethereum Display | Enhanced Ethereum with wallet display |
| 0xFC | Delegation | Delegate to other lock scripts |
| 0xFD | Exec | Execute other lock scripts |
| 0xFE | Dynamic Linking | Dynamic delegation capabilities |

## Script Structure

### Lock Script Args Format

```
auth (21 bytes) + omni_lock_flags (1 byte) + [optional configs]
```

#### Auth Field (21 bytes)
- **Byte 0**: Authentication flag (see table above)
- **Bytes 1-20**: Identity data (pubkey hash, address hash, etc.)

#### Omni Lock Flags (1 byte)
Bitmask controlling optional features:
- **Bit 0**: Administrator mode enabled
- **Bit 1**: Anyone-can-pay mode enabled  
- **Bit 2**: Time lock enabled
- **Bit 3**: Supply mode enabled

#### Optional Configuration Data
Appended based on enabled flags:
- **Administrator**: 32-byte SMT root hash
- **Time Lock**: 8-byte since value (absolute/relative time)
- **Anyone-Can-Pay**: 8-byte minimum CKB amount + 8-byte minimum UDT amount
- **Supply**: 32-byte info cell type script hash

### Witness Structure

```molecule
table OmniLockWitnessLock {
    signature: BytesOpt,
    omni_identity: IdentityOpt, 
    preimage: BytesOpt,
}

table Identity {
    identity: Auth,
    proofs: SmtProofEntryVec,
}
```

## Signature Verification Methods

### Bitcoin (Flag 0x04)

Supports three Bitcoin address formats:

```rust
// P2WPKH (Native SegWit)
// Identity: RIPEMD160(SHA256(pubkey))
let pubkey_hash = ripemd160(sha256(pubkey));
let signature = sign_bitcoin_message(tx_hash, private_key);

// P2SH-P2WPKH (Wrapped SegWit)  
// Identity: RIPEMD160(SHA256(redeem_script))
let redeem_script = witness_script(pubkey_hash);
let script_hash = ripemd160(sha256(redeem_script));

// P2PKH (Legacy)
// Identity: RIPEMD160(SHA256(pubkey))
let signature = sign_legacy_bitcoin(tx_hash, private_key);
```

### Ethereum (Flag 0x01)

Standard Ethereum ECDSA signature with recovery:

```javascript
// Ethereum signature generation
const messageHash = keccak256(ckbTransactionHash);
const signature = await web3.eth.sign(messageHash, ethereumAddress);

// Recovery and verification
const recoveredPubkey = ecrecover(messageHash, v, r, s);
const recoveredAddress = keccak256(recoveredPubkey.slice(1)).slice(-20);
```

### Tron (Flag 0x03)

Tron-style signature compatible with TronLink:

```javascript
// Tron signature with TronLink
const tronSignature = await tronWeb.trx.sign(messageHash);
const recoveredAddress = tronWeb.address.fromHex(
    tronWeb.utils.crypto.getAddressFromPriKey(privateKey)
);
```

### CKB MultiSig (Flag 0x06)

Native CKB multisignature support:

```rust
// MultiSig configuration
struct MultiSigConfig {
    first_n: u8,      // Require first N signatures
    threshold: u8,    // Total threshold required
    pubkeys_cnt: u8,  // Total number of pubkeys
    pubkeys: Vec<[u8; 33]>, // Compressed pubkeys
}

// Signature format: threshold of N signatures
let multisig_signature = MultiSigSignature {
    signatures: vec![sig1, sig2, sig3], // Up to threshold
    bitmap: 0b101, // Which pubkeys signed (bits 0, 2)
};
```

## Administrator Mode (RCE)

Administrator mode enables regulatory compliance through Regulation Compliance Extension (RCE) cells organized in a Sparse Merkle Tree structure.

### RCE Cell Structure

```molecule
table RCRule {
    identity: IdentityOpt,
    is_black: Uint8,
    message: Bytes,
}

table SmtProofEntry {
    mask: Byte,
    proof: Bytes,
}
```

### Whitelist/Blacklist Operations

```rust
// Check if identity is authorized
fn verify_rce_authorization(
    identity: &Auth,
    smt_root: &H256,
    proofs: &[SmtProofEntry]
) -> Result<bool, Error> {
    let identity_hash = blake2b_256(identity);
    
    // Verify SMT proof
    let proof_result = verify_smt_proof(
        &identity_hash,
        smt_root,
        proofs
    )?;
    
    match proof_result {
        Some(rule) => {
            let rce_rule = RCRule::from_slice(&rule)?;
            Ok(!rce_rule.is_black()) // Allow if not blacklisted
        }
        None => Ok(false) // Deny if not in whitelist
    }
}
```

### RCE Management

```javascript
// Add identity to whitelist
async function addToWhitelist(identity, smtRoot) {
    const rceRule = {
        identity: identity,
        is_black: 0,
        message: "Approved for trading"
    };
    
    return updateSmtTree(smtRoot, identity, rceRule);
}

// Blacklist identity
async function blacklistIdentity(identity, smtRoot) {
    const rceRule = {
        identity: identity,
        is_black: 1,
        message: "Blacklisted due to compliance violation"
    };
    
    return updateSmtTree(smtRoot, identity, rceRule);
}
```

## Anyone-Can-Pay Mode

ACP mode allows partial spending while maintaining minimum balances:

```rust
// ACP Configuration
struct AcpConfig {
    ckb_minimum: u64,  // Minimum CKB to retain
    udt_minimum: u64,  // Minimum UDT tokens to retain
}

// ACP Validation Logic
fn validate_acp_transaction(
    inputs: &[Cell],
    outputs: &[Cell],
    config: &AcpConfig
) -> Result<(), Error> {
    let input_capacity = inputs.iter().map(|c| c.capacity).sum::<u64>();
    let output_capacity = outputs.iter().map(|c| c.capacity).sum::<u64>();
    
    // Must retain minimum CKB
    if output_capacity < config.ckb_minimum {
        return Err(Error::InsufficientMinimumBalance);
    }
    
    // Validate UDT balances if applicable
    validate_udt_minimums(inputs, outputs, config.udt_minimum)?;
    
    Ok(())
}
```

## Time Lock Mode

Built-in time-based restrictions using CKB's since field:

```rust
// Time Lock Types
enum TimeLockType {
    BlockNumber(u64),    // Absolute block number
    Timestamp(u64),      // Absolute timestamp
    BlockSpan(u64),      // Relative block count
    TimeSpan(u64),       // Relative time duration
}

// Time Lock Validation
fn validate_time_lock(
    since_value: u64,
    current_block: &Header,
    input_block: Option<&Header>
) -> Result<(), Error> {
    let since_flag = (since_value >> 56) as u8;
    let since_value = since_value & 0x00FFFFFFFFFFFFFF;
    
    match since_flag {
        0x00 => {
            // Absolute block number
            if current_block.number() < since_value {
                return Err(Error::TimeLockNotMet);
            }
        }
        0x20 => {
            // Absolute timestamp
            if current_block.timestamp() < since_value {
                return Err(Error::TimeLockNotMet);
            }
        }
        0x80 => {
            // Relative block count
            let input_block = input_block.ok_or(Error::MissingInputBlock)?;
            if current_block.number() - input_block.number() < since_value {
                return Err(Error::TimeLockNotMet); 
            }
        }
        0xA0 => {
            // Relative time duration
            let input_block = input_block.ok_or(Error::MissingInputBlock)?;
            if current_block.timestamp() - input_block.timestamp() < since_value {
                return Err(Error::TimeLockNotMet);
            }
        }
        _ => return Err(Error::InvalidTimeLockFlag),
    }
    
    Ok(())
}
```

## Supply Mode

Token supply management for regulatory compliance:

```rust
// Supply Mode Configuration
struct SupplyConfig {
    info_cell_type_hash: H256,  // Points to token info cell
}

// Supply Validation
fn validate_supply_mode(
    transaction: &Transaction,
    config: &SupplyConfig
) -> Result<(), Error> {
    // Find info cell in cell deps
    let info_cell = find_info_cell(
        &transaction.cell_deps(),
        &config.info_cell_type_hash
    )?;
    
    // Parse supply information
    let supply_info = SupplyInfo::from_slice(info_cell.output_data())?;
    
    // Validate current supply against limits
    let current_supply = calculate_current_supply(&transaction)?;
    
    if current_supply > supply_info.max_supply {
        return Err(Error::SupplyLimitExceeded);
    }
    
    // Validate issuer authorization
    validate_issuer_signature(&transaction, &supply_info.issuer_lock_hash)?;
    
    Ok(())
}
```

## Deployment Information

### Mainnet (Mirana)

- **Code Hash**: `0x79f90bb5e892d80dd213439eeab551120eb417678824f453d0c94b0c15dc3c8c`
- **Hash Type**: `type`
- **TX Hash**: `0x71a7ba8fc96349fea0ed3a5c47992e3b4084b031a42264a018e0072e8172e46c`

### Testnet (Pudge)

- **Code Hash**: `0x79f90bb5e892d80dd213439eeab551120eb417678824f453d0c94b0c15dc3c8c`
- **Hash Type**: `type`  
- **TX Hash**: `0x57a62003daeab9d54aa29b944fc3b451213a5ebdf2e232216a3cfed0dde61b38`

## Integration Examples

### Web3 Wallet Integration

```javascript
// Connect Ethereum wallet to CKB via Omnilock
async function connectEthereumWallet() {
    const accounts = await ethereum.request({method: 'eth_requestAccounts'});
    const ethAddress = accounts[0];
    
    // Generate Omnilock script
    const omnilockScript = {
        codeHash: "0x79f90bb5e892d80dd213439eeab551120eb417678824f453d0c94b0c15dc3c8c",
        hashType: "type",
        args: generateOmnilockArgs(ethAddress, ETHEREUM_FLAG)
    };
    
    return {
        address: generateCkbAddress(omnilockScript),
        script: omnilockScript,
        signTransaction: (txHash) => signWithEthereum(txHash, ethAddress)
    };
}

function generateOmnilockArgs(ethAddress, flag) {
    const authBytes = new Uint8Array(21);
    authBytes[0] = flag; // 0x01 for Ethereum
    authBytes.set(hexToBytes(ethAddress.slice(2)), 1); // Remove 0x prefix
    
    const flagsByte = 0x00; // No additional features enabled
    
    return bytesToHex([...authBytes, flagsByte]);
}
```

### Bitcoin Wallet Integration

```javascript
// Connect Bitcoin wallet to CKB via Omnilock
async function connectBitcoinWallet(bitcoinAddress) {
    let authFlag, identityHash;
    
    // Determine address type and extract identity
    if (bitcoinAddress.startsWith('bc1') || bitcoinAddress.startsWith('tb1')) {
        // P2WPKH (Bech32)
        authFlag = 0x04; 
        identityHash = decodeBech32PubkeyHash(bitcoinAddress);
    } else if (bitcoinAddress.startsWith('3') || bitcoinAddress.startsWith('2')) {
        // P2SH-P2WPKH  
        authFlag = 0x04;
        identityHash = decodeBase58ScriptHash(bitcoinAddress);
    } else {
        // P2PKH (Legacy)
        authFlag = 0x04;
        identityHash = decodeBase58PubkeyHash(bitcoinAddress);
    }
    
    const omnilockScript = {
        codeHash: "0x79f90bb5e892d80dd213439eeab551120eb417678824f453d0c94b0c15dc3c8c",
        hashType: "type",
        args: generateBitcoinOmnilockArgs(authFlag, identityHash)
    };
    
    return {
        address: generateCkbAddress(omnilockScript),
        script: omnilockScript,
        signTransaction: (txHash) => signWithBitcoin(txHash, bitcoinAddress)
    };
}
```

## Best Practices

### Security Considerations

1. **Signature Verification**: Always validate signatures according to the specific blockchain's rules
2. **Identity Mapping**: Ensure proper mapping between blockchain addresses and CKB identities  
3. **RCE Updates**: Keep RCE trees updated and validate SMT proofs thoroughly
4. **Time Lock Validation**: Verify time locks against both block numbers and timestamps
5. **Supply Limits**: Enforce token supply limits strictly in supply mode

### Performance Optimization

1. **Batch Verification**: Group signature verifications when possible
2. **SMT Caching**: Cache SMT roots and proofs for frequently accessed identities
3. **Witness Optimization**: Minimize witness data size for transaction efficiency
4. **Script Caching**: Cache compiled omnilock scripts for reuse

### Development Guidelines

1. **Test Coverage**: Test all authentication flags and mode combinations
2. **Error Handling**: Provide clear error messages for signature failures
3. **Compatibility**: Maintain backward compatibility when updating
4. **Documentation**: Document all supported signature formats clearly

Omnilock represents a significant advancement in blockchain interoperability, enabling seamless cross-chain interactions while maintaining security and regulatory compliance. Its modular design allows for easy extension to support additional blockchain ecosystems as they emerge.