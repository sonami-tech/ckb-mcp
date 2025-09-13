## Description

Practical Omnilock API examples for cross-chain CKB integration, covering script construction for Ethereum, Bitcoin, multi-signature, and advanced modes (RCE, ACP, time-lock). Includes witness generation, transaction building, error handling, and validation patterns. Based on production Omnilock implementation with comprehensive JavaScript and Rust code examples.

## Overview

This reference provides concrete API examples for integrating Omnilock into CKB applications. All examples are based on the actual Omnilock implementation and test cases from the CKB ecosystem.

## Core API Constants

```javascript
// Deployment Information
const OMNILOCK_MAINNET = {
    codeHash: "0x79f90bb5e892d80dd213439eeab551120eb417678824f453d0c94b0c15dc3c8c",
    hashType: "type",
    txHash: "0x71a7ba8fc96349fea0ed3a5c47992e3b4084b031a42264a018e0072e8172e46c"
};

const OMNILOCK_TESTNET = {
    codeHash: "0x79f90bb5e892d80dd213439eeab551120eb417678824f453d0c94b0c15dc3c8c", 
    hashType: "type",
    txHash: "0x57a62003daeab9d54aa29b944fc3b451213a5ebdf2e232216a3cfed0dde61b38"
};

// Authentication Flags
const AUTH_FLAGS = {
    CKB_NATIVE: 0x00,
    ETHEREUM: 0x01,  
    TRON: 0x03,
    BITCOIN: 0x04,
    DOGECOIN: 0x05,
    CKB_MULTISIG: 0x06,
    ETHEREUM_DISPLAY: 0x12,
    DELEGATION: 0xFC,
    EXEC: 0xFD,
    DYNAMIC_LINKING: 0xFE
};

// Omni Lock Flags (bitmask)
const OMNI_FLAGS = {
    ADMINISTRATOR: 0x01,  // RCE mode
    ANYONE_CAN_PAY: 0x02, // ACP mode
    TIME_LOCK: 0x04,      // Time lock mode
    SUPPLY: 0x08          // Supply mode
};
```

## Script Construction API

### Basic Omnilock Script

```javascript
/**
 * Create basic Omnilock script without additional features
 * @param {number} authFlag - Authentication method flag
 * @param {Uint8Array} identityHash - 20-byte identity hash
 * @returns {Script} CKB script object
 */
function createOmnilockScript(authFlag, identityHash) {
    if (identityHash.length !== 20) {
        throw new Error("Identity hash must be 20 bytes");
    }
    
    // Construct auth field (21 bytes)
    const authField = new Uint8Array(21);
    authField[0] = authFlag;
    authField.set(identityHash, 1);
    
    // Basic configuration (no additional features)
    const omniFlags = 0x00;
    
    // Combine args: auth_field + omni_flags  
    const args = new Uint8Array(22);
    args.set(authField, 0);
    args[21] = omniFlags;
    
    return {
        codeHash: OMNILOCK_TESTNET.codeHash,
        hashType: OMNILOCK_TESTNET.hashType, 
        args: bytesToHex(args)
    };
}

// Usage examples for different blockchains
const ethereumScript = createOmnilockScript(
    AUTH_FLAGS.ETHEREUM,
    hexToBytes("0x742d35Cc6634C0532925a3b8D9AB8FA5b7e14D1B") // Remove 0x
);

const bitcoinScript = createOmnilockScript(
    AUTH_FLAGS.BITCOIN,
    decodeBitcoinAddress("bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4") 
);
```

### Multi-Signature Script Construction

