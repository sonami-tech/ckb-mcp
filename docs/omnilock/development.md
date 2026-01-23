## Description

Integrate Omnilock universal lock script for cross-chain wallet compatibility on CKB. Learn Ethereum, Bitcoin, and multi-signature implementations, advanced modes including administrator RCE, anyone-can-pay, and time locks. Covers testing patterns, production deployment guidelines, security considerations, and monitoring strategies for robust cross-chain applications.


## Basic Integration Patterns

### Simple Transfer with Ethereum Wallet

```javascript
import { ccc } from "@ckb-ccc/core";

// Create Omnilock script for Ethereum address
function createEthereumOmnilockScript(ethereumAddress) {
    // Auth field: flag (1 byte) + pubkey hash (20 bytes)
    const authBytes = new Uint8Array(21);
    authBytes[0] = 0x01; // Ethereum flag
    authBytes.set(
        ccc.hexToBytes(ethereumAddress.slice(2)), // Remove 0x prefix
        1
    );
    
    // Omni flags: no additional features
    const omniFlagsByte = 0x00;
    
    const args = ccc.bytesToHex([...authBytes, omniFlagsByte]);
    
    return {
        codeHash: "0x79f90bb5e892d80dd213439eeab551120eb417678824f453d0c94b0c15dc3c8c",
        hashType: "type",
        args: args
    };
}

// Sign transaction with Ethereum wallet
async function signOmnilockTransaction(transaction, ethereumAddress) {
    // Get transaction hash for signing
    const txHash = transaction.hash(); 
    
    // Create message hash (Ethereum style)
    const messageHash = ccc.keccak256(ccc.hexToBytes(txHash));
    
    // Sign with MetaMask or other Ethereum wallet
    const signature = await ethereum.request({
        method: 'personal_sign',
        params: [ccc.bytesToHex(messageHash), ethereumAddress]
    });
    
    // Create witness structure  
    const witnessLock = {
        signature: signature,
        omni_identity: null,
        preimage: null
    };
    
    return ccc.WitnessArgs.pack({
        lock: ccc.OmniLockWitnessLock.pack(witnessLock)
    });
}

// Complete transfer example
async function transferWithEthereumWallet(from, to, amount) {
    const client = new ccc.ClientPublicTestnet();
    
    // Create scripts
    const fromScript = createEthereumOmnilockScript(from.address);
    const toScript = createEthereumOmnilockScript(to.address);
    
    // Build transaction
    const txBuilder = ccc.Transaction.builder()
        .addInput({
            previousOutput: from.outPoint,
            since: "0x0"
        })
        .addOutput({
            lock: toScript,
            capacity: amount
        })
        .addCellDep({
            outPoint: OMNILOCK_DEP_OUT_POINT,
            depType: "code"
        });
        
    const tx = txBuilder.build();
    
    // Sign transaction
    const witness = await signOmnilockTransaction(tx, from.address);
    const signedTx = tx.setWitnesses([witness]);
    
    // Submit transaction
    return await client.sendTransaction(signedTx);
}
```

### Bitcoin Wallet Integration

```javascript
// Support different Bitcoin address formats
function createBitcoinOmnilockScript(bitcoinAddress) {
    let identityHash;
    
    if (bitcoinAddress.startsWith('bc1') || bitcoinAddress.startsWith('tb1')) {
        // P2WPKH (Native SegWit) - Bech32
        identityHash = decodeBech32Address(bitcoinAddress);
    } else if (bitcoinAddress.startsWith('3') || bitcoinAddress.startsWith('2')) {
        // P2SH-P2WPKH (Wrapped SegWit)
        identityHash = decodeBase58ScriptHash(bitcoinAddress);
    } else if (bitcoinAddress.startsWith('1')) {
        // P2PKH (Legacy)
        identityHash = decodeBase58PubkeyHash(bitcoinAddress);
    } else {
        throw new Error(`Unsupported Bitcoin address format: ${bitcoinAddress}`);
    }
    
    // Auth field construction
    const authBytes = new Uint8Array(21);
    authBytes[0] = 0x04; // Bitcoin flag
    authBytes.set(identityHash, 1);
    
    const omniFlagsByte = 0x00;
    const args = ccc.bytesToHex([...authBytes, omniFlagsByte]);
    
    return {
        codeHash: "0x79f90bb5e892d80dd213439eeab551120eb417678824f453d0c94b0c15dc3c8c",
        hashType: "type",
        args: args
    };
}

// Bitcoin signature creation
async function signBitcoinOmnilockTransaction(transaction, bitcoinWallet) {
    const txHash = transaction.hash();
    
    // Bitcoin uses double SHA256 for message hash
    const messageHash = sha256(sha256(ccc.hexToBytes(txHash)));
    
    // Sign with Bitcoin wallet (implementation depends on wallet)
    const signature = await bitcoinWallet.signMessage(messageHash);
    
    const witnessLock = {
        signature: signature,
        omni_identity: null,
        preimage: null
    };
    
    return ccc.WitnessArgs.pack({
        lock: ccc.OmniLockWitnessLock.pack(witnessLock)
    });
}
```

