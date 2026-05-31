## Description

Fiber invoices and payments: `new_invoice` (the four-name expiry trap — use `final_expiry_delta`, not the no-op `final_cltv`; release minimum 2h40m / `0x927c00`, not the documented "16h"), HOLD invoices (set `payment_hash` without a preimage, release with `settle_invoice`), the `CkbInvoiceStatus` lifecycle, and bech32m invoice encoding (HRP `fibb`/`fibt`/`fibd`, not BOLT11). Payment sending via `send_payment` (invoice, keysend, `target_pubkey`, `max_parts` for MPP, `dry_run`) and `send_payment_with_router`; the `PaymentStatus` machine; the fee model (`max_fee_rate` per-thousand default 5, `tlc_fee_proportional_millionths` per-million default 1000, ceil formula, outbound directionality, UDT fees paid in the UDT). The authoritative settlement check — `get_invoice` status `Paid` (recipient) or `get_payment` status `Success` (sender) — and what is *not* settlement. A `failed_error` troubleshooting table. Worked JSON with current pubkey API and release-valid values.

## Related Resources

- [ckb://docs/fiber/overview](ckb://docs/fiber/overview) - Status enum tables and the four expiry parameters
- [ckb://docs/fiber/rpc-reference](ckb://docs/fiber/rpc-reference) - Exact invoice/payment method parameters
- [ckb://docs/fiber/channels](ckb://docs/fiber/channels) - Open a channel before paying over it
- [ckb://docs/fiber/routing-and-graph](ckb://docs/fiber/routing-and-graph) - Multi-hop, trampoline, and why a payment finds no route
- [ckb://docs/fiber/udt-channels](ckb://docs/fiber/udt-channels) - UDT invoices and UDT payment fees
- [ckb://docs/fiber/cross-chain-hub](ckb://docs/fiber/cross-chain-hub) - Pay a Bitcoin Lightning invoice from Fiber

Receiving a Fiber payment means generating an invoice; sending one means calling `send_payment`. Both sides have subtle state machines, and "the payment succeeded" has one correct definition. This page is the working flow with the traps marked.

## Overview

### The Four Expiry Parameters (read this first)

The single most common Fiber coding error is the expiry-parameter name. **For the four names and which call each belongs to, see the canonical table in [overview](ckb://docs/fiber/overview) ("The Four Expiry Parameters").** For invoices the relevant one is `final_expiry_delta` (on `new_invoice`); for payments it is `final_tlc_expiry_delta` (on `send_payment`). `final_cltv` is a no-op.

**Release-vs-debug value trap:** the `0xDFFA0` (≈15 min) `final_expiry_delta` seen in the repo's Bruno e2e tests works only because those run against a debug build. On a release node the minimum is `0x927c00` (160 min) and smaller values are **rejected**. Use `0x927c00` or larger.

### Create an Invoice

```json
{"id":1,"jsonrpc":"2.0","method":"new_invoice","params":[{
  "amount":"0x9502f9000",
  "currency":"Fibt",
  "description":"demo payment",
  "payment_preimage":"0x1a2b3c4d5e6f78901a2b3c4d5e6f78901a2b3c4d5e6f78901a2b3c4d5e6f7890",
  "final_expiry_delta":"0x927c00",
  "hash_algorithm":"ckb_hash"
}]}
```

- `amount` is hex Shannons for CKB (`0x9502f9000` = 400 CKB), or UDT base units for a UDT invoice (add `udt_type_script`; see udt-channels).
- `currency` must match the node's chain: `Fibb` mainnet, `Fibt` testnet, `Fibd` devnet.
- `hash_algorithm` defaults to `ckb_hash`; use `sha256` only for cross-chain.
- The response includes `invoice_address` (a bech32m string starting `fibt…`) — give that to the payer.

Fiber invoices are **bech32m** with HRP `fibb`/`fibt`/`fibd` — they are **not** BOLT11-compatible (cross-chain compatibility goes through the CCH).

### HOLD Invoices

Set `payment_hash` **without** `payment_preimage` to create a HOLD invoice: the node accepts an incoming TLC and holds it (status `Received`) until you call `settle_invoice(payment_hash, payment_preimage)` with a preimage that hashes to that `payment_hash`. Setting both is an error; setting neither generates a random preimage. HOLD invoices are how you make payment acceptance conditional on an external event.

### Invoice Status Lifecycle

`Open → (Received) → Paid`, or terminal `Cancelled` / `Expired`.

| Status | Meaning |
|--------|---------|
| `Open` | Payable (auto-reported as `Expired` once past expiry) |
| `Received` | TLC arrived but not settled — a HOLD invoice awaiting its preimage |
| `Paid` | Settled — **the recipient-side proof of payment** |
| `Cancelled` | Cancelled before settlement |
| `Expired` | Past expiry |

`cancel_invoice(payment_hash)` is rejected **only when status is `Paid` or `Cancelled`** — `Open`, `Received`, and `Expired` are all cancellable (cancelling a `Received`/held invoice releases its TLC set). It is *not* "Open-only."

### Send a Payment

By invoice (the common case):

```json
{"id":2,"jsonrpc":"2.0","method":"send_payment","params":[{
  "invoice":"fibt1q...."
}]}
```

By keysend (no invoice — pay a pubkey directly):

```json
{"id":2,"jsonrpc":"2.0","method":"send_payment","params":[{
  "target_pubkey":"0291a6576bd5a94bd74b27080a48340875338fff9f6d6361fe6b8db8d0d1912fcc",
  "amount":"0x5f5e100",
  "keysend":true
}]}
```

`send_payment` returns immediately with status `Created`; settlement is asynchronous. Useful options: `max_parts` (MPP, payment side — note `allow_mpp` is the *invoice* counterpart), `final_tlc_expiry_delta` (payment-side timelock), `max_fee_amount` / `max_fee_rate` (fee caps), `trampoline_hops` (see routing-and-graph), and `dry_run: true` to validate the route and get the exact fee **without sending**.

### Payment Status

`Created → Inflight → Success | Failed`.

| Status | Meaning |
|--------|---------|
| `Created` | Session created, no HTLC dispatched |
| `Inflight` | First-hop AddTlc sent (note the lowercase `f`) |
| `Success` | All HTLCs settled — **the sender-side proof of payment** |
| `Failed` | Terminated; inspect `failed_error` |

### Verify Settlement — the one correct definition

A payment is settled **only** when:
- **Recipient side:** `get_invoice(payment_hash).status == "Paid"`, or
- **Sender side:** `get_payment(payment_hash).status == "Success"`.

```json
{"method":"get_invoice","params":[{"payment_hash":"0x..."}]}   // poll for status "Paid"
{"method":"get_payment","params":[{"payment_hash":"0x..."}]}   // poll for status "Success"
```

**Settlement is NOT:** invoice creation, an accepted/in-flight TLC, the payer having sent `send_payment`, or the preimage merely being known mid-flight. Always poll to a terminal state and handle both outcomes (invoice: `Paid`/`Cancelled`/`Expired`; payment: `Success`/`Failed`). This dual-check pattern is the one proven in production integrations.

### Fees

- `max_fee_rate`: **per-thousand**, default **5** (= 0.5%) — the rate cap for the whole payment.
- `max_fee_amount`: absolute fee cap in shannons (and, in trampoline mode, the total fee budget).
- Per-hop forwarding fee uses the channel's `tlc_fee_proportional_millionths` (**per-million**, default **1000** = 0.1%): `fee = ceil(amount_forwarded × rate / 1_000_000)`.
- **Directionality:** the fee a hop B charges in A → B → C is computed from B's **outbound** (B → C) channel config, not the inbound one.
- **UDT payments: routing fees are paid in the UDT itself**, not in CKB. On-chain transaction fees (for open/close) are always separate and in CKB. See udt-channels.

### Payment Failure Troubleshooting

Inspect `get_payment(...).failed_error` (and `Channel.failure_detail` for channel-level issues). This table is **curated, not exhaustive** — the node emits many more validation strings (sourced in `rpc/invoice.rs`, `rpc/payment.rs`, `fiber/network.rs`); always handle an unknown `failed_error` gracefully rather than matching only these:

| Symptom / error | Cause | Fix |
|-----------------|-------|-----|
| No route found, right after `ChannelReady` | Gossip graph not synced | Wait ~30 s and retry (see routing-and-graph) |
| No route found, persistent | No path with capacity, or channel `enabled: false` | Check `graph_channels`; open/rebalance a channel; verify `enabled` |
| Error `"final_expiry_delta must be greater than or equal to {min}"` (on `new_invoice`) | Invoice `final_expiry_delta` below the release minimum | Use `0x927c00` (2h40m) or larger |
| Expiry too soon (on `send_payment`) | `final_tlc_expiry_delta` / `tlc_expiry_limit` too small for the accumulated per-hop deltas | Raise the limit to cover the sum of each channel's `tlc_expiry_delta` along the path |
| Fee cap too low | `max_fee_rate` / `max_fee_amount` below the route's cost | Raise the cap |
| MPP / trampoline rejected at invoice creation | Issuing node lacks the feature | The node must enable MPP / trampoline; `allow_mpp` / `allow_trampoline_routing` are invoice params |
| Error `"udt_type_script does not match the invoice"` | Payment-command script ≠ invoice script | Match the script exactly (see udt-channels) |
| Large payment fails, tiny one works | Insufficient outbound liquidity along the path | Use a smaller amount, `trampoline_hops`, or add liquidity |