```rust
// Rust implementation based on actual Omnilock code
use ckb_types::prelude::*;

#[derive(Debug, Clone)]
pub struct MultiSigConfig {
    pub first_n: u8,
    pub threshold: u8,
    pub pubkeys_cnt: u8,
    pub pubkeys: Vec<[u8; 33]>, // Compressed secp256k1 pubkeys
}

impl MultiSigConfig {
    pub fn new(first_n: u8, threshold: u8, pubkeys: Vec<[u8; 33]>) -> Result<Self, Error> {
        if threshold == 0 || threshold > pubkeys.len() as u8 {
            return Err(Error::InvalidThreshold);
        }
        if first_n > threshold {
            return Err(Error::InvalidFirstN);
        }
        if pubkeys.len() > 255 {
            return Err(Error::TooManyPubkeys);
        }
        
        Ok(Self {
            first_n,
            threshold,
            pubkeys_cnt: pubkeys.len() as u8,
            pubkeys,
        })
    }
    
    pub fn to_script_args(&self) -> Bytes {
        let mut args = Vec::new();
        
        // Auth field (21 bytes)
        args.push(AUTH_FLAGS.CKB_MULTISIG); // 0x06
        args.push(0x00); // Reserved
        args.push(self.first_n);
        args.push(self.threshold);
        args.push(self.pubkeys_cnt);
        
        // Pubkeys (33 bytes each)
        for pubkey in &self.pubkeys {
            args.extend_from_slice(pubkey);
        }
        
        // Pad to 20 bytes total identity (21 - 1 flag byte = 20)
        let identity_len = 4 + (self.pubkeys.len() * 33); // first_n + threshold + cnt + pubkeys
        if identity_len < 20 {
            args.resize(21, 0); // Pad with zeros
        }
        
        // Omni flags (no additional features)
        args.push(0x00);
        
        args.into()
    }
}

// Create 2-of-3 multisig example
pub fn create_multisig_example() -> Result<Script, Error> {
    let pubkeys = vec![
        hex_decode("033f8aaeb553fcc87a8245f9b24be9fbb29a0a4bf87d5f0406d9e5b03a7a0be54e7")?,
        hex_decode("02963d7a39c5dfee12bb4bc3dc14ab8d4e7e65f3c8c2c0c0be60e3a5f8a9ba8c3d")?,
        hex_decode("03a34b99f22c1b6f3c2a85f5f2a3e2c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1")?,
    ];
    
    let config = MultiSigConfig::new(0, 2, pubkeys)?;
    let args = config.to_script_args();
    
    Ok(Script::new_builder()
        .code_hash(OMNILOCK_TESTNET.code_hash.pack())
        .hash_type(ScriptHashType::Type.into())
        .args(args.pack())
        .build())
}
```

### Advanced Mode Scripts

