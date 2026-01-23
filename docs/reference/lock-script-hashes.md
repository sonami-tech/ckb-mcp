# Lock Script Hashes

## Description

Code hashes, type IDs, and cell dependencies for CKB lock scripts on mainnet and testnet. Omnilock (universal cross-chain lock), ACP (Anyone-Can-Pay), PW Lock (deprecated), JoyID (WebAuthn/passkey authentication), and Nostr Lock (schnorr signature verification). Args format and deployment transaction hashes.

## Omnilock

Omnilock is a universal lock script designed for cross-chain interoperability.

**Mainnet (Mirana)**
- **Code Hash**: `0x9b819793a64463aed77c615d6cb226eea5487ccfc0783043a587254cda2b6f26`
- **Hash Type**: `type`
- **Args**: `<auth: 21 bytes> + <flags: 1 byte> + [optional configs]`
  - **Auth byte 0**: Authentication method (0x00=CKB, 0x01=Ethereum, 0x03=Tron, 0x04=Bitcoin, 0x05=Dogecoin, 0x06=Multisig, 0xFC=Script Hash, 0xFD=Exec, 0xFE=Dynamic Linking)
  - **Auth bytes 1-20**: Identity hash (blake160 of pubkey for most methods, address-specific for Bitcoin)
  - **Flags byte**: Mode bitmask (0x01=Administrator, 0x02=Anyone-Can-Pay, 0x04=Time-Lock, 0x08=Supply)
  - **Optional configs**: Appended based on enabled flags (32-byte SMT root, 8-byte since value, etc.)
  - **Full spec**: [Omnilock Protocol](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0042-omnilock/0042-omnilock.md)

**Cell Dependency (Mainnet)**
- **TX Hash**: `0xc76edf469816aa22f416503c38d0b533d2a018e253e379f134c3985b3472c842`
- **Index**: `0x0`
- **Dep Type**: `code`

**Testnet (Pudge)**
- **Code Hash**: `0xf329effd1c475a2978453c8600e1eaf0bc2087ee093c3ee64cc96ec6847752cb`
- **Hash Type**: `type`
- **Args**: `<auth: 21 bytes> + <flags: 1 byte> + [optional configs]`
  - **Auth byte 0**: Authentication method (0x00=CKB, 0x01=Ethereum, 0x03=Tron, 0x04=Bitcoin, 0x05=Dogecoin, 0x06=Multisig, 0xFC=Script Hash, 0xFD=Exec, 0xFE=Dynamic Linking)
  - **Auth bytes 1-20**: Identity hash (blake160 of pubkey for most methods, address-specific for Bitcoin)
  - **Flags byte**: Mode bitmask (0x01=Administrator, 0x02=Anyone-Can-Pay, 0x04=Time-Lock, 0x08=Supply)
  - **Optional configs**: Appended based on enabled flags (32-byte SMT root, 8-byte since value, etc.)
  - **Full spec**: [Omnilock Protocol](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0042-omnilock/0042-omnilock.md)

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

## PW Lock Script (Deprecated)

PW Lock is a lock script for PW-SDK compatibility, enabling Ethereum-style authentication on CKB. PW-SDK is deprecated; use Omnilock for new Ethereum-compatible deployments.

**Mainnet**
- **Code Hash**: `0xbf43c3602455798c1a61a596e0d95278864c552fafe231c063b3fabf97a8febc`
- **Hash Type**: `type`
- **Args**: Multi-ecosystem authentication data (format varies by chain).
  - **Note**: PW-SDK is deprecated. Use [Omnilock](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0042-omnilock/0042-omnilock.md) for cross-chain lock scripts in new deployments.

**Cell Dependency (Mainnet)**
- **TX Hash**: `0x1d60cb8f4666e039f418ea94730b1a8c5aa0bf2f7781474406387462924d15d4`
- **Index**: `0x0`
- **Dep Type**: `code`

**Testnet**
- **Code Hash**: `0x58c5f491aba6d61678b7cf7edf4910b1f5e00ec0cde2f42e0abb4fd9aff25a63`
- **Hash Type**: `type`
- **Args**: Multi-ecosystem authentication data (format varies by chain).
  - **Note**: PW-SDK is deprecated. Use [Omnilock](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0042-omnilock/0042-omnilock.md) for cross-chain lock scripts in new deployments.

