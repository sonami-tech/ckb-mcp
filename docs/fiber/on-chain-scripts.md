## Description

Fiber's on-chain CKB lock scripts (the `fiber-scripts` repo): **funding-lock** (112-byte witness = 16-byte empty WitnessArgs + 32-byte Schnorr pubkey + 64-byte signature; single input; message = `blake2b_256` of the raw tx with cell_deps cleared; ckb-auth Schnorr id 7) and **commitment-lock** (57-byte args; an 85-byte HTLC sub-script whose byte 0 bit-packs htlc_type and hash algorithm; preimage verified against `blake2b_256`/`sha256` truncated to 20 bytes; revocation vs settlement paths; 1/3, 2/3, full `delay_epoch` timelocks). The canonical way to reference deployed scripts — `code_hash` + `hash_type: type` + the type-id cell-dep args + the ckb-auth code cell-dep — and why you must never pin a deployment `tx_hash` (the locks are upgradeable type-id scripts redeployed multiple times on testnet). Building custom locks with ckb-script-templates (ckb-std 1.1), with the caveat that fiber-scripts itself is on ckb-std 0.18.

## Related Resources

- [ckb://docs/fiber/overview](ckb://docs/fiber/overview) - HTLC-not-PTLC and the tx_hash-pinning footgun
- [ckb://docs/fiber/channels](ckb://docs/fiber/channels) - External funding builds against the funding lock
- [ckb://docs/fiber/udt-channels](ckb://docs/fiber/udt-channels) - UDT cell data layout on channel cells
- [ckb://docs/fiber/rpc-reference](ckb://docs/fiber/rpc-reference) - The external-funding RPC methods
- [ckb://docs/concepts/script-groups](ckb://docs/concepts/script-groups) - CKB script execution model
- [ckb://docs/scripts/type-id](ckb://docs/scripts/type-id) - Type-ID upgradeable script pattern
- [ckb://docs/tools/offckb-workflow](ckb://docs/tools/offckb-workflow) - Local devnet for script development

Two CKB lock scripts secure a Fiber channel: a **funding-lock** that guards the on-chain funding cell, and a **commitment-lock** that guards commitment-transaction outputs (enabling revocation and HTLC resolution). Both verify signatures through `ckb-auth`. These byte layouts exist only in the `fiber-scripts` source and are needed for watchtowers, channel forensics, and external-funding integrations.

## Overview

### Reference Deployed Scripts by Type-ID — Never a tx_hash

The Fiber locks are deployed as **type-id (upgradeable) scripts**. The deployment `tx_hash` changes on every redeploy (testnet has been redeployed multiple times) — so pinning a tx_hash will break. The durable referents are the script `code_hash` (a type-id hash), `hash_type: type`, and the **type-id cell-dep args**, plus a separate cell-dep for the ckb-auth code cell.

**⚠️ The hex literals in the block below are illustrative and likely already stale.** Do not copy them — extract the current `code_hash`, type-id `args`, and ckb-auth `out_point` from the running node's live `config/testnet/config.yml`. The structure shown is what to read, not the values:

Structural template only — every `0x…` below is a placeholder. Read the real values from the live config; do not copy these:

```yaml
scripts:
  - name: FundingLock
    script:
      code_hash: <FUNDING_LOCK_CODE_HASH from live config>
      hash_type: type            # Type-ID (upgradeable), NOT data
      args: 0x                   # per-channel args supplied at runtime
    cell_deps:
      - type_id:                 # reference the script CELL by its Type-ID, not a tx_hash
          code_hash: 0x00000000000000000000000000000000000000000000000000545950455f4944  # "TYPE_ID" (constant)
          hash_type: type
          args: <FUNDING_LOCK_TYPE_ID_ARGS from live config>
      - cell_dep:                # the ckb-auth code cell (a plain out_point dep)
          out_point:
            tx_hash: <CKB_AUTH_TX_HASH from live config>
            index: 0x0
          dep_type: code
  - name: CommitmentLock
    script:
      code_hash: <COMMITMENT_LOCK_CODE_HASH from live config>
      hash_type: type
      args: 0x
    cell_deps:
      - type_id:
          code_hash: 0x00000000000000000000000000000000000000000000000000545950455f4944  # "TYPE_ID" (constant)
          hash_type: type
          args: <COMMITMENT_LOCK_TYPE_ID_ARGS from live config>
      - cell_dep:
          out_point:
            tx_hash: <CKB_AUTH_TX_HASH from live config>
            index: 0x0
          dep_type: code
```

Notes and warnings:
- The lock `code_hash` + `hash_type: type` is the durable identity; the `type_id` cell-dep `args` is the Type-ID that survives upgrades. Read both live from config — they change on redeployment.
- The only `tx_hash` here is the **ckb-auth code cell-dep**, referenced by out_point because auth is a stable dependency; even so, prefer reading it live from config.
- The config file's own comment points at an older migration JSON — that comment is stale while the `cell_deps` are current. **Trust the code_hashes, not the comment.**
- Mainnet (`config/mainnet/config.yml`) deploys via a 3-of-5 multisig; testnet is the development target.

### funding-lock

Guards the on-chain funding cell; both channel parties' funds sit behind it.

- **Witness must be exactly 112 bytes** = `EMPTY_WITNESS_ARGS` (16 bytes, an empty `WitnessArgs` molecule placeholder for xUDT compatibility: `[16,0,0,0, 16,0,0,0, 16,0,0,0, 16,0,0,0]`) + 32-byte Schnorr pubkey + 64-byte Schnorr signature.
- Exactly **one** group input is allowed (multiple inputs error out).
- The signed **message** is `blake2b_256` of the raw transaction with its `cell_deps` cleared.
- `args[0..20]` is the pubkey hash.
- Verification is via `ckb-auth` with **algorithm id 7 (Schnorr)**, invoked through `exec_cell`.

### commitment-lock

Guards commitment-transaction outputs; supports revocation (penalty) and HTLC settlement.

**Args — exactly 57 bytes:**

| Offset | Bytes | Field |
|--------|-------|-------|
| `0..20` | 20 | local pubkey hash (revocation/settlement) |
| `20..28` | 8 | `delay_epoch` (Since-encoded EpochNumberWithFraction, LE) |
| `28..36` | 8 | `version` (LE; revocation requires new_version ≥ current) |
| `36..56` | 20 | settlement / pending-HTLC script hash = `blake2b_256(settlement_script)[0..20]` |
| `56` | 1 | `is_first_settlement` flag (`0x00` = first commitment cell, `0x01` = subsequent) |

**HTLC sub-script — exactly 85 bytes each:**

| Offset | Bytes | Field |
|--------|-------|-------|
| `0` | 1 | **type/flags byte (bit-packed)**: bit 0 = htlc_type (0 Offered / 1 Received); bit 1 = payment-hash algorithm (0 Blake2b / 1 Sha256) |
| `1..17` | 16 | `payment_amount` (u128 little-endian) |
| `17..37` | 20 | `payment_hash` (**truncated to 20 bytes**, not 32) |
| `37..57` | 20 | `remote_htlc_pubkey_hash` |
| `57..77` | 20 | `local_htlc_pubkey_hash` |
| `77..85` | 8 | `htlc_expiry` (u64 LE, absolute timestamp Since) |

**Preimage check** (this is why Fiber is HTLC, not PTLC): the contract requires `payment_hash == blake2b_256(preimage)[0..20]` (Blake2b) or `sha256(preimage)[0..20]` (Sha256). `PREIMAGE_LEN = 32`. There is no adaptor-signature / point-locked path.

**Unlock paths:**
- **Revocation** (penalty path; lets a watchtower claim the whole balance if a revoked commitment is broadcast). Witness layout: the contract first drains the 16-byte `EMPTY_WITNESS_ARGS` prefix (`[16,0,0,0, 16,0,0,0, 16,0,0,0, 16,0,0,0]`) and requires it to match exactly, then reads byte 0 as `unlock_count` — `0x00` selects revocation. The remaining bytes are `[new_version (8 bytes)][Schnorr signature]`. It verifies a Schnorr signature (auth id 7) over `blake2b_256(output ‖ (output_data_len as u32 LE, 4 bytes) ‖ output_data ‖ args[0..28] ‖ new_version)` where `output`/`output_data` are the first output cell, and requires `new_version >= current_version` (the contract errors only when `current_version > new_version`).
- **Settlement**: HTLC resolution, signed with auth id 0 (Ckb/secp256k1). The per-HTLC settlement record is `[unlock_type][with_preimage flag][65-byte signature][optional 32-byte preimage]`. The `unlock_type` sentinels `0xFE`/`0xFF` (remote/local **party** settlement, the non-pending-HTLC close) carry an additional structured payload (pubkey hashes + u128 LE amounts at fixed offsets) beyond this record — **read `commitment-lock/src/main.rs` (the `0xFE`/`0xFF` branches) before constructing a party-settlement witness**; the layout above covers only the per-HTLC records.

**Timelocks** (fractions of `delay_epoch`): preimage unlock allowed after **1/3**, expiry unlock after **2/3**, party/non-pending settlement after the **full** delay. Verification uses `spawn_cell` + `wait`.

### ckb-auth

Both locks delegate signature verification to the `ckb-auth` binary, loaded by its `code_hash` with `ScriptHashType::Data1` (the auth cell-dep is `dep_type: code`, distinct from the locks' type-id cell-deps). Fiber uses **algorithm id 7 (Schnorr)** for funding-lock unlock and commitment-lock revocation, and **algorithm id 0 (Ckb/secp256k1)** for commitment-lock settlement signatures.

### Building Custom Locks

`fiber-scripts` was bootstrapped with **ckb-script-templates**, the canonical Rust on-chain scaffolding tool (use it for new locks; it now targets **ckb-std 1.1** / ckb-testtool 1.1 with native-simulator coverage). **Caveat:** `fiber-scripts` itself is pinned to **ckb-std 0.18** — a different major line. If you scaffold a fresh lock with today's templates and then read fiber-scripts source as a reference, the ckb-std API will differ. `offckb` provides a local devnet for testing deployed scripts.