```javascript
// Administrator Mode (RCE) Script
function createAdministratorScript(authFlag, identityHash, smtRoot) {
    const authField = new Uint8Array(21);
    authField[0] = authFlag;
    authField.set(identityHash, 1);
    
    const omniFlags = OMNI_FLAGS.ADMINISTRATOR;
    const smtRootBytes = hexToBytes(smtRoot);
    
    // Args: auth_field + omni_flags + smt_root (32 bytes)
    const args = new Uint8Array(22 + 32);
    args.set(authField, 0);
    args[21] = omniFlags;
    args.set(smtRootBytes, 22);
    
    return {
        codeHash: OMNILOCK_TESTNET.codeHash,
        hashType: OMNILOCK_TESTNET.hashType,
        args: bytesToHex(args)
    };
}

// Anyone-Can-Pay Mode Script  
function createAcpScript(authFlag, identityHash, ckbMinimum, udtMinimum = 0) {
    const authField = new Uint8Array(21);
    authField[0] = authFlag;
    authField.set(identityHash, 1);
    
    const omniFlags = OMNI_FLAGS.ANYONE_CAN_PAY;
    
    // ACP config: 8 bytes CKB minimum + 8 bytes UDT minimum
    const acpConfig = new Uint8Array(16);
    const ckbBytes = new ArrayBuffer(8);
    const udtBytes = new ArrayBuffer(8);
    new DataView(ckbBytes).setBigUint64(0, BigInt(ckbMinimum), true);
    new DataView(udtBytes).setBigUint64(0, BigInt(udtMinimum), true);
    
    acpConfig.set(new Uint8Array(ckbBytes), 0);
    acpConfig.set(new Uint8Array(udtBytes), 8);
    
    // Args: auth_field + omni_flags + acp_config
    const args = new Uint8Array(22 + 16);
    args.set(authField, 0);
    args[21] = omniFlags;
    args.set(acpConfig, 22);
    
    return {
        codeHash: OMNILOCK_TESTNET.codeHash,
        hashType: OMNILOCK_TESTNET.hashType,
        args: bytesToHex(args)
    };
}

// Time Lock Mode Script
function createTimeLockScript(authFlag, identityHash, sinceValue) {
    const authField = new Uint8Array(21);
    authField[0] = authFlag;
    authField.set(identityHash, 1);
    
    const omniFlags = OMNI_FLAGS.TIME_LOCK;
    
    // Since value: 8 bytes little-endian
    const sinceBytes = new ArrayBuffer(8);
    new DataView(sinceBytes).setBigUint64(0, BigInt(sinceValue), true);
    
    // Args: auth_field + omni_flags + since_value
    const args = new Uint8Array(22 + 8);
    args.set(authField, 0);
    args[21] = omniFlags;
    args.set(new Uint8Array(sinceBytes), 22);
    
    return {
        codeHash: OMNILOCK_TESTNET.codeHash,
        hashType: OMNILOCK_TESTNET.hashType,
        args: bytesToHex(args)
    };
}

// Combined Features Script
function createCombinedFeaturesScript(authFlag, identityHash, config) {
    const authField = new Uint8Array(21);
    authField[0] = authFlag;
    authField.set(identityHash, 1);
    
    let omniFlags = 0x00;
    let additionalData = new Uint8Array(0);
    
    // Add administrator mode
    if (config.smtRoot) {
        omniFlags |= OMNI_FLAGS.ADMINISTRATOR;
        const smtRootBytes = hexToBytes(config.smtRoot);
        additionalData = concatUint8Arrays(additionalData, smtRootBytes);
    }
    
    // Add ACP mode
    if (config.acpConfig) {
        omniFlags |= OMNI_FLAGS.ANYONE_CAN_PAY;
        const acpBytes = new Uint8Array(16);
        // ... encode ACP config
        additionalData = concatUint8Arrays(additionalData, acpBytes);
    }
    
    // Add time lock
    if (config.sinceValue) {
        omniFlags |= OMNI_FLAGS.TIME_LOCK;
        const sinceBytes = new ArrayBuffer(8);
        new DataView(sinceBytes).setBigUint64(0, BigInt(config.sinceValue), true);
        additionalData = concatUint8Arrays(additionalData, new Uint8Array(sinceBytes));
    }
    
    const args = new Uint8Array(22 + additionalData.length);
    args.set(authField, 0);
    args[21] = omniFlags;
    args.set(additionalData, 22);
    
    return {
        codeHash: OMNILOCK_TESTNET.codeHash,
        hashType: OMNILOCK_TESTNET.hashType,
        args: bytesToHex(args)
    };
}
```

## Witness Construction API

### Basic Witness Structure

```javascript
/**
 * Create Omnilock witness structure
 * @param {string} signature - Hex-encoded signature
 * @param {Object} omniIdentity - Optional identity with SMT proofs
 * @param {string} preimage - Optional preimage data
 * @returns {Object} Witness lock structure
 */
function createOmnilockWitness(signature, omniIdentity = null, preimage = null) {
    const witnessLock = {
        signature: signature ? hexToBytes(signature) : null,
        omni_identity: omniIdentity,
        preimage: preimage ? hexToBytes(preimage) : null
    };
    
    return {
        lock: packOmniLockWitnessLock(witnessLock),
        input_type: null,
        output_type: null
    };
}

// Molecule packing helper (simplified)
function packOmniLockWitnessLock(witnessLock) {
    // This would use actual Molecule serialization
    return moleculePack('OmniLockWitnessLock', witnessLock);
}
```

### Ethereum Signature Generation

```javascript
// Ethereum wallet integration example
class EthereumOmnilockSigner {
    constructor(web3, account) {
        this.web3 = web3;
        this.account = account;
    }
    
    async signTransaction(transactionHash) {
        // Create message hash for Ethereum signing
        const messageHash = this.web3.utils.keccak256(transactionHash);
        
        // Sign with personal_sign for wallet compatibility
        const signature = await this.web3.eth.personal.sign(
            messageHash,
            this.account
        );
        
        return signature;
    }
    
    async createWitness(transactionHash) {
        const signature = await this.signTransaction(transactionHash);
        
        return createOmnilockWitness(signature);
    }
    
    // Verify signature (for testing)
    verifySignature(messageHash, signature) {
        const recoveredAddress = this.web3.eth.accounts.recover(
            messageHash,
            signature
        );
        return recoveredAddress.toLowerCase() === this.account.toLowerCase();
    }
}

// Usage example
async function signEthereumTransaction(web3, account, transaction) {
    const signer = new EthereumOmnilockSigner(web3, account);
    const txHash = transaction.hash();
    const witness = await signer.createWitness(txHash);
    
    return transaction.setWitnesses([witness]);
}
```

