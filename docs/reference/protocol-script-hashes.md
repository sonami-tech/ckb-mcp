## Description

Code hashes, type IDs, and cell dependencies for CKB protocol scripts on mainnet and testnet. Spore (NFT/digital objects), Cluster, CoTA (compact token aggregator), CKBFS (file storage), iCKB (NervosDAO liquidity), RGB++ (Bitcoin interoperability), BTC Time Lock, and special purpose scripts (Type ID, Always Success, Zero Lock, proxy locks).

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

**Source**: [spore-contract VERSIONS.md](https://github.com/sporeprotocol/spore-contract/blob/master/docs/VERSIONS.md)

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
- **Code Hash**: `0x0bbe768b519d8ea7b96d58f1182eb7e6ef96c541fbd9526975077ee09f049058`
- **Hash Type**: `data1`
- **Args**: Type ID or empty for standard Cluster deployment.

**Cell Dependency (Testnet)**
- **TX Hash**: `0x49551a20dfe39231e7db49431d26c9c08ceec96a29024eef3acc936deeb2ca76`
- **Index**: `0x0`
- **Dep Type**: `code`

**Source**: [spore-contract VERSIONS.md](https://github.com/sporeprotocol/spore-contract/blob/master/docs/VERSIONS.md)

### CoTA (Compact Token Aggregator)

**Mainnet**
- **Code Hash**: `0x1122a4fb54697cf2e6e3a96c9d80fd398a936559b90954c6e88eb7ba0cf652df`
- **Hash Type**: `type`
- **Type ID**: `0x8b8c859723698f5fd38372b6eadb8f1b4aaa823169baec7c203ed9b269953f0b`
- **Args**: `0x`

**Cell Dependency (Mainnet)**
- **TX Hash**: `0xabaa25237554f0d6c586dc010e7e85e6870bcfd9fb8773257ecacfbe1fd738a0`
- **Index**: `0x0`
- **Dep Type**: `dep_group` (3 cells)

**Testnet**
- **Code Hash**: `0x89cd8003a0eaf8e65e0c31525b7d1d5c1becefd2ea75bb4cff87810ae37764d8`
- **Hash Type**: `type`
- **Args**: `0x`

**Cell Dependency (Testnet)**
- **TX Hash**: `0x636a786001f87cb615acfcf408be0f9a1f077001f0bbc75ca54eadfe7e221713`
- **Index**: `0x0`
- **Dep Type**: `dep_group` (3 cells)

**Source**: [nervina-labs/cota-sdk-js constants](https://github.com/nervina-labs/cota-sdk-js/blob/develop/src/constants/index.ts)

### CoTA Registry

**Mainnet**
- **Code Hash**: `0x90ca618be6c15f5857d3cbd09f9f24ca6770af047ba9ee70989ec3b229419ac7`
- **Hash Type**: `type`
- **Type ID**: `0xf89559d113f2814d62f087e155c767e5297967aabc6ffe918c34d8e00442c19c`
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
- **Type ID Args**: `0x4a8629bac7f1d135dc13c33596e29c6fd5ccfca8043a1546929438291fbd6e36`

**Cell Dependency (Mainnet)**
- **TX Hash**: `0xfab07962ed7178ed88d450774e2a6ecd50bae856bdb9b692980be8c5147d1bfa`
- **Index**: `0x0`
- **Dep Type**: `dep_group` (2 cells)

**Testnet**
- **Code Hash**: `0x31e6376287d223b8c0410d562fb422f04d1d617b2947596a14c3d2efb7218d3a`
- **Hash Type**: `data1`
- **Type ID**: `0x7c6dcab8268201f064dc8676b5eafa60ca2569e5c6209dcbab0eb64a9cb3aaa3` (use with hash_type `type`)

**Cell Dependency (Testnet)**
- **TX Hash**: `0x469af0d961dcaaedd872968a9388b546717a6ccfa47b3165b3f9c981e9d66aaa`
- **Index**: `0x0`
- **Dep Type**: `dep_group` (2 cells)

#### Adler32 Hasher Script (Version 20241025)

**Mainnet**
- **Code Hash**: `0x2138683f76944437c0c643664120d620bdb5858dd6c9d1d156805e279c2c536f`
- **Hash Type**: `data1`
- **Type ID**: `0x641c01d590833a3f5471bd441651d9f2a8a200141949cdfeef2d68d8094c5876` (use with hash_type `type`)
- **Type ID Args**: `0xb8c16cf1dc255118176787b580c5c5cd8c327d5667cf3421b34bff05ffe77e7d`

**Testnet**
- **Code Hash**: `0x2138683f76944437c0c643664120d620bdb5858dd6c9d1d156805e279c2c536f`
- **Hash Type**: `data1`
- **Type ID**: `0x5f73f128be76e397f5a3b56c94ca16883a8ee91b498bc0ee80473818318c05ac` (use with hash_type `type`)

**Source**: [CKBFS README.md](https://github.com/nervape/ckbfs/blob/master/README.md)

## NervosDAO Liquidity

### iCKB Protocol

iCKB is a NervosDAO liquidity protocol that tokenizes DAO deposits into transferable iCKB tokens. All scripts use `data1` hash type (immutable) for security - no entity can upgrade scripts to steal deposited funds.

**Shared Cell Dependency (all iCKB scripts)**
- **Mainnet TX Hash**: `0x621a6f38de3b9f453016780edac3b26bfcbfa3e2ecb47c2da275471a5d3ed165`
- **Testnet TX Hash**: `0xf7ece4fb33d8378344cab11fcd6a4c6f382fd4207ac921cf5821f30712dcd311`
- **Index**: `0x0`, **Dep Type**: `dep_group` (8 cells: 3 iCKB scripts + xUDT + 4 genesis scripts)

#### iCKB Logic Script

- **Code Hash**: `0x2a8100ab5990fa055ab1b50891702e1e895c7bd1df6322cd725c1a6115873bd3`
- **Hash Type**: `data1`

#### Limit Order Script

- **Code Hash**: `0x49dfb6afee5cc8ac4225aeea8cb8928b150caf3cd92fea33750683c74b13254a`
- **Hash Type**: `data1`

#### Owned-Owner Script

- **Code Hash**: `0xacc79e07d107831feef4c70c9e683dac5644d5993b9cb106dca6e74baa381bd0`
- **Hash Type**: `data1`

#### iCKB xUDT Type Script

- **Code Hash**: `0x50bd8d6680b8b9cf98b73f3c08faf8b2a21914311954118ad6609be6e78a1b95`
- **Hash Type**: `data1`
- **Args**: `0xb73b6ab39d79390c6de90a09c96b290c331baf1798ed6f97aed02590929734e800000080`

*Note: Args encode the iCKB Logic script hash plus extension flags.*

**Source**: [iCKB Whitepaper - Deployment](https://github.com/ickb/whitepaper#mainnet-deployment)

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

*Note: Multiple community deployments exist. Choose based on your VM version requirements.*

#### CoTA SDK Deployment (CKB VM v0)

**Mainnet**
- **Code Hash**: `0xd483925160e4232b2cb29f012e8380b7b612d71cf4e79991476b6bcf610735f6`
- **Hash Type**: `data`
- **Args**: `0x`

**Cell Dependency (Mainnet)**
- **TX Hash**: `0x81e22f4bb39080b112e5efb18e3fad65ebea735eac2f9c495b7f4d3b4faa377d`
- **Index**: `0x0`
- **Dep Type**: `code`

**Testnet**
- **Code Hash**: `0x1157470ca9de091c21c262bf0754b777f3529e10d2728db8f6b4e04cfc2fbb5f`
- **Hash Type**: `data`
- **Args**: `0x`

**Cell Dependency (Testnet)**
- **TX Hash**: `0x81e22f4bb39080b112e5efb18e3fad65ebea735eac2f9c495b7f4d3b4faa377d`
- **Index**: `0x0`
- **Dep Type**: `code`

**Source**: [CoTA SDK Constants](https://github.com/nervina-labs/cota-sdk-js/blob/develop/src/constants/index.ts)

#### CCC SDK Deployment (CKB VM v1, Recommended)

**Mainnet & Testnet** (same code hash)
- **Code Hash**: `0x3b521cc4b552f109d092d8cc468a8048acb53c5952dbe769d2b2f9cf6e47f7f1`
- **Hash Type**: `data1`
- **Args**: `0x`

**Cell Dependency (Mainnet)**
- **TX Hash**: `0x10d63a996157d32c01078058000052674ca58d15f921bec7f1dcdac2160eb66b`
- **Index**: `0x0`
- **Dep Type**: `code`

**Cell Dependency (Testnet)**
- **TX Hash**: `0xb4f171c9c9caf7401f54a8e56225ae21d95032150a87a4678eac3f66a3137b93`
- **Index**: `0x0`
- **Dep Type**: `code`

**Source**: [CCC SDK Mainnet Config](https://github.com/ckb-ecofund/ccc/blob/master/packages/core/src/client/clientPublicMainnet.advanced.ts)

### Zero Lock (Always Fail)

**Pattern**: Used for code cells and data storage to ensure permanent locking
- **Code Hash**: `0x0000000000000000000000000000000000000000000000000000000000000000`
- **Hash Type**: `data`
- **Args**: `0x`
- **Note**: This lock always fails because no cell can have an all-zero data hash, making it impossible to unlock.

### SECP256K1_BLAKE160_MULTISIG V2

Updated version of the multi-signature lock script using CKB VM v1.

**Mainnet & Testnet** (same code hash)
- **Code Hash**: `0x36c971b8d41fbd94aabca77dc75e826729ac98447b46f91e00796155dddb0d29`
- **Hash Type**: `data1`
- **Args**: Contains multisig configuration and public key hashes.

**Cell Dependency (Mainnet)**
- **TX Hash**: `0x6888aa39ab30c570c2c30d9d5684d3769bf77265a7973211a3c087fe8efbf738`
- **Index**: `0x0`
- **Dep Type**: `dep_group`

**Cell Dependency (Testnet)**
- **TX Hash**: `0x2eefdeb21f3a3edf697c28a52601b4419806ed60bb427420455cc29a090b26d5`
- **Index**: `0x0`
- **Dep Type**: `dep_group`

**Source**: [CCC SDK](https://github.com/ckb-ecofund/ccc)

### Unique Type Script

Ensures a cell with this type script is unique on-chain, useful for singleton patterns.

**Mainnet (data1, immutable)**
- **Code Hash**: `0x2c8c11c985da60b0a330c61a85507416d6382c130ba67f0c47ab071e00aec628`
- **Hash Type**: `data1`
- **Args**: Type script args for uniqueness verification.

**Cell Dependency (Mainnet)**
- **TX Hash**: `0x67524c01c0cb5492e499c7c7e406f2f9d823e162d6b0cf432eacde0c9808c2ad`
- **Index**: `0x0`
- **Dep Type**: `code`

**Testnet (type, upgradable)**
- **Code Hash**: `0x8e341bcfec6393dcd41e635733ff2dca00a6af546949f70c57a706c0f344df8b`
- **Hash Type**: `type`
- **Args**: Type script args for uniqueness verification.

**Cell Dependency (Testnet)**
- **TX Hash**: `0xff91b063c78ed06f10a1ed436122bd7d671f9a72ef5f5fa28d05252c17cf4cef`
- **Index**: `0x0`
- **Dep Type**: `code`

*Note: Mainnet uses `data1` (immutable deployment), testnet uses `type` (upgradable via Type ID). Different code hashes due to different hash types.*

**Source**: [CCC SDK](https://github.com/ckb-ecofund/ccc)

### Single Use Lock

A lock script that can only be unlocked once, useful for one-time authorization patterns.

**Mainnet & Testnet** (same code hash)
- **Code Hash**: `0x8290467a512e5b9a6b816469b0edabba1f4ac474e28ffdd604c2a7c76446bbaf`
- **Hash Type**: `data1`
- **Args**: `0x`

**Cell Dependency (Mainnet)**
- **TX Hash**: `0x10d63a996157d32c01078058000052674ca58d15f921bec7f1dcdac2160eb66b`
- **Index**: `0x4`
- **Dep Type**: `code`

**Cell Dependency (Testnet)**
- **TX Hash**: `0xb4f171c9c9caf7401f54a8e56225ae21d95032150a87a4678eac3f66a3137b93`
- **Index**: `0x4`
- **Dep Type**: `code`

**Source**: [CCC SDK](https://github.com/ckb-ecofund/ccc)

### Time Lock

A lock script that enforces time-based unlock conditions.

**Mainnet & Testnet** (same code hash)
- **Code Hash**: `0x6fac4b2e89360a1e692efcddcb3a28656d8446549fb83da6d896db8b714f4451`
- **Hash Type**: `data1`
- **Args**: Contains time lock parameters.

**Cell Dependency (Mainnet)**
- **TX Hash**: `0xb0ed754fb27d67fd8388c97fed914fb7998eceaa01f3e6f967e498de1ba0ac9b`
- **Index**: `0x1`
- **Dep Type**: `code`

**Cell Dependency (Testnet)**
- **TX Hash**: `0x1b4ffcad55ecd36ffb2715b6816b83da73851f1a24fe594f263c4f34dad90792`
- **Index**: `0x1`
- **Dep Type**: `code`

**Source**: [CCC SDK](https://github.com/ckb-ecofund/ccc)

### Input Type Proxy Lock

A lock script that delegates authorization to the type script of an input cell. Useful for scripts that need to authorize based on input type.

**Mainnet & Testnet** (same code hash)
- **Code Hash**: `0x5123908965c711b0ffd8aec642f1ede329649bda1ebdca6bd24124d3796f768a`
- **Hash Type**: `data1`
- **Args**: `0x`

**Cell Dependency (Mainnet)**
- **TX Hash**: `0x10d63a996157d32c01078058000052674ca58d15f921bec7f1dcdac2160eb66b`
- **Index**: `0x1`
- **Dep Type**: `code`

**Cell Dependency (Testnet)**
- **TX Hash**: `0xb4f171c9c9caf7401f54a8e56225ae21d95032150a87a4678eac3f66a3137b93`
- **Index**: `0x1`
- **Dep Type**: `code`

**Source**: [CCC SDK](https://github.com/ckb-ecofund/ccc)

### Output Type Proxy Lock

A lock script that delegates authorization to the type script of an output cell. Useful for scripts that need to authorize based on output type.

**Mainnet & Testnet** (same code hash)
- **Code Hash**: `0x2df53b592db3ae3685b7787adcfef0332a611edb83ca3feca435809964c3aff2`
- **Hash Type**: `data1`
- **Args**: `0x`

**Cell Dependency (Mainnet)**
- **TX Hash**: `0x10d63a996157d32c01078058000052674ca58d15f921bec7f1dcdac2160eb66b`
- **Index**: `0x2`
- **Dep Type**: `code`

**Cell Dependency (Testnet)**
- **TX Hash**: `0xb4f171c9c9caf7401f54a8e56225ae21d95032150a87a4678eac3f66a3137b93`
- **Index**: `0x2`
- **Dep Type**: `code`

**Source**: [CCC SDK](https://github.com/ckb-ecofund/ccc)

### Lock Proxy Lock

A lock script that delegates authorization to another lock script. Useful for meta-lock patterns.

**Mainnet & Testnet** (same code hash)
- **Code Hash**: `0x5d41e32e224c15f152b7e6529100ebeac83b162f5f692a5365774dad2c1a1d02`
- **Hash Type**: `data1`
- **Args**: `0x`

**Cell Dependency (Mainnet)**
- **TX Hash**: `0x10d63a996157d32c01078058000052674ca58d15f921bec7f1dcdac2160eb66b`
- **Index**: `0x3`
- **Dep Type**: `code`

**Cell Dependency (Testnet)**
- **TX Hash**: `0xb4f171c9c9caf7401f54a8e56225ae21d95032150a87a4678eac3f66a3137b93`
- **Index**: `0x3`
- **Dep Type**: `code`

**Source**: [CCC SDK](https://github.com/ckb-ecofund/ccc)

### Type Burn Lock

A lock script that requires the type script to be "burned" (removed) for the cell to be unlocked. Useful for token burning patterns.

**Mainnet & Testnet** (same code hash)
- **Code Hash**: `0xff78bae0abf17d7a404c0be0f9ad9c9185b3f88dcc60403453d5ba8e1f22f53a`
- **Hash Type**: `data1`
- **Args**: `0x`

**Cell Dependency (Mainnet)**
- **TX Hash**: `0x10d63a996157d32c01078058000052674ca58d15f921bec7f1dcdac2160eb66b`
- **Index**: `0x5`
- **Dep Type**: `code`

**Cell Dependency (Testnet)**
- **TX Hash**: `0xb4f171c9c9caf7401f54a8e56225ae21d95032150a87a4678eac3f66a3137b93`
- **Index**: `0x5`
- **Dep Type**: `code`

**Source**: [CCC SDK](https://github.com/ckb-ecofund/ccc)

### Easy To Discover Type

A type script that makes cells easily discoverable on-chain by embedding identifying information.

**Mainnet & Testnet** (same code hash)
- **Code Hash**: `0xaba4430cc7110d699007095430a1faa72973edf2322ddbfd4d1d219cacf237af`
- **Hash Type**: `data1`
- **Args**: Discovery parameters.

**Cell Dependency (Mainnet)**
- **TX Hash**: `0xb0ed754fb27d67fd8388c97fed914fb7998eceaa01f3e6f967e498de1ba0ac9b`
- **Index**: `0x0`
- **Dep Type**: `code`

**Cell Dependency (Testnet)**
- **TX Hash**: `0x1b4ffcad55ecd36ffb2715b6816b83da73851f1a24fe594f263c4f34dad90792`
- **Index**: `0x0`
- **Dep Type**: `code`

**Source**: [CCC SDK](https://github.com/ckb-ecofund/ccc)

## Hash Type Values

- `type`: Upgradable smart contract using type script verification (most commonly Type ID system).
- `data`: Script identified by data hash using CKB VM v0 (Lina).
- `data1`: Script identified by data hash using CKB VM v1 (CKB2021 hardfork, Mirana).
- `data2`: Script identified by data hash using CKB VM v2 (CKB2023 hardfork, Meepo).

## References

- [CCC SDK](https://github.com/ckb-ecofund/ccc) - Primary CKB ecosystem SDK
- [RFC 0024: CKB Genesis Script List](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0024-ckb-genesis-script-list/0024-ckb-genesis-script-list.md) - Type ID
- [spore-contract VERSIONS.md](https://github.com/sporeprotocol/spore-contract/blob/master/docs/VERSIONS.md) - Spore, Cluster
- [nervina-labs/cota-sdk-js constants](https://github.com/nervina-labs/cota-sdk-js/blob/develop/src/constants/index.ts) - CoTA
- [CKBFS README.md](https://github.com/nervape/ckbfs/blob/master/README.md) - CKBFS
- [iCKB Whitepaper](https://github.com/ickb/whitepaper#mainnet-deployment) - iCKB
- [RGB++ SDK Constants](https://github.com/RGBPlusPlus/rgbpp-sdk/blob/main/packages/ckb/src/constants/index.ts) - RGB++, BTC Time Lock
