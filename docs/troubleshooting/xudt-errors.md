## Description

xUDT script error debugging guide covering extension validation failures, amount overflow errors, type script hash mismatches, owner mode violations, regulatory flag issues, token minting/burning errors, and compatibility problems with code examples and solutions.

## Related Resources

- Protocol Spec: ckb://docs/protocols/xudt-protocol
- Token Creation: ckb://docs/patterns/token-creation-guide
- UDT Patterns: ckb://docs/patterns/udt-tokens-guide

## Common xUDT Errors

### ERROR: Extension Validation Failed

**Error Code**: `0x50` (ScriptValidationFailure: -80)

**Cause**: Invalid or unsupported extension in xUDT cell data

**Solution**:
```rust
// Ensure extension follows xUDT format
let mut data = amount.to_le_bytes().to_vec(); // First 16 bytes: amount
data.extend_from_slice(&extension_data);

// Extension must be valid molecule encoding
let extension = Extension::new_builder()
    .set(ExtensionType::OwnerMode, owner_lock_hash)
    .build();
```

### ERROR: Amount Overflow

**Error Code**: `0x51` (ArithmeticError: -81)

**Cause**: Token amount exceeds u128 maximum or causes overflow in calculation

**Debug**:
```typescript
// JavaScript/TypeScript - use BigInt for safety
const MAX_XUDT = 2n ** 128n - 1n;
const amount1 = BigInt("100000000000000000000");
const amount2 = BigInt("200000000000000000000");

// Check for overflow before operation
if (amount1 + amount2 > MAX_XUDT) {
  throw new Error("xUDT amount overflow");
}

// In Rust
use primitive_types::U128;
let amount = U128::from_little_endian(&data[0..16]);
let new_amount = amount.checked_add(transfer_amount)
    .ok_or("Amount overflow")?;
```

### ERROR: Type Script Hash Mismatch

**Error Code**: `0x52` (ScriptValidationFailure: -82)

**Cause**: Output cells have different type script hash than inputs

**Validation**:
```typescript
// All xUDT cells must have same type script hash
function validateXudtTransfer(tx: Transaction) {
  const inputTypeHash = tx.inputs[0].cellOutput.type?.hash();
  
  for (const output of tx.outputs) {
    if (output.type && output.type.hash() !== inputTypeHash) {
      throw new Error("Type script hash mismatch");
    }
  }
}
```

### ERROR: Owner Mode Violation

**Error Code**: `0x53` (PermissionDenied: -83)

**Cause**: Non-owner attempting restricted operation (mint/burn)

**Owner Mode Check**:
```typescript
// Extract owner lock from extension
function extractOwnerLock(cellData: Uint8Array): Uint8Array | null {
  if (cellData.length <= 16) return null; // No extension
  
  const extension = cellData.slice(16);
  // Parse extension for owner mode flag (0x01)
  if (extension[0] === 0x01) {
    return extension.slice(1, 33); // 32-byte owner lock hash
  }
  return null;
}

// Verify owner authorization
function verifyOwnerMode(tx: Transaction, cellData: Uint8Array) {
  const ownerLockHash = extractOwnerLock(cellData);
  if (!ownerLockHash) return true; // No owner mode
  
  // Check if any input has the owner lock
  const hasOwnerLock = tx.inputs.some(input => 
    input.cellOutput.lock.hash() === ownerLockHash
  );
  
  if (!hasOwnerLock) {
    throw new Error("Owner authorization required");
  }
}
```

### ERROR: Invalid Regulatory Flags

**Error Code**: `0x54` (ScriptValidationFailure: -84)

**Cause**: Regulatory extension contains invalid flags or data

**Regulatory Extension Format**:
```rust
// Regulatory flags in extension
const REGULATORY_FLAG: u8 = 0x02;
const BLACKLIST_FLAG: u8 = 0x04;
const WHITELIST_FLAG: u8 = 0x08;

struct RegulatoryExtension {
    flags: u8,
    admin_lock_hash: [u8; 32],
    list_cell_type_hash: Option<[u8; 32]>,
}

// Validate regulatory compliance
fn check_regulatory_compliance(
    extension: &[u8],
    from_address: &Address,
    to_address: &Address,
) -> Result<(), Error> {
    let flags = extension[0];
    
    if flags & BLACKLIST_FLAG != 0 {
        // Check blacklist
        if is_blacklisted(from_address) || is_blacklisted(to_address) {
            return Err(Error::Blacklisted);
        }
    }
    
    if flags & WHITELIST_FLAG != 0 {
        // Check whitelist
        if !is_whitelisted(from_address) || !is_whitelisted(to_address) {
            return Err(Error::NotWhitelisted);
        }
    }
    
    Ok(())
}
```