### Multi-Signature Pattern

```rust
// Rust implementation for CKB MultiSig with Omnilock
use ckb_types::prelude::*;

pub struct MultiSigConfig {
    pub first_n: u8,
    pub threshold: u8, 
    pub pubkeys: Vec<[u8; 33]>,
}

impl MultiSigConfig {
    pub fn new(first_n: u8, threshold: u8, pubkeys: Vec<[u8; 33]>) -> Self {
        assert!(threshold <= pubkeys.len() as u8);
        assert!(first_n <= threshold);
        
        Self {
            first_n,
            threshold,
            pubkeys,
        }
    }
    
    pub fn to_omnilock_args(&self) -> Bytes {
        let mut args = Vec::new();
        
        // Auth field
        args.push(0x06); // MultiSig flag
        
        // MultiSig data structure
        args.push(0x00); // Reserved
        args.push(self.first_n);
        args.push(self.threshold);
        args.push(self.pubkeys.len() as u8);
        
        // Add all pubkeys
        for pubkey in &self.pubkeys {
            args.extend_from_slice(pubkey);
        }
        
        // Omni flags
        args.push(0x00); // No additional features
        
        args.into()
    }
}

// Sign multisig transaction
pub fn create_multisig_signature(
    tx_hash: &[u8; 32],
    config: &MultiSigConfig,
    signatures: Vec<(usize, [u8; 65])>, // (pubkey_index, signature)
) -> Result<Bytes, Error> {
    if signatures.len() < config.threshold as usize {
        return Err(Error::InsufficientSignatures);
    }
    
    // Validate first_n requirement
    let first_n_signed = signatures.iter()
        .filter(|(index, _)| *index < config.first_n as usize)
        .count();
    
    if first_n_signed < config.first_n as usize {
        return Err(Error::FirstNRequirementNotMet);
    }
    
    // Create signature bitmap
    let mut bitmap = 0u32;
    let mut sig_data = Vec::new();
    
    for (index, signature) in signatures {
        bitmap |= 1 << index;
        sig_data.extend_from_slice(&signature);
    }
    
    // Pack multisig signature
    let mut result = Vec::new();
    result.extend_from_slice(&bitmap.to_le_bytes());
    result.extend_from_slice(&sig_data);
    
    Ok(result.into())
}
```

## Advanced Mode Patterns

### Administrator Mode with RCE

```javascript
// Administrator mode with whitelist/blacklist
class OmnilockRCEManager {
    constructor(smtRoot, rpcClient) {
        this.smtRoot = smtRoot;
        this.rpcClient = rpcClient;
    }
    
    // Create administrator mode Omnilock script
    createAdminOmnilockScript(identity, smtRoot) {
        const authBytes = new Uint8Array(21);
        authBytes[0] = identity.flag;
        authBytes.set(identity.hash, 1);
        
        const omniFlags = 0x01; // Administrator mode enabled
        
        // Args: auth + flags + SMT root
        const args = ccc.bytesToHex([
            ...authBytes,
            omniFlags,
            ...ccc.hexToBytes(smtRoot)
        ]);
        
        return {
            codeHash: "0x79f90bb5e892d80dd213439eeab551120eb417678824f453d0c94b0c15dc3c8c",
            hashType: "type",
            args: args
        };
    }
    
    // Generate SMT proof for identity authorization
    async generateAuthorizationProof(identity) {
        const identityHash = ccc.blake2b(identity.toBytes());
        
        // Query SMT proof from RCE service
        const proof = await this.rpcClient.getSmtProof(
            this.smtRoot,
            identityHash
        );
        
        return {
            identity: identity,
            proofs: proof.siblings.map(sibling => ({
                mask: sibling.mask,
                proof: sibling.data
            }))
        };
    }
    
    // Sign transaction with RCE proof
    async signWithRCEProof(transaction, identity, privateKey) {
        const txHash = transaction.hash();
        const signature = await this.signMessage(txHash, privateKey);
        
        // Generate authorization proof
        const authProof = await this.generateAuthorizationProof(identity);
        
        const witnessLock = {
            signature: signature,
            omni_identity: authProof,
            preimage: null
        };
        
        return ccc.WitnessArgs.pack({
            lock: ccc.OmniLockWitnessLock.pack(witnessLock)
        });
    }
}

// Usage example
async function transferWithRCE(sender, recipient, amount) {
    const rceManager = new OmnilockRCEManager(SMT_ROOT, client);
    
    // Create scripts with RCE enabled
    const senderScript = rceManager.createAdminOmnilockScript(
        sender.identity, 
        SMT_ROOT
    );
    
    // Build and sign transaction
    const tx = buildTransferTransaction(senderScript, recipient, amount);
    const witness = await rceManager.signWithRCEProof(
        tx, 
        sender.identity, 
        sender.privateKey
    );
    
    return await client.sendTransaction(tx.setWitnesses([witness]));
}
```

