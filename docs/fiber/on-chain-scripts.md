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

```yaml
scripts:
  - name: FundingLock
    script:
      code_hash: 0x6c67887fe201ee0c7853f1682c0b77c0e6214044c156c7558269390a8afa6d7c
      hash_type: type            # Type-ID (upgradeable), NOT data
      args: 0x                   # per-channel args supplied at runtime
    cell_deps:
      - type_id:                 # reference the script CELL by its Type-ID, not a tx_hash
          code_hash: 0x00000000000000000000000000000000000000000000000000545950455f4944  # "TYPE_ID"
          hash_type: type
          args: 0x3cb7c0304fe53f75bb5727e2484d0beae4bd99d979813c6fc97c3cca569f10f6
      - cell_dep:                # the ckb-auth code cell (a plain out_point dep)
          out_point:
            tx_hash: 0x12c569a258dd9c5bd99f632bb8314b1263b90921ba31496467580d6b79dd14a7  # ckb_auth
            index: 0x0
          dep_type: code
  - name: CommitmentLock
    script:
      code_hash: 0x740dee83f87c6f309824d8fd3fbdd3c8380ee6fc9acc90b1a748438afcdf81d8
      hash_type: type
      args: 0x
    cell_deps:
      - type_id:
          code_hash: 0x00000000000000000000000000000000000000000000000000545950455f4944
          hash_type: type
          args: 0xf7e458887495cf70dd30d1543cad47dc1dfe9d874177bf19291e4db478d5751b
      - cell_dep:
          out_point:
            tx_hash: 0x12c569a258dd9c5bd99f632bb8314b1263b90921ba31496467580d6b79dd14a7  # ckb_auth
            index: 0x0
          dep_type: code
```

Notes and warnings:
- The lock `code_hash` + `hash_type: type` is the durable identity; the `type_id` cell-dep `args` (`0x3cb7c030…` funding, `0xf7e45888…` commitment) is the Type-ID that survives upgrades.
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
- **Revocation** (selected when the witness's `unlock_count` byte is `0x00`): a penalty path verifying a Schnorr signature (auth id 7) over `blake2b_256(output ‖ output_data_len ‖ output_data ‖ args[0..28] ‖ new_version)`, requiring `new_version >= current_version` (the contract errors only when `current_version > new_version`). Lets a watchtower claim the whole balance if a revoked commitment is broadcast.
- **Settlement**: HTLC resolution, signed with auth id 0 (Ckb/secp256k1). The settlement record is `[unlock_type][with_preimage flag][65-byte signature][optional 32-byte preimage]`; `unlock_type` sentinels `0xFE`/`0xFF` denote remote/local party settlement.

**Timelocks** (fractions of `delay_epoch`): preimage unlock allowed after **1/3**, expiry unlock after **2/3**, party/non-pending settlement after the **full** delay. Verification uses `spawn_cell` + `wait`.

### ckb-auth

Both locks delegate signature verification to the `ckb-auth` binary, loaded by its `code_hash` with `ScriptHashType::Data1` (the auth cell-dep is `dep_type: code`, distinct from the locks' type-id cell-deps). Fiber uses **algorithm id 7 (Schnorr)** for funding-lock unlock and commitment-lock revocation, and **algorithm id 0 (Ckb/secp256k1)** for commitment-lock settlement signatures.

### Building Custom Locks

`fiber-scripts` was bootstrapped with **ckb-script-templates**, the canonical Rust on-chain scaffolding tool (use it for new locks; it now targets **ckb-std 1.1** / ckb-testtool 1.1 with native-simulator coverage). **Caveat:** `fiber-scripts` itself is pinned to **ckb-std 0.18** — a different major line. If you scaffold a fresh lock with today's templates and then read fiber-scripts source as a reference, the ckb-std API will differ. `offckb` provides a local devnet for testing deployed scripts.
