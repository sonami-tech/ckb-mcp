## Description

CKB Fiber Network: an off-chain payment channel network on Nervos CKB for CKB and UDT assets, analogous to Bitcoin's Lightning Network. Architecture, the JSON-RPC surface (41 methods across 10 method-bearing modules plus a pubsub subscription and biscuit auth), channel types, the canonical enum/status tables (PaymentStatus, CkbInvoiceStatus, CchOrderStatus, Currency, HashAlgorithm, ChannelState), the four expiry-parameter names, and the stale-to-current API translation table. Current payments are hash-based HTLC/TLC; PTLC is roadmap terminology. Version anchor (v0.8.0 baseline, current v0.9.0-rc2) and the source-of-truth rule that the fiber-json-types wire crate outranks generated docs. fiber-js wraps the snake_case RPC with camelCase TypeScript. Top footguns digest with pointers to the file that fixes each.

## Related Resources

- [ckb://docs/fiber/node-setup](ckb://docs/fiber/node-setup) - Install, configure, fund, and safely expose a Fiber node; first peer connection and channel
- [ckb://docs/fiber/rpc-reference](ckb://docs/fiber/rpc-reference) - Per-method JSON-RPC parameters, returns, encoding rules, and enum tables
- [ckb://docs/fiber/channels](ckb://docs/fiber/channels) - Channel lifecycle, state machine, collateral, external funding, shutdown
- [ckb://docs/fiber/invoices-and-payments](ckb://docs/fiber/invoices-and-payments) - Invoice lifecycle, payment sending, fees, settlement verification
- [ckb://docs/fiber/routing-and-graph](ckb://docs/fiber/routing-and-graph) - Gossip graph sync, multi-hop, trampoline, rebalancing
- [ckb://docs/fiber/on-chain-scripts](ckb://docs/fiber/on-chain-scripts) - funding-lock and commitment-lock layouts, ckb-auth, deployed-script referencing
- [ckb://docs/fiber/udt-channels](ckb://docs/fiber/udt-channels) - UDT-funded channels and UDT invoices (RUSD testnet, USDI mainnet)
- [ckb://docs/fiber/cross-chain-hub](ckb://docs/fiber/cross-chain-hub) - Fiber-to-Bitcoin-Lightning atomic swaps via the Cross-Chain Hub
- [ckb://docs/ecosystem/project-directory](ckb://docs/ecosystem/project-directory) - CKB ecosystem project directory including Fiber
- [ckb://docs/sdk/rust-sdk-basic](ckb://docs/sdk/rust-sdk-basic) - ckb-sdk-rust basics including RpcClient configuration

CKB Fiber Network (FNN = Fiber Network Node, the reference implementation of the Fiber Network Protocol) is an off-chain payment/swap network on Nervos CKB. Transactions are settled only by the involved peers — no global consensus — giving high throughput, low latency, and high privacy, with multi-hop routing so any node can forward payments and earn fees.

## Overview

### What Fiber Is

- **Off-chain payment channels on CKB**, like Bitcoin's Lightning Network, but **multi-asset**: native CKB, UDT/xUDT tokens (e.g. RUSD on testnet, USDI on mainnet), and RGB++ assets.
- **Channels** are funded by an on-chain transaction; only opening and closing touch the chain. In between, balances move off-chain via signed commitment transactions.
- **Multi-hop**: payments route across a path of channels (A → B → C); intermediate nodes earn forwarding fees.
- **Cross-network**: the Cross-Chain Hub (CCH) bridges Fiber and Bitcoin Lightning via atomic swaps sharing one payment hash.

### HTLC Today, PTLC Roadmap

The top-level project README markets Fiber as using "PTLC not HTLC." **This is aspirational.** The shipping implementation uses **hash-based HTLC/TLC**: the `commitment-lock` contract verifies a payment hash against `blake2b_256(preimage)` or `sha256(preimage)`. The glossary states it directly: *"PTLC support is a planned direction; today Fiber still runs on hash-based TLC/HTLC flows."* When describing Fiber's mechanics, say **HTLC/TLC**, not PTLC.

### Channel Types

- **Public** (`public: true`, the default): broadcast to the network via gossip; usable to forward other parties' payments.
- **Private** (`public: false`): not broadcast; reachable only via `hop_hints` as a last hop.
- **One-way** (`one_way: true`): a *private* channel (it is **not broadcast**) that can only move payments in one direction (funder → acceptor). `public` and `one_way` are **mutually exclusive** — `open_channel` rejects `{"public": true, "one_way": true}` with "An one-way channel cannot be public", so a one-way channel is always private. `list_channels` still returns `is_public` and `is_one_way` as separate fields; direction is read from `is_one_way` together with `is_acceptor`.

### Version Anchor and Source of Truth

- **Baseline: v0.8.0** (2026-03-28, the last release with a CHANGELOG). **Current checked-out binary: v0.9.0-rc2.** The CHANGELOG trails the code — there are post-v0.8.0 changes not documented there.
- **Authority order for API shapes** (highest first):
  1. The `crates/fiber-json-types` wire-type crate (canonical serde structs/enums).
  2. The RPC handlers and validation code (`crates/fiber-lib/src/rpc/*.rs`).
  3. The generated RPC README (`crates/fiber-lib/src/rpc/README.md`) — **can drift; its prose has known errors** (e.g. it claims a 16-hour invoice expiry minimum that is actually 2h40m).
  4. The CHANGELOG.
  5. Community demos / blog posts — **most are stale** (see translation table).
- Even the wire crate's *doc comments* occasionally lie (the "16 hours" minimum is wrong in the crate too); trust the **validation code** for numeric limits.

### Wire Encoding Rules (apply to every RPC)

- **Integer amounts** (`u128`, `u64`, `u32`): lowercase `0x`-hex strings, no redundant leading zeros — `0x0` is valid, `0x00` is rejected. Amounts are in **Shannons** for CKB (1 CKB = 10^8 shannons = `0x5f5e100`); for UDT channels the unit is the UDT's raw base unit.
- **Pubkey**: 33-byte compressed secp256k1 as hex **without** `0x` (66 hex chars). Input accepts an optional `0x`; output never has it.
- **Hash256** (payment_hash, channel_id, preimage, chain_hash): 32-byte hex **with** `0x` (66 chars total).
- **Enum casing is not uniform** — see the tables below. `state_name` and the status enums are PascalCase; `HashAlgorithm` and invoice `Attribute` keys are snake_case.

### fiber-js (TypeScript)

The official `@nervosnetwork/fiber-js` client (in the `fiber-js/` directory of the Fiber repo) exposes **camelCase** methods (`openChannel`, `sendPayment`, `newInvoice`) that wrap the **snake_case** JSON-RPC (`open_channel`, `send_payment`, `new_invoice`). Method names map 1:1; only the casing differs. A `randomSecretKey()` helper is provided.

**Browser/WASM runtime warning:** in the browser, fiber-js runs its client runtime across **two Web Workers** (a database worker and a fiber worker) communicating over a **`SharedArrayBuffer`**, which requires the page to be served with **COOP/COEP headers** (`Cross-Origin-Opener-Policy: same-origin` + `Cross-Origin-Embedder-Policy: require-corp`). Browser bootnodes must use `wss` multiaddrs. The browser build is a **client**, not a standalone WASM full node. Raw JSON-RPC over HTTP remains snake_case regardless of client.

## Core Enums and Status Tables

These are the canonical values (verified against `fiber-json-types`). Casing is exact and load-bearing.

### PaymentStatus

`Created → Inflight → Success | Failed`

| Value | Meaning |
|-------|---------|
| `Created` | Payment session created; no HTLC dispatched yet |
| `Inflight` | First-hop AddTlc sent, awaiting response (note the lowercase `f`) |
| `Success` | All HTLCs settled — **the settlement proof on the sender side** |
| `Failed` | Session terminated; inspect `failed_error` |

Stale demos write `Succeeded` or `InFlight` — both are wrong.

### CkbInvoiceStatus

| Value | Meaning |
|-------|---------|
| `Open` | Payable (auto-reported as `Expired` once past its expiry) |
| `Received` | TLC received but not settled (a HOLD invoice awaiting its preimage) |
| `Paid` | Settled — **the settlement proof on the recipient side** |
| `Cancelled` | Cancelled before payment |
| `Expired` | Past expiry |

### CchOrderStatus

| Value | Meaning |
|-------|---------|
| `Pending` | Waiting for the incoming invoice to collect TLCs |
| `IncomingAccepted` | Incoming TLCs collected; ready to send the outgoing leg |
| `OutgoingInFlight` | Outgoing payment in flight (note the capital `F` — differs from `PaymentStatus::Inflight`) |
| `OutgoingSuccess` | Outgoing leg settled, preimage obtained |
| `Success` | Both legs settled, order complete |
| `Failed` | Order failed |

v0.8.0 renamed `Succeeded → Success` and `OutgoingSucceeded → OutgoingSuccess`.

### Currency

| Value | Network | Invoice HRP |
|-------|---------|-------------|
| `Fibb` | mainnet | `fibb` |
| `Fibt` | testnet | `fibt` |
| `Fibd` | devnet (default) | `fibd` |

`new_invoice` rejects a currency that does not match the node's chain.

### HashAlgorithm (snake_case)

| Value | Use |
|-------|-----|
| `ckb_hash` | default — blake2b_256 (truncated to 20 bytes on-chain) |
| `sha256` | **required** for cross-chain (Bitcoin Lightning) |

The internal Rust name is `CkbHash`; the JSON wire value is `ckb_hash`.

### ChannelState

`state_name` is PascalCase; `state_flags` (when present) is SCREAMING_SNAKE_CASE joined by ` | `.

| state_name | carries state_flags? |
|------------|----------------------|
| `NegotiatingFunding` | yes |
| `CollaboratingFundingTx` | yes |
| `SigningCommitment` | yes |
| `AwaitingTxSignatures` | yes |
| `AwaitingChannelReady` | yes |
| `ChannelReady` | **no** (emits just `{"state_name":"ChannelReady"}`) |
| `ShuttingDown` | yes |
| `Closed` | yes |

**Poll for `state.state_name == "ChannelReady"`.** Note `CHANNEL_READY` is only a *flag string* inside `AwaitingChannelReady.state_flags` — it is not the ready *state*. A channel with `enabled: false` will not route even while `ChannelReady`.

### The Four Expiry Parameters (the #1 documentation trap)

Canonical table (single source of truth; other files link here):

| Name | Real? | Which call | Unit | Notes |
|------|-------|-----------|------|-------|
| `final_cltv` | **NO-OP** | (none) | — | Exists nowhere in Fiber; appears in the official public-nodes examples; silently dropped |
| `final_expiry_delta` | yes | `new_invoice` (invoice) | ms | Release min **2h40m** (`0x927c00`), max 14d, default = min. The "16h" in docs/rustdoc is wrong |
| `final_tlc_expiry_delta` | yes | `send_payment` (payment) | ms | Payment-side final-hop timelock |
| `min_final_cltv_expiry_delta` | yes | Bitcoin Lightning invoice (CCH only) | CLTV blocks | The BTC-side delta, a different network |

### Stale → Current Translation Table

Community demos (`ckb-fiber-testnet-demo`, `ckb-fiber-docker` which pins fnn v0.5.1) predate v0.8.0. Map old forms to current:

| Stale form | Current form |
|-----------|--------------|
| `peer_id` (RPC param, base58 `Qm…`) | `pubkey` (hex, no `0x`). `Qm…` PeerIds remain valid **only inside multiaddrs** |
| `state_name: "CHANNEL_READY"` | `state_name: "ChannelReady"` |
| `CkbHash` | `ckb_hash` |
| `Succeeded` | `Success` |
| `OutgoingSucceeded` | `OutgoingSuccess` |
| `final_cltv` | `final_expiry_delta` (invoice) / `final_tlc_expiry_delta` (payment) |

## Top Footguns Digest

Each is corrected in the file noted:

1. **Reference deployed scripts by `code_hash` + `hash_type: type` + type-id cell-dep args + the ckb-auth cell-dep — never by a deployment `tx_hash`** (it changes on every upgrade). → on-chain-scripts
2. **`connect_peer` by `pubkey` alone fails on a fresh node** (`PeerNotFound`); first contact needs a multiaddr `address`. → node-setup, channels
3. **Public RPC binding hard-refuses to start without `rpc.biscuit_public_key`.** → node-setup
4. **`graph_channels[].outbound_liquidity` is `null` for remote channels — that is expected, not a bug** (only your own channels report it). → routing-and-graph
5. **UDT routing fees are paid in the UDT, not CKB; `udt_type_script` must be byte-identical across open/invoice/payment; UDT amounts are raw base units.** → udt-channels
6. **Settlement = `get_invoice` status `Paid` (recipient) or `get_payment` status `Success` (sender)** — not invoice creation, not TLC forwarding, not an in-flight session. → invoices-and-payments
7. **The `final_*` expiry name cluster** (see table) and the release-vs-debug value gap (`0xDFFA0` from example tests is rejected on release nodes). → invoices-and-payments
8. **Cooperative `shutdown_channel` is rejected if the peer is offline;** use `force: true` for unilateral close. → channels
9. **fiber-js browser builds need two Web Workers + SharedArrayBuffer + COOP/COEP;** they are clients, not WASM full nodes. → this page (fiber-js section above)
10. **It's HTLC/TLC, not PTLC.** → this page

## Glossary

- **FNN / FNP** — Fiber Network Node (the implementation) / Fiber Network Protocol.
- **TLC / HTLC** — (Hashed) Time-Locked Contract; the conditional-payment primitive Fiber ships today.
- **Channel** — a two-party off-chain balance secured by on-chain funding and commitment locks.
- **Commitment transaction** — the latest signed balance, broadcastable if a peer disappears.
- **Gossip** — the protocol that propagates channel/node presence and fee policy (not live balances).
- **CCH** — Cross-Chain Hub, bridging Fiber and Bitcoin Lightning.
- **Watchtower** — a third party that monitors for revoked-commitment broadcasts and can penalize a cheating counterparty.
- **Bootnode** — a peer-discovery-only node; cannot open channels (connect to a *public node* for that).
