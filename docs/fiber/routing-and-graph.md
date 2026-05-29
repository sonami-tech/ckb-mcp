## Description

Fiber routing and the network graph: gossip sync (active pull then passive subscribe, Cursor = timestamp + message id, "received ≠ applied", ban/rate-limiting) and why a freshly started node returns "no path found" until it syncs — the single most common demo-works-but-fresh-start-fails symptom. The `graph_nodes` / `graph_channels` response shapes with directional `ChannelUpdateInfo`, and the `outbound_liquidity` field that is populated only for your own channels (`null` for remote channels is expected, not a bug). Multi-hop fee and expiry accumulation; trampoline routing (max 5 hops, mandatory `max_fee_amount`, feature-gated, incompatible with self-payment); and channel rebalancing — there is no dedicated rebalance RPC, so use self-keysend or `build_router` + `send_payment_with_router` with `dry_run` first. A routing-failure troubleshooting table keyed by `failed_error` / `failure_detail`.

## Related Resources

- [ckb://docs/fiber/overview](ckb://docs/fiber/overview) - Concepts and footgun digest
- [ckb://docs/fiber/invoices-and-payments](ckb://docs/fiber/invoices-and-payments) - send_payment, fees, and the payment failure table
- [ckb://docs/fiber/rpc-reference](ckb://docs/fiber/rpc-reference) - graph_nodes / graph_channels / build_router parameters
- [ckb://docs/fiber/channels](ckb://docs/fiber/channels) - enabled flag and channel fields that affect routing
- [ckb://docs/fiber/node-setup](ckb://docs/fiber/node-setup) - Connect peers so the graph can populate
- [ckb://docs/fiber/udt-channels](ckb://docs/fiber/udt-channels) - Same-asset path requirement for UDT payments

Fiber finds multi-hop paths from gossip-synced graph data. The graph carries topology and fee policy but not live remote balances, so a path can exist yet still lack capacity — and a fresh node has no graph at all until gossip syncs. Both facts drive most routing confusion.

## Overview

### Gossip Sync — Why Payments Fail Right After Startup

A node builds its routing graph from gossip: on connecting to peers it **actively pulls** history (`GetBroadcastMessages`), then **passively subscribes** (`BroadcastMessagesFilter`). Messages are ordered by a `Cursor` = `(timestamp_ms, message_id)`.

**The footgun:** even with connected peers, `graph_nodes` and `graph_channels` can be empty for ~10-60 seconds after startup. During that window `send_payment` fails with "no path found." This is the classic "it worked in the demo but not on a fresh node" symptom — it is graph-sync latency, not a bad invoice or insufficient balance.

Other gossip behavior worth knowing: "received ≠ applied" — some announcements are dropped after signature or chain validation, so a received message may never enter the graph; and the gossip layer rate-limits and can ban noisy peers. A `Cursor` set too far in the future can permanently miss immutable channel announcements.

**Practical sequence:** start node → connect to peers → wait for `graph_nodes`/`graph_channels` to return non-empty (or simply retry the payment for a while) → then send.

### Inspecting the Graph

```json
{"method":"graph_nodes","params":[{"limit":"0x14"}]}
{"method":"graph_channels","params":[{"limit":"0x14"}]}
```

Both paginate via a `JsonBytes` cursor: pass the previous response's `last_cursor` as `after` to continue. (Note this cursor is `JsonBytes`, unlike `list_payments` which uses a Hash256 cursor.)

- `graph_nodes` → `{ nodes, last_cursor }`; each `NodeInfo` has `pubkey`, `addresses`, `features`, `timestamp`, `chain_hash`, `auto_accept_min_ckb_funding_amount`, `udt_cfg_infos`.
- `graph_channels` → `{ channels, last_cursor }`; each `ChannelInfo` has `channel_outpoint`, `node1`, `node2`, `capacity`, `udt_type_script?`, and two **directional** updates (`update_info_of_node1`, `update_info_of_node2`).

Each `ChannelUpdateInfo` has `timestamp`, `enabled`, **`outbound_liquidity?`**, `tlc_expiry_delta`, `tlc_minimum_value`, `fee_rate`.

**Key nuance:** `outbound_liquidity` is populated **only for channels where your node is a party**. For purely gossip-learned remote channels it is `null` — that is **expected**, not a bug. Gossip never propagates live remote balances; it carries only presence, capacity, and fee policy. So "path exists" does not guarantee "payment succeeds."

`enabled: false` on a channel update means that direction will not route, even if the channel is `ChannelReady`.

### Multi-Hop Payments

`send_payment` with an `invoice` (or `target_pubkey`) automatically finds a multi-hop path. Along a path A → B → C:
- Each intermediate node's forwarding fee uses **its outbound channel's** `tlc_fee_proportional_millionths` (`fee = ceil(amount × rate / 1_000_000)`), paid by the sender.
- Expiry deltas accumulate: each hop adds **its channel's** configured `tlc_expiry_delta` (4h default per hop for both `open_channel` and `accept_channel`), so `tlc_expiry_limit` must cover the sum across the path.

### Trampoline Routing

When the local graph is incomplete, trampoline routing lets a chosen node finish path-finding for you:

```json
{"method":"send_payment","params":[{
  "invoice":"fibt1q...",
  "trampoline_hops":["02b6d4e3ab86a2ca2fad6fae0ecb2e1e559e0b911939872a90abdda6d20302be71"],
  "max_fee_amount":"0x186a0"
}]}
```

- `trampoline_hops = [t1, t2, …]` routes only to `t1`; the inner onion encodes `t1 → … → final`.
- **`max_fee_amount` is mandatory** with trampoline (it is the total fee budget, split across trampoline hops).
- Maximum **5** trampoline hops.
- Incompatible with `allow_self_payment`.
- Trampoline must be enabled on the issuing node (the invoice's `allow_trampoline_routing` feature gate).

### Rebalancing Liquidity

There is **no dedicated rebalance RPC.** Rebalancing shifts liquidity between your own channels via a circular self-payment; only routing fees are spent.

**Automatic (self-keysend):**

```json
{"method":"send_payment","params":[{
  "target_pubkey":"<your_own_node_pubkey>",
  "amount":"0x5f5e100",
  "keysend":true,
  "allow_self_payment":true
}]}
```

**Manual (pin the exact circular route):**

```json
{"method":"build_router","params":[{
  "amount":"0x5f5e100",
  "hops_info":[{"pubkey":"<peer_A>"},{"pubkey":"<peer_B>"},{"pubkey":"<your_own_node>"}]
}]}
```

then feed the returned `router_hops` into `send_payment_with_router` with `keysend: true`. `build_router` returns the explicit `router_hops` (the inspectable, replayable route); add `dry_run: true` on the send to confirm the `fee` with **no side effects** (the response's `routers` array itself is only populated in debug builds).

### Routing Failure Troubleshooting

| `failed_error` / symptom | Cause | Fix |
|--------------------------|-------|-----|
| No path found, just after `ChannelReady` | Gossip graph not synced | Wait ~30 s, retry; check `graph_channels` is non-empty |
| No path found, persistent | No path with capacity; or a channel `enabled: false` | Open/rebalance a channel; verify `enabled` on the route |
| Remote `outbound_liquidity: null` | Expected for gossip-learned channels | Not an error — gossip omits remote balances |
| Expiry too soon | `tlc_expiry_limit` below the accumulated per-hop deltas | Raise the limit |
| Fee cap too low | Route cost exceeds `max_fee_rate` / `max_fee_amount` | Raise the cap |
| Trampoline error | `max_fee_amount` missing, >5 hops, or feature off on issuer | Provide `max_fee_amount`, ≤5 hops, enable trampoline |
| UDT payment no route | No same-asset path; or `udt_type_script` mismatch | Need a path of channels funded with the same UDT (see udt-channels) |
