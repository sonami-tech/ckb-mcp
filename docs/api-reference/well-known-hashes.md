# CKB Well-Known Hashes Reference

## Description

Comprehensive reference of well-known script hashes, code hashes, transaction hashes, and dependency hashes for both CKB mainnet and testnet networks. Includes system scripts, popular protocols (Omnilock, xUDT, SUDT), deployment transactions, and cell dependencies with complete hash values for development and integration.

This document provides a centralized reference for all well-known hashes in the CKB ecosystem, organized by network and script type for easy lookup during development.

## System Scripts

### SECP256K1_BLAKE160 (Fallback Lock Script)

*Note: Sometimes called the default lock script*

**Mainnet**
- **Code Hash**: `0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8`
- **Hash Type**: `type`
- **TX Hash**: `0xe2fb199810d49a4d8beec56718ba2593b665db9d52299a0f9e6e75416d73ff5c`
- **Index**: `0x1`

**Official Documentation**:
- [CKB System Scripts - secp256k1_blake160_sighash_all.c](https://github.com/nervosnetwork/ckb-system-scripts/blob/master/c/secp256k1_blake160_sighash_all.c)
- [secp256k1-blake160-sighash-all Documentation](https://nervosnetwork.github.io/ckb-system-scripts/c/secp256k1_blake160_sighash_all)
- [CKB Genesis Script List RFC](https://github.com/nervosnetwork/rfcs/blob/780b2f98068ed2337f3a97b02ec6b5336b6fb143/rfcs/0024-ckb-genesis-script-list/0024-ckb-genesis-script-list.md)

**Cell Dependency (Mainnet)**
- **TX Hash**: `0x71a7ba8fc96349fea0ed3a5c47992e3b4084b031a42264a018e0072e8172e46c`
- **Index**: `0x0`
- **Dep Type**: `dep_group`

**Testnet**
- **Code Hash**: `0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8`
- **Hash Type**: `type`
- **TX Hash**: `0x8f8c79eb6671709633fe6a46de93c0fedc9c1b8a6527a18d3983879542635c9f`
- **Index**: `0x1`

**Cell Dependency (Testnet)**
- **TX Hash**: `0xf8de3bb47d055cdf460d93a2a6e1b05f7432f9777c8c474abf4eec1d4aee5d37`
- **Index**: `0x0`
- **Dep Type**: `dep_group`

### SECP256K1_BLAKE160_MULTISIG (Multi-Signature Lock Script)

**Mainnet**
- **Code Hash**: `0x5c5069eb0857efc65e1bca0c07df34c31663b3622fd3876c876320fc9634e2a8`
- **Hash Type**: `type`
- **TX Hash**: `0xe2fb199810d49a4d8beec56718ba2593b665db9d52299a0f9e6e75416d73ff5c`
- **Index**: `0x4`

**Official Documentation**:
- [CKB System Scripts - secp256k1_blake160_multisig_all.c](https://github.com/nervosnetwork/ckb-system-scripts/blob/master/c/secp256k1_blake160_multisig_all.c)
- [How to sign transaction Wiki](https://github.com/nervosnetwork/ckb-system-scripts/wiki/How-to-sign-transaction)

**Cell Dependency (Mainnet)**
- **TX Hash**: `0x71a7ba8fc96349fea0ed3a5c47992e3b4084b031a42264a018e0072e8172e46c`
- **Index**: `0x1`
- **Dep Type**: `dep_group`

**Testnet**
- **Code Hash**: `0x5c5069eb0857efc65e1bca0c07df34c31663b3622fd3876c876320fc9634e2a8`
- **Hash Type**: `type`
- **TX Hash**: `0x8f8c79eb6671709633fe6a46de93c0fedc9c1b8a6527a18d3983879542635c9f`
- **Index**: `0x4`

**Cell Dependency (Testnet)**
- **TX Hash**: `0xf8de3bb47d055cdf460d93a2a6e1b05f7432f9777c8c474abf4eec1d4aee5d37`
- **Index**: `0x1`
- **Dep Type**: `dep_group`

### DAO (Nervos DAO Type Script)

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

**Official Documentation**:
- [CKB System Scripts - dao.c](https://github.com/nervosnetwork/ckb-system-scripts/blob/master/c/dao.c)
- [Nervos DAO RFC](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0023-dao-deposit-withdraw/0023-dao-deposit-withdraw.md)

## Token Scripts

### SUDT (Simple User Defined Token)

**Mainnet**
- **Code Hash**: `0x5e7a36a77e68eecc013dfa2fe6a23f3b6c344b04005808694ae6dd45eea4cfd5`
- **Hash Type**: `type`
- **TX Hash**: `0xc7813f6a415144643970c2e88e0bb6ca6a8edc5dd7c1022746f628284a9936d5`

**Cell Dependency (Mainnet)**
- **TX Hash**: `0xc7813f6a415144643970c2e88e0bb6ca6a8edc5dd7c1022746f628284a9936d5`
- **Index**: `0x0`
- **Dep Type**: `code`

**Testnet**
- **Code Hash**: `0xc5e5dcf215925f7ef4dfaf5f4b4f105bc321c02776d6e7d52a1db3fcd9d011a4`
- **Hash Type**: `type`
- **TX Hash**: `0xe12877ebd2c3c364dc46c5c992bcfaf4fee33fa13eebdf82c591fc9825aab769`

**Cell Dependency (Testnet)**
- **TX Hash**: `0xe12877ebd2c3c364dc46c5c992bcfaf4fee33fa13eebdf82c591fc9825aab769`
- **Index**: `0x0`
- **Dep Type**: `code`

### xUDT (Extensible User Defined Token)

**Mainnet**
- **Code Hash**: `0x50bd8d6680b8b9cf98b73f3c08faf8b2a21914311954118ad6609be6e78a1b95`
- **Hash Type**: `data1`
- **TX Hash**: `0xc07844ce21b38e4b071dd0e1ee3b0e27afd8d7532491327f39b786343f558ab7`

**Cell Dependency (Mainnet)**
- **TX Hash**: `0xc07844ce21b38e4b071dd0e1ee3b0e27afd8d7532491327f39b786343f558ab7`
- **Index**: `0x0`
- **Dep Type**: `code`

**Testnet**
- **Code Hash**: `0x50bd8d6680b8b9cf98b73f3c08faf8b2a21914311954118ad6609be6e78a1b95`
- **Hash Type**: `data1`
- **TX Hash**: `0xbf6fb538763efec2a70a6a3dcb7242787087e1030c4e7d86585bc63a9d337f5f`

**Cell Dependency (Testnet)**
- **TX Hash**: `0xbf6fb538763efec2a70a6a3dcb7242787087e1030c4e7d86585bc63a9d337f5f`
- **Index**: `0x0`
- **Dep Type**: `code`

## Universal Lock Scripts

### Omnilock

**Mainnet**
- **Code Hash**: `0x79f90bb5e892d80dd213439eeab551120eb417678824f453d0c94b0c15dc3c8c`
- **Hash Type**: `type`
- **TX Hash**: `0x71a7ba8fc96349fea0ed3a5c47992e3b4084b031a42264a018e0072e8172e46c`

**Official Documentation**:
- [Omnilock RFC](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0042-omnilock/0042-omnilock.md)
- [Omnilock Script Documentation](https://docs-new.nervos.org/docs/common-scripts/omnilock)
- [Omnilock GitHub Repository](https://github.com/cryptape/omnilock)

**Testnet**
- **Code Hash**: `0x79f90bb5e892d80dd213439eeab551120eb417678824f453d0c94b0c15dc3c8c`
- **Hash Type**: `type`
- **TX Hash**: `0x57a62003daeab9d54aa29b944fc3b451213a5ebdf2e232216a3cfed0dde61b38`

**Cell Dependency (Testnet)**
- **TX Hash**: `0x57a62003daeab9d54aa29b944fc3b451213a5ebdf2e232216a3cfed0dde61b38`
- **Index**: `0x0`
- **Dep Type**: `dep_group`

**Mainnet - Deprecated**
- **Code Hash**: `0x00000000000000000000000000000000000000000000000000545950455f4944`
- **Hash Type**: `type`
- **Note**: Earlier Omnilock deployments using Type ID, use current version above

### PW Lock Script

**Official Documentation**:
- [PW Core Constants](https://github.com/jordanmack/pw-core/blob/dev/src/constants.ts)

**Mainnet**
- **Code Hash**: `0xbf43c3602455798c1a61a596e0d95278864c552fafe231c063b3fabf97a8febc`
- **Hash Type**: `type`

**Testnet**
- **Code Hash**: `0x58c5f491aba6d61678b7cf7edf4910b1f5e00ec0cde2f42e0abb4fd9aff25a63`
- **Hash Type**: `type`

### ACP (Anyone Can Pay) Lock Script

**Mainnet**
- **Code Hash**: `0x9b819793a64463aed77c615d6cb226eea5487ccfc0783043a587254cda2b6f26`
- **Hash Type**: `type`
- **TX Hash**: `0xc76edf469816aa22f416503c38d0b533d2a018e253e379f134c3985b3472c842`
- **Index**: `0x0`

**Cell Dependency (Mainnet)**
- **TX Hash**: `0xc76edf469816aa22f416503c38d0b533d2a018e253e379f134c3985b3472c842`
- **Index**: `0x0`
- **Dep Type**: `code`

**Testnet**
- **Code Hash**: `0xf329effd1c475a2978453c8600e1eaf0bc2087ee093c3ee64cc96ec6847752cb`
- **Hash Type**: `type`
- **TX Hash**: `0x3d4296df1bd2cc2bd3f483f61ab7ebeac462a2f336f2b944168fe6ba5d81c014`
- **Index**: `0x0`

**Cell Dependency (Testnet)**
- **TX Hash**: `0x3d4296df1bd2cc2bd3f483f61ab7ebeac462a2f336f2b944168fe6ba5d81c014`
- **Index**: `0x0`
- **Dep Type**: `code`

**Mainnet - Deprecated**
- **Code Hash**: `0xd369597ff47f29fbc0d47d2e3775370d1250b85140c670e4718af712983a2354`
- **Hash Type**: `type`
- **Note**: Earlier deployment, use current version above

## NFT and Digital Objects

### Spore Protocol

**Code Hashes**
- **SPORE**: `0x4a4dce1df3dffff7f8b2cd7dff7303df3b6150c9788cb75dcf6747247132b9f5`
- **CLUSTER**: `0x7366a61534fa7c7e6225ecc0d828ea3b5366adec2b58206f2ee84995fe030075`
- **Hash Type**: `type`

### CoTA (Compact Token Aggregator)

**Mainnet**
- **TX Hash**: `0x71a7ba8fc96349fea0ed3a5c47992e3b4084b031a42264a018e0072e8172e46c`
- **Index**: `0x0`

**Testnet**
- **TX Hash**: `0xf8de3bb47d055cdf460d93a2a6e1b05f7432f9777c8c474abf4eec1d4aee5d37`
- **Index**: `0x0`

## Storage and File Systems

### CKBFS (CKB File System)

**Mainnet Deployments**
- **CKBFS Contract**: `0x31e6376287d223b8c0410d562fb422f04d1d617b2947596a14c3d2efb7218d3a`
- **Adler32 Contract**: `0x2138683f76944437c0c643664120d620bdb5858dd6c9d1d156805e279c2c536f`

**Type ID Example**
- **Type ID**: `0xbce89252cece632ef819943bed9cd0e2576f8ce26f9f02075b621b1c9a28056a`

## Liquid Staking

### iCKB Protocol

**Mainnet**
- **Transaction**: `0xd7309191381f5a8a2904b8a79958a9be2752dbba6871fa193dab6aeb29dc8f44`
- **Status**: Non-upgradable (zero lock)

**Testnet**
- **Transaction**: `0x9ac989b3355764f76cdce02c69dedb819fdfbcbda49a7db1a2c9facdfdb9a7fe`
- **Purpose**: Development and testing

## Authentication and Identity

### JoyID Lock Script

**Mainnet**
- **TX Hash**: `0xf05188e5f3a6767fc4687faf45ba5f1a6e25d3ada6129dae8722cb282f262493`
- **Index**: `0x0`

**Cell Dependency (Mainnet)**
- **TX Hash**: `0xf05188e5f3a6767fc4687faf45ba5f1a6e25d3ada6129dae8722cb282f262493`
- **Index**: `0x0`
- **Dep Type**: `dep_group`

**Testnet**
- **TX Hash**: `0x759f281588c96979764cb21c196478cf8e13ea81fede7f4ba26d1ff29dbc6a81`
- **Index**: `0x0`

**Cell Dependency (Testnet)**
- **TX Hash**: `0x759f281588c96979764cb21c196478cf8e13ea81fede7f4ba26d1ff29dbc6a81`
- **Index**: `0x0`
- **Dep Type**: `dep_group`

**Official Documentation**: [JoyID Mainnet Contract Upgrade](https://nervina.notion.site/JoyID-Mainnet-Contract-Upgrade-253c046a93fd801cac98fb793c1b3613)

## Special Purpose Scripts

### Type ID Script

**Pattern**: Used for unique type script identification
- **Code Hash**: Generated from first input outpoint + output index
- **Hash Type**: `type`
- **Args**: 32-byte calculated type ID

### Zero Lock (Always Fail)

**Pattern**: Used for code cells and data storage to ensure permanent locking
- **Code Hash**: `0x0000000000000000000000000000000000000000000000000000000000000000`
- **Hash Type**: `data`
- **Args**: `0x`
- **Note**: This lock always fails because no cell can have an all-zero data hash, making it impossible to unlock

## Common Cell Dependencies

### Standard Dependencies

**SECP256K1_BLAKE160 Dependency**
- **TX Hash**: `0x71a7ba8fc96349fea0ed3a5c47992e3b4084b031a42264a018e0072e8172e46c`
- **Index**: `0x0`
- **Dep Type**: `dep_group`

## Usage Notes

### Hash Type Values
- `type`: Upgradable smart contract using type script verification (most commonly Type ID system)
- `data`: Script identified by data hash using CKB VM v0 (Lina)
- `data1`: Script identified by data hash using CKB VM v1 (CKB2021 hardfork, Mirana)
- `data2`: Script identified by data hash using CKB VM v2 (CKB2023 hardfork, Meepo)

### Integration Guidelines

1. **Always verify hash values** against the latest network deployments
2. **Use Type ID hashes** for production deployments when available
3. **Check network compatibility** before using specific hashes
4. **Reference transaction hashes** for deployment verification

### Version Considerations

- Hashes from the reference pw-core constants (circa 2021) serve as the baseline
- Recent documentation takes precedence for conflicting values
- Always verify current deployments for production use

## References

- **Source**: [CKB MCP Documentation](ckb-dev-context://api-reference/well-known-hashes)
- **Historical Reference**: [pw-core constants.ts](https://raw.githubusercontent.com/jordanmack/pw-core/refs/heads/dev/src/constants.ts) (jordanmack/pw-core)
- **JoyID Documentation**: [JoyID Mainnet Contract Upgrade](https://nervina.notion.site/JoyID-Mainnet-Contract-Upgrade-253c046a93fd801cac98fb793c1b3613)
- **Network Status**: Check current CKB explorer for deployment verification