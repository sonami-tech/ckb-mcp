## Description

UDT-funded Fiber channels and UDT invoices — Fiber's multi-asset feature beyond native CKB. Opening a channel with `funding_udt_type_script`, the `udt_whitelist` config requirement (an absent entry rejects the channel; a missing `auto_accept_amount` forces manual accept), and the rule that `udt_type_script` must be **byte-identical** (including args) across channel open, invoice, and payment or the transfer is silently rejected. UDT amounts are raw integer base units, not decimal display units. Routing fees on UDT channels are paid **in the UDT**, while on-chain transaction fees remain in CKB, and UDT channels reserve **more** than the 99-CKB CKB baseline. The testnet RUSD and mainnet USDI type scripts (verified from config and the public-node directory), with the caveat that mainnet public nodes hold no USDI yet. Same-asset path requirement for routing UDT payments. Worked open + invoice JSON using the current API.

## Related Resources

- [ckb://docs/fiber/overview](ckb://docs/fiber/overview) - Multi-asset model and footgun digest
- [ckb://docs/fiber/channels](ckb://docs/fiber/channels) - Channel lifecycle (UDT channels follow the same flow)
- [ckb://docs/fiber/invoices-and-payments](ckb://docs/fiber/invoices-and-payments) - Invoice and payment mechanics and fees
- [ckb://docs/fiber/node-setup](ckb://docs/fiber/node-setup) - The ckb.udt_whitelist config section
- [ckb://docs/fiber/routing-and-graph](ckb://docs/fiber/routing-and-graph) - Same-asset path requirement
- [ckb://docs/tokens/udt-overview](ckb://docs/tokens/udt-overview) - CKB UDT/xUDT token model
- [ckb://docs/fiber/on-chain-scripts](ckb://docs/fiber/on-chain-scripts) - UDT cell data on channel cells

Fiber channels can carry a CKB UDT asset (e.g. RUSD on testnet, USDI on mainnet) instead of native CKB. The mechanics mirror CKB channels, with three extra rules that cause most UDT bugs: the node must whitelist the UDT, the type script must match byte-for-byte everywhere, and amounts/fees use the UDT's units.

## Overview

### Whitelist the UDT (required)

A node will only accept a UDT channel if the UDT is in its `ckb.udt_whitelist`:

```yaml
ckb:
  udt_whitelist:
    - name: RUSD
      script:
        code_hash: 0x1142755a044bf2ee358cba9f2da187ce928c91cd4dc8692ded0337efa677d21a
        hash_type: type
        args: 0x878fcc6f1f08d48e87bb1c3b3d5083f23f8a39c5d5c764f253b55b998526439b
      cell_deps:
        - type_id:
            code_hash: 0x00000000000000000000000000000000000000000000000000545950455f4944
            hash_type: type
            args: 0x97d30b723c0b2c66e9cb8d4d0df4ab5d7222cbb00d4a9a2055ce2e5d7f0d8b0f
      auto_accept_amount: 1000000000
```

- **Absent whitelist entry → the node rejects the UDT channel.**
- **Missing `auto_accept_amount` → the channel must be manually accepted** (`accept_channel`); a present amount auto-accepts at that threshold.

### Open a UDT Channel

Same as a CKB channel, plus `funding_udt_type_script`:

```json
{"method":"open_channel","params":[{
  "pubkey":"02b6d4e3ab86a2ca2fad6fae0ecb2e1e559e0b911939872a90abdda6d20302be71",
  "funding_amount":"0x3b9aca00",
  "public":true,
  "funding_udt_type_script":{
    "code_hash":"0x1142755a044bf2ee358cba9f2da187ce928c91cd4dc8692ded0337efa677d21a",
    "hash_type":"type",
    "args":"0x878fcc6f1f08d48e87bb1c3b3d5083f23f8a39c5d5c764f253b55b998526439b"
  }
}]}
```

Here `funding_amount` is the **UDT base unit** amount, not CKB. The CKB collateral is reserved separately (and is *larger* than the 99-CKB CKB-channel baseline — the channel cell carries 16 extra bytes of UDT amount data plus the UDT type script). The wallet still needs CKB capacity for that reserve and for on-chain fees.

### Create and Pay a UDT Invoice

Add `udt_type_script` to `new_invoice` (matching the channel's `funding_udt_type_script` byte-for-byte):

```json
{"method":"new_invoice","params":[{
  "amount":"0x5f5e100",
  "currency":"Fibt",
  "final_expiry_delta":"0x927c00",
  "payment_preimage":"0x1a2b3c4d5e6f78901a2b3c4d5e6f78901a2b3c4d5e6f78901a2b3c4d5e6f7890",
  "udt_type_script":{
    "code_hash":"0x1142755a044bf2ee358cba9f2da187ce928c91cd4dc8692ded0337efa677d21a",
    "hash_type":"type",
    "args":"0x878fcc6f1f08d48e87bb1c3b3d5083f23f8a39c5d5c764f253b55b998526439b"
  }
}]}
```

`amount` (`0x5f5e100`) is in the UDT's base units. To pay, use the returned invoice with `send_payment` (which also accepts a matching `udt_type_script` for keysend).

### The Three UDT Rules That Bite

1. **Byte-identical type script everywhere.** The `udt_type_script` on the invoice/payment must match the channel's `funding_udt_type_script` exactly — same `code_hash`, `hash_type`, and `args` (including arg byte order). A mismatch causes a silent rejection / no-route.
2. **Raw base units, not decimals.** `amount` is the integer on-chain UDT amount, never a human-readable decimal. RUSD/USDI use their own base-unit scale; do not divide or format.
3. **Fees are in the UDT.** Routing/TLC forwarding fees on a UDT channel are denominated in the UDT itself (`tlc_fee_proportional_millionths` applies to the UDT amount). Only the on-chain open/close transaction fees are in CKB. An assistant that assumes CKB fees on a UDT payment is wrong.

### Routing UDT Payments

A multi-hop UDT payment needs a path of channels **all funded with the same UDT** — you cannot route a UDT payment across CKB channels or a different UDT. Graph and liquidity checks (see routing-and-graph) apply per asset.

### Known Network Assets

| Asset | Network | code_hash | args |
|-------|---------|-----------|------|
| RUSD | testnet | `0x1142755a044bf2ee358cba9f2da187ce928c91cd4dc8692ded0337efa677d21a` | `0x878fcc6f1f08d48e87bb1c3b3d5083f23f8a39c5d5c764f253b55b998526439b` |
| USDI | mainnet | `0xbfa35a9c38a676682b65ade8f02be164d48632281477e36f8dc2f41f79e56bfc` | `0xd591ebdc69626647e056e13345fd830c8b876bb06aa07ba610479eb77153ea9f` |

All `hash_type: type`. The **type scripts** above are stable identifiers. The per-node **auto-accept thresholds, liquidity, and availability** are point-in-time and live — never hardcode them; query a node's `node_info` (its `udt_cfg_infos` and auto-accept amounts), the node's `config.yml`, or the live directory. In particular, mainnet public nodes may hold little or no USDI liquidity, so confirm a node actually offers the asset before attempting a mainnet UDT channel.

### Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Node rejects the UDT channel | UDT not in `udt_whitelist` | Add the type script to the node config |
| Channel stuck pending, never accepts | No `auto_accept_amount` for that UDT | Peer must `accept_channel`, or set the threshold |
| Payment rejected / no route on a UDT invoice | `udt_type_script` not byte-identical to the channel | Match `code_hash`/`hash_type`/`args` exactly |
| Amounts off by orders of magnitude | Decimal display units used | Use raw integer base units |
| Fee accounting looks wrong | Assumed CKB fees | UDT routing fees are in the UDT; only on-chain fees are CKB |
