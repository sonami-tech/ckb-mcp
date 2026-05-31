## Description

Fiber Network JSON-RPC reference (JSON-RPC 2.0 over HTTP via jsonrpsee, with a WebSocket path for subscriptions): 41 methods across 10 method-bearing modules (channel, payment, invoice, peer, graph, info, cch, plus debug-only dev, feature-gated prof and watchtower), a `subscribe_store_changes` WebSocket subscription, and biscuit auth. Wire encoding rules (u64/u128/u32 as lowercase `0x`-hex with no redundant leading zeros; pubkey hex without `0x`; Hash256 with `0x`; PascalCase status/state names; snake_case HashAlgorithm and invoice Attribute keys). Per-method parameters with types, required/optional, and defaults, plus return fields, verified against the `fiber-json-types` wire crate — which outranks the generated RPC README (whose prose has known errors, e.g. the 16-hour invoice minimum that is actually 2h40m). Enum tables, biscuit scopes, the public-IP bind requirement, and the `final_cltv` / `final_expiry_delta` / `final_tlc_expiry_delta` parameter traps.

## Related Resources

- [ckb://docs/fiber/overview](ckb://docs/fiber/overview) - Master enum tables, encoding rules, and the four expiry parameters
- [ckb://docs/fiber/channels](ckb://docs/fiber/channels) - Lifecycle context for the channel methods
- [ckb://docs/fiber/invoices-and-payments](ckb://docs/fiber/invoices-and-payments) - Invoice and payment flows and fees
- [ckb://docs/fiber/routing-and-graph](ckb://docs/fiber/routing-and-graph) - Graph methods and error-string vocabulary
- [ckb://docs/fiber/on-chain-scripts](ckb://docs/fiber/on-chain-scripts) - Scripts the external-funding methods build
- [ckb://docs/fiber/node-setup](ckb://docs/fiber/node-setup) - Endpoint, biscuit auth, and bind requirement
- [ckb://docs/sdk/rust-sdk-basic](ckb://docs/sdk/rust-sdk-basic) - ckb-sdk-rust RpcClient and types

Fiber exposes JSON-RPC 2.0 over HTTP (served by `jsonrpsee`), with a WebSocket path for the `subscribe_store_changes` subscription. The authoritative shape of every request and response lives in the `crates/fiber-json-types` wire-type crate; the generated RPC README is convenient but can drift, so this reference takes parameter limits from the validation code.

## Overview

### Transport and Endpoint

JSON-RPC 2.0 over HTTP (served by `jsonrpsee` on `hyper`) at `rpc.listening_addr` (default `127.0.0.1:8227`); the `subscribe_store_changes` pubsub uses a WebSocket on the same endpoint. CORS is opt-in (`rpc.cors_enabled` / `rpc.cors_allowed_origins`). See node-setup for the hard rule that public binding requires `rpc.biscuit_public_key`.

### Encoding Rules

| Type | JSON form | Rule |
|------|-----------|------|
| `u128` / `u64` / `u32` amount | `"0x…"` lowercase hex | No redundant leading zero: `0x0` valid, `0x00` rejected. CKB unit = Shannons (1 CKB = `0x5f5e100`); UDT channels use the UDT base unit |
| Pubkey | hex **without** `0x` (66 chars) | Input accepts optional `0x`; output never has it |
| Hash256 | hex **with** `0x` (66 chars) | payment_hash, channel_id, preimage, chain_hash |
| Duration (invoice `expiry`, CCH timestamps) | `"0x{seconds}"` | Seconds, hex |
| `Script` / `CellDep` | CKB JSON object | Standard CKB conventions |
| `OutPoint` (e.g. every `channel_outpoint`) | `0x`+molecule hex string (`EntityHex`) | **Not** a `{ tx_hash, index }` JSON object. Every `channel_outpoint` field in these wire types — `HopHint`, `HopRequire`, `RouterHop`, `Channel`, `ChannelInfo`, `SessionRouteNode` — encodes the `OutPoint` as molecule-serialized `0x` hex |
| `state_flags` | `"FLAG_A | FLAG_B"` | SCREAMING_SNAKE, ` | `-joined. The `state_flags` key is **absent** for `ChannelReady` (no flags payload) |
| `custom_records` | `{ "0x{u32}": "0x{bytes}" }` | Keys 0..=65535; total value bytes ≤ 2048 |

Enum casing is in the overview tables. Status enums and `state_name` are PascalCase; `HashAlgorithm` and invoice `Attribute` keys are snake_case.

### Module and Method Index

41 methods across the modules below. `dev` is compiled **only in debug builds** (absent on release nodes). `prof` requires the `pprof` feature; `watchtower` is feature-gated. `cch`, `pubsub`, and `biscuit` are non-WASM. Default-enabled RPC namespaces are `channel, payment, invoice, peer, graph, info, cch` (+ `watchtower` when built). **`pubsub` is NOT default** — add it to `rpc.enabled_modules` or `subscribe_store_changes` is unavailable (needed by the standalone Cross-Chain Hub; see cross-chain-hub).

| Module | Methods |
|--------|---------|
| `peer` | connect_peer, disconnect_peer, list_peers |
| `channel` | open_channel, accept_channel, abandon_channel, list_channels, shutdown_channel, update_channel, open_channel_with_external_funding, submit_signed_funding_tx |
| `invoice` | new_invoice, parse_invoice, get_invoice, cancel_invoice, settle_invoice |
| `payment` | send_payment, get_payment, build_router, send_payment_with_router, list_payments |
| `info` | node_info |
| `graph` | graph_nodes, graph_channels |
| `cch` | send_btc, receive_btc, get_cch_order |
| `watchtower` (feat) | create_watch_channel, remove_watch_channel, update_revocation, update_pending_remote_settlement, update_local_settlement, create_preimage, remove_preimage |
| `dev` (debug only) | commitment_signed, add_tlc, remove_tlc, submit_commitment_transaction, check_channel_shutdown, sign_external_funding_tx |
| `prof` (feat) | pprof |
| `pubsub` | subscribe_store_changes (WebSocket subscription, not request/response) |

### Biscuit Scopes (per-method auth)

When `rpc.biscuit_public_key` is set, each method requires a scope. The rule is a `read(domain)`/`write(domain)` Datalog check (source: `rpc/biscuit.rs`). `write` does **not** imply `read`; domain names are **plural** where the module is plural. Default by pattern: queries need `read(domain)`, mutations need `write(domain)`, with `domain` = the module name below.

| Domain | `read(domain)` methods | `write(domain)` methods |
|--------|------------------------|--------------------------|
| `channels` | list_channels | open_channel, accept_channel, abandon_channel, shutdown_channel, update_channel, submit_signed_funding_tx, add_tlc, remove_tlc, check_channel_shutdown, sign_external_funding_tx |
| `payments` | get_payment, list_payments, build_router | send_payment, send_payment_with_router |
| `invoices` | parse_invoice, get_invoice | new_invoice, cancel_invoice, settle_invoice |
| `peers` | list_peers | connect_peer, disconnect_peer |
| `graph` | graph_nodes, graph_channels | — |
| `node` | node_info | — |
| `cch` | receive_btc, get_cch_order, subscribe_store_changes | send_btc |
| `watchtower` | — | all watchtower methods |

**Exceptions that don't follow the module→domain pattern** (an AI will get these wrong without the list): `commitment_signed` → `write("messages")`; `submit_commitment_transaction` → `write("chain")`; `pprof` → `write("pprof")`. `biscuit.rs` is the live source — verify against it before issuing tokens.

## Peer Module

### connect_peer → `null`

| Param | Type | Req | Default |
|-------|------|-----|---------|
| `address` | String (multiaddr) | one of address/pubkey | — |
| `pubkey` | Pubkey | one of address/pubkey | resolves address from synced graph |
| `save` | bool | opt | `true` |
| `addr_type` | `tcp`/`ws`/`wss` | opt | native→tcp, wasm→ws/wss |

**Fresh-node trap:** pubkey-only fails (`PeerNotFound`) until gossip knows the address; first contact needs `address`.

### disconnect_peer → `null`
`pubkey` (Pubkey, required).

### list_peers → `{ peers: [{ pubkey, address }] }`
No params.

## Channel Module

### open_channel → `{ temporary_channel_id: Hash256 }`

| Param | Type | Req | Default |
|-------|------|-----|---------|
| `pubkey` | Pubkey | **yes** | peer must be connected first |
| `funding_amount` | u128 | **yes** | — |
| `public` | bool | opt | `true` |
| `one_way` | bool | opt | `false` |
| `funding_udt_type_script` | Script | opt | none (= CKB channel) |
| `shutdown_script` | Script | opt | node sighash script |
| `commitment_delay_epoch` | EpochNumberWithFraction (u64 hex) | opt | 1 epoch (4h) = `"0x10000000001"` (number=1, index=0, length=1) |
| `commitment_fee_rate` / `funding_fee_rate` | u64 | opt | node default |
| `tlc_expiry_delta` | u64 ms | opt | **4h** (`14400000`) |
| `tlc_min_value` | u128 | opt | 0 |
| `tlc_fee_proportional_millionths` | u128 | opt | **1000** (0.1%) |
| `max_tlc_value_in_flight` | u128 | opt | effectively `u128::MAX` if unset; immutable after open |
| `max_tlc_number_in_flight` | u64 | opt | **125** default; immutable after open. Hard cap **253** (`SYS_MAX_TLC_NUMBER_IN_FLIGHT`) — a higher value is rejected with `"Max TLC number in flight {} is greater than the system maximal value 253"` |

Returns `temporary_channel_id`, **not** the final `channel_id` — poll `list_channels` for `ChannelReady`, where the real `channel_id` appears.

### accept_channel → `{ channel_id: Hash256 }`
Only needed when the peer does not auto-accept.

| Param | Type | Req | Default |
|-------|------|-----|---------|
| `temporary_channel_id` | Hash256 | **yes** | — |
| `funding_amount` | u128 | **yes** | — |
| `shutdown_script` | Script | opt | node sighash |
| `max_tlc_value_in_flight` | u128 | opt | `u128::MAX`; immutable |
| `max_tlc_number_in_flight` | u64 | opt | 125 default; immutable; hard cap 253 (see open_channel) |
| `tlc_min_value` | u128 | opt | 0 |
| `tlc_fee_proportional_millionths` | u128 | opt | 1000 |
| `tlc_expiry_delta` | u64 ms | opt | **4h** (the node default; the wire-crate rustdoc/CLI help saying "1 day" is stale, like the "16h" invoice-min) |

### list_channels → `{ channels: [Channel] }`

| Param | Type | Req | Default |
|-------|------|-----|---------|
| `pubkey` | Pubkey | opt | none → all |
| `include_closed` | bool | opt | `false` |
| `only_pending` | bool | opt | `false` (mutually exclusive with `include_closed`) |

`Channel` fields (amounts u128 hex): `channel_id`, `is_public`, `is_acceptor`, `is_one_way`, `channel_outpoint?`, `pubkey` (counterparty), `funding_udt_type_script?`, `state` (ChannelState), `local_balance`, `offered_tlc_balance`, `remote_balance`, `received_tlc_balance`, `pending_tlcs` ([Htlc]), `latest_commitment_transaction_hash?`, `created_at`, **`enabled`** (false ⇒ not routed even when ChannelReady), `tlc_expiry_delta`, `tlc_fee_proportional_millionths`, `shutdown_transaction_hash?`, **`failure_detail`** (always present as a field; carries a human-readable reason only when an open failed, otherwise `null`).

`Htlc`: `id`, `amount`, `payment_hash`, `expiry`, `forwarding_channel_id?`, `forwarding_tlc_id?`, `status` (TlcStatus: `{"Outbound": …}` | `{"Inbound": …}`).

### shutdown_channel → `null`

| Param | Type | Req | Default |
|-------|------|-----|---------|
| `channel_id` | Hash256 | **yes** | — |
| `close_script` | Script | opt | node `default_funding_lock_script` (the wallet sighash lock), **not** the Fiber `funding-lock` contract; sighash only |
| `fee_rate` | u64 | opt | **1000 shannons/KW** |
| `force` | bool | opt | `false` |

**Cooperative close (`force:false`) is rejected if the peer is offline.** Use `force:true` for unilateral close (ignores `close_script`/`fee_rate`).

### update_channel → `null`
`channel_id` (required); optional `enabled` (default `true`; set `false` to leave routing), `tlc_expiry_delta`, `tlc_minimum_value` (note: `tlc_minimum_value` here, not `tlc_min_value`), `tlc_fee_proportional_millionths`.

### open_channel_with_external_funding → `{ channel_id: Hash256, unsigned_funding_tx: Transaction }`
Like `open_channel` but `shutdown_script` and `funding_lock_script` are **required** (plus optional `funding_lock_script_cell_deps`). Returns the real `channel_id` and an unsigned funding tx to sign offline.

### submit_signed_funding_tx → `{ channel_id, funding_tx_hash }`
`channel_id` + `signed_funding_tx`. **Invariant:** the signer may only add witnesses — `inputs`, `outputs`, `outputs_data`, and `cell_deps` must be unchanged.

### abandon_channel → `null`
`channel_id`. Only for channels not in `ChannelReady` or `Closed`.

## Invoice Module

### new_invoice → `{ invoice_address: String, invoice: CkbInvoice }`

| Param | Type | Req | Default / validation |
|-------|------|-----|----------------------|
| `amount` | u128 | **yes** | Shannons (CKB) or UDT base units |
| `currency` | Currency | **yes** | must match node chain (Fibb/Fibt/Fibd) |
| `description` | String | opt | none |
| `payment_preimage` | Hash256 | opt | if absent and `payment_hash` absent → random; if set, `payment_hash` must be absent |
| `payment_hash` | Hash256 | opt | set without preimage → **HOLD invoice** |
| `expiry` | u64 seconds | opt | none (no auto-expiry) |
| `fallback_address` | String | opt | none |
| `final_expiry_delta` | u64 ms | opt | default = min = **2h40m** (`0x927c00`); min 2h40m, max 14d. (Docs/rustdoc "16h" is wrong) |
| `udt_type_script` | Script | opt | none → CKB; if set, must byte-match the channel `funding_udt_type_script` |
| `hash_algorithm` | HashAlgorithm | opt | `ckb_hash`; use `sha256` for cross-chain |
| `allow_mpp` | bool | opt | false; if true the node must have the MPP feature, else error (**invoice** param) |
| `allow_trampoline_routing` | bool | opt | false; node must have the trampoline feature (**invoice** param) |

`CkbInvoice`: `currency`, `amount?`, `signature?` (hex), `data { timestamp, payment_hash, attrs: [Attribute] }`. Invoice strings are bech32m with HRP `fibb`/`fibt`/`fibd` — **not BOLT11**.

### get_invoice → `{ invoice_address, invoice, status }`
`payment_hash`. **`status == "Paid"` is the recipient-side settlement proof.** `Open` auto-reports as `Expired` once past expiry.

### settle_invoice → `{}`
`payment_hash` + `payment_preimage`. For HOLD invoices: saves the preimage to release the held TLC.

### cancel_invoice → `{ invoice_address, invoice, status }`
`payment_hash`. **Rejected only when status is `Paid` or `Cancelled`** — `Open`, `Received`, and `Expired` are all cancellable (cancelling a held invoice releases its TLC set). (It is *not* "Open-only.")

### parse_invoice → `{ invoice: CkbInvoice }`
`invoice` (String).

## Payment Module

### send_payment → `GetPaymentCommandResult`

| Param | Type | Req | Notes |
|-------|------|-----|-------|
| `target_pubkey` | Pubkey | opt* | required unless derivable from `invoice` |
| `amount` | u128 | opt | uses invoice amount if absent |
| `payment_hash` | Hash256 | opt | from invoice, or random if `keysend`, else required |
| `final_tlc_expiry_delta` | u64 ms | opt | payment-side final-hop timelock (distinct from invoice `final_expiry_delta`) |
| `tlc_expiry_limit` | u64 ms | opt | whole-payment cap; sums each channel's `tlc_expiry_delta` (4h default per hop) |
| `invoice` | String | opt | supplies target/amount/hash |
| `timeout` | u64 seconds | opt | cancel if unpaid |
| `max_fee_amount` | u128 | opt | absolute fee cap; in trampoline mode used as the total budget |
| `max_fee_rate` | u64 | opt | **default 5 = 0.5%** (per-thousand) |
| `max_parts` | u64 | opt | MPP knob (payment side) |
| `trampoline_hops` | [Pubkey] | opt | explicit trampoline path; incompatible with `allow_self_payment` |
| `keysend` | bool | opt | false |
| `udt_type_script` | Script | opt | must byte-match channel/invoice UDT script |
| `allow_self_payment` | bool | opt | false; true + keysend + self target = rebalance |
| `custom_records` | map | opt | u32→bytes, ≤ 2048 bytes total |
| `hop_hints` | [HopHint] | opt | reach a private last-hop channel (a hint, not a guarantee) |
| `dry_run` | bool | opt | **true validates routability and returns the exact `fee` with no side effects** (the `routers` route array is only populated in debug builds; use `build_router` for an inspectable route) |

`HopHint`: `{ pubkey, channel_outpoint, fee_rate, tlc_expiry_delta }`. `channel_outpoint` is `EntityHex` (see the encoding table — `0x` molecule hex, **not** `{ tx_hash, index }`); `fee_rate` and `tlc_expiry_delta` are `0x`-hex `u64`. Required to route to a private/one-way channel as the last hop.

There is **no dedicated rebalance RPC** — rebalance via self-payment or build_router + send_payment_with_router.

### get_payment → `GetPaymentCommandResult`
`payment_hash`. Result: `payment_hash`, `status` (PaymentStatus), `created_at`, `last_updated_at`, `failed_error?`, `fee` (u128), `custom_records?` (and, debug builds only, `routers`). **`status == "Success"` is the sender-side settlement proof.**

### build_router → `{ router_hops: [RouterHop] }`

| Param | Type | Req | Default |
|-------|------|-----|---------|
| `hops_info` | [HopRequire] | **yes** | `{pubkey, channel_outpoint?}` list (`channel_outpoint` is `EntityHex` `0x` hex, not an object); excludes source; last = target. A **strong** restriction (must be honored, unlike hop_hints) |
| `amount` | u128 | opt | min routable `1` |
| `udt_type_script` | Script | opt | none |
| `final_tlc_expiry_delta` | u64 ms | opt | final-hop timelock |

`RouterHop`: `target`, `channel_outpoint` (`EntityHex` `0x` hex), `amount_received`, `incoming_tlc_expiry`.

### send_payment_with_router → `GetPaymentCommandResult`
`router` ([RouterHop], required) + optional `payment_hash`, `invoice`, `custom_records`, `keysend`, `udt_type_script`, `dry_run`. Feed `router_hops` from `build_router` for deterministic replay.

### list_payments → `{ payments: [GetPaymentCommandResult], last_cursor? }`
`status?`, `limit?` (default **15**), `after?` (exclusive Hash256 cursor).

## Info Module

### node_info → `NodeInfoResult`
No params. Fields: `version`, `commit_hash`, **`pubkey`** (your own node id), `features`, `node_name?`, `addresses` (multiaddrs), `chain_hash`, `open_channel_auto_accept_min_ckb_funding_amount`, `auto_accept_channel_ckb_funding_amount`, `default_funding_lock_script`, `tlc_expiry_delta`, `tlc_min_value`, `tlc_fee_proportional_millionths`, `channel_count`, `pending_channel_count`, `peers_count`, `udt_cfg_infos`.

`features` is a `[String]` of capability flags. Each is a base name + `_REQUIRED` or `_OPTIONAL` suffix. Base names (source: `fiber-types/src/protocol.rs`): `GOSSIP_QUERIES`, `BASIC_MPP`, `TRAMPOLINE_ROUTING`. Example: a node advertising MPP returns `"BASIC_MPP_OPTIONAL"` or `"BASIC_MPP_REQUIRED"` — check for the `BASIC_MPP` / `TRAMPOLINE_ROUTING` prefix to detect capability before sending `max_parts` / trampoline payments. `udt_cfg_infos` lists supported UDTs (each entry: name, the UDT `Script`, `auto_accept_amount?`, decimals); see node-setup's `udt_whitelist` for the shape.

## Graph Module

### graph_nodes → `{ nodes: [NodeInfo], last_cursor }`
`limit?`, `after?` (JsonBytes cursor — not Hash256). `NodeInfo`: `node_name`, `version`, `addresses`, `features`, `pubkey`, `timestamp`, `chain_hash`, `auto_accept_min_ckb_funding_amount`, `udt_cfg_infos`.

### graph_channels → `{ channels: [ChannelInfo], last_cursor }`
`ChannelInfo`: `channel_outpoint`, `node1`, `node2`, `created_timestamp`, `update_info_of_node1?`, `update_info_of_node2?`, `capacity`, `chain_hash`, `udt_type_script?`. Each `ChannelUpdateInfo`: `timestamp`, `enabled`, **`outbound_liquidity?`** (populated only for your own channels; `null` for remote gossip-learned channels is expected), `tlc_expiry_delta`, `tlc_minimum_value`, `fee_rate` (the same proportional-millionths forwarding fee that other methods expose as `tlc_fee_proportional_millionths` — the graph just names the field `fee_rate`).

## CCH Module

`#[cfg(not(wasm32))]`, enabled by default. See cross-chain-hub for flows.

- **send_btc** → `CchOrderResponse` — params `{ btc_pay_req: String, currency: Currency }`. Fiber → BTC-Lightning.
- **receive_btc** → `CchOrderResponse` — params `{ fiber_pay_req: String }`. BTC-Lightning → Fiber.
- **get_cch_order** → `CchOrderResponse` — params `{ payment_hash: Hash256 }`.

`CchOrderResponse`: `timestamp`, `expiry_delta_seconds`, `wrapped_btc_type_script`, `incoming_invoice` (CchInvoice = `{"Fiber": String}` | `{"Lightning": String}`), `outgoing_pay_req`, `payment_hash` (shared across both chains), `amount_sats`, `fee_sats`, `status` (CchOrderStatus). The BTC side requires `sha256`.

## Watchtower Module (feature-gated)

`create_watch_channel`, `remove_watch_channel`, `update_revocation`, `update_pending_remote_settlement`, `update_local_settlement`, `create_preimage`, `remove_preimage` — all return `null`. A watchtower stores per-state revocation data so it can broadcast a penalty transaction if a counterparty publishes a revoked commitment. Requires the `write("watchtower")` scope.

Example — `create_watch_channel` params: `{ channel_id: Hash256, funding_tx_lock: Script, remote_settlement_data: SettlementData }` (where `SettlementData` carries the per-commitment revocation keys/version). The other methods key off `channel_id` plus their respective settlement/preimage payloads. **For the exact param structs of all 7 methods, read `crates/fiber-lib/src/rpc/watchtower.rs`** (the `#[method]` declarations) — this is a niche operator-side surface, not transcribed in full here. On-chain penalty witness layout is in on-chain-scripts.

## Dev Module (debug builds only)

`commitment_signed`, `add_tlc`, `remove_tlc`, `submit_commitment_transaction`, `check_channel_shutdown`, `sign_external_funding_tx`. Compiled with `#[cfg(debug_assertions)]` — **these methods do not exist on a release node.** Never write production code against them.

## Prof Module (pprof feature)

`pprof` — profiling; useful for diagnosing stuck TLCs.

## Pubsub

`subscribe_store_changes` is a **WebSocket subscription** (notification `store_changes`, unsubscribe `unsubscribe_store_changes`), used by a standalone Cross-Chain Hub. Requires the `read("cch")` scope. Not a request/response method. The notification payload is a `StoreChange` object — for its shape read `crates/fiber-lib/src/rpc/pubsub.rs` (the type is not transcribed here as it is an internal CCH-replication surface).

## Enums and Error Fields

The canonical enum tables (PaymentStatus, CkbInvoiceStatus, CchOrderStatus, Currency, HashAlgorithm, ChannelState, the four expiry params, and the stale→current translation) are in overview. Two error fields carry human-readable cause strings: `Channel.failure_detail` (failed channel opens) and `GetPaymentCommandResult.failed_error` (failed payments) — e.g. no-path-found, expiry-too-soon, insufficient-balance. See routing-and-graph for the troubleshooting table.
