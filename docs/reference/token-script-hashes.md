# Token Script Hashes

## Description

Code hashes, type IDs, and cell dependencies for CKB token scripts on mainnet and testnet. SUDT (Simple User Defined Token) and xUDT (Extensible User Defined Token) deployment information. Args format, transaction hashes, and output indices for token type scripts.

## SUDT (Simple User Defined Token)

**Mainnet**
- **Code Hash**: `0x5e7a36a77e68eecc013dfa2fe6a23f3b6c344b04005808694ae6dd45eea4cfd5`
- **Hash Type**: `type`
- **Args**: Contains the owner's lock script hash (32 bytes).

**Cell Dependency (Mainnet)**
- **TX Hash**: `0xc7813f6a415144643970c2e88e0bb6ca6a8edc5dd7c1022746f628284a9936d5`
- **Index**: `0x0`
- **Dep Type**: `code`

**Testnet**
- **Code Hash**: `0xc5e5dcf215925f7ef4dfaf5f4b4f105bc321c02776d6e7d52a1db3fcd9d011a4`
- **Hash Type**: `type`
- **Args**: Contains the owner's lock script hash (32 bytes).

**Cell Dependency (Testnet)**
- **TX Hash**: `0xe12877ebd2c3c364dc46c5c992bcfaf4fee33fa13eebdf82c591fc9825aab769`
- **Index**: `0x0`
- **Dep Type**: `code`

**Source**: [RFC 0025: Simple UDT](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0025-simple-udt/0025-simple-udt.md)

## xUDT (Extensible User Defined Token)

**Mainnet**
- **Code Hash**: `0x50bd8d6680b8b9cf98b73f3c08faf8b2a21914311954118ad6609be6e78a1b95`
- **Hash Type**: `data1`
- **Args**: Contains the owner's lock script hash (32 bytes).

**Cell Dependency (Mainnet)**
- **TX Hash**: `0xc07844ce21b38e4b071dd0e1ee3b0e27afd8d7532491327f39b786343f558ab7`
- **Index**: `0x0`
- **Dep Type**: `code`

**Testnet (Version 1 - data1)**
- **Code Hash**: `0x50bd8d6680b8b9cf98b73f3c08faf8b2a21914311954118ad6609be6e78a1b95`
- **Hash Type**: `data1`
- **Args**: Contains the owner's lock script hash (32 bytes).

**Cell Dependency (Testnet V1)**
- **TX Hash**: `0xbf6fb538763efec2a70a6a3dcb7242787087e1030c4e7d86585bc63a9d337f5f`
- **Index**: `0x0`
- **Dep Type**: `code`

**Testnet (Version 2 - type, used by CCC)**
- **Code Hash**: `0x25c29dc317811a6f6f3985a7a9ebc4838bd388d19d0feeecf0bcd60f6c0975bb`
- **Hash Type**: `type`
- **Args**: Contains the owner's lock script hash (32 bytes).

**Cell Dependency (Testnet V2)**
- **TX Hash**: `0xbf6fb538763efec2a70a6a3dcb7242787087e1030c4e7d86585bc63a9d337f5f`
- **Index**: `0x0`
- **Dep Type**: `code`

*Note: Testnet has two xUDT versions. Version 1 uses `data1` hash type (immutable). Version 2 uses `type` hash type (upgradable via Type ID) and is the default in CCC SDK.*

**Source**: [RFC 0052: Extensible UDT](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0052-extensible-udt/0052-extensible-udt.md)

## Hash Type Values

- `type`: Upgradable smart contract using type script verification (most commonly Type ID system).
- `data`: Script identified by data hash using CKB VM v0 (Lina).
- `data1`: Script identified by data hash using CKB VM v1 (CKB2021 hardfork, Mirana).
- `data2`: Script identified by data hash using CKB VM v2 (CKB2023 hardfork, Meepo).

## References

- [RFC 0025: Simple UDT](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0025-simple-udt/0025-simple-udt.md) - SUDT
- [RFC 0052: Extensible UDT](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0052-extensible-udt/0052-extensible-udt.md) - xUDT
- [CCC SDK Mainnet Config](https://github.com/ckb-ecofund/ccc/blob/master/packages/core/src/client/clientPublicMainnet.advanced.ts) - Mainnet script configurations
- [CCC SDK Testnet Config](https://github.com/ckb-ecofund/ccc/blob/master/packages/core/src/client/clientPublicTestnet.advanced.ts) - Testnet script configurations
