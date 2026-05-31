## Description

Run a CKB Fiber node (FNN) end to end: install via Docker (`nervos/fiber`, `ghcr.io/nervosnetwork/fiber`) or build from source (Rust 1.93, ckb-cli v2.0.0); the data-directory layout and `ckb/key` extraction (raw hex, no `0x`, `chmod 600`); `FIBER_SECRET_KEY_PASSWORD` and `RUST_LOG`; the full `config.yml` reference (`fiber` / `rpc` / `ckb` / `services` plus SOCKS5 proxy and Tor onion blocks); funding requirements (~499 CKB for a public channel) and faucets; the first peer connection (multiaddr first — pubkey-only fails on a fresh node); a first-public-channel checklist; safe RPC exposure (the node refuses to bind a public address without `rpc.biscuit_public_key`; biscuit scopes; reverse-proxy TLS); `fnn-cli`; upgrades and `fnn-migrate`; testing with the Bruno e2e suite; and a startup troubleshooting table.

## Related Resources

- [ckb://docs/fiber/overview](ckb://docs/fiber/overview) - Fiber concepts, enum tables, version anchor, and footgun digest
- [ckb://docs/fiber/channels](ckb://docs/fiber/channels) - Open a channel, poll for ChannelReady, shutdown
- [ckb://docs/fiber/rpc-reference](ckb://docs/fiber/rpc-reference) - JSON-RPC method parameters and returns
- [ckb://docs/fiber/invoices-and-payments](ckb://docs/fiber/invoices-and-payments) - Create and pay invoices
- [ckb://docs/fiber/udt-channels](ckb://docs/fiber/udt-channels) - UDT channels and the udt_whitelist
- [ckb://docs/tools/offckb-workflow](ckb://docs/tools/offckb-workflow) - offckb local devnet for CKB development
- [ckb://docs/tools/development-tools](ckb://docs/tools/development-tools) - CKB tooling including ckb-cli

Running a Fiber node requires a CKB private key in the node's data directory, an encryption password, a config file for the target chain, and enough on-chain CKB to fund channels. This page is the working recipe, with the non-obvious steps and operational traps called out.

## Overview

### Install

**Docker (recommended):** release tags publish to `nervos/fiber` (Docker Hub) and `ghcr.io/nervosnetwork/fiber` (GHCR). Images bundle `fnn`, `fnn-cli`, and `fnn-migrate`.

```bash
export FIBER_IMAGE=ghcr.io/nervosnetwork/fiber:<release-tag>
docker run --rm ${FIBER_IMAGE} fnn --help
```

**Build from source:** the repo is a Cargo workspace pinned to **Rust 1.93** (see `rust-toolchain.toml`). Build with `cargo build --release`; the binaries are `fnn`, `fnn-cli`, `fnn-migrate`. The companion `ckb-cli` (v2.0.0) is needed for key management.

### Create the Data Directory and Key

The node needs a data directory containing the `fnn` binary (or use Docker), `config.yml`, and `ckb/key` — a **raw** secp256k1 private key.

```bash
mkdir -p my-fnn/ckb
cp config/testnet/config.yml my-fnn/
# Export a key with ckb-cli, then keep only the first line (the raw private key):
ckb-cli account export --lock-arg <lock_arg> --extended-privkey-path ./my-fnn/ckb/exported-key
head -n 1 ./my-fnn/ckb/exported-key > ./my-fnn/ckb/key
rm ./my-fnn/ckb/exported-key
chmod 600 ./my-fnn/ckb/key
```

**Trap:** the key file must be **raw hex with no `0x` prefix**. FNN wants only the private key, not the extended key's chain code (hence `head -n 1`). Never commit `ckb/key`.

### Start the Node

```bash
FIBER_SECRET_KEY_PASSWORD='YOUR_PASSWORD' RUST_LOG='info,fnn=debug' ./fnn -c config.yml -d .
```

- `FIBER_SECRET_KEY_PASSWORD` is **required** — it encrypts the on-disk key file on first start.
- `-c` = config path, `-d` = data directory. The RocksDB store lives at `<data-dir>/fiber/store`.
- `RUST_LOG` controls verbosity; `info,fnn=debug` enables debug logs only for the `fnn` module.

Docker equivalent (`-p 8228:8228` publishes the **P2P** port only; the RPC port 8227 is intentionally **not** published — see Safe RPC Exposure):

```bash
docker run --rm -it --name fiber-node \
  -e FIBER_SECRET_KEY_PASSWORD='YOUR_PASSWORD' -e RUST_LOG='info' \
  -v "$(pwd)/my-fnn:/fiber" -p 8228:8228 ${FIBER_IMAGE}   # 8228 = P2P
```

### config.yml Reference

The shipped `config/testnet/config.yml` carries only the testnet-necessary keys; `fnn --help` is the exhaustive option reference. Top-level structure:

- **`fiber:`** — `listening_addr` (P2P, default `/ip4/0.0.0.0/tcp/8228`), `announced_node_name`, `bootnode_addrs[]`, `announce_listening_addr`, `announced_addrs[]` (set your public IP here to be reachable), `chain` (`testnet`/`mainnet`/`dev`), optional `proxy` (SOCKS5) and `onion` (Tor) blocks, and `scripts[]` (the funding-lock/commitment-lock cell deps — see on-chain-scripts).
- **`rpc:`** — `listening_addr` (JSON-RPC, **default `127.0.0.1:8227`, localhost-only by design**), optional `biscuit_public_key`, optional `cors_enabled`/`cors_allowed_origins`.
- **`ckb:`** — `rpc_url` (testnet default `https://testnet.ckbapp.dev/`), `udt_whitelist[]` (see udt-channels).
- **`services:`** — `[fiber, rpc, ckb]`. This array gates which subsystems run (and is how a standalone CCH is composed).

**Local multi-node demos:** add `announce_private_addr: true` under `fiber:` so private IPs are announced.

**Privacy (optional):** the `proxy:` block routes outbound P2P through a SOCKS5 proxy (`socks5://127.0.0.1:9050`) for Tor stream isolation; the `onion:` block (with `tor_controller: "127.0.0.1:9051"`) makes the node reachable at a `.onion` address via a running Tor daemon.

### Fund the Node

- **The only protocol-level floor is the per-side reserve: 99 CKB** (98 CKB occupied capacity + 1 CKB shutdown fee) — below this a channel cell cannot exist. There is **no** hardcoded "100 CKB minimum" in the validation path; the 100 CKB figure is a *default auto-accept policy*, not a protocol constant (see next bullet). See channels for the math.
- **A public node's auto-accept minimum is per-node policy, not a constant — query it, don't hardcode.** Read `open_channel_auto_accept_min_ckb_funding_amount` from that node's `node_info`. The source default is **100 CKB** (`DEFAULT_OPEN_CHANNEL_AUTO_ACCEPT_MIN_CKB_FUNDING_AMOUNT`), but public nodes set their own; many currently run **499 CKB** (leaving ~400 usable after the 99-CKB reserve). Budget funding + a change cell + fee. Never bake 499 into code or fixtures — it changes when a node reconfigures.
- Testnet CKB faucet: `https://faucet.nervos.org`. Testnet RUSD faucet: `https://testnet0815.stablepp.xyz/faucet`.

### Inspect the Node

Call `node_info` (no params) to confirm the node is up and get **your own pubkey**, `chain_hash`, `version`, `addresses`, and `channel_count` / `peers_count`:

```bash
curl -s -X POST http://127.0.0.1:8227 -H 'Content-Type: application/json' \
  --data '{"id":1,"jsonrpc":"2.0","method":"node_info","params":[]}'
```

### Connect to the First Peer

**Trap (the most common first-contact failure):** `connect_peer` by `pubkey` alone **fails with `PeerNotFound` on a fresh node**, because the node has no gossip-learned address for that pubkey yet. First contact to an unknown peer **must pass a multiaddr `address`**:

```json
{"id":1,"jsonrpc":"2.0","method":"connect_peer",
 "params":[{"address":"/ip4/54.179.226.154/tcp/8228/p2p/Qmes1EBD4yNo9Ywkfe6eRw9tG1nVNGLDmMud1xJMsoYFKy"}]}
```

A successful `connect_peer` returns `null`. The multiaddr still embeds a base58 `Qm…` PeerId — that is fine; only RPC *parameters* moved from `peer_id` to `pubkey`. After gossip sync, reconnects can use `pubkey`. The optional `addr_type` (`tcp`/`ws`/`wss`) filters which resolved address is used.

The shipped testnet bootnodes (peer discovery only — **bootnodes cannot open channels**) are:
- `/ip4/54.179.226.154/tcp/8228/p2p/Qmes1EBD4yNo9Ywkfe6eRw9tG1nVNGLDmMud1xJMsoYFKy`
- `/ip4/16.163.7.105/tcp/8228/p2p/QmdyQWjPtbK4NWWsvy8s69NGJaQULwgeQDT5ZpNDrTNaeV`

For channel-capable public nodes (with stable pubkeys), see the live directory at `https://dashboard.fiber.channel/nodes` and the repo's `docs/network-nodes.md` — these are live infrastructure, so resolve them at use time rather than hardcoding.

### First Public Channel Checklist

1. Fund the node wallet with ~499 CKB (faucet).
2. `connect_peer` to a **public node** by its multiaddr (not a bootnode).
3. `open_channel` with that node's `pubkey`, `funding_amount` (hex shannons, e.g. `"0xb9e459300"` = 499 CKB), `public: true`. You get back a `temporary_channel_id`.
4. Poll `list_channels` until `state.state_name == "ChannelReady"` (the funding tx must confirm; ~10-30 s on testnet). The real `channel_id` appears here.
5. Routing may still fail immediately after `ChannelReady` until the gossip graph syncs — see routing-and-graph.

Full lifecycle JSON is in channels and invoices-and-payments.

### Safe RPC Exposure

**Hard rule, enforced at startup:** the node **refuses to start** if `rpc.listening_addr` is a public address and `rpc.biscuit_public_key` is not set. The error is:

> `Cannot listen on a public address without a biscuit public key set in the config. Please set rpc.biscuit_public_key or listen on a private interface.`

A "public address" is anything not loopback / private / link-local (including `0.0.0.0`).

To expose RPC safely:
1. Set `rpc.biscuit_public_key: "ed25519/<hex>"` (or the `RPC_BISCUIT_PUBLIC_KEY` env var).
2. Issue **least-privilege biscuit tokens**. Scopes are Datalog rules with **plural resource names**, e.g. `allow if write("channels");`, `allow if read("payments");`. `write` does **not** imply `read`. Available scopes include `read`/`write` over `channels`, `payments`, `invoices`, `peers`, `graph` (read), `node` (read), `cch`, and `watchtower`.
3. Clients send `Authorization: Bearer <token>`. `fnn-cli` accepts `--auth-token`, `--auth-token-file`, or `FNN_AUTH_TOKEN`.
4. Terminate TLS at a reverse proxy (required for `wss` pubsub).

**Never** expose an unauthenticated RPC to an untrusted network.

### fnn-cli

A one-shot + interactive REPL that mirrors the RPC modules; defaults to `http://127.0.0.1:8227`. Flags are kebab-case; output via `-o yaml|json`.

```bash
fnn-cli info                                  # node_info
fnn-cli channel list_channels
fnn-cli channel open_channel --pubkey 02b6d4e3... --funding-amount 49900000000
fnn-cli invoice new_invoice --amount 100 --currency Fibt --description "demo"
docker exec -it fiber-node fnn-cli info        # against a Docker node
```

### Upgrades and Migration

The protocol and storage format may change between versions. **Close channels before upgrading** unless the migration is verified for your version delta — otherwise you risk losing channel data and funds. Alternatively run `fnn-migrate -d <data-dir>` — note `fnn-migrate` resolves the store at `<data-dir>/store`, whereas the running node (`fnn -d <data-dir>`) keeps its store at `<data-dir>/fiber/store`; pass `fnn-migrate` the path such that it finds the actual store directory. Back up the store folder first. v0.8.0 re-keyed several RocksDB column families (PeerId → Pubkey); a unified migration system was introduced after rc2.

### Testing with Bruno

The Fiber repo's `tests/bruno/e2e/` directory holds runnable JSON-RPC sequences (14 scenarios) that use the **current** API — the best source of copy-paste request bodies. Scenarios include `open-use-close-a-channel` (the canonical lifecycle), `3-nodes-transfer`, `router-pay`, `udt`, `cross-chain-hub`, `external-funding-open`, and `watchtower`. **Adapt numeric values before using them on a release node:** Bruno runs against a debug build, so its invoice `final_expiry_delta` values (e.g. `0xDFFA0` ≈ 15 min) are **rejected on release nodes**, where the minimum is `0x927c00` (160 min).

### Startup and Connectivity Troubleshooting

| Symptom | Cause / fix |
|---------|-------------|
| Node won't start, complains about a public address | `rpc.listening_addr` is public but `rpc.biscuit_public_key` is unset — set it or bind to `127.0.0.1` |
| Key decryption / "invalid private key" error | Key file has a `0x` prefix or includes chain code — use raw hex, `head -n 1` of the exported key |
| `connect_peer` returns `PeerNotFound` | Pubkey-only connect on a node with no gossip address — pass a multiaddr `address` |
| Docker node unreachable on 8227 | RPC binds `127.0.0.1` and the port isn't published; change `rpc.listening_addr` and expose with auth |
| Channel never reaches `ChannelReady` | Funding tx not yet confirmed; wait. Confirm wallet had enough CKB (≥ funding + ~62 CKB overhead) |
| `send_payment` "no path found" right after `ChannelReady` | Gossip graph not synced yet — wait and retry (see routing-and-graph) |
