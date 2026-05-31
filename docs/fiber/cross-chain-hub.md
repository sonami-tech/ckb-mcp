## Description

Fiber's Cross-Chain Hub (CCH): atomic swaps between Fiber payments and Bitcoin Lightning invoices through `send_btc` (Fiber → Lightning), `receive_btc` (Lightning → Fiber), and `get_cch_order`. Both legs share one `payment_hash`, which is what makes the swap atomic — so the invoice **must** use `hash_algorithm: "sha256"` (Bitcoin Lightning's hash), not the Fiber default `ckb_hash`; a mismatch breaks settlement even when the Fiber-only flow is valid. The `CchOrderResponse` shape and `CchInvoice` union (`{"Fiber": String}` | `{"Lightning": String}`, encoded invoice strings), the `CchOrderStatus` lifecycle (Pending → IncomingAccepted → OutgoingInFlight → OutgoingSuccess → Success | Failed; v0.8.0 renamed `Succeeded`/`OutgoingSucceeded`), running CCH standalone via the `services` array (HTTP RPC + the `subscribe_store_changes` pubsub), and the BTC-side `min_final_cltv_expiry_delta` versus Fiber's `final_expiry_delta`. End-to-end flow and a pointer to the Bruno cross-chain scenarios.

## Related Resources