### Anyone-Can-Pay Mode

```rust
// ACP mode implementation in Rust
use ckb_types::prelude::*;

pub struct AcpConfig {
    pub ckb_minimum: u64,  // Minimum CKB to retain (shannon)
    pub udt_minimum: u64,  // Minimum UDT tokens to retain
}

impl AcpConfig {
    pub fn to_omnilock_args(&self, base_auth: &[u8; 21]) -> Bytes {
        let mut args = Vec::new();
        
        // Base auth field
        args.extend_from_slice(base_auth);
        
        // Omni flags with ACP enabled
        args.push(0x02); // ACP mode bit set
        
        // ACP configuration
        args.extend_from_slice(&self.ckb_minimum.to_le_bytes());
        args.extend_from_slice(&self.udt_minimum.to_le_bytes());
        
        args.into()
    }
}

// ACP transaction builder
pub struct AcpTransactionBuilder {
    config: AcpConfig,
    inputs: Vec<CellInput>,
    outputs: Vec<CellOutput>,
    outputs_data: Vec<Bytes>,
}

impl AcpTransactionBuilder {
    pub fn new(config: AcpConfig) -> Self {
        Self {
            config,
            inputs: Vec::new(),
            outputs: Vec::new(),
            outputs_data: Vec::new(),
        }
    }
    
    pub fn add_acp_input(&mut self, cell_input: CellInput) -> &mut Self {
        self.inputs.push(cell_input);
        self
    }
    
    pub fn add_output(&mut self, output: CellOutput, data: Bytes) -> &mut Self {
        self.outputs.push(output);
        self.outputs_data.push(data);
        self
    }
    
    pub fn build(&self) -> Result<Transaction, Error> {
        // Validate ACP rules
        self.validate_acp_rules()?;
        
        let tx = TransactionBuilder::default()
            .inputs(self.inputs.clone())
            .outputs(self.outputs.clone())
            .outputs_data(self.outputs_data.clone())
            .build();
            
        Ok(tx)
    }
    
    fn validate_acp_rules(&self) -> Result<(), Error> {
        let total_input_capacity = self.inputs.iter()
            .map(|input| input.capacity)
            .sum::<u64>();
            
        let total_output_capacity = self.outputs.iter()
            .map(|output| output.capacity().unwrap())
            .sum::<u64>();
        
        // Must retain minimum CKB
        if total_output_capacity < self.config.ckb_minimum {
            return Err(Error::InsufficientMinimumBalance);
        }
        
        // Validate UDT minimums if applicable
        self.validate_udt_minimums()?;
        
        Ok(())
    }
}
```

### Time Lock Mode