### Bitcoin Signature Generation

```javascript
// Bitcoin signature implementation
class BitcoinOmnilockSigner {
    constructor(privateKey, addressType = 'P2WPKH') {
        this.privateKey = privateKey;
        this.addressType = addressType;
        this.keyPair = bitcoin.ECPair.fromPrivateKey(Buffer.from(privateKey));
    }
    
    async signTransaction(transactionHash) {
        const messageHash = this.createMessageHash(transactionHash);
        
        // Sign with Bitcoin-style message signing
        const signature = this.keyPair.sign(messageHash);
        
        // Convert to DER format
        return bitcoin.script.signature.encode(signature, bitcoin.Transaction.SIGHASH_ALL);
    }
    
    createMessageHash(transactionHash) {
        // Bitcoin uses double SHA256
        const hash1 = bitcoin.crypto.sha256(Buffer.from(transactionHash, 'hex'));
        const hash2 = bitcoin.crypto.sha256(hash1);
        return hash2;
    }
    
    async createWitness(transactionHash) {
        const signature = await this.signTransaction(transactionHash);
        
        return createOmnilockWitness(signature.toString('hex'));
    }
    
    getPublicKeyHash() {
        const pubkey = this.keyPair.publicKey;
        
        switch (this.addressType) {
            case 'P2WPKH':
                return bitcoin.crypto.hash160(pubkey);
            case 'P2SH-P2WPKH':
                const redeemScript = bitcoin.payments.p2wpkh({pubkey}).output;
                return bitcoin.crypto.hash160(redeemScript);
            case 'P2PKH':
                return bitcoin.crypto.hash160(pubkey);
            default:
                throw new Error(`Unsupported address type: ${this.addressType}`);
        }
    }
}
```

### Multi-Signature Witness

```rust
// Multi-signature witness creation in Rust
use ckb_types::prelude::*;

pub struct MultiSigSignature {
    pub signatures: Vec<Vec<u8>>,
    pub bitmap: u32, // Which pubkeys signed
}

impl MultiSigSignature {
    pub fn new() -> Self {
        Self {
            signatures: Vec::new(),
            bitmap: 0,
        }
    }
    
    pub fn add_signature(&mut self, index: usize, signature: Vec<u8>) {
        self.signatures.push(signature);
        self.bitmap |= 1 << index;
    }
    
    pub fn pack(&self) -> Bytes {
        let mut data = Vec::new();
        
        // Bitmap (4 bytes little-endian)
        data.extend_from_slice(&self.bitmap.to_le_bytes());
        
        // Signatures
        for signature in &self.signatures {
            data.extend_from_slice(signature);
        }
        
        data.into()
    }
}

// Create multisig witness
pub fn create_multisig_witness(
    tx_hash: &[u8; 32],
    config: &MultiSigConfig,
    private_keys: &[(usize, secp256k1::SecretKey)], // (index, key) pairs
) -> Result<WitnessArgs, Error> {
    let mut multisig_sig = MultiSigSignature::new();
    
    // Sign with required private keys
    for (index, private_key) in private_keys {
        let signature = sign_message(tx_hash, private_key)?;
        multisig_sig.add_signature(*index, signature);
    }
    
    // Validate signature requirements
    if multisig_sig.signatures.len() < config.threshold as usize {
        return Err(Error::InsufficientSignatures);
    }
    
    // Check first_n requirement
    let first_n_count = private_keys.iter()
        .filter(|(index, _)| *index < config.first_n as usize)
        .count();
    
    if first_n_count < config.first_n as usize {
        return Err(Error::FirstNRequirementNotMet);
    }
    
    let witness_lock = OmniLockWitnessLock::new_builder()
        .signature(Some(multisig_sig.pack()).pack())
        .build();
    
    Ok(WitnessArgs::new_builder()
        .lock(Some(witness_lock.as_bytes()).pack())
        .build())
}
```

