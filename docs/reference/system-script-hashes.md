# System Script Hashes

## Description

Code hashes, type IDs, and cell dependencies for CKB system scripts on mainnet and testnet. Genesis block transactions, SECP256K1/blake160 lock script, SECP256K1/blake160 multisig lock script, NervosDAO type script, and quantum-resistant lock script. Deployment transaction hashes and output indices for each script.

## Script Structure Overview

### Script Definition

A CKB script consists of three essential components:

- **Code Hash**: The hash identifying the script code (either data hash or type script hash).
- **Hash Type**: Specifies how the code hash should be interpreted (`type`, `data`, `data1`, `data2`).
- **Args**: Additional parameters passed to the script. Can be empty (`0x`) or contain data like identity hashes, configuration flags, or operational parameters. **The script itself (identified by code_hash + hash_type) defines how args are interpreted.** For example, the SECP256K1 lock script uses args to store the 20-byte pubkey hash, meaning two cells with the same code_hash and hash_type but different args represent different users.

### Cell Dependencies

Every script requires at least one cell dependency to function. Cell dependencies specify where the script code is located on-chain:

- **TX Hash**: Transaction hash containing the script code cell.
- **Index**: Output index of the script code cell within the transaction.
- **Dep Type**: How the dependency should be loaded (`code`, `dep_group`).

Scripts cannot execute without their corresponding cell dependencies being included in the transaction's `cell_deps` field.

## Genesis Block

The genesis block (block 0) contains foundational system scripts referenced by multiple protocols.

**Mainnet Genesis Transaction**

**TX Hash**: `0xe2fb199810d49a4d8beec56718ba2593b665db9d52299a0f9e6e75416d73ff5c`

**Outputs:**
- Index 1: SECP256K1/blake160 lock binary
- Index 2: NervosDAO type script
- Index 3: secp256k1_data (precomputed elliptic curve parameters)
- Index 4: Multisig lock binary

**Testnet Genesis Transaction**

**TX Hash**: `0x8f8c79eb6671709633fe6a46de93c0fedc9c1b8a6527a18d3983879542635c9f`

Contains the same system scripts at identical indices. Testnet uses separate dep_group transactions (`0xf8de3bb...`) that bundle these genesis scripts for easier reference.

**Usage:** Genesis scripts appear as dependencies in SECP256K1, Multisig, ACP, JoyID, and iCKB dep groups.

## SECP256K1_BLAKE160 (Fallback Lock Script)

*Note: Sometimes called the default lock script.*

**Mainnet**
- **Code Hash**: `0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8`
- **Hash Type**: `type`
- **Args**: Contains the 20-byte Blake160 hash of the public key.

**Cell Dependency (Mainnet)**
- **TX Hash**: `0x71a7ba8fc96349fea0ed3a5c47992e3b4084b031a42264a018e0072e8172e46c`
- **Index**: `0x0`
- **Dep Type**: `dep_group` (2 cells)

**Testnet**
- **Code Hash**: `0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8`
- **Hash Type**: `type`
- **Args**: Contains the 20-byte Blake160 hash of the public key.

**Cell Dependency (Testnet)**
- **TX Hash**: `0xf8de3bb47d055cdf460d93a2a6e1b05f7432f9777c8c474abf4eec1d4aee5d37`
- **Index**: `0x0`
- **Dep Type**: `dep_group` (2 cells)

**Source**: [RFC 0024: CKB Genesis Script List](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0024-ckb-genesis-script-list/0024-ckb-genesis-script-list.md)

## SECP256K1_BLAKE160_MULTISIG (Multi-Signature Lock Script)

*Note: This is the multi-signature variant of the SECP256K1/blake160 lock script, using the same cryptographic foundation (secp256k1 elliptic curve + blake160 hashing) but enabling M-of-N threshold signing instead of single-key authorization.*

**Mainnet**
- **Code Hash**: `0x5c5069eb0857efc65e1bca0c07df34c31663b3622fd3876c876320fc9634e2a8`
- **Hash Type**: `type`
- **Args**: `<multisig_hash: 20 bytes> + <S|R|M: 1 byte> + <pubkeys: N×33 bytes>`
  - **Bytes 0-19**: Blake160 hash of multisig script
  - **Byte 20**: Configuration byte (bits 0-4: threshold M, bits 5-6: require_first_n R, bit 7: reserved S)
  - **Bytes 21+**: Compressed secp256k1 public keys (33 bytes each)
  - **Example**: 2-of-3 multisig = 20-byte hash + 1 config byte + 3×33 pubkey bytes = 120 bytes total

**Cell Dependency (Mainnet)**
- **TX Hash**: `0x71a7ba8fc96349fea0ed3a5c47992e3b4084b031a42264a018e0072e8172e46c`
- **Index**: `0x1`
- **Dep Type**: `dep_group` (2 cells)