```javascript
// Time lock implementation
class OmnilockTimeLock {
    constructor(lockType, value) {
        this.lockType = lockType; // 'absolute_block', 'absolute_time', 'relative_block', 'relative_time'
        this.value = value;
    }
    
    // Create time lock Omnilock script
    createTimeLockScript(baseAuth) {
        const authBytes = new Uint8Array(21);
        authBytes.set(baseAuth);
        
        const omniFlags = 0x04; // Time lock mode enabled
        const sinceValue = this.encodeSinceValue();
        
        const args = ccc.bytesToHex([
            ...authBytes,
            omniFlags,
            ...sinceValue
        ]);
        
        return {
            codeHash: "0x79f90bb5e892d80dd213439eeab551120eb417678824f453d0c94b0c15dc3c8c",
            hashType: "type", 
            args: args
        };
    }
    
    encodeSinceValue() {
        let since = this.value;
        
        switch (this.lockType) {
            case 'absolute_block':
                // No flags, just the value
                break;
            case 'absolute_time':
                since |= 0x20 << 56; // Set timestamp flag
                break;
            case 'relative_block':
                since |= 0x80 << 56; // Set relative flag
                break;
            case 'relative_time':
                since |= 0xA0 << 56; // Set relative + timestamp flags
                break;
            default:
                throw new Error(`Invalid lock type: ${this.lockType}`);
        }
        
        // Convert to 8-byte little-endian
        const buffer = new ArrayBuffer(8);
        const view = new DataView(buffer);
        view.setBigUint64(0, BigInt(since), true);
        
        return new Uint8Array(buffer);
    }
    
    // Validate if time lock can be unlocked
    validateTimeLock(currentHeader, inputHeader = null) {
        const currentBlock = currentHeader.number;
        const currentTime = currentHeader.timestamp;
        
        switch (this.lockType) {
            case 'absolute_block':
                return currentBlock >= this.value;
                
            case 'absolute_time':
                return currentTime >= this.value;
                
            case 'relative_block':
                if (!inputHeader) {
                    throw new Error('Input header required for relative time lock');
                }
                return (currentBlock - inputHeader.number) >= this.value;
                
            case 'relative_time':
                if (!inputHeader) {
                    throw new Error('Input header required for relative time lock');
                }
                return (currentTime - inputHeader.timestamp) >= this.value;
                
            default:
                return false;
        }
    }
}

// Usage example
async function createTimeLockTransfer(lockDuration) {
    const timeLock = new OmnilockTimeLock('relative_time', lockDuration);
    
    const authBytes = new Uint8Array(21);
    authBytes[0] = 0x01; // Ethereum flag
    authBytes.set(ccc.hexToBytes(senderAddress.slice(2)), 1);
    
    const lockScript = timeLock.createTimeLockScript(authBytes);
    
    // Create transaction with time lock
    const tx = ccc.Transaction.builder()
        .addOutput({
            lock: lockScript,
            capacity: amount
        })
        .build();
        
    return tx;
}
```

## Testing Patterns

### Unit Testing Omnilock Integration

```javascript
describe('Omnilock Integration Tests', () => {
    let client, omnilockScript;
    
    beforeEach(() => {
        client = new ccc.ClientPublicTestnet();
        omnilockScript = createEthereumOmnilockScript(TEST_ETH_ADDRESS);
    });
    
    test('should create valid Ethereum Omnilock script', () => {
        expect(omnilockScript.codeHash).toBe(OMNILOCK_CODE_HASH);
        expect(omnilockScript.hashType).toBe('type');
        expect(omnilockScript.args.length).toBe((21 + 1) * 2 + 2); // 44 chars + 0x
    });
    
    test('should sign transaction with Ethereum wallet', async () => {
        const tx = createTestTransaction(omnilockScript);
        const witness = await signOmnilockTransaction(tx, TEST_ETH_ADDRESS);
        
        expect(witness).toBeDefined();
        expect(witness.lock).toBeDefined();
    });
    
    test('should validate multisig configuration', () => {
        const config = new MultiSigConfig(1, 2, [PUBKEY1, PUBKEY2, PUBKEY3]);
        const args = config.to_omnilock_args();
        
        expect(args[0]).toBe(0x06); // MultiSig flag
        expect(args[2]).toBe(1); // first_n
        expect(args[3]).toBe(2); // threshold
        expect(args[4]).toBe(3); // pubkeys count
    });
    
    test('should enforce ACP minimum balance rules', () => {
        const acpConfig = new AcpConfig(100, 50);
        const builder = new AcpTransactionBuilder(acpConfig);
        
        // Add insufficient output
        builder.add_output(createTestOutput(50), new Uint8Array());
        
        expect(() => builder.build()).toThrow('InsufficientMinimumBalance');
    });
    
    test('should validate time lock constraints', () => {
        const timeLock = new OmnilockTimeLock('absolute_block', 1000);
        const currentHeader = { number: 900, timestamp: Date.now() };
        
        expect(timeLock.validateTimeLock(currentHeader)).toBe(false);
        
        currentHeader.number = 1100;
        expect(timeLock.validateTimeLock(currentHeader)).toBe(true);
    });
});
```