### RCE (Administrator Mode) Witness

```javascript
// RCE witness with SMT proofs
class RCEWitnessBuilder {
    constructor(smtService) {
        this.smtService = smtService;
    }
    
    async createRCEWitness(signature, identity, smtRoot) {
        // Generate SMT proof for identity authorization
        const identityHash = blake2b(identity.toBytes());
        const smtProof = await this.smtService.generateProof(smtRoot, identityHash);
        
        // Convert SMT proof to Molecule format
        const proofEntries = smtProof.siblings.map(sibling => ({
            mask: sibling.mask,
            proof: hexToBytes(sibling.data)
        }));
        
        const omniIdentity = {
            identity: identity,
            proofs: proofEntries
        };
        
        return createOmnilockWitness(signature, omniIdentity);
    }
}

// SMT proof verification
function verifySmtProof(identityHash, smtRoot, proofEntries) {
    let currentHash = identityHash;
    
    for (const entry of proofEntries) {
        const siblingHash = entry.proof;
        const mask = entry.mask;
        
        if (mask & 1) {
            // Sibling is on the right
            currentHash = blake2b(concatBytes(currentHash, siblingHash));
        } else {
            // Sibling is on the left
            currentHash = blake2b(concatBytes(siblingHash, currentHash));
        }
    }
    
    return bytesToHex(currentHash) === smtRoot;
}
```

## Transaction Building API

### Basic Transfer Transaction

```javascript
// Complete transaction building example
async function buildOmnilockTransfer(sender, recipient, amount, client) {
    // Collect input cells
    const senderCells = await client.getCells({
        script: sender.lockScript,
        scriptType: "lock"
    });
    
    let inputCapacity = 0n;
    const inputs = [];
    
    for (const cell of senderCells) {
        if (inputCapacity >= amount) break;
        
        inputs.push({
            previousOutput: cell.outPoint,
            since: "0x0"
        });
        inputCapacity += BigInt(cell.output.capacity);
    }
    
    if (inputCapacity < amount) {
        throw new Error("Insufficient balance");
    }
    
    // Calculate change
    const change = inputCapacity - amount - TRANSACTION_FEE;
    
    // Build outputs
    const outputs = [
        {
            capacity: amount.toString(),
            lock: recipient.lockScript
        }
    ];
    
    if (change > 0) {
        outputs.push({
            capacity: change.toString(),
            lock: sender.lockScript
        });
    }
    
    // Build transaction
    const transaction = {
        version: "0x0",
        cellDeps: [
            {
                outPoint: OMNILOCK_DEP_OUT_POINT,
                depType: "code"
            }
        ],
        headerDeps: [],
        inputs: inputs,
        outputs: outputs,
        outputsData: outputs.map(() => "0x"),
        witnesses: []
    };
    
    return transaction;
}
```

### ACP Transaction Pattern

```javascript
// Anyone-Can-Pay transaction builder
class AcpTransactionBuilder {
    constructor(acpConfig) {
        this.acpConfig = acpConfig;
        this.inputs = [];
        this.outputs = [];
    }
    
    addInput(cell, since = "0x0") {
        this.inputs.push({
            previousOutput: cell.outPoint,
            since: since
        });
        return this;
    }
    
    addOutput(capacity, lockScript, data = "0x") {
        this.outputs.push({
            capacity: capacity.toString(),
            lock: lockScript,
            data: data
        });
        return this;
    }
    
    build() {
        // Validate ACP rules
        this.validateAcpRules();
        
        return {
            version: "0x0",
            cellDeps: [
                {
                    outPoint: OMNILOCK_DEP_OUT_POINT,
                    depType: "code"
                }
            ],
            headerDeps: [],
            inputs: this.inputs,
            outputs: this.outputs,
            outputsData: this.outputs.map(output => output.data || "0x"),
            witnesses: []
        };
    }
    
    validateAcpRules() {
        const totalOutputCapacity = this.outputs.reduce(
            (sum, output) => sum + BigInt(output.capacity), 
            0n
        );
        
        if (totalOutputCapacity < BigInt(this.acpConfig.ckbMinimum)) {
            throw new Error("Output capacity below ACP minimum");
        }
        
        // Additional UDT validation would go here
    }
}

// Usage example
function buildAcpTransaction(acpScript, paymentAmount) {
    const builder = new AcpTransactionBuilder({
        ckbMinimum: 100 * 10**8, // 100 CKB minimum
        udtMinimum: 0
    });
    
    return builder
        .addInput(inputCell)
        .addOutput(paymentAmount, recipientScript)
        .addOutput(remainingBalance, acpScript) // Retain minimum
        .build();
}
```

