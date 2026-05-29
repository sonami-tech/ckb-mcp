## Description

Fiber channel lifecycle and management: `open_channel` (returns a `temporary_channel_id`, not the final `channel_id`), polling `list_channels` for `state.state_name == "ChannelReady"` (PascalCase; `CHANNEL_READY` is only a flag string), manual vs auto-accept, `update_channel`, and `shutdown_channel` (cooperative close is rejected when the peer is offline — use `force: true`) plus `abandon_channel`. The full `ChannelState` machine and SCREAMING_SNAKE `state_flags`; the `Channel` field set including `enabled` (false stops routing even while ready) and `failure_detail`; collateral math (99 CKB per side = 98 occupied + 1 shutdown fee, driven by the shutdown script; UDT channels reserve more); and the external-funding flow (`open_channel_with_external_funding` → sign witnesses only → `submit_signed_funding_tx`). Worked JSON for connect → open → poll → close using the current pubkey API and release-valid values.

## Related Resources

- [ckb://docs/fiber/overview](ckb://docs/fiber/overview) - ChannelState table and footgun digest
- [ckb://docs/fiber/node-setup](ckb://docs/fiber/node-setup) - Connect to a peer and the first-channel checklist
- [ckb://docs/fiber/rpc-reference](ckb://docs/fiber/rpc-reference) - Exact channel-method parameters and returns
- [ckb://docs/fiber/invoices-and-payments](ckb://docs/fiber/invoices-and-payments) - Pay over an open channel
- [ckb://docs/fiber/routing-and-graph](ckb://docs/fiber/routing-and-graph) - Why a ready channel may not route yet
- [ckb://docs/fiber/on-chain-scripts](ckb://docs/fiber/on-chain-scripts) - funding-lock and commitment-lock that secure the channel
- [ckb://docs/fiber/udt-channels](ckb://docs/fiber/udt-channels) - UDT-funded channels

A Fiber channel is a two-party off-chain balance secured by an on-chain funding transaction. Opening and closing touch the chain; payments in between are off-chain. The lifecycle has several non-obvious steps — the temporary-vs-final id, the readiness poll, the routing-enabled flag, and the offline-peer close rule.

## Overview

### Channel Lifecycle

```
NegotiatingFunding → CollaboratingFundingTx → SigningCommitment
  → AwaitingTxSignatures → AwaitingChannelReady → ChannelReady → ShuttingDown → Closed
```

### Open a Channel

The peer must be connected first (see node-setup; first contact needs a multiaddr). Then:

```json
{"id":1,"jsonrpc":"2.0","method":"open_channel","params":[{
  "pubkey":"02b6d4e3ab86a2ca2fad6fae0ecb2e1e559e0b911939872a90abdda6d20302be71",
  "funding_amount":"0xb9e459300",
  "public":true
}]}
```

`0xb9e459300` = 49,900,000,000 shannons = 499 CKB. The response is a **temporary** id:

```json
{"jsonrpc":"2.0","id":1,"result":{"temporary_channel_id":"0x284a31f9591e79669d1b4118fe3f5da5050a9a746d83e8a65d02605d6f22d16c"}}
```

**Trap:** `temporary_channel_id` is **not** the channel's identity. The real `channel_id` only appears in `list_channels` once the channel is open. Do not store the temporary id as durable identity.

`commitment_delay_epoch` (optional) is an `EpochNumberWithFraction` (24-bit number, 16-bit index, 16-bit length); the 1-epoch (4h) default encodes as `"0x10000000001"` (number=1, index=0, length=1).

### Poll for ChannelReady

```json
{"id":2,"jsonrpc":"2.0","method":"list_channels","params":[{
  "pubkey":"02b6d4e3ab86a2ca2fad6fae0ecb2e1e559e0b911939872a90abdda6d20302be71"
}]}
```

Poll until the channel's `state.state_name == "ChannelReady"` (the funding tx must confirm, ~10-30 s on testnet). A ready channel looks like:

```json
{"channel_id":"0x8cfd17...","is_public":true,"pubkey":"02b6d4e3...",
 "state":{"state_name":"ChannelReady"},
 "local_balance":"0x9502f9000","remote_balance":"0x38407b700","enabled":true,
 "tlc_expiry_delta":"0xdbba00","tlc_fee_proportional_millionths":"0x3e8","failure_detail":null}
```

**Two traps here:**
- `ChannelReady` is the bare state (no `state_flags`). `CHANNEL_READY` is only a *flag string* inside `AwaitingChannelReady.state_flags` — do not poll for the all-caps form (that is the pre-v0.8.0 API the demos use).
- Even at `ChannelReady`, a payment can fail until the gossip graph syncs (see routing-and-graph), and `enabled: false` keeps the channel out of routing entirely.

### Manual vs Auto-Accept

If the peer enables auto-accept above a threshold (`open_channel_auto_accept_min_ckb_funding_amount`), the channel proceeds without the peer calling `accept_channel`. Otherwise the peer must `accept_channel(temporary_channel_id, funding_amount, …)`, which returns the final `channel_id`. Public Fiber nodes auto-accept CKB channels at a 499-CKB minimum and contribute ~250 CKB of inbound liquidity.

### ChannelState and Flags

`state_name` is PascalCase; `state_flags` (when present) is SCREAMING_SNAKE_CASE joined by ` | `. See the overview table for all eight states and their flags. Example mid-shutdown:

```json
"state":{"state_name":"ShuttingDown","state_flags":"OUR_SHUTDOWN_SENT | THEIR_SHUTDOWN_SENT"}
```

### Channel Fields

Key fields from `list_channels` (full list in rpc-reference): identity (`channel_id`, `channel_outpoint`, counterparty `pubkey`), capabilities (`is_public`, `is_acceptor`, `is_one_way`), balances (`local_balance`, `remote_balance`, `offered_tlc_balance`, `received_tlc_balance` — all u128 hex shannons, or UDT base units for UDT channels), routing (`enabled`, `tlc_expiry_delta`, `tlc_fee_proportional_millionths`), and diagnostics (`pending_tlcs`, `shutdown_transaction_hash`, **`failure_detail`** — always present as a field, carrying a human-readable reason only when an open failed, otherwise `null`).

`is_acceptor` + `is_one_way` together determine send capability: a one-way channel sends only from funder to acceptor. A one-way channel is necessarily **private** — `open_channel` rejects `public: true` together with `one_way: true` ("An one-way channel cannot be public"), so it is never broadcast to the gossip graph.

### Collateral and Capacity

Each channel side reserves **99 CKB**: **98 CKB** of commitment-lock occupied capacity (`MIN_OCCUPIED_CAPACITY`) + **1 CKB** shutdown fee (`DEFAULT_MIN_SHUTDOWN_FEE`). So funding 499 CKB leaves ~400 CKB usable; `local_balance` will be less than `funding_amount`.

- **Protocol minimum to open:** 100 CKB.
- The reserve is driven by the **shutdown script** (a larger custom `shutdown_script` raises occupied capacity), not the funding lock.
- **UDT channels reserve more than 99 CKB** — the cell carries an extra 16 bytes of UDT amount data plus the UDT type script.

### Update a Channel

```json
{"method":"update_channel","params":[{"channel_id":"0x8cfd17...","enabled":false}]}
```

`update_channel` adjusts `enabled` (set `false` to remove the channel from routing while keeping it open), `tlc_expiry_delta`, `tlc_minimum_value`, and `tlc_fee_proportional_millionths`. Forwarding fees use the **outbound** channel's config (the fee for hop B in A → B → C comes from B → C).

### External Funding (hardware wallet / multisig / browser signer)

When the funding key lives outside the node:

1. `open_channel_with_external_funding(pubkey, funding_amount, shutdown_script, funding_lock_script, …)` → returns the real `channel_id` **and** an `unsigned_funding_tx`.
2. Sign the transaction with your external wallet. **Invariant: add witnesses only.** You must not change `inputs`, `outputs`, `outputs_data`, or `cell_deps` — the peer has committed to that exact transaction shape, and any structural change invalidates the negotiation. (CCC's `withFundingTxWitnesses` helper preserves structure; watch out for libraries that auto-insert change cells or reorder cell deps.)
3. Verify the transaction, then `submit_signed_funding_tx(channel_id, signed_funding_tx)` → `{ channel_id, funding_tx_hash }`.

See on-chain-scripts for the funding-lock witness format the signer must satisfy.

### Shutdown and Force Close

```json
{"method":"shutdown_channel","params":[{
  "channel_id":"0x8cfd17...",
  "close_script":{"code_hash":"0x...","hash_type":"type","args":"0x..."},
  "fee_rate":"0x3e8"
}]}
```

`fee_rate` defaults to the channel's `commitment_fee_rate` (1000 shannons/KW unless set otherwise at open); `close_script` defaults to `default_funding_lock_script` from the node's `CkbConfig` — i.e. the node's own wallet sighash lock (secp256k1_blake160_sighash_all only), not the on-chain `funding-lock` contract. 

**Cooperative close is rejected if the counterparty is offline.** For a unilateral close, set `force: true` (which ignores `close_script`/`fee_rate` and uses the open-time defaults):

```json
{"method":"shutdown_channel","params":[{"channel_id":"0x8cfd17...","force":true}]}
```

`abandon_channel(channel_id)` removes a channel that is **not** in `ChannelReady` or `Closed` (e.g. a stuck opening attempt).