### Integration Testing with Mock Wallets

```javascript
// Mock wallet implementations for testing
class MockEthereumWallet {
    constructor(privateKey) {
        this.privateKey = privateKey;
        this.address = ethers.utils.computeAddress(privateKey);
    }
    
    async signMessage(messageHash) {
        const signature = ethers.utils.joinSignature(
            ethers.utils.splitSignature(
                ethers.utils.signMessage(messageHash, this.privateKey)
            )
        );
        return signature;
    }
}

class MockBitcoinWallet {
    constructor(privateKey, addressType = 'P2WPKH') {
        this.privateKey = privateKey;
        this.addressType = addressType;
        this.address = this.generateAddress();
    }
    
    generateAddress() {
        switch (this.addressType) {
            case 'P2WPKH':
                return bitcoin.payments.p2wpkh({
                    pubkey: bitcoin.ECPair.fromPrivateKey(this.privateKey).publicKey
                }).address;
            case 'P2SH':
                return bitcoin.payments.p2sh({
                    redeem: bitcoin.payments.p2wpkh({
                        pubkey: bitcoin.ECPair.fromPrivateKey(this.privateKey).publicKey
                    })
                }).address;
            default:
                throw new Error(`Unsupported address type: ${this.addressType}`);
        }
    }
    
    async signMessage(messageHash) {
        const keyPair = bitcoin.ECPair.fromPrivateKey(this.privateKey);
        return keyPair.sign(messageHash);
    }
}

// Integration test with mock wallets
describe('Cross-Chain Wallet Integration', () => {
    test('should work with Ethereum wallet', async () => {
        const wallet = new MockEthereumWallet(TEST_ETH_PRIVATE_KEY);
        const result = await transferWithEthereumWallet(
            wallet, 
            RECIPIENT_ADDRESS, 
            TRANSFER_AMOUNT
        );
        
        expect(result.txHash).toBeDefined();
    });
    
    test('should work with Bitcoin wallet', async () => {
        const wallet = new MockBitcoinWallet(TEST_BTC_PRIVATE_KEY, 'P2WPKH');
        const result = await transferWithBitcoinWallet(
            wallet,
            RECIPIENT_ADDRESS,
            TRANSFER_AMOUNT
        );
        
        expect(result.txHash).toBeDefined();
    });
});
```

## Production Deployment Guidelines

### Security Checklist

- Validate all signature formats according to blockchain specifications
- Ensure proper args format and length validation
- Verify enabled modes match expected functionality
- Thoroughly validate SMT proofs in administrator mode
- Confirm time lock constraints are properly enforced

### Performance Optimization

- Group multiple signature verifications
- Cache compiled scripts and RCE proofs
- Minimize witness data to reduce transaction fees
- Pool connections to external blockchain RPCs

### Monitoring and Alerting

```javascript
// Monitoring helper for Omnilock transactions
class OmnilockMonitor {
    constructor(client, alerting) {
        this.client = client;
        this.alerting = alerting;
        this.metrics = {
            transactions: 0,
            failures: 0,
            authMethods: new Map()
        };
    }
    
    async monitorTransaction(txHash) {
        try {
            const tx = await this.client.getTransaction(txHash);
            this.analyzeOmnilockUsage(tx);
            this.metrics.transactions++;
        } catch (error) {
            this.metrics.failures++;
            this.alerting.error('Omnilock transaction failed', {
                txHash,
                error: error.message
            });
        }
    }
    
    analyzeOmnilockUsage(transaction) {
        for (const input of transaction.inputs) {
            const script = input.lock;
            if (script.codeHash === OMNILOCK_CODE_HASH) {
                const authFlag = script.args[2]; // First byte of auth field
                const method = this.getAuthMethodName(authFlag);
                
                this.metrics.authMethods.set(
                    method,
                    (this.metrics.authMethods.get(method) || 0) + 1
                );
            }
        }
    }
    
    getAuthMethodName(flag) {
        const methods = {
            0x00: 'CKB Native',
            0x01: 'Ethereum',
            0x03: 'Tron',
            0x04: 'Bitcoin',
            0x05: 'Dogecoin',
            0x06: 'CKB MultiSig'
        };
        return methods[flag] || 'Unknown';
    }
    
    getMetrics() {
        return {
            ...this.metrics,
            successRate: this.metrics.transactions / 
                (this.metrics.transactions + this.metrics.failures)
        };
    }
}
```