### ERROR: Token Conservation Violation

**Error Code**: `0x55` (ScriptValidationFailure: -85)

**Cause**: Input token sum doesn't equal output token sum (except for mint/burn)

**Validation Logic**:
```typescript
function validateTokenConservation(tx: Transaction) {
  let inputSum = 0n;
  let outputSum = 0n;
  
  // Calculate input tokens
  for (const input of tx.inputs) {
    const cellData = input.data;
    if (cellData.length >= 16) {
      inputSum += BigInt(readUInt128LE(cellData.slice(0, 16)));
    }
  }
  
  // Calculate output tokens
  for (const output of tx.outputs) {
    const cellData = output.data;
    if (cellData.length >= 16) {
      outputSum += BigInt(readUInt128LE(cellData.slice(0, 16)));
    }
  }
  
  // Check conservation (unless owner mode allows mint/burn)
  if (inputSum !== outputSum && !hasOwnerAuthorization(tx)) {
    throw new Error(`Token conservation failed: ${inputSum} != ${outputSum}`);
  }
}
```

### ERROR: Invalid Token Amount Encoding

**Error Code**: `0x56` (EncodingError: -86)

**Cause**: Token amount not properly encoded as 16-byte little-endian

**Correct Encoding**:
```typescript
// JavaScript/TypeScript
function encodeAmount(amount: bigint): Uint8Array {
  const buffer = new ArrayBuffer(16);
  const view = new DataView(buffer);
  
  // Write as little-endian u128
  view.setBigUint64(0, amount & 0xFFFFFFFFFFFFFFFFn, true);
  view.setBigUint64(8, amount >> 64n, true);
  
  return new Uint8Array(buffer);
}

// Rust
fn encode_amount(amount: u128) -> [u8; 16] {
    amount.to_le_bytes()
}
```

### ERROR: Mint Without Type ID

**Error Code**: `0x57` (ScriptValidationFailure: -87)

**Cause**: Attempting to mint tokens without proper type ID in first input

**Solution**:
```typescript
// Ensure first input contains type ID for minting
const firstInput = tx.inputs[0];
const typeScript = firstInput.cellOutput.type;

if (!typeScript || typeScript.codeHash !== TYPE_ID_CODE_HASH) {
  throw new Error("First input must have type ID for minting");
}

// Type ID ensures unique token identity
const typeId = typeScript.args; // 32-byte unique identifier
```

## Debugging Utilities

### xUDT Data Parser

```typescript
function parseXudtData(data: Uint8Array): {
  amount: bigint;
  extension?: any;
} {
  if (data.length < 16) {
    throw new Error("Invalid xUDT data: too short");
  }
  
  const amount = readUInt128LE(data.slice(0, 16));
  
  if (data.length > 16) {
    // Parse extension based on first byte flag
    const extensionFlag = data[16];
    const extensionData = data.slice(16);
    
    switch (extensionFlag) {
      case 0x01: // Owner mode
        return {
          amount,
          extension: {
            type: 'owner',
            ownerLockHash: extensionData.slice(1, 33)
          }
        };
      case 0x02: // Regulatory
        return {
          amount,
          extension: {
            type: 'regulatory',
            flags: extensionData[1],
            adminLock: extensionData.slice(2, 34)
          }
        };
      default:
        return { amount, extension: { raw: extensionData } };
    }
  }
  
  return { amount };
}
```

### Transaction Validator

```typescript
async function validateXudtTransaction(tx: Transaction) {
  const errors = [];
  
  try {
    validateTokenConservation(tx);
  } catch (e) {
    errors.push(`Conservation: ${e.message}`);
  }
  
  try {
    validateTypeScriptConsistency(tx);
  } catch (e) {
    errors.push(`Type Script: ${e.message}`);
  }
  
  try {
    validateExtensions(tx);
  } catch (e) {
    errors.push(`Extensions: ${e.message}`);
  }
  
  if (errors.length > 0) {
    console.error("xUDT Validation Errors:", errors);
    return false;
  }
  
  return true;
}
```

## Common Mistakes

1. **Using wrong endianness**: xUDT uses little-endian for amounts
2. **Forgetting 16-byte padding**: Amount must always be exactly 16 bytes
3. **Missing owner lock for mint/burn**: These operations require owner authorization
4. **Incorrect extension format**: Extensions must follow xUDT specification
5. **Type script in wrong position**: xUDT must be in type script, not lock script
6. **Not checking for overflow**: Always validate arithmetic operations
7. **Ignoring regulatory flags**: Some tokens have transfer restrictions