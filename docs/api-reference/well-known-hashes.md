# Well-Known Script Hashes

## Description

Script hashes, code hashes, transaction hashes, and cell dependency hashes for CKB mainnet and testnet. System scripts, Omnilock, xUDT, SUDT, Spore, CoTA, iCKB, JoyID, CKBFS, RGB++, Nostr Lock deployment transactions and cell dependencies.


## Script Structure Overview

### Script Definition

A CKB script consists of three essential components:

- **Code Hash**: The hash identifying the script code (either data hash or type script hash).
- **Hash Type**: Specifies how the code hash should be interpreted (`type`, `data`, `data1`, `data2`).
- **Args**: Additional parameters passed to the script (can be empty `0x` or contain specific data).

### Cell Dependencies

Every script requires at least one cell dependency to function. Cell dependencies specify where the script code is located on-chain:

- **TX Hash**: Transaction hash containing the script code cell.
- **Index**: Output index of the script code cell within the transaction.
- **Dep Type**: How the dependency should be loaded (`code`, `dep_group`).

Scripts cannot execute without their corresponding cell dependencies being included in the transaction's `cell_deps` field.

## System Scripts

### SECP256K1_BLAKE160 (Fallback Lock Script)

*Note: Sometimes called the default lock script.*

**Mainnet**
- **Code Hash**: `0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8`
- **Hash Type**: `type`
- **Args**: Contains the 20-byte Blake160 hash of the public key.

**Cell Dependency (Mainnet)**
- **TX Hash**: `0x71a7ba8fc96349fea0ed3a5c47992e3b4084b031a42264a018e0072e8172e46c`
- **Index**: `0x0`
- **Dep Type**: `dep_group`

**Testnet**
- **Code Hash**: `0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8`
- **Hash Type**: `type`
- **Args**: Contains the 20-byte Blake160 hash of the public key.

**Cell Dependency (Testnet)**
- **TX Hash**: `0xf8de3bb47d055cdf460d93a2a6e1b05f7432f9777c8c474abf4eec1d4aee5d37`
- **Index**: `0x0`
- **Dep Type**: `dep_group`

**Source**: [RFC 0024: CKB Genesis Script List](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0024-ckb-genesis-script-list/0024-ckb-genesis-script-list.md)

### SECP256K1_BLAKE160_MULTISIG (Multi-Signature Lock Script)

**Mainnet**
- **Code Hash**: `0x5c5069eb0857efc65e1bca0c07df34c31663b3622fd3876c876320fc9634e2a8`
- **Hash Type**: `type`
- **Args**: Contains multisig configuration and public key hashes.

**Cell Dependency (Mainnet)**
- **TX Hash**: `0x71a7ba8fc96349fea0ed3a5c47992e3b4084b031a42264a018e0072e8172e46c`
- **Index**: `0x1`
- **Dep Type**: `dep_group`

**Testnet**
- **Code Hash**: `0x5c5069eb0857efc65e1bca0c07df34c31663b3622fd3876c876320fc9634e2a8`
- **Hash Type**: `type`
- **Args**: Contains multisig configuration and public key hashes.

**Cell Dependency (Testnet)**
- **TX Hash**: `0xf8de3bb47d055cdf460d93a2a6e1b05f7432f9777c8c474abf4eec1d4aee5d37`
- **Index**: `0x1`
- **Dep Type**: `dep_group`

**Source**: [RFC 0024: CKB Genesis Script List](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0024-ckb-genesis-script-list/0024-ckb-genesis-script-list.md)

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

**Source**: [RFC 0024: CKB Genesis Script List](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0024-ckb-genesis-script-list/0024-ckb-genesis-script-list.md)

## Token Scripts

### SUDT (Simple User Defined Token)

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

### xUDT (Extensible User Defined Token)

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

## Universal Lock Scripts

### Omnilock

Omnilock is a universal lock script supporting multiple authentication methods including secp256k1, Ethereum, Bitcoin, Dogecoin, and more. It also includes an anyone-can-pay mode.