**Cell Dependency (Testnet)**
- **TX Hash**: `0x57a62003daeab9d54aa29b944fc3b451213a5ebdf2e232216a3cfed0dde61b38`
- **Index**: `0x0`
- **Dep Type**: `code`

**Source**: [pw-core constants.ts](https://github.com/jordanmack/pw-core/blob/dev/src/constants.ts)

## ACP (Anyone Can Pay) Lock Script

ACP is a lock script that allows anyone to transfer CKB or UDT tokens to a cell. The receiver can accept payments without signing. Optional minimum transfer amounts protect against DDoS attacks.

*Note: The anyone-can-pay pattern is also available in other locks:*
- *Omnilock: Built-in ACP mode via flag 0x02 in the args.*
- *PW Lock (deprecated): Has ACP logic built in.*
- *ACP Proxy: Extends ACP mechanics to JoyID and Multisig locks.*

**Mainnet (Lina)**
- **Code Hash**: `0xd369597ff47f29fbc0d47d2e3775370d1250b85140c670e4718af712983a2354`
- **Hash Type**: `type`
- **Type ID**: `0xde8b879bd1e98399de0dc9be163e703fc1fb82d9379ee1e85143b9f5a863610c`
- **Args**: `<pubkey_hash: 20 bytes> [+ <ckb_min: 1 byte>] [+ <udt_min: 1 byte>]`
  - **Bytes 0-19**: Blake160 hash of public key
  - **Byte 20** (optional): Minimum CKB transfer (exponential: value n = 10^n shannons, e.g., 9 = 10 CKB)
  - **Byte 21** (optional): Minimum UDT transfer (exponential: value n = 10^n base units)
  - **Note**: Omitted minimums default to 0 (no minimum enforced)
  - **Full spec**: [RFC 0026](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0026-anyone-can-pay/0026-anyone-can-pay.md)

**Cell Dependency (Mainnet)**
- **TX Hash**: `0x4153a2014952d7cac45f285ce9a7c5c0c0e1b21f2d378b82ac1433cb11c25c4d`
- **Index**: `0x0`
- **Dep Type**: `dep_group` (2 cells)

**Testnet (Aggron)**
- **Code Hash**: `0x3419a1c09eb2567f6552ee7a8ecffd64155cffe0f1796e6e61ec088d740c1356`
- **Hash Type**: `type`
- **Args**: `<pubkey_hash: 20 bytes> [+ <ckb_min: 1 byte>] [+ <udt_min: 1 byte>]`
  - **Bytes 0-19**: Blake160 hash of public key
  - **Byte 20** (optional): Minimum CKB transfer (exponential: value n = 10^n shannons, e.g., 9 = 10 CKB)
  - **Byte 21** (optional): Minimum UDT transfer (exponential: value n = 10^n base units)
  - **Note**: Omitted minimums default to 0 (no minimum enforced)
  - **Full spec**: [RFC 0026](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0026-anyone-can-pay/0026-anyone-can-pay.md)

**Cell Dependency (Testnet)**
- **TX Hash**: `0xec26b0f85ed839ece5f11c4c4e837ec359f5adc4420410f6453b1f6b60fb96a6`
- **Index**: `0x0`
- **Dep Type**: `dep_group` (2 cells)

**Source**: [RFC 0026: Anyone-Can-Pay Lock](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0026-anyone-can-pay/0026-anyone-can-pay.md)

## JoyID Lock Script

JoyID is a passwordless authentication solution using WebAuthn and passkeys for CKB.

**Mainnet**
- **Code Hash**: `0xd00c84f0ec8fd441c38bc3f87a371f547190f2fcff88e642bc5bf54b9e318323`
- **Hash Type**: `type`
- **Args**: Contains JoyID-specific authentication parameters.

**Cell Dependencies (Mainnet - CCC SDK)**

CCC SDK uses 5 individual code deps with Type IDs for verification:

| Index | TX Hash | Type ID |
|-------|---------|---------|
| 0 | `0x8a605a4402cadda69fa64fd25cbbd74058e3eb86a7a72aee3d25df278564d31b` | `0x2d1f2d4d1514ccc3bb4f04f5437a5ae30d00636ee57cedd2c70ab3ea75b62adc` |
| 1 | `0x8a605a4402cadda69fa64fd25cbbd74058e3eb86a7a72aee3d25df278564d31b` | `0xc086090432098835ec542a1b94bdd1b842c5aa1ccd1616873fe77f4a04044417` |
| 2 | `0x8a605a4402cadda69fa64fd25cbbd74058e3eb86a7a72aee3d25df278564d31b` | `0x165b225c6fbed7e655b024384d9083de3243375f9893706f4452858ecd694e96` |
| 3 | `0x8a605a4402cadda69fa64fd25cbbd74058e3eb86a7a72aee3d25df278564d31b` | `0xafb8408d0094ab944e6286aac750b9bb854ac0bcb66dfe5c60559744a700e70c` |
| 4 | `0x8a605a4402cadda69fa64fd25cbbd74058e3eb86a7a72aee3d25df278564d31b` | `0x773bf0647be24b4e18ef44068fd069b9de5549c4b86be227779ceb9179598ec4` |

**Cell Dependency (Mainnet - Alternative dep_group)**
- **TX Hash**: `0xf05188e5f3a6767fc4687faf45ba5f1a6e25d3ada6129dae8722cb282f262493`
- **Index**: `0x0`
- **Dep Type**: `dep_group`

*Note: The dep_group bundles the same 5 JoyID scripts plus SECP256K1 into one dependency. Both approaches load identical code (same Type IDs).*

**Testnet**
- **Code Hash**: `0xd23761b364210735c19c60561d213fb3beae2fd6172743719eff6920e020baac`
- **Hash Type**: `type`
- **Args**: Contains JoyID-specific authentication parameters.

**Cell Dependencies (Testnet - CCC SDK)**

| Index | TX Hash | Type ID |
|-------|---------|---------|
| 0 | `0x4a596d31dc35e88fb1591debbf680b04a44b4a434e3a94453c21ea8950ffb4d9` | `0x1c9fc299ba0570d077b4d7fb9acff1ccc0de69d369942d82678bae937c44ec30` |
| 1 | `0x4a596d31dc35e88fb1591debbf680b04a44b4a434e3a94453c21ea8950ffb4d9` | `0x27f0d3ccdc2fcd52ae31fbacad5f86b97bc147d7093e4807cd6e3d21c1fe6841` |
| 2 | `0xf2c9dbfe7438a8c622558da8fa912d36755271ea469d3a25cb8d3373d35c8638` | `0x0ac15fe5b2d059ec39de03f2d3159d5463abb918a1a07a9fa00d2b9c61d89ef3` |
| 3 | `0x95ecf9b41701b45d431657a67bbfa3f07ef7ceb53bf87097f3674e1a4a19ce62` | `0xc7bafc5550ccad7cea32c27764f5df6aca4de547da65e3e67fa08477a1af7f5e` |
| 4 | `0x8b3255491f3c4dcc1cfca33d5c6bcaec5409efe4bbda243900f9580c47e0242e` | `0x71decef9ca8725e64ec99a5521790d16b8d5daadb4989b45dd6ab51806a8c0e4` |

**Cell Dependency (Testnet - Alternative dep_group)**
- **TX Hash**: `0x4dcf3f3b09efac8995d6cbee87c5345e812d310094651e0c3d9a730f32dc9263`
- **Index**: `0x0`
- **Dep Type**: `dep_group`

**Source**: [CCC SDK](https://github.com/ckb-ecofund/ccc), [JoyID Smart Contract Docs](https://docs.joyid.dev/guide/ckb/smart-contract)

## Nostr Lock Script

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

## Hash Type Values

- `type`: Upgradable smart contract using type script verification (most commonly Type ID system).
- `data`: Script identified by data hash using CKB VM v0 (Lina).
- `data1`: Script identified by data hash using CKB VM v1 (CKB2021 hardfork, Mirana).
- `data2`: Script identified by data hash using CKB VM v2 (CKB2023 hardfork, Meepo).

## References

- [RFC 0026: Anyone-Can-Pay](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0026-anyone-can-pay/0026-anyone-can-pay.md) - ACP
- [RFC 0042: Omnilock](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0042-omnilock/0042-omnilock.md) - Omnilock
- [CCC SDK](https://github.com/ckb-ecofund/ccc) - Primary CKB ecosystem SDK
- [JoyID Smart Contract Docs](https://docs.joyid.dev/guide/ckb/smart-contract) - JoyID
- [Nostr Lock Script Specification](https://github.com/cryptape/nostr-binding/blob/main/docs/nostr-lock-script.md) - Nostr Lock