- [ckb://docs/fiber/overview](ckb://docs/fiber/overview) - CchOrderStatus table and the expiry parameters
- [ckb://docs/fiber/invoices-and-payments](ckb://docs/fiber/invoices-and-payments) - Fiber invoice/payment mechanics and the sha256 hash algorithm
- [ckb://docs/fiber/rpc-reference](ckb://docs/fiber/rpc-reference) - send_btc / receive_btc / get_cch_order parameters
- [ckb://docs/fiber/node-setup](ckb://docs/fiber/node-setup) - The services array and pubsub
- [ckb://docs/fiber/routing-and-graph](ckb://docs/fiber/routing-and-graph) - Graph sync before the Fiber leg can route

The Cross-Chain Hub bridges Fiber and Bitcoin Lightning. A swap has two legs — one on each network — locked to the **same** payment hash, so revealing the preimage to claim one leg automatically enables claiming the other. This is Fiber's headline interoperability feature and the only built-in path between the two networks.

## Overview

### Model

A CCH order pairs a Fiber payment with a Bitcoin Lightning payment under one `payment_hash`. The hub holds an HTLC on each side and does **not** know the preimage in advance: it pays the *outgoing* leg first, learns the preimage from that settlement, then uses it to claim the *incoming* leg — which is what makes the swap atomic. Because the Lightning side hashes with SHA-256, the Fiber side must too — see the hash-algorithm rule below.

`CchInvoice` (the proxy invoice the hub generates for the incoming leg) is a union of encoded invoice strings: `{"Fiber": "fibt…"}` or `{"Lightning": "lnbc…"}` — not structured objects.

### Fiber → Bitcoin Lightning (`send_btc`)

You hold a Bitcoin Lightning invoice (a `bolt11` string) and want to pay it from Fiber:

```json
{"method":"send_btc","params":[{
  "btc_pay_req":"lnbc1...",
  "currency":"Fibt"
}]}
```

The hub returns a `CchOrderResponse` containing an `incoming_invoice` (a Fiber invoice for you to pay) and tracks the order. You pay the Fiber invoice; the hub uses the shared preimage to settle the Lightning invoice. Poll `get_cch_order(payment_hash)` until `status == "Success"`.

### Bitcoin Lightning → Fiber (`receive_btc`)

You want to receive CKB on Fiber from a Bitcoin sender. Create a Fiber invoice (with `hash_algorithm: "sha256"`), then:

```json
{"method":"receive_btc","params":[{
  "fiber_pay_req":"fibt1..."
}]}
```

The hub returns a `CchOrderResponse` with an `outgoing_pay_req` (a Lightning invoice for the Bitcoin sender to pay). When they pay it, the hub pays your Fiber invoice using the shared preimage. The Fiber invoice you supply **must** use `hash_algorithm: "sha256"` so its payment hash matches the Lightning leg — a default `ckb_hash` invoice produces a blake2b payment hash that the SHA-256 Lightning leg can never match, so the swap cannot settle atomically across both networks.

### Order Tracking

```json
{"method":"get_cch_order","params":[{"payment_hash":"0x..."}]}
```

`CchOrderResponse` fields: `timestamp`, `expiry_delta_seconds`, `wrapped_btc_type_script`, `incoming_invoice` (CchInvoice), `outgoing_pay_req`, `payment_hash` (the shared atomicity link), `amount_sats`, `fee_sats`, `status` (CchOrderStatus).

### CchOrderStatus Lifecycle

`Pending → IncomingAccepted → OutgoingInFlight → OutgoingSuccess → Success`, or `Failed`.

| Status | Meaning |
|--------|---------|
| `Pending` | Waiting for the incoming invoice to collect TLCs |
| `IncomingAccepted` | Incoming TLCs collected; ready to send the outgoing leg |
| `OutgoingInFlight` | Outgoing payment in flight (capital `F` — distinct from `PaymentStatus::Inflight`) |
| `OutgoingSuccess` | Outgoing leg settled, preimage obtained |
| `Success` | Both legs settled, order complete |
| `Failed` | Order failed |

(v0.8.0 renamed `Succeeded → Success` and `OutgoingSucceeded → OutgoingSuccess`; older code/tutorials use the old `Succeeded`/`OutgoingSucceeded` names.)

### Hash Algorithm — the cross-chain requirement

Bitcoin Lightning uses **SHA-256** payment hashes; Fiber defaults to `ckb_hash` (blake2b). For a swap the two legs must share the same preimage *and* hash function, so **the Fiber invoice in a CCH flow must set `hash_algorithm: "sha256"`**. A mismatch produces an order that cannot settle atomically even though the Fiber-only invoice flow would otherwise be valid. This is the most common CCH integration error.

### Expiry — two different "final CLTV" concepts

Do not confuse the two networks' final-hop timelocks:
- **Fiber side:** `final_expiry_delta` on the Fiber invoice (ms; release min 2h40m).
- **Bitcoin Lightning side:** `min_final_cltv_expiry_delta` on the BTC invoice (in CLTV blocks).

The hub enforces a dependency between them (the BTC invoice's `min_final_cltv_expiry_delta`, converted to seconds at ~10 min/block, must be safely below the CKB-side final TLC expiry) so the legs cannot time out against each other. See the four-expiry table in overview.

### Running CCH Standalone

Since v0.8.0, the Cross-Chain Hub can run as its own process rather than inside the Fiber node. It connects to a Fiber node over HTTP RPC and subscribes to store changes via the `subscribe_store_changes` WebSocket (requires the `read("cch")` biscuit scope). **The Fiber node must have `pubsub` in its `rpc.enabled_modules` — it is NOT default** (`DEFAULT_ENABLED_MODULES` = `cch,channel,graph,payment,info,invoice,peer[,watchtower]`, no `pubsub`); without it `subscribe_store_changes` is not registered and the standalone hub cannot receive store events. Composition is controlled by the node's `services` array (see node-setup) — enabling or omitting `cch` determines whether the hub runs in-process. A standalone hub additionally needs a Bitcoin Lightning node (e.g. LND) for the Lightning leg. The `cch:` block is **not** in the shipped `config/testnet/config.yml`; its keys (source: `cch/config.rs`, each also settable via the env var shown):

| Key | Env var | Default | Notes |
|-----|---------|---------|-------|
| `fiber_rpc_url` | `CCH_FIBER_RPC_URL` | — | **Standalone mode only:** the remote Fiber node's RPC URL. Set this to run CCH as a separate process |
| `base_dir` | `CCH_BASE_DIR` | `$BASE_DIR/cch` | cch data dir |
| `lnd_rpc_url` | `CCH_LND_RPC_URL` | `https://127.0.0.1:10009` | LND gRPC endpoint |
| `lnd_cert_path` | `CCH_LND_CERT_PATH` | — | LND TLS cert |
| `lnd_macaroon_path` | `CCH_LND_MACAROON_PATH` | — | LND auth macaroon |
| `wrapped_btc_type_script_args` | `CCH_WRAPPED_BTC_TYPE_SCRIPT_ARGS` | — | args of the wrapped-BTC UDT type script |
| `wrapped_btc_type_script` | — | — | full Script JSON; **`validate_standalone()` rejects startup if unset/unparseable**; takes precedence over `_args` |
| `order_expiry_delta_seconds` | `CCH_ORDER_EXPIRY_DELTA_SECONDS` | (source default) | order relative expiry |
| `base_fee_sats` | `CCH_BASE_FEE_SATS` | `0` | base fee per order |
| `fee_rate_per_million_sats` | `CCH_FEE_RATE_PER_MILLION_SATS` | (source default) | proportional fee |
| `btc_final_tlc_expiry_delta_blocks` | `CCH_BTC_FINAL_TLC_EXPIRY_DELTA_BLOCKS` | (source default) | BTC-leg final timelock (blocks) |
| `ckb_final_tlc_expiry_delta_seconds` | `CCH_CKB_FINAL_TLC_EXPIRY_DELTA_SECONDS` | (source default) | CKB-leg final timelock (s) |
| `min_outgoing_invoice_expiry_delta_seconds` | `CCH_MIN_OUTGOING_INVOICE_EXPIRY_DELTA_SECONDS` | (source default) | min outgoing invoice expiry |
| `ignore_startup_failure` | — | `false` | continue if the hub fails to start |

For a working end-to-end example see the `cross-chain-hub-separate` Bruno fixtures (`tests/nodes/`); treat `cch/config.rs` as the live source for key names and defaults.

### End-to-End and Examples

A complete swap: create the order (`send_btc` or `receive_btc`) → pay the source-side invoice → poll `get_cch_order` to a terminal status → confirm Fiber settlement (`get_invoice` `Paid` / `get_payment` `Success`) and Lightning settlement. The Fiber repo's Bruno e2e suite includes `cross-chain-hub` and `cross-chain-hub-separate` (the standalone-CCH-with-LND variant) scenarios with runnable current-API request bodies — adapt any debug-only numeric values for a release node.