**Mainnet (Mirana)**
- **Code Hash**: `0x9b819793a64463aed77c615d6cb226eea5487ccfc0783043a587254cda2b6f26`
- **Hash Type**: `type`
- **Args**: 21-byte auth content followed by Omnilock args for mode flags.

**Cell Dependency (Mainnet)**
- **TX Hash**: `0xc76edf469816aa22f416503c38d0b533d2a018e253e379f134c3985b3472c842`
- **Index**: `0x0`
- **Dep Type**: `code`

**Testnet (Pudge)**
- **Code Hash**: `0xf329effd1c475a2978453c8600e1eaf0bc2087ee093c3ee64cc96ec6847752cb`
- **Hash Type**: `type`
- **Args**: 21-byte auth content followed by Omnilock args for mode flags.

**Cell Dependency (Testnet - RFC 0042)**
- **TX Hash**: `0x3d4296df1bd2cc2bd3f483f61ab7ebeac462a2f336f2b944168fe6ba5d81c014`
- **Index**: `0x0`
- **Dep Type**: `code`

**Cell Dependency (Testnet - Lumos)**
- **TX Hash**: `0xec18bf0d857c981c3d1f4e17999b9b90c484b303378e94de1a57b0872f5d4602`
- **Index**: `0x0`
- **Dep Type**: `code`

*Note: Multiple Omnilock deployments exist on testnet. The RFC 0042 deployment and CCC deployment use different transaction hashes. Both reference the same code hash.*

**Source**: [RFC 0042: Omnilock](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0042-omnilock/0042-omnilock.md)

### PW Lock Script (Deprecated)

PW Lock is a lock script for PW-SDK compatibility, enabling Ethereum-style authentication on CKB. PW-SDK is deprecated; use Omnilock for new Ethereum-compatible deployments.

**Mainnet**
- **Code Hash**: `0xbf43c3602455798c1a61a596e0d95278864c552fafe231c063b3fabf97a8febc`
- **Hash Type**: `type`
- **Args**: Contains Ethereum-style authentication data.

**Cell Dependency (Mainnet)**
- **TX Hash**: `0x1d60cb8f4666e039f418ea94730b1a8c5aa0bf2f7781474406387462924d15d4`
- **Index**: `0x0`
- **Dep Type**: `code`

**Testnet**
- **Code Hash**: `0x58c5f491aba6d61678b7cf7edf4910b1f5e00ec0cde2f42e0abb4fd9aff25a63`
- **Hash Type**: `type`
- **Args**: Contains Ethereum-style authentication data.

**Cell Dependency (Testnet)**
- **TX Hash**: `0x57a62003daeab9d54aa29b944fc3b451213a5ebdf2e232216a3cfed0dde61b38`
- **Index**: `0x0`
- **Dep Type**: `code`