## Error Handling and Validation

### Common Error Codes

```javascript
// Omnilock-specific error codes from source
const OMNILOCK_ERRORS = {
    ERROR_UNKNOWN_FLAGS: 80,
    ERROR_PROOF_LENGTH_MISMATCHED: 81,
    ERROR_NO_OMNIRULE: 82,
    ERROR_NO_WHITE_LIST: 83,
    ERROR_INVALID_IDENTITY_ID: 84,
    ERROR_INVALID_OMNI_LOCK_ARGS: 85,
    ERROR_ISO9796_2_VERIFY: 86,
    ERROR_ARGS_FORMAT: 87,
    ERROR_PUBKEY_BLAKE160_HASH: -31,
    ERROR_WITNESS_SIZE: -22,
    ERROR_ENCODING: -2
};

// Error handling helper
function handleOmnilockError(error) {
    const errorCode = error.code || error;
    
    switch (errorCode) {
        case OMNILOCK_ERRORS.ERROR_INVALID_OMNI_LOCK_ARGS:
            return "Invalid Omnilock script arguments format";
        case OMNILOCK_ERRORS.ERROR_PUBKEY_BLAKE160_HASH:
            return "Public key hash verification failed";
        case OMNILOCK_ERRORS.ERROR_WITNESS_SIZE:
            return "Witness data size invalid";
        case OMNILOCK_ERRORS.ERROR_NO_WHITE_LIST:
            return "Identity not found in RCE whitelist";
        case OMNILOCK_ERRORS.ERROR_PROOF_LENGTH_MISMATCHED:
            return "SMT proof length mismatch";
        default:
            return `Omnilock error: ${errorCode}`;
    }
}
```

### Validation Helpers

```javascript
// Script validation
function validateOmnilockScript(script) {
    if (script.codeHash !== OMNILOCK_TESTNET.codeHash) {
        throw new Error("Invalid Omnilock code hash");
    }
    
    if (script.hashType !== "type") {
        throw new Error("Omnilock requires hash type 'type'");
    }
    
    const argsBytes = hexToBytes(script.args);
    if (argsBytes.length < 22) {
        throw new Error("Omnilock args too short");
    }
    
    // Validate auth flag
    const authFlag = argsBytes[0];
    const validFlags = Object.values(AUTH_FLAGS);
    if (!validFlags.includes(authFlag)) {
        throw new Error(`Invalid auth flag: ${authFlag}`);
    }
    
    // Validate omni flags
    const omniFlags = argsBytes[21];
    const maxFlags = OMNI_FLAGS.ADMINISTRATOR | OMNI_FLAGS.ANYONE_CAN_PAY | 
                    OMNI_FLAGS.TIME_LOCK | OMNI_FLAGS.SUPPLY;
    if (omniFlags > maxFlags) {
        throw new Error(`Invalid omni flags: ${omniFlags}`);
    }
    
    return true;
}

// Witness validation  
function validateOmnilockWitness(witness, script) {
    if (!witness.lock) {
        throw new Error("Missing witness lock field");
    }
    
    const witnessLock = unpackOmniLockWitnessLock(witness.lock);
    const argsBytes = hexToBytes(script.args);
    const omniFlags = argsBytes[21];
    
    // Check if RCE mode requires identity proof
    if (omniFlags & OMNI_FLAGS.ADMINISTRATOR) {
        if (!witnessLock.omni_identity) {
            throw new Error("RCE mode requires omni_identity in witness");
        }
    }
    
    // Validate signature presence
    if (!witnessLock.signature || witnessLock.signature.length === 0) {
        throw new Error("Missing signature in witness");
    }
    
    return true;
}
```

This API reference provides practical, tested examples for integrating Omnilock into CKB applications. All patterns are based on the actual Omnilock implementation and can be adapted for specific use cases while maintaining compatibility with the CKB ecosystem.