# xUDT Protocol Specification

## Description

eXtensible User Defined Token protocol extending sUDT with modular extension capabilities for advanced token features. Covers flag-based extension system, standard extensions (supply cap, metadata, oracle), validation rules, implementation patterns, migration strategies, and security considerations for sophisticated token functionality.

xUDT (eXtensible User Defined Token) extends sUDT with modular extension capabilities for advanced token features.

## Core Structure

### Type Script
```
code_hash: 0x50bd8d6680b8b9cf98b73f3c08faf8b2a21914311954118ad6609be6e78a1b95
hash_type: data1
args: <owner_lock_hash(32 bytes)> + <xudt_flags(4 bytes)> + <extension_data>
```

### Cell Data Format
```
amount: u128 (16 bytes, little-endian)
extension_data: bytes (optional, based on flags)
```

## xUDT Flags

32-bit flags control enabled extensions:
- `0x00000000`: Basic sUDT compatibility mode
- `0x00000001`: Reserved
- `0x00000002`: Reserved
- `0x20000000`: Extension data in input cell
- `0x40000000`: Extension data in output cell
- `0x80000000`: Extension data in witness

## Standard Extensions

### Extension Script Structure
```rust
struct ExtensionScript {
    code_hash: Byte32,
    hash_type: u8,
    args: Bytes,
}
```

### Common Extensions

**1. Supply Cap Extension**
```
code_hash: SUPPLY_CAP_CODE_HASH
args: max_supply (u128, 16 bytes)
```

**2. Metadata Extension**
```
code_hash: METADATA_CODE_HASH
args: metadata_cell_type_id (32 bytes)
```

**3. Oracle Extension**
```
code_hash: ORACLE_CODE_HASH
args: oracle_cell_lock_hash (32 bytes)
```

## Implementation Examples

### Basic xUDT Transfer
```rust
// Type script args construction
let owner_lock_hash = calculate_lock_hash(&owner_lock);
let xudt_flags = 0u32; // No extensions
let type_args = [owner_lock_hash.as_bytes(), &xudt_flags.to_le_bytes()].concat();

// Create xUDT type script
let xudt_type = Script::new_builder()
    .code_hash(XUDT_CODE_HASH.pack())
    .hash_type(ScriptHashType::Data1.into())
    .args(type_args.pack())
    .build();

// Cell data: amount only
let amount = 1000000u128;
let cell_data = amount.to_le_bytes().to_vec();
```

### xUDT with Extensions
```rust
// Enable witness extension
let xudt_flags = 0x80000000u32;

// Extension script in args
let extension_script = ExtensionScript {
    code_hash: SUPPLY_CAP_CODE_HASH,
    hash_type: ScriptHashType::Type,
    args: max_supply.to_le_bytes().to_vec(),
};

// Serialize extension
let mut extension_data = vec![];
extension_data.extend_from_slice(&extension_script.code_hash);
extension_data.push(extension_script.hash_type);
extension_data.extend_from_slice(&(extension_script.args.len() as u32).to_le_bytes());
extension_data.extend_from_slice(&extension_script.args);

// Complete type args
let type_args = [
    owner_lock_hash.as_bytes(),
    &xudt_flags.to_le_bytes(),
    &extension_data
].concat();
```

## Validation Rules

### Amount Rules
1. Total input amount >= Total output amount (conservation)
2. Owner mode: If any input uses owner lock, skip amount validation
3. Amount stored as little-endian u128

### Extension Validation
1. Parse extension scripts from args based on flags
2. Execute each extension script in order
3. Extension scripts access via `CKB_SOURCE_GROUP_INPUT/OUTPUT`
4. All extensions must return success (0)

### Backwards Compatibility
- If no flags set, behaves identically to sUDT
- Existing sUDT cells can migrate by adding 4 zero bytes for flags

## Common Patterns

### Token Issuance
```rust
// Owner issues new tokens
let tx = TransactionBuilder::default()
    .input(owner_cell)
    .output(
        CellOutput::new_builder()
            .lock(recipient_lock)
            .type_(Some(xudt_type).pack())
            .capacity(capacity.pack())
            .build()
    )
    .output_data(issue_amount.to_le_bytes().pack())
    .build();
```

### Token Burning
```rust
// Burn by sending less output than input
let burn_amount = 1000u128;
let input_amount = get_xudt_amount(&input_cell);
let output_amount = input_amount - burn_amount;

// Create output with reduced amount
let output_data = output_amount.to_le_bytes().to_vec();
```

### Extension Discovery
```rust
fn parse_xudt_args(args: &[u8]) -> Result<(H256, u32, Vec<ExtensionScript>)> {
    let owner_lock_hash = H256::from_slice(&args[0..32])?;
    let flags = u32::from_le_bytes(args[32..36].try_into()?);
    
    let mut extensions = vec![];
    let mut offset = 36;
    
    while offset < args.len() {
        let code_hash = H256::from_slice(&args[offset..offset+32])?;
        let hash_type = args[offset+32];
        let args_len = u32::from_le_bytes(args[offset+33..offset+37].try_into()?);
        let ext_args = args[offset+37..offset+37+args_len as usize].to_vec();
        
        extensions.push(ExtensionScript {
            code_hash,
            hash_type,
            args: ext_args,
        });
        
        offset += 37 + args_len as usize;
    }
    
    Ok((owner_lock_hash, flags, extensions))
}
```

## Security Considerations

1. **Extension Trust**: Only use verified extension scripts
2. **Flag Validation**: Ensure flags match actual data locations
3. **Owner Key**: Protect owner lock private key for issuance control
4. **Amount Overflow**: Validate u128 arithmetic operations
5. **Extension Order**: Extensions execute sequentially, order matters

## Migration from sUDT

```rust
// Old sUDT args: owner_lock_hash (32 bytes)
let sudt_args = owner_lock_hash.as_bytes();

// New xUDT args: owner_lock_hash + flags
let xudt_args = [sudt_args, &0u32.to_le_bytes()].concat();

// Update type script
let xudt_type = Script::new_builder()
    .code_hash(XUDT_CODE_HASH.pack())
    .hash_type(ScriptHashType::Data1.into())
    .args(xudt_args.pack())
    .build();
```

## Reference Implementation

See: `resources/omnilock/c/xudt_rce.mol` for Molecule schemas
See: `resources/ckb-production-scripts/rust/xudt` for Rust implementation