**Source**: [pw-core constants.ts](https://github.com/jordanmack/pw-core/blob/dev/src/constants.ts)

### ACP (Anyone Can Pay) Lock Script

ACP is a lock script that allows anyone to transfer CKB or UDT tokens to a cell. The receiver can accept payments without signing. Note: Omnilock also supports an anyone-can-pay mode via its mode flags.

**Mainnet (Lina)**
- **Code Hash**: `0xd369597ff47f29fbc0d47d2e3775370d1250b85140c670e4718af712983a2354`
- **Hash Type**: `type`
- **Args**: 20-byte public key hash, optional minimum CKB/UDT amount.

**Cell Dependency (Mainnet)**
- **TX Hash**: `0x4153a2014952d7cac45f285ce9a7c5c0c0e1b21f2d378b82ac1433cb11c25c4d`
- **Index**: `0x0`
- **Dep Type**: `dep_group`

**Testnet (Aggron)**
- **Code Hash**: `0x3419a1c09eb2567f6552ee7a8ecffd64155cffe0f1796e6e61ec088d740c1356`
- **Hash Type**: `type`
- **Args**: 20-byte public key hash, optional minimum CKB/UDT amount.

**Cell Dependency (Testnet)**
- **TX Hash**: `0xec26b0f85ed839ece5f11c4c4e837ec359f5adc4420410f6453b1f6b60fb96a6`
- **Index**: `0x0`
- **Dep Type**: `dep_group`

**Source**: [RFC 0026: Anyone-Can-Pay Lock](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0026-anyone-can-pay/0026-anyone-can-pay.md)

## NFT and Digital Objects

### Spore Protocol

**Mainnet**
- **Code Hash**: `0x4a4dce1df3dffff7f8b2cd7dff7303df3b6150c9788cb75dcf6747247132b9f5`
- **Hash Type**: `data1`
- **Args**: Type ID or empty for standard Spore NFT deployment.

**Cell Dependency (Mainnet)**
- **TX Hash**: `0x96b198fb5ddbd1eed57ed667068f1f1e55d07907b4c0dbd38675a69ea1b69824`
- **Index**: `0x0`
- **Dep Type**: `code`

**Testnet**
- **Code Hash**: `0xbbad126377d45f90a8ee120da988a2d7332c78ba8fd679aab478a19d6c133494`
- **Hash Type**: `data1`
- **Args**: Type ID or empty for standard Spore NFT deployment.

**Cell Dependency (Testnet)**
- **TX Hash**: `0xfd694382e621f175ddf81ce91ce2ecf8bfc027d53d7d31b8438f7d26fc37fd19`
- **Index**: `0x0`
- **Dep Type**: `code`

**Source**: [Spore Docs - Contracts](https://docs.spore.pro/resources/contracts) (mainnet), [spore-sdk predefined.ts](https://github.com/sporeprotocol/spore-sdk/blob/main/packages/core/src/config/predefined.ts) (testnet)

### Cluster Protocol

**Mainnet**
- **Code Hash**: `0x7366a61534fa7c7e6225ecc0d828ea3b5366adec2b58206f2ee84995fe030075`
- **Hash Type**: `data1`
- **Args**: Type ID or empty for standard Cluster deployment.

**Cell Dependency (Mainnet)**
- **TX Hash**: `0xe464b7fb9311c5e2820e61c99afc615d6b98bdefbe318c34868c010cbd0dc938`
- **Index**: `0x0`
- **Dep Type**: `code`

**Testnet**
- **Code Hash**: `0x598d793defef36e2eeba54a9b45130e4ca92822e1d193671f490950c3b856080`
- **Hash Type**: `data1`
- **Args**: Type ID or empty for standard Cluster deployment.

**Cell Dependency (Testnet)**
- **TX Hash**: `0x49551a20dfe39231e7db49431d26c9c08ceec96a29024eef3acc936deeb2ca76`
- **Index**: `0x0`
- **Dep Type**: `code`

**Source**: [Spore Docs - Contracts](https://docs.spore.pro/resources/contracts) (mainnet), [spore-sdk predefined.ts](https://github.com/sporeprotocol/spore-sdk/blob/main/packages/core/src/config/predefined.ts) (testnet)

### CoTA (Compact Token Aggregator)

**Mainnet**
- **Code Hash**: `0x1122a4fb54697cf2e6e3a96c9d80fd398a936559b90954c6e88eb7ba0cf652df`
- **Hash Type**: `type`
- **Args**: `0x`

**Cell Dependency (Mainnet)**
- **TX Hash**: `0xabaa25237554f0d6c586dc010e7e85e6870bcfd9fb8773257ecacfbe1fd738a0`
- **Index**: `0x0`
- **Dep Type**: `dep_group`

**Testnet**
- **Code Hash**: `0x89cd8003a0eaf8e65e0c31525b7d1d5c1becefd2ea75bb4cff87810ae37764d8`
- **Hash Type**: `type`
- **Args**: `0x`

**Cell Dependency (Testnet)**
- **TX Hash**: `0x636a786001f87cb615acfcf408be0f9a1f077001f0bbc75ca54eadfe7e221713`
- **Index**: `0x0`
- **Dep Type**: `dep_group`

**Source**: [nervina-labs/cota-sdk-js constants](https://github.com/nervina-labs/cota-sdk-js/blob/develop/src/constants/index.ts)

### CoTA Registry

**Mainnet**
- **Code Hash**: `0x90ca618be6c15f5857d3cbd09f9f24ca6770af047ba9ee70989ec3b229419ac7`
- **Hash Type**: `type`
- **Args**: `0x563631b49cee549f3585ab4dde5f9d590f507f1f`

**Testnet**
- **Code Hash**: `0x9302db6cc1344b81a5efee06962abcb40427ecfcbe69d471b01b2658ed948075`
- **Hash Type**: `type`
- **Args**: `0xf9910364e0ca81a0e074f3aa42fe78cfcc880da6`

**Source**: [nervina-labs/cota-sdk-js constants](https://github.com/nervina-labs/cota-sdk-js/blob/develop/src/constants/index.ts)

## Storage and File Systems

### CKBFS (CKB File System)

CKBFS is a witnesses-based file storage protocol for CKB. Scripts can be referenced by code hash (hash_type `data1`) or by type ID (hash_type `type`).

#### CKBFS Script (Version 20241025)

**Mainnet**
- **Code Hash**: `0x31e6376287d223b8c0410d562fb422f04d1d617b2947596a14c3d2efb7218d3a`
- **Hash Type**: `data1`
- **Type ID**: `0xfd2058c9a0c0183354cf637e25d2707ffa9bb6fa2ba9b29f4ebc6be3e54ad7eb` (use with hash_type `type`)

**Cell Dependency (Mainnet)**
- **TX Hash**: `0xfab07962ed7178ed88d450774e2a6ecd50bae856bdb9b692980be8c5147d1bfa`
- **Index**: `0x0`
- **Dep Type**: `dep_group`

**Testnet**
- **Code Hash**: `0x31e6376287d223b8c0410d562fb422f04d1d617b2947596a14c3d2efb7218d3a`
- **Hash Type**: `data1`
- **Type ID**: `0x7c6dcab8268201f064dc8676b5eafa60ca2569e5c6209dcbab0eb64a9cb3aaa3` (use with hash_type `type`)

**Cell Dependency (Testnet)**
- **TX Hash**: `0x469af0d961dcaaedd872968a9388b546717a6ccfa47b3165b3f9c981e9d66aaa`
- **Index**: `0x0`
- **Dep Type**: `dep_group`

#### Adler32 Hasher Script (Version 20241025)

**Mainnet**
- **Code Hash**: `0x2138683f76944437c0c643664120d620bdb5858dd6c9d1d156805e279c2c536f`
- **Hash Type**: `data1`
- **Type ID**: `0x641c01d590833a3f5471bd441651d9f2a8a200141949cdfeef2d68d8094c5876` (use with hash_type `type`)

**Testnet**
- **Code Hash**: `0x2138683f76944437c0c643664120d620bdb5858dd6c9d1d156805e279c2c536f`
- **Hash Type**: `data1`
- **Type ID**: `0x5f73f128be76e397f5a3b56c94ca16883a8ee91b498bc0ee80473818318c05ac` (use with hash_type `type`)

**Source**: [CKBFS Repository](https://github.com/nervape/ckbfs)

## Liquid Staking

### iCKB Protocol

iCKB is a liquid staking protocol that tokenizes NervosDAO deposits into transferable iCKB tokens. All scripts are deployed non-upgradably with a zero lock.

#### iCKB Logic Script

Used for iCKB token minting and burning logic.

**Mainnet & Testnet** (same code hash)
- **Code Hash**: `0x2a8100ab5990fa055ab1b50891702e1e895c7bd1df6322cd725c1a6115873bd3`
- **Hash Type**: `data1`

**Cell Dependency (Mainnet)**
- **TX Hash**: `0x621a6f38de3b9f453016780edac3b26bfcbfa3e2ecb47c2da275471a5d3ed165`
- **Index**: `0x0`
- **Dep Type**: `dep_group`

**Cell Dependency (Testnet)**
- **TX Hash**: `0xf7ece4fb33d8378344cab11fcd6a4c6f382fd4207ac921cf5821f30712dcd311`
- **Index**: `0x0`
- **Dep Type**: `dep_group`

#### Limit Order Script

Used for iCKB limit order matching.

**Mainnet & Testnet** (same code hash)
- **Code Hash**: `0x49dfb6afee5cc8ac4225aeea8cb8928b150caf3cd92fea33750683c74b13254a`
- **Hash Type**: `data1`

**Cell Dependency (Mainnet)**
- **TX Hash**: `0x621a6f38de3b9f453016780edac3b26bfcbfa3e2ecb47c2da275471a5d3ed165`
- **Index**: `0x0`
- **Dep Type**: `dep_group`

**Cell Dependency (Testnet)**
- **TX Hash**: `0xf7ece4fb33d8378344cab11fcd6a4c6f382fd4207ac921cf5821f30712dcd311`
- **Index**: `0x0`
- **Dep Type**: `dep_group`

#### Owned-Owner Script

Used for ownership verification in iCKB operations.

**Mainnet & Testnet** (same code hash)
- **Code Hash**: `0xacc79e07d107831feef4c70c9e683dac5644d5993b9cb106dca6e74baa381bd0`
- **Hash Type**: `data1`

**Cell Dependency (Mainnet)**
- **TX Hash**: `0x621a6f38de3b9f453016780edac3b26bfcbfa3e2ecb47c2da275471a5d3ed165`
- **Index**: `0x0`
- **Dep Type**: `dep_group`

**Cell Dependency (Testnet)**
- **TX Hash**: `0xf7ece4fb33d8378344cab11fcd6a4c6f382fd4207ac921cf5821f30712dcd311`
- **Index**: `0x0`
- **Dep Type**: `dep_group`

#### iCKB xUDT Type Script

The iCKB token itself, implemented as an xUDT with specific args encoding the iCKB Logic script hash.

**Mainnet & Testnet** (same values)
- **Code Hash**: `0x50bd8d6680b8b9cf98b73f3c08faf8b2a21914311954118ad6609be6e78a1b95`
- **Hash Type**: `data1`
- **Args**: `0xb73b6ab39d79390c6de90a09c96b290c331baf1798ed6f97aed02590929734e800000080`

**Cell Dependency (Mainnet)**
- **TX Hash**: `0x621a6f38de3b9f453016780edac3b26bfcbfa3e2ecb47c2da275471a5d3ed165`
- **Index**: `0x0`
- **Dep Type**: `dep_group`

**Cell Dependency (Testnet)**
- **TX Hash**: `0xf7ece4fb33d8378344cab11fcd6a4c6f382fd4207ac921cf5821f30712dcd311`
- **Index**: `0x0`
- **Dep Type**: `dep_group`

*Note: The args encode the iCKB Logic script hash plus extension flags. This xUDT references the standard xUDT code hash with data1 hash type.*

**Source**: [iCKB Whitepaper](https://github.com/ickb/whitepaper)

## Authentication and Identity

### JoyID Lock Script

JoyID is a passwordless authentication solution using WebAuthn and passkeys for CKB.

**Mainnet**
- **Code Hash**: `0xd00c84f0ec8fd441c38bc3f87a371f547190f2fcff88e642bc5bf54b9e318323`
- **Hash Type**: `type`
- **Args**: Contains JoyID-specific authentication parameters.

**Cell Dependency (Mainnet)**
- **TX Hash**: `0xf05188e5f3a6767fc4687faf45ba5f1a6e25d3ada6129dae8722cb282f262493`
- **Index**: `0x0`
- **Dep Type**: `dep_group`

**Testnet**
- **Code Hash**: `0xd23761b364210735c19c60561d213fb3beae2fd6172743719eff6920e020baac`
- **Hash Type**: `type`
- **Args**: Contains JoyID-specific authentication parameters.

**Cell Dependency (Testnet)**
- **TX Hash**: `0x4dcf3f3b09efac8995d6cbee87c5345e812d310094651e0c3d9a730f32dc9263`
- **Index**: `0x0`
- **Dep Type**: `dep_group`

**Source**: [JoyID Smart Contract Docs](https://docs.joyid.dev/guide/ckb/smart-contract)

### Nostr Lock Script

Nostr Lock enables CKB interoperability with the Nostr protocol, supporting schnorr signature verification and optional proof-of-work mechanics.

**Mainnet**
- **Code Hash**: `0x641a89ad2f77721b803cd50d01351c1f308444072d5fa20088567196c0574c68`
- **Hash Type**: `type`
- **Args**: 1-byte PoW difficulty + 20-byte schnorr pubkey hash (blake160 of 32-byte pubkey).

**Cell Dependency (Mainnet)**
- **TX Hash**: `0x1911208b136957d5f7c1708a8835edfe8ae1d02700d5cb2c3a6aacf4d5906306`
- **Index**: `0x0`
- **Dep Type**: `code`

**Testnet**
- **Code Hash**: `0x6ae5ee0cb887b2df5a9a18137315b9bdc55be8d52637b2de0624092d5f0c91d5`
- **Hash Type**: `type`
- **Args**: 1-byte PoW difficulty + 20-byte schnorr pubkey hash (blake160 of 32-byte pubkey).

**Cell Dependency (Testnet)**
- **TX Hash**: `0xa2a434dcdbe280b9ed75bb7d6c7d68186a842456aba0fc506657dc5ed7c01d68`
- **Index**: `0x0`
- **Dep Type**: `code`

**Source**: [Nostr Lock Script Specification](https://github.com/cryptape/nostr-binding/blob/main/docs/nostr-lock-script.md)

## Bitcoin Interoperability

### RGB++ Lock Script

RGB++ enables isomorphic binding between Bitcoin UTXOs and CKB cells, allowing Bitcoin assets to leverage CKB's smart contract capabilities.

**Mainnet**
- **Code Hash**: `0xbc6c568a1a0d0a09f6844dc9d74ddb4343c32143ff25f727c59edf4fb72d6936`
- **Hash Type**: `type`
- **Args**: Contains Bitcoin UTXO binding information.

**Cell Dependency (Mainnet)**
- **TX Hash**: `0xcb4d9f9726e66306bfda6359d39d3bea8b4e5345d0f95f26a3e51626ebe82a63`
- **Index**: `0x0`
- **Dep Type**: `code`

**RGB++ Config Cell Dependency (Mainnet)**
- **TX Hash**: `0xcb4d9f9726e66306bfda6359d39d3bea8b4e5345d0f95f26a3e51626ebe82a63`
- **Index**: `0x1`
- **Dep Type**: `code`

**Testnet**
- **Code Hash**: `0x61ca7a4796a4eb19ca4f0d065cb9b10ddcf002f10f7cbb810c706cb6bb5c3248`
- **Hash Type**: `type`
- **Args**: Contains Bitcoin UTXO binding information.

**Cell Dependency (Testnet)**
- **TX Hash**: `0x0d1567da0979f78b297d5311442669fbd1bd853c8be324c5ab6da41e7a1ed6e5`
- **Index**: `0x0`
- **Dep Type**: `code`

**RGB++ Config Cell Dependency (Testnet)**
- **TX Hash**: `0x0d1567da0979f78b297d5311442669fbd1bd853c8be324c5ab6da41e7a1ed6e5`
- **Index**: `0x1`
- **Dep Type**: `code`

**Source**: [RGB++ SDK Constants](https://github.com/RGBPlusPlus/rgbpp-sdk/blob/main/packages/ckb/src/constants/index.ts)

### BTC Time Lock Script

BTC Time Lock enforces a block confirmation waiting period when assets transfer from Bitcoin to CKB, providing security for cross-layer transactions.

**Mainnet**
- **Code Hash**: `0x70d64497a075bd651e98ac030455ea200637ee325a12ad08aff03f1a117e5a62`
- **Hash Type**: `type`
- **Args**: Contains time lock parameters and Bitcoin transaction reference.

**Cell Dependency (Mainnet)**
- **TX Hash**: `0x3d1c26b966504b09253ad84173bf3baa7b8135c5ff520c32cf70b631c1d08b9b`
- **Index**: `0x0`
- **Dep Type**: `code`

**BTC Time Lock Config Cell Dependency (Mainnet)**
- **TX Hash**: `0x3d1c26b966504b09253ad84173bf3baa7b8135c5ff520c32cf70b631c1d08b9b`
- **Index**: `0x1`
- **Dep Type**: `code`

**Testnet**
- **Code Hash**: `0x00cdf8fab0f8ac638758ebf5ea5e4052b1d71e8a77b9f43139718621f6849326`
- **Hash Type**: `type`
- **Args**: Contains time lock parameters and Bitcoin transaction reference.

**Cell Dependency (Testnet)**
- **TX Hash**: `0x8fb747ff0416a43e135c583b028f98c7b81d3770551b196eb7ba1062dd9acc94`
- **Index**: `0x0`
- **Dep Type**: `code`

**BTC Time Lock Config Cell Dependency (Testnet)**
- **TX Hash**: `0x8fb747ff0416a43e135c583b028f98c7b81d3770551b196eb7ba1062dd9acc94`
- **Index**: `0x1`
- **Dep Type**: `code`

**Source**: [RGB++ SDK Constants](https://github.com/RGBPlusPlus/rgbpp-sdk/blob/main/packages/ckb/src/constants/index.ts)

## Special Purpose Scripts

### Type ID Script

Type ID is a built-in system script that enables upgradable smart contracts by providing a unique, persistent identifier for cells regardless of their data content.

**Mainnet & Testnet** (same on both networks)
- **Code Hash**: `0x00000000000000000000000000000000000000000000000000545950455f4944`
- **Hash Type**: `type`
- **Args**: 32-byte type ID, calculated from first input outpoint + output index.

*Note: The code hash is a fixed value (ASCII hex for "TYPE_ID" padded with zeros). The args field contains the calculated type ID that uniquely identifies a cell across upgrades.*

**Source**: [RFC 0024: CKB Genesis Script List](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0024-ckb-genesis-script-list/0024-ckb-genesis-script-list.md)

### Always Success Lock Script

Always Success is a simple lock script that always returns success (exit code 0). Used for testing and development purposes.

*Note: These are community deployments without official documentation. Verify on-chain before production use.*

**Mainnet (Unverified Community Deployment)**
- **Code Hash**: `0xd483925160e4232b2cb29f012e8380b7b612d71cf4e79991476b6bcf610735f6`
- **Hash Type**: `data`
- **Args**: `0x`

**Cell Dependency (Mainnet)**
- **TX Hash**: `0x81e22f4bb39080b112e5efb18e3fad65ebea735eac2f9c495b7f4d3b4faa377d`
- **Index**: `0x0`
- **Dep Type**: `code`

**Testnet (Unverified Community Deployment)**
- **Code Hash**: `0x1157470ca9de091c21c262bf0754b777f3529e10d2728db8f6b4e04cfc2fbb5f`
- **Hash Type**: `data`
- **Args**: `0x`

**Cell Dependency (Testnet)**
- **TX Hash**: `0x81e22f4bb39080b112e5efb18e3fad65ebea735eac2f9c495b7f4d3b4faa377d`
- **Index**: `0x0`
- **Dep Type**: `code`

### Zero Lock (Always Fail)

**Pattern**: Used for code cells and data storage to ensure permanent locking
- **Code Hash**: `0x0000000000000000000000000000000000000000000000000000000000000000`
- **Hash Type**: `data`
- **Args**: `0x`
- **Note**: This lock always fails because no cell can have an all-zero data hash, making it impossible to unlock.

### SECP256K1_BLAKE160_MULTISIG V2 (Testnet Only)

Updated version of the multi-signature lock script using CKB VM v1.

**Testnet**
- **Code Hash**: `0x36c971b8d41fbd94aabca77dc75e826729ac98447b46f91e00796155dddb0d29`
- **Hash Type**: `data1`
- **Args**: Contains multisig configuration and public key hashes.

**Cell Dependency (Testnet)**
- **TX Hash**: `0x2eefdeb21f3a3edf697c28a52601b4419806ed60bb427420455cc29a090b26d5`
- **Index**: `0x0`
- **Dep Type**: `dep_group`

**Source**: [CCC Testnet Config](https://github.com/ckb-ecofund/ccc/blob/master/packages/core/src/client/clientPublicTestnet.advanced.ts)

### Unique Type Script (Testnet Only)

Ensures a cell with this type script is unique on-chain, useful for singleton patterns.

**Testnet**
- **Code Hash**: `0x8e341bcfec6393dcd41e635733ff2dca00a6af546949f70c57a706c0f344df8b`
- **Hash Type**: `type`
- **Args**: Type script args for uniqueness verification.

**Cell Dependency (Testnet)**
- **TX Hash**: `0xff91b063c78ed06f10a1ed436122bd7d671f9a72ef5f5fa28d05252c17cf4cef`
- **Index**: `0x0`
- **Dep Type**: `code`

**Source**: [CCC Testnet Config](https://github.com/ckb-ecofund/ccc/blob/master/packages/core/src/client/clientPublicTestnet.advanced.ts)

### Single Use Lock (Testnet Only)

A lock script that can only be unlocked once, useful for one-time authorization patterns.

**Testnet**
- **Code Hash**: `0x8290467a512e5b9a6b816469b0edabba1f4ac474e28ffdd604c2a7c76446bbaf`
- **Hash Type**: `data1`
- **Args**: `0x`

**Cell Dependency (Testnet)**
- **TX Hash**: `0xb4f171c9c9caf7401f54a8e56225ae21d95032150a87a4678eac3f66a3137b93`
- **Index**: `0x4`
- **Dep Type**: `code`

**Source**: [CCC Testnet Config](https://github.com/ckb-ecofund/ccc/blob/master/packages/core/src/client/clientPublicTestnet.advanced.ts)

### Time Lock (Testnet Only)

A lock script that enforces time-based unlock conditions.

**Testnet**
- **Code Hash**: `0x6fac4b2e89360a1e692efcddcb3a28656d8446549fb83da6d896db8b714f4451`
- **Hash Type**: `data1`
- **Args**: Contains time lock parameters.

**Cell Dependency (Testnet)**
- **TX Hash**: `0x1b4ffcad55ecd36ffb2715b6816b83da73851f1a24fe594f263c4f34dad90792`
- **Index**: `0x1`
- **Dep Type**: `code`

**Source**: [CCC Testnet Config](https://github.com/ckb-ecofund/ccc/blob/master/packages/core/src/client/clientPublicTestnet.advanced.ts)

## Usage Notes

### Hash Type Values
- `type`: Upgradable smart contract using type script verification (most commonly Type ID system).
- `data`: Script identified by data hash using CKB VM v0 (Lina).
- `data1`: Script identified by data hash using CKB VM v1 (CKB2021 hardfork, Mirana).
- `data2`: Script identified by data hash using CKB VM v2 (CKB2023 hardfork, Meepo).

### Integration Guidelines

1. **Always verify hash values** against the latest network deployments.
2. **Use Type ID hashes** for production deployments when available.
3. **Check network compatibility** before using specific hashes.
4. **Reference transaction hashes** for deployment verification.

### Version Considerations

- CCC SDK configurations serve as the primary reference for current deployments.
- RFCs serve as the authoritative source for protocol specifications.
- Always verify current deployments for production use.

## References

### Official RFCs

- [RFC 0024: CKB Genesis Script List](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0024-ckb-genesis-script-list/0024-ckb-genesis-script-list.md) - SECP256K1, Multisig, DAO, Type ID
- [RFC 0025: Simple UDT](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0025-simple-udt/0025-simple-udt.md) - SUDT
- [RFC 0026: Anyone-Can-Pay](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0026-anyone-can-pay/0026-anyone-can-pay.md) - ACP
- [RFC 0042: Omnilock](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0042-omnilock/0042-omnilock.md) - Omnilock
- [RFC 0052: Extensible UDT](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0052-extensible-udt/0052-extensible-udt.md) - xUDT

### SDK References

- [CCC Mainnet Config](https://github.com/ckb-ecofund/ccc/blob/master/packages/core/src/client/clientPublicMainnet.advanced.ts) - Primary SDK mainnet configurations
- [CCC Testnet Config](https://github.com/ckb-ecofund/ccc/blob/master/packages/core/src/client/clientPublicTestnet.advanced.ts) - Primary SDK testnet configurations
- [Lumos Config Manager](https://github.com/ckb-js/lumos/blob/develop/packages/config-manager/src/predefined.ts) - Legacy SDK configurations (deprecated)

### Verification

Always verify hash values against the CKB explorer or by querying the chain directly before production use.