## Description

Common Omnilock script errors and solutions. Covers signature verification failures for Bitcoin/Ethereum/Tron wallets, administrator mode issues, anyone-can-pay validation errors, time lock problems, witness structure mismatches, authentication flag errors, and cross-chain compatibility issues with debugging strategies and code examples.

## Related Resources

- Protocol Spec: ckb://docs/omnilock/protocol
- Development Guide: ckb://docs/omnilock/development
- API Examples: ckb://docs/omnilock/api-examples

## Common Errors

### ERROR: Invalid Signature

**Error Code**: `0x40` (ValidationFailure: -64)

**Causes**:
- Wrong authentication flag for wallet type
- Incorrect message hash calculation
- Mismatched public key recovery

**Solutions**:

```typescript
// Ethereum signature - ensure correct flag
const witness = {
  lock: {
    signature: ethSignature,
    flag: 0x01, // Must be 0x01 for Ethereum
    message: messageHash
  }
};

// Bitcoin signature - use correct flag based on address type
const btcFlag = addressType === 'P2WPKH' ? 0x04 : 0x04;
```

### ERROR: Administrator Check Failed

**Error Code**: `0x41` (ValidationFailure: -65)

**Cause**: Transaction violates administrator whitelist/blacklist rules

**Debug Steps**:
```typescript
// Check if address is blacklisted
const adminList = await getAdminList(lockScript);
const isBlacklisted = adminList.blacklist.includes(address);

// Verify administrator signature
const adminSig = witness.lock.adminSignature;
const isValidAdmin = verifyAdminSignature(adminSig, adminPubkey);
```

### ERROR: ACP Amount Insufficient

**Error Code**: `0x43` (ValidationFailure: -67)

**Cause**: Anyone-can-pay mode requires minimum amount not met

**Solution**:
```typescript
// Ensure output maintains minimum amount
const MIN_ACP_AMOUNT = 61n * 10n ** 8n; // 61 CKB minimum
const outputCell = {
  capacity: originalCapacity - transferAmount,
  lock: acpLockScript
};

if (outputCell.capacity < MIN_ACP_AMOUNT) {
  throw new Error(`ACP requires minimum ${MIN_ACP_AMOUNT} shannons`);
}
```

### ERROR: Time Lock Not Satisfied

**Error Code**: `0x44` (ValidationFailure: -68)

**Cause**: Transaction attempted before time lock expiry

**Debug**:
```typescript
// Check time lock in args
const args = lockScript.args;
const timeLock = extractTimeLock(args); // bytes 28-35

// Compare with header timestamp
const headerDep = tx.headerDeps[0];
const header = await getHeader(headerDep);
const currentTime = header.timestamp;

if (currentTime < timeLock) {
  const waitTime = timeLock - currentTime;
  console.log(`Wait ${waitTime}ms until unlock`);
}
```

### ERROR: Invalid Witness Structure

**Error Code**: `0x45` (ValidationFailure: -69)

**Common Issues**:
1. Missing witness fields
2. Incorrect witness encoding
3. Wrong witness position in transaction

**Correct Structure**:
```typescript
// Standard Omnilock witness
const witness = {
  lock: {
    signature: new Uint8Array(65), // 65 bytes for secp256k1
    flag: 0x0, // Authentication flag
    // Optional fields based on mode:
    adminProof?: Uint8Array,
    preimage?: Uint8Array, // For hash lock
  }
};

// Ensure witness matches input index
tx.witnesses[inputIndex] = serializeWitness(witness);
```

### ERROR: Wrong Authentication Flag

**Error Code**: `0x46` (ValidationFailure: -70)

**Flag Reference**:
```typescript
const AUTH_FLAGS = {
  CKB: 0x0,
  Ethereum: 0x01,
  Tron: 0x03,
  Bitcoin: 0x04,
  Dogecoin: 0x05,
  CKBMultiSig: 0x06,
  EthereumDisplay: 0x12,
  Delegation: 0xFC,
  Exec: 0xFD,
  DynamicLinking: 0xFE
};

// Validate flag matches wallet type
function validateAuthFlag(walletType: string, flag: number) {
  const expected = AUTH_FLAGS[walletType];
  if (flag !== expected) {
    throw new Error(`Expected flag ${expected} for ${walletType}, got ${flag}`);
  }
}
```

### ERROR: Invalid Public Key Recovery

**Error Code**: `0x47` (ValidationFailure: -71)

**Ethereum/Tron Specific**:
```typescript
// Ethereum requires recovery ID
const signature = ethers.utils.splitSignature(rawSig);
const recoveryId = signature.recoveryParam; // 0 or 1

// Pack signature with recovery ID
const packedSig = new Uint8Array(65);
packedSig.set(signature.r, 0);
packedSig.set(signature.s, 32);
packedSig[64] = recoveryId;

// For Tron, add 27 to recovery ID
if (walletType === 'Tron') {
  packedSig[64] = recoveryId + 27;
}
```

### ERROR: Supply Mode Validation Failed

**Error Code**: `0x48` (ValidationFailure: -72)

**Cause**: Token supply constraints violated

**Debug**:
```typescript
// Check supply mode configuration
const supplyMode = extractSupplyMode(lockScript.args);
if (supplyMode.enabled) {
  const totalSupply = calculateTotalSupply(tx.outputs);
  const maxSupply = supplyMode.maxSupply;
  
  if (totalSupply > maxSupply) {
    throw new Error(`Supply ${totalSupply} exceeds max ${maxSupply}`);
  }
}
```

## Debugging Strategies

### 1. Enable Debug Logs

```typescript
// CCC SDK
const client = new ccc.Client({
  url: RPC_URL,
  debug: true // Enable debug output
});

// ckb-sdk-rust
std::env::set_var("RUST_LOG", "debug");
```

### 2. Validate Witness Encoding

```typescript
// Use molecule to validate witness structure
import { WitnessArgs } from '@ckb-lumos/base';

function validateWitness(witness: any) {
  try {
    const decoded = WitnessArgs.unpack(witness);
    console.log('Valid witness:', decoded);
    return true;
  } catch (e) {
    console.error('Invalid witness encoding:', e);
    return false;
  }
}
```

### 3. Test with Mock Transaction

```typescript
// Create minimal test transaction
async function testOmnilockUnlock() {
  const testTx = {
    inputs: [{
      previousOutput: {
        txHash: '0x...',
        index: '0x0'
      },
      since: '0x0'
    }],
    outputs: [{
      capacity: '0x174876e800', // 100 CKB
      lock: receiverLock
    }],
    witnesses: [omnilockWitness]
  };
  
  // Run local validation
  const result = await debugger.runTransaction(testTx);
  console.log('Cycles used:', result.cycles);
}
```

### 4. Common Configuration Issues

```typescript
// Incorrect: Using wrong Omnilock deployment
const WRONG_CODE_HASH = '0x0000...'; // System script hash

// Correct: Use proper Omnilock code hash
const OMNILOCK_CODE_HASH = {
  mainnet: '0x9f3aeaf2fc439549cbc870c653374943af96a0658bd6b51be8d8983183e6f52f',
  testnet: '0xf329effd1c475a2978453c8600e1eaf0bc2087ee093c3ee64cc96ec6847752cb'
};
```

## Prevention Checklist

- [ ] Verify authentication flag matches wallet type
- [ ] Ensure witness structure follows Omnilock specification
- [ ] Check time locks and administrator rules before signing
- [ ] Validate minimum amounts for ACP mode
- [ ] Test with correct network's Omnilock deployment
- [ ] Include all required header dependencies
- [ ] Verify signature format for cross-chain wallets
- [ ] Test recovery ID calculation for Ethereum/Tron