**Testnet**
- **Code Hash**: `0x5c5069eb0857efc65e1bca0c07df34c31663b3622fd3876c876320fc9634e2a8`
- **Hash Type**: `type`
- **Args**: `<multisig_hash: 20 bytes> + <S|R|M: 1 byte> + <pubkeys: N×33 bytes>`
  - **Bytes 0-19**: Blake160 hash of multisig script
  - **Byte 20**: Configuration byte (bits 0-4: threshold M, bits 5-6: require_first_n R, bit 7: reserved S)
  - **Bytes 21+**: Compressed secp256k1 public keys (33 bytes each)
  - **Example**: 2-of-3 multisig = 20-byte hash + 1 config byte + 3×33 pubkey bytes = 120 bytes total

**Cell Dependency (Testnet)**
- **TX Hash**: `0xf8de3bb47d055cdf460d93a2a6e1b05f7432f9777c8c474abf4eec1d4aee5d37`
- **Index**: `0x1`
- **Dep Type**: `dep_group` (2 cells)

**Source**: [RFC 0024: CKB Genesis Script List](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0024-ckb-genesis-script-list/0024-ckb-genesis-script-list.md)

## DAO (Nervos DAO Type Script)

*Note: Code hash is identical on both networks*

**Mainnet**
- **Code Hash**: `0x82d76d1b75fe2fd9a27dfbaa65a039221a380d76c926f378d3f81cf3e7e13f2e`
- **Hash Type**: `type`
- **Args**: `0x`

**Cell Dependency (Mainnet)**
- **TX Hash**: `0xe2fb199810d49a4d8beec56718ba2593b665db9d52299a0f9e6e75416d73ff5c`
- **Index**: `0x2`
- **Dep Type**: `code`

**Testnet**
- **Code Hash**: `0x82d76d1b75fe2fd9a27dfbaa65a039221a380d76c926f378d3f81cf3e7e13f2e`
- **Hash Type**: `type`
- **Args**: `0x`

**Cell Dependency (Testnet)**
- **TX Hash**: `0x8f8c79eb6671709633fe6a46de93c0fedc9c1b8a6527a18d3983879542635c9f`
- **Index**: `0x2`
- **Dep Type**: `code`

**Source**: [RFC 0024: CKB Genesis Script List](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0024-ckb-genesis-script-list/0024-ckb-genesis-script-list.md)

## Quantum-Resistant Lock Script

Post-quantum cryptography lock script using SPHINCS+ signature algorithm, resistant to attacks from quantum computers.

**Mainnet**
- **Code Hash**: `0x302d35982f865ebcbedb9a9360e40530ed32adb8e10b42fbbe70d8312ff7cedf`
- **Hash Type**: `type`

**Cell Dependency (Mainnet)**
- **TX Hash**: `0x4598d00df2f3dc8bc40eee38689a539c94f6cc3720b7a2a6746736daa60f500a`
- **Index**: `0x0`
- **Dep Type**: `code`

**Testnet**
- **Code Hash**: `0x147ecbb5c5127d982ee1362d2c2bb4267803da2eb006d150e88af6caaa0a7eaf`
- **Hash Type**: `data1`

**Cell Dependency (Testnet)**
- **TX Hash**: `0x631d9a6049fb1fc3790e89d9daf35abe535b5e754cd8c3404319319710f0b106`
- **Index**: `0x0`
- **Dep Type**: `code`

**Source**: [Quantum-Resistant Lock Script](https://github.com/cryptape/quantum-resistant-lock-script)

## Hash Type Values

- `type`: Upgradable smart contract using type script verification (most commonly Type ID system).
- `data`: Script identified by data hash using CKB VM v0 (Lina).
- `data1`: Script identified by data hash using CKB VM v1 (CKB2021 hardfork, Mirana).
- `data2`: Script identified by data hash using CKB VM v2 (CKB2023 hardfork, Meepo).

## References

- [RFC 0024: CKB Genesis Script List](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0024-ckb-genesis-script-list/0024-ckb-genesis-script-list.md) - SECP256K1, Multisig, DAO, Type ID
- [Quantum-Resistant Lock Script](https://github.com/cryptape/quantum-resistant-lock-script) - Post-quantum SPHINCS+ lock script
- [CCC SDK Mainnet Config](https://github.com/ckb-ecofund/ccc/blob/master/packages/core/src/client/clientPublicMainnet.advanced.ts) - Mainnet script configurations
- [CCC SDK Testnet Config](https://github.com/ckb-ecofund/ccc/blob/master/packages/core/src/client/clientPublicTestnet.advanced.ts) - Testnet script configurations
