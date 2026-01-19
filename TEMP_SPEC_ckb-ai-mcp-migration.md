# CKB-AI-MCP Migration Specification

> **TEMPORARY FILE** - Delete after migration is completed, verified, and accepted.

## Overview

This specification details the migration from three separate MCP servers to a single unified server (`ckb-ai-mcp`) implementing MCP protocol version 2025-11-25 with Streamable HTTP transport.

---

## Table of Contents

1. [Project Metadata](#1-project-metadata)
2. [Architecture Changes](#2-architecture-changes)
3. [Technology Stack](#3-technology-stack)
4. [CLI Specification](#4-cli-specification)
5. [MCP Protocol Implementation](#5-mcp-protocol-implementation)
6. [Tool Specifications](#6-tool-specifications)
7. [Resource Specifications](#7-resource-specifications)
8. [Prompt Specifications](#8-prompt-specifications)
9. [Search Implementation](#9-search-implementation)
10. [Statistics Integration](#10-statistics-integration)
11. [HTTP Endpoints](#11-http-endpoints)
12. [Directory Structure](#12-directory-structure)
13. [Cargo Configuration](#13-cargo-configuration)
14. [Implementation Phases](#14-implementation-phases)
15. [Testing Requirements](#15-testing-requirements)
16. [Cleanup Tasks](#16-cleanup-tasks)

---

## 1. Project Metadata

| Field | Value |
|-------|-------|
| Binary Name | `ckb-ai-mcp` |
| Package Name | `ckb-ai-mcp` |
| Version | `1.0.0` |
| Rust Edition | `2024` |
| Default Port | `3112` |
| Protocol Version | `2025-11-25` |
| Transport | Streamable HTTP (via rmcp) |

---

## 2. Architecture Changes

### Current Architecture (3 Servers)

```
┌─────────────────────┐  ┌─────────────────────┐  ┌─────────────────────┐
│  ckb-rpc-server     │  │  ckb-docs-server    │  │  ckb-tools-server   │
│  Port: 8001         │  │  Port: 8002         │  │  Port: 8003         │
│  51 tools           │  │  87 resources       │  │  8 tools            │
│  HTTP POST /mcp     │  │  HTTP POST /mcp     │  │  HTTP POST /mcp     │
│  Protocol: 2024-11  │  │  Protocol: 2024-11  │  │  + POST /deploy/file│
└─────────────────────┘  └─────────────────────┘  └─────────────────────┘
```

### New Architecture (1 Server)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                            ckb-ai-mcp                                   │
│                            Port: 3112                                   │
├─────────────────────────────────────────────────────────────────────────┤
│  MCP Endpoint: /mcp (Streamable HTTP)                                   │
│  - POST: Client requests/notifications                                  │
│  - GET:  Server-initiated messages (SSE)                                │
│  - DELETE: Session termination                                          │
├─────────────────────────────────────────────────────────────────────────┤
│  Additional Endpoints:                                                  │
│  - GET /health                                                          │
│  - GET /stats                                                           │
│  - POST /deploy/file (multipart upload)                                 │
├─────────────────────────────────────────────────────────────────────────┤
│  Capabilities:                                                          │
│  - 61 tools (51 rpc + 8 dev + 2 search)                                │
│  - 87 resources (documentation)                                         │
│  - 4 prompts (workflows)                                                │
│  - Protocol: 2025-11-25                                                 │
└─────────────────────────────────────────────────────────────────────────┘
```

### Feature Flags

| Flag | Effect | CKB Node Required |
|------|--------|-------------------|
| (default) | All features enabled | Yes |
| `--docs-only` | Only resources and prompts | No |
| `--rpc-only` | Only RPC tools | Yes |
| `--tools-only` | Only dev tools | Yes |
| `--no-prompts` | Disable prompts feature | Depends on other flags |

---

## 3. Technology Stack

### Core Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| `rmcp` | `0.13` | MCP protocol implementation |
| `axum` | `0.8` | HTTP framework |
| `tokio` | `1` | Async runtime |
| `serde` | `1` | Serialization |
| `schemars` | `1.0` | JSON Schema generation |
| `clap` | `4` | CLI parsing |
| `tracing` | `0.1` | Logging |

### CKB Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| `ckb-sdk` | `4.4` | Transaction building |
| `ckb-types` | `0.202` | Core types |
| `ckb-jsonrpc-types` | `0.202` | RPC types |
| `ckb-hash` | `0.202` | Hashing |

### rmcp Features

```toml
rmcp = { version = "0.13", features = [
    "server",
    "macros",
    "transport-streamable-http-server"
] }
```

---

## 4. CLI Specification

### Arguments

| Argument | Type | Default | Description |
|----------|------|---------|-------------|
| `--port` | u16 | `3112` | Server port |
| `--host` | String | `0.0.0.0` | Bind address |
| `--ckb-rpc` | String | `http://127.0.0.1:8114` | CKB node URL |
| `--private-key` | String | (test key) | Private key for signing |
| `--docs-path` | PathBuf | `./docs` | Documentation directory |
| `--stats-db` | PathBuf | `./data/ckb-ai-mcp-stats.redb` | Stats database |
| `--log-level` | String | `info` | Log level |
| `--docs-only` | bool | `false` | Enable only docs/prompts |
| `--rpc-only` | bool | `false` | Enable only RPC tools |
| `--tools-only` | bool | `false` | Enable only dev tools |
| `--no-prompts` | bool | `false` | Disable prompts |

### Usage Examples

```bash
# Full server (all features)
ckb-ai-mcp --port 3112 --ckb-rpc http://192.168.0.73:28114

# Docs-only mode (no CKB node)
ckb-ai-mcp --port 3112 --docs-only

# RPC-only with custom node
ckb-ai-mcp --rpc-only --ckb-rpc http://192.168.0.73:18114

# Production with custom key
ckb-ai-mcp --ckb-rpc http://node:8114 --private-key 0x...

# Debug mode
ckb-ai-mcp --log-level debug --ckb-rpc http://localhost:8114
```

---

## 5. MCP Protocol Implementation

### Initialize Response

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocolVersion": "2025-11-25",
    "capabilities": {
      "tools": { "listChanged": true },
      "resources": { "listChanged": true, "subscribe": true },
      "prompts": { "listChanged": true }
    },
    "serverInfo": {
      "name": "ckb-ai-mcp",
      "version": "1.0.0"
    },
    "instructions": "CKB blockchain development server providing RPC queries, development tools, documentation, and guided workflows."
  }
}
```

### Session Management

- Session ID assigned via `Mcp-Session-Id` header on initialize
- Sessions managed by rmcp's `LocalSessionManager`
- Session required for all requests after initialize

### Supported Methods

| Method | Description |
|--------|-------------|
| `initialize` | Protocol handshake |
| `notifications/initialized` | Client ready notification |
| `tools/list` | List available tools |
| `tools/call` | Execute a tool |
| `resources/list` | List available resources |
| `resources/read` | Read a resource |
| `prompts/list` | List available prompts |
| `prompts/get` | Get prompt messages |

---

## 6. Tool Specifications

### Tool Definition Schema (Enhanced)

```json
{
  "name": "rpc_get_block",
  "title": "Get Block by Hash",
  "description": "Retrieve a complete CKB block by its hash. Returns header, transactions, proposals, and uncles.",
  "category": "query",
  "inputSchema": {
    "type": "object",
    "properties": {
      "block_hash": {
        "type": "string",
        "description": "Block hash (0x-prefixed, 64 hex characters)"
      }
    },
    "required": ["block_hash"]
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "header": { "type": "object" },
      "transactions": { "type": "array" },
      "proposals": { "type": "array" },
      "uncles": { "type": "array" }
    }
  },
  "annotations": {
    "audience": ["assistant"],
    "priority": 0.8
  }
}
```

### RPC Tools (51 tools)

#### Category: query (22 tools)

| Old Name | New Name | Description |
|----------|----------|-------------|
| `get_block` | `rpc_get_block` | Get block by hash |
| `get_block_by_number` | `rpc_get_block_by_number` | Get block by number |
| `get_header` | `rpc_get_header` | Get block header by hash |
| `get_header_by_number` | `rpc_get_header_by_number` | Get block header by number |
| `get_transaction` | `rpc_get_transaction` | Get transaction by hash |
| `get_block_hash` | `rpc_get_block_hash` | Get block hash by number |
| `get_tip_header` | `rpc_get_tip_header` | Get tip block header |
| `get_tip_block_number` | `rpc_get_tip_block_number` | Get tip block number |
| `get_current_epoch` | `rpc_get_current_epoch` | Get current epoch info |
| `get_epoch_by_number` | `rpc_get_epoch_by_number` | Get epoch by number |
| `get_live_cell` | `rpc_get_live_cell` | Get live cell by outpoint |
| `get_fork_block` | `rpc_get_fork_block` | Get fork block by hash |

#### Category: search (4 tools)

| Old Name | New Name | Description |
|----------|----------|-------------|
| `get_indexer_tip` | `rpc_get_indexer_tip` | Get indexer sync tip |
| `get_cells` | `rpc_search_cells` | Search cells by criteria |
| `get_transactions` | `rpc_search_transactions` | Search transactions by criteria |
| `get_cells_capacity` | `rpc_get_cells_capacity` | Get total capacity of matching cells |

#### Category: submit (2 tools)

| Old Name | New Name | Description |
|----------|----------|-------------|
| `send_transaction` | `rpc_submit_transaction` | Submit transaction to network |
| `test_tx_pool_accept` | `rpc_test_transaction` | Test transaction acceptance |

#### Category: status (11 tools)

| Old Name | New Name | Description |
|----------|----------|-------------|
| `local_node_info` | `rpc_get_node_info` | Get local node info |
| `sync_state` | `rpc_get_sync_state` | Get sync state |
| `get_peers` | `rpc_get_peers` | Get connected peers |
| `tx_pool_info` | `rpc_get_pool_info` | Get tx pool info |
| `tx_pool_ready` | `rpc_get_pool_ready` | Check pool readiness |
| `get_raw_tx_pool` | `rpc_get_pool_transactions` | Get pool transaction IDs |
| `get_pool_tx_detail_info` | `rpc_get_pool_tx_detail` | Get pool tx details |
| `get_blockchain_info` | `rpc_get_blockchain_info` | Get blockchain info |
| `get_consensus` | `rpc_get_consensus` | Get consensus params |
| `get_deployments_info` | `rpc_get_deployments` | Get deployment info |

#### Category: calculate (6 tools)

| Old Name | New Name | Description |
|----------|----------|-------------|
| `estimate_cycles` | `rpc_estimate_cycles` | Estimate script cycles |
| `estimate_fee_rate` | `rpc_estimate_fee_rate` | Estimate fee rate |
| `calculate_dao_maximum_withdraw` | `rpc_calculate_dao_withdraw` | Calculate DAO withdrawal |
| `get_block_economic_state` | `rpc_get_block_economics` | Get block economics |
| `get_block_median_time` | `rpc_get_block_median_time` | Get median time |
| `get_block_filter` | `rpc_get_block_filter` | Get BIP-157 filter |

#### Category: verify (2 tools)

| Old Name | New Name | Description |
|----------|----------|-------------|
| `get_transaction_proof` | `rpc_get_transaction_proof` | Generate tx proof |
| `verify_transaction_proof` | `rpc_verify_transaction_proof` | Verify tx proof |

### Development Tools (8 tools)

#### Category: deploy (1 tool)

| Old Name | New Name | Description |
|----------|----------|-------------|
| `DeployCellData` | `dev_deploy_cell` | Deploy cell with data (max 1KB inline) |

#### Category: account (4 tools)

| Old Name | New Name | Description |
|----------|----------|-------------|
| `GetAddressBalance` | `dev_get_balance` | Get address balance |
| `GetDefaultAccountInfo` | `dev_get_account_info` | Get default account info |
| `GenerateLockInfo` | `dev_generate_lock_info` | Generate lock from private key |
| `GetLockInfoFromAddress` | `dev_get_lock_from_address` | Extract lock from address |

#### Category: chain (2 tools)

| Old Name | New Name | Description |
|----------|----------|-------------|
| `GetChainType` | `dev_get_chain_type` | Get chain type |
| `GetGenesisHash` | `dev_get_genesis_hash` | Get genesis hash |

#### Category: faucet (1 tool)

| Old Name | New Name | Description |
|----------|----------|-------------|
| `RequestTestnetFunds` | `dev_request_faucet` | Request testnet funds |

### Search Tools (2 new tools)

| Name | Description | Category |
|------|-------------|----------|
| `find_tools` | Search tools by keyword/category | search |
| `find_resources` | Search resources by keyword/category | search |

#### find_tools Schema

```json
{
  "name": "find_tools",
  "title": "Find Tools",
  "description": "Search available tools by keyword or category. Returns matching tool names and descriptions.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "query": {
        "type": "string",
        "description": "Search keyword (matches tool name and description)"
      },
      "category": {
        "type": "string",
        "enum": ["query", "search", "submit", "status", "calculate", "verify", "deploy", "account", "chain", "faucet"],
        "description": "Filter by category"
      },
      "limit": {
        "type": "integer",
        "default": 10,
        "description": "Maximum results to return"
      }
    }
  }
}
```

#### find_resources Schema

```json
{
  "name": "find_resources",
  "title": "Find Resources",
  "description": "Search documentation resources by keyword or category. Returns matching resource URIs and descriptions.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "query": {
        "type": "string",
        "description": "Search keyword (matches resource name and description)"
      },
      "category": {
        "type": "string",
        "enum": ["start", "concepts", "patterns", "api", "protocols", "troubleshoot", "deploy", "examples", "ecosystem", "reference"],
        "description": "Filter by category"
      },
      "limit": {
        "type": "integer",
        "default": 10,
        "description": "Maximum results to return"
      }
    }
  }
}
```

---

## 7. Resource Specifications

### Resource Definition Schema (Enhanced)

```json
{
  "uri": "ckb-dev-context://concepts/cell-model",
  "name": "cell-model",
  "title": "CKB Cell Model",
  "description": "Core CKB data structure. Cells are the fundamental storage unit containing capacity, lock script, type script, and data.",
  "mimeType": "text/markdown",
  "size": 8192,
  "category": "concepts",
  "annotations": {
    "audience": ["assistant"],
    "priority": 0.9
  }
}
```

### Category Mapping

| New Category | Old Path(s) | Count | Description |
|--------------|-------------|-------|-------------|
| `start` | getting-started/ | 4 | Getting started guides, tool recommendations |
| `concepts` | concepts/, concepts-for-coding/ | 11 | Core CKB concepts |
| `patterns` | patterns/ | 30 | Development patterns and best practices |
| `api` | api-reference/ | 14 | SDK examples and API references |
| `protocols` | protocols/ | 11 | Protocol specifications |
| `troubleshoot` | troubleshooting/ | 7 | Error guides and debugging |
| `deploy` | deployment/ | 2 | Deployment guides |
| `examples` | examples/, integration-examples/ | 3 | Code examples |
| `ecosystem` | ecosystem/, education/, tools/ | 4 | Project directory, education |
| `reference` | (root) | 1 | AI quick reference |

### Complete Resource List

#### Category: start (4 resources)

| URI | Name |
|-----|------|
| `ckb-dev-context://start/developer-resources` | developer-resources |
| `ckb-dev-context://start/offckb-workflow` | offckb-workflow |
| `ckb-dev-context://start/tool-recommendations` | tool-recommendations |

#### Category: concepts (11 resources)

| URI | Name |
|-----|------|
| `ckb-dev-context://concepts/cell-model` | cell-model |
| `ckb-dev-context://concepts/advanced-cells` | advanced-cells |
| `ckb-dev-context://concepts/transaction-structure` | transaction-structure |
| `ckb-dev-context://concepts/molecule-serialization` | molecule-serialization |
| `ckb-dev-context://concepts/script-groups` | script-groups |
| `ckb-dev-context://concepts/syscalls` | syscalls |
| `ckb-dev-context://concepts/network-history` | network-history |
| `ckb-dev-context://concepts/header-deps` | header-deps |
| `ckb-dev-context://concepts/lock-value-relationships` | lock-value-relationships |
| `ckb-dev-context://concepts/cell-lifecycle` | cell-lifecycle |
| `ckb-dev-context://concepts/transaction-lifecycle` | transaction-lifecycle |

#### Category: patterns (30 resources)

| URI | Name |
|-----|------|
| `ckb-dev-context://patterns/minimal-lock-script` | minimal-lock-script |
| `ckb-dev-context://patterns/minimal-type-script` | minimal-type-script |
| `ckb-dev-context://patterns/token-creation` | token-creation |
| `ckb-dev-context://patterns/udt-tokens` | udt-tokens |
| `ckb-dev-context://patterns/dao-development` | dao-development |
| `ckb-dev-context://patterns/omnilock-development` | omnilock-development |
| `ckb-dev-context://patterns/omnilock-interoperability` | omnilock-interoperability |
| `ckb-dev-context://patterns/spore-development` | spore-development |
| `ckb-dev-context://patterns/cota-nft-development` | cota-nft-development |
| `ckb-dev-context://patterns/ickb-development` | ickb-development |
| `ckb-dev-context://patterns/ickb-liquidity` | ickb-liquidity |
| `ckb-dev-context://patterns/molecule-schema` | molecule-schema |
| `ckb-dev-context://patterns/rust-script-development` | rust-script-development |
| `ckb-dev-context://patterns/script-development` | script-development |
| `ckb-dev-context://patterns/script-source` | script-source |
| `ckb-dev-context://patterns/transaction-building` | transaction-building |
| `ckb-dev-context://patterns/simple-transfer` | simple-transfer |
| `ckb-dev-context://patterns/operation-detection` | operation-detection |
| `ckb-dev-context://patterns/type-id` | type-id |
| `ckb-dev-context://patterns/seed-cell` | seed-cell |
| `ckb-dev-context://patterns/file-storage` | file-storage |
| `ckb-dev-context://patterns/system-scripts` | system-scripts |
| `ckb-dev-context://patterns/development-tools` | development-tools |
| `ckb-dev-context://patterns/c-to-rust-migration` | c-to-rust-migration |
| `ckb-dev-context://patterns/cobuild-integration` | cobuild-integration |
| `ckb-dev-context://patterns/ssri-implementation` | ssri-implementation |
| `ckb-dev-context://patterns/dob-development` | dob-development |
| `ckb-dev-context://patterns/proxy-lock` | proxy-lock |
| `ckb-dev-context://patterns/proxy-lock-testing` | proxy-lock-testing |
| `ckb-dev-context://patterns/contract-workspace` | contract-workspace |

#### Category: api (14 resources)

| URI | Name |
|-----|------|
| `ckb-dev-context://api/ccc-patterns` | ccc-patterns |
| `ckb-dev-context://api/ccc-cross-chain` | ccc-cross-chain |
| `ckb-dev-context://api/ccc-ssri` | ccc-ssri |
| `ckb-dev-context://api/ckb-rust-sdk` | ckb-rust-sdk |
| `ckb-dev-context://api/molecule-examples` | molecule-examples |
| `ckb-dev-context://api/omnilock-examples` | omnilock-examples |
| `ckb-dev-context://api/omnilock-ethereum` | omnilock-ethereum |
| `ckb-dev-context://api/spore-sdk` | spore-sdk |
| `ckb-dev-context://api/cota-sdk` | cota-sdk |
| `ckb-dev-context://api/ickb-sdk` | ickb-sdk |
| `ckb-dev-context://api/syscalls-ref` | syscalls-ref |
| `ckb-dev-context://api/well-known-hashes` | well-known-hashes |
| `ckb-dev-context://api/sdk-patterns` | sdk-patterns |
| `ckb-dev-context://api/xudt-minting` | xudt-minting |

#### Category: protocols (11 resources)

| URI | Name |
|-----|------|
| `ckb-dev-context://protocols/omnilock` | omnilock |
| `ckb-dev-context://protocols/spore` | spore |
| `ckb-dev-context://protocols/spore-digital-objects` | spore-digital-objects |
| `ckb-dev-context://protocols/cota` | cota |
| `ckb-dev-context://protocols/ickb` | ickb |
| `ckb-dev-context://protocols/rgb-plus-plus` | rgb-plus-plus |
| `ckb-dev-context://protocols/ckbfs` | ckbfs |
| `ckb-dev-context://protocols/cobuild` | cobuild |
| `ckb-dev-context://protocols/open-transaction` | open-transaction |
| `ckb-dev-context://protocols/ssri` | ssri |
| `ckb-dev-context://protocols/xudt` | xudt |

#### Category: troubleshoot (7 resources)

| URI | Name |
|-----|------|
| `ckb-dev-context://troubleshoot/common-script-errors` | common-script-errors |
| `ckb-dev-context://troubleshoot/rust-script-issues` | rust-script-issues |
| `ckb-dev-context://troubleshoot/ickb-debugging` | ickb-debugging |
| `ckb-dev-context://troubleshoot/omnilock-errors` | omnilock-errors |
| `ckb-dev-context://troubleshoot/xudt-errors` | xudt-errors |
| `ckb-dev-context://troubleshoot/transaction-errors` | transaction-errors |
| `ckb-dev-context://troubleshoot/spore-errors` | spore-errors |

#### Category: deploy (2 resources)

| URI | Name |
|-----|------|
| `ckb-dev-context://deploy/binary-deployment` | binary-deployment |
| `ckb-dev-context://deploy/cota-infrastructure` | cota-infrastructure |

#### Category: examples (3 resources)

| URI | Name |
|-----|------|
| `ckb-dev-context://examples/cell-collection` | cell-collection |
| `ckb-dev-context://examples/calculate-file-hashes` | calculate-file-hashes |
| `ckb-dev-context://examples/consolidate-cells` | consolidate-cells |

#### Category: ecosystem (4 resources)

| URI | Name |
|-----|------|
| `ckb-dev-context://ecosystem/project-directory` | project-directory |
| `ckb-dev-context://ecosystem/interactive-courses` | interactive-courses |
| `ckb-dev-context://ecosystem/ssri-server` | ssri-server |

#### Category: reference (1 resource)

| URI | Name |
|-----|------|
| `ckb-dev-context://reference/ai-quick-ref` | ai-quick-ref |

---

## 8. Prompt Specifications

### Prompt Definition Schema

```json
{
  "name": "explain_concept",
  "title": "Explain CKB Concept",
  "description": "Get a detailed explanation of a CKB blockchain concept with embedded documentation",
  "arguments": [
    {
      "name": "concept",
      "description": "The concept to explain (e.g., cell-model, transaction-structure)",
      "required": true
    }
  ]
}
```

### Prompt 1: explain_concept

**Name:** `explain_concept`
**Title:** Explain CKB Concept
**Description:** Get a detailed explanation of a CKB concept using embedded documentation
**Arguments:**
- `concept` (required): Concept name matching a resource in concepts category

**Returned Messages:**

```json
{
  "messages": [
    {
      "role": "user",
      "content": {
        "type": "text",
        "text": "Explain the CKB concept: {concept}\n\nUse the embedded documentation below as your primary source. Explain in clear terms suitable for a developer new to CKB. Include practical code examples where helpful."
      }
    },
    {
      "role": "user",
      "content": {
        "type": "resource",
        "resource": {
          "uri": "ckb-dev-context://concepts/{concept}",
          "mimeType": "text/markdown",
          "text": "{content of matching documentation}"
        }
      }
    }
  ]
}
```

### Prompt 2: analyze_transaction

**Name:** `analyze_transaction`
**Title:** Analyze CKB Transaction
**Description:** Analyze a CKB transaction structure and explain its components
**Arguments:**
- `tx_hash` (required): Transaction hash to analyze

**Returned Messages:**

```json
{
  "messages": [
    {
      "role": "user",
      "content": {
        "type": "text",
        "text": "Analyze this CKB transaction: {tx_hash}\n\nExplain:\n1. What inputs are being consumed (cells, capacity)\n2. What outputs are being created\n3. What scripts (lock/type) are being executed\n4. Whether this is a transfer, token operation, or other operation\n5. Any potential issues or unusual patterns\n\nUse the rpc_get_transaction tool to fetch the transaction data."
      }
    },
    {
      "role": "user",
      "content": {
        "type": "resource",
        "resource": {
          "uri": "ckb-dev-context://concepts/transaction-structure",
          "mimeType": "text/markdown",
          "text": "{content of transaction-structure doc}"
        }
      }
    }
  ]
}
```

### Prompt 3: debug_script_error

**Name:** `debug_script_error`
**Title:** Debug Script Error
**Description:** Debug a CKB script error using troubleshooting guides
**Arguments:**
- `error` (required): Error message or code to debug

**Returned Messages:**

```json
{
  "messages": [
    {
      "role": "user",
      "content": {
        "type": "text",
        "text": "Debug this CKB script error: {error}\n\nSteps to follow:\n1. Identify the error code meaning from the troubleshooting guide\n2. Explain the likely cause\n3. Suggest debugging steps\n4. Provide example fixes if applicable"
      }
    },
    {
      "role": "user",
      "content": {
        "type": "resource",
        "resource": {
          "uri": "ckb-dev-context://troubleshoot/common-script-errors",
          "mimeType": "text/markdown",
          "text": "{content of common-script-errors doc}"
        }
      }
    }
  ]
}
```

### Prompt 4: deploy_guide

**Name:** `deploy_guide`
**Title:** Cell Deployment Guide
**Description:** Step-by-step guide for deploying a cell/contract to CKB
**Arguments:**
- `contract_name` (optional): Name of contract being deployed

**Returned Messages:**

```json
{
  "messages": [
    {
      "role": "user",
      "content": {
        "type": "text",
        "text": "Guide me through deploying {contract_name or 'a cell/contract'} to CKB.\n\nFollow this workflow:\n1. Verify I have the compiled binary ready\n2. Check my account balance using dev_get_account_info\n3. Explain the deployment options (inline vs file upload)\n4. Walk me through the actual deployment using dev_deploy_cell or /deploy/file\n5. Verify the deployment succeeded\n6. Explain how to reference the deployed cell in transactions"
      }
    },
    {
      "role": "user",
      "content": {
        "type": "resource",
        "resource": {
          "uri": "ckb-dev-context://deploy/binary-deployment",
          "mimeType": "text/markdown",
          "text": "{content of binary-deployment doc}"
        }
      }
    }
  ]
}
```

---

## 9. Search Implementation

### Search Algorithm

```
function search_tools(query, category, limit):
    results = []

    for tool in all_tools:
        if category and tool.category != category:
            continue

        score = 0
        query_lower = query.lower()

        # Exact name match (highest priority)
        if query_lower == tool.name.lower():
            score = 100
        # Name contains query
        elif query_lower in tool.name.lower():
            score = 80
        # Description contains query
        elif query_lower in tool.description.lower():
            score = 60
        # Word match in description
        elif any(word in tool.description.lower() for word in query_lower.split()):
            score = 40

        if score > 0:
            results.append((tool, score))

    # Sort by score descending
    results.sort(key=lambda x: x[1], reverse=True)

    return results[:limit]
```

### Search Response Format

```json
{
  "content": [{
    "type": "text",
    "text": "Found 5 tools matching 'transaction':\n\n1. rpc_get_transaction (query)\n   Get transaction by hash\n\n2. rpc_search_transactions (search)\n   Search transactions by criteria\n\n3. rpc_submit_transaction (submit)\n   Submit transaction to network\n\n..."
  }]
}
```

---

## 10. Statistics Integration

### Stats Recording Points

| Event | Method | Location |
|-------|--------|----------|
| Tool call | `stats.record_tool_call(name)` | After successful tool execution |
| Resource read | `stats.record_resource_read(uri)` | After successful resource read |
| Prompt get | `stats.record_prompt_get(name)` | After successful prompt retrieval |
| Error | `stats.record_error()` | On any error response |

### Stats Database

**Path:** `./data/ckb-ai-mcp-stats.redb`

**Tables:**
- `tool_calls`: Tool name → call count
- `tool_last_called`: Tool name → timestamp
- `resource_reads`: Resource URI → read count
- `resource_last_read`: Resource URI → timestamp
- `prompt_gets`: Prompt name → get count (NEW)
- `prompt_last_get`: Prompt name → timestamp (NEW)
- `metadata`: Server metadata

### Stats Endpoint

**Path:** `GET /stats`

**Query Parameters:**
- `format=human` (default): Human-readable text
- `format=json`: JSON object
- `format=prometheus`: Prometheus metrics

---

## 11. HTTP Endpoints

### MCP Endpoint (Streamable HTTP)

**Path:** `/mcp`

| Method | Purpose | Headers |
|--------|---------|---------|
| POST | Client requests/notifications | `Content-Type: application/json`, `Accept: application/json, text/event-stream`, `Mcp-Session-Id` |
| GET | Server-initiated messages (SSE) | `Accept: text/event-stream`, `Mcp-Session-Id`, `Last-Event-ID` (optional) |
| DELETE | Session termination | `Mcp-Session-Id` |

### Health Endpoint

**Path:** `GET /health`
**Response:** `200 OK` with body `"OK"`

### Stats Endpoint

**Path:** `GET /stats`
**Query:** `?format=human|json|prometheus`
**Response:** Statistics in requested format

### File Upload Endpoint

**Path:** `POST /deploy/file`
**Content-Type:** `multipart/form-data`
**Field:** `file` (binary data)

**Response:**
```json
{
  "tx_hash": "0x...",
  "output_index": 0,
  "data_size": 12345,
  "capacity": 100000000000
}
```

**Errors:**
- `400 Bad Request`: Missing file or invalid format
- `500 Internal Server Error`: Deployment failed

---

## 12. Directory Structure

```
ckb-mcp/
├── Cargo.toml                           # Workspace config (add ckb-ai-mcp)
├── TEMP_SPEC_ckb-ai-mcp-migration.md    # THIS FILE (delete after migration)
├── crates/
│   ├── shared/                          # KEEP - Update mcp.rs types
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── mcp.rs                   # Update: Add title, outputSchema, annotations
│   │       ├── error.rs                 # KEEP
│   │       ├── types.rs                 # KEEP
│   │       ├── params.rs                # KEEP
│   │       ├── ckb_client.rs            # KEEP
│   │       └── stats.rs                 # Update: Add prompt tracking
│   │
│   ├── ckb-ai-mcp/                      # NEW - Unified server
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs                  # CLI, feature flags, startup
│   │       ├── server.rs                # rmcp integration, router setup
│   │       ├── capabilities.rs          # ServerHandler implementation
│   │       │
│   │       ├── rpc/                     # RPC tools module
│   │       │   ├── mod.rs               # Module exports
│   │       │   ├── tools.rs             # Tool definitions (51 tools)
│   │       │   └── handlers.rs          # RPC call handlers
│   │       │
│   │       ├── docs/                    # Documentation resources module
│   │       │   ├── mod.rs               # Module exports
│   │       │   ├── resources.rs         # Resource definitions
│   │       │   └── loader.rs            # Doc loading, description extraction
│   │       │
│   │       ├── tools/                   # Development tools module
│   │       │   ├── mod.rs               # Module exports
│   │       │   ├── definitions.rs       # Tool definitions (8 tools)
│   │       │   └── handlers.rs          # Deploy, account handlers
│   │       │
│   │       ├── prompts/                 # Prompts module
│   │       │   ├── mod.rs               # Module exports
│   │       │   └── definitions.rs       # 4 prompt definitions
│   │       │
│   │       └── search/                  # Search module
│   │           ├── mod.rs               # Module exports
│   │           └── engine.rs            # Search algorithm
│   │
│   ├── test-common/                     # KEEP - Update for new server
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   │
│   ├── ckb-rpc-server/                  # KEEP during migration → DELETE Phase 8
│   ├── ckb-docs-server/                 # KEEP during migration → DELETE Phase 8
│   └── ckb-tools-server/                # KEEP during migration → DELETE Phase 8
│
├── docs/                                # KEEP - Documentation content
│   ├── ai-quick-reference.md
│   ├── api-reference/
│   ├── concepts/
│   ├── concepts-for-coding/
│   ├── deployment/
│   ├── ecosystem/
│   ├── education/
│   ├── examples/
│   ├── getting-started/
│   ├── integration-examples/
│   ├── patterns/
│   ├── protocols/
│   ├── tools/
│   └── troubleshooting/
│
└── resources/                           # KEEP - Example contracts
```

---

## 13. Cargo Configuration

### Workspace Cargo.toml Changes

```toml
[workspace]
resolver = "2"
members = [
    "crates/shared",
    "crates/test-common",
    "crates/ckb-ai-mcp",        # ADD
    # Keep during migration:
    "crates/ckb-rpc-server",
    "crates/ckb-docs-server",
    "crates/ckb-tools-server",
]
```

### crates/ckb-ai-mcp/Cargo.toml

```toml
[package]
name = "ckb-ai-mcp"
version = "1.0.0"
edition = "2024"
description = "Unified MCP server for CKB blockchain development"
license = "MIT"

[[bin]]
name = "ckb-ai-mcp"
path = "src/main.rs"

[dependencies]
# MCP Protocol
rmcp = { version = "0.13", features = [
    "server",
    "macros",
    "transport-streamable-http-server"
] }

# Web Framework
axum = { version = "0.8", features = ["multipart"] }
tokio = { version = "1", features = ["full"] }
tower-http = { version = "0.6", features = ["cors"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
schemars = "1.0"

# CKB
ckb-sdk = "4.4"
ckb-types = "0.202"
ckb-jsonrpc-types = "0.202"
ckb-hash = "0.202"
secp256k1 = "0.29"
hex = "0.4"

# Utilities
clap = { version = "4", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
reqwest = { version = "0.12", features = ["json"] }
anyhow = "1"
thiserror = "2"
uuid = { version = "1", features = ["v4"] }

# Shared crate
shared = { path = "../shared" }

[dev-dependencies]
tokio-test = "0.4"
```

---

## 14. Implementation Phases

### Phase 1: Server Skeleton

**Goal:** Create working server with rmcp transport

**Tasks:**
1. Create `crates/ckb-ai-mcp/` directory
2. Create `Cargo.toml` with dependencies
3. Implement `main.rs`:
   - CLI argument parsing with clap
   - Feature flag handling
   - Logging initialization
   - Stats database initialization
4. Implement `server.rs`:
   - rmcp StreamableHttpService setup
   - Axum router with /mcp, /health, /stats
   - Session management
5. Implement basic `capabilities.rs`:
   - ServerHandler trait implementation
   - `get_info()` returning capabilities
   - `initialize` handling
6. Add to workspace `Cargo.toml`
7. Verify: `cargo build -p ckb-ai-mcp`
8. Test: Server starts, health endpoint works, MCP handshake succeeds

**Deliverable:** Server that passes MCP initialize handshake

### Phase 2: RPC Tools Migration

**Goal:** Migrate 51 RPC tools with new naming

**Tasks:**
1. Create `src/rpc/mod.rs`
2. Create `src/rpc/tools.rs`:
   - Define all 51 tools with new names
   - Add title, outputSchema, annotations, category
3. Create `src/rpc/handlers.rs`:
   - Port handler logic from ckb-rpc-server
   - Use shared CkbRpcClient
4. Update `capabilities.rs`:
   - Implement `list_tools()`
   - Implement `call_tool()` routing
5. Add stats recording after tool execution
6. Test each tool category

**Deliverable:** All 51 RPC tools working with new names

### Phase 3: Resources Migration

**Goal:** Migrate 87 documentation resources with new categories

**Tasks:**
1. Create `src/docs/mod.rs`
2. Create `src/docs/loader.rs`:
   - Port DocsProvider from ckb-docs-server
   - Update resource mappings with new URIs
3. Create `src/docs/resources.rs`:
   - Define resource definitions with new categories
   - Add title, size, annotations
4. Update `capabilities.rs`:
   - Implement `list_resources()`
   - Implement `read_resource()`
5. Add stats recording after resource read
6. Test resource listing and reading

**Deliverable:** All 87 resources working with new URIs/categories

### Phase 4: Development Tools Migration

**Goal:** Migrate 8 dev tools and file upload endpoint

**Tasks:**
1. Create `src/tools/mod.rs`
2. Create `src/tools/definitions.rs`:
   - Define 8 tools with new names
   - Add title, outputSchema, annotations, category
3. Create `src/tools/handlers.rs`:
   - Port ToolsProvider from ckb-tools-server
   - Deployment, account, chain info handlers
4. Add `/deploy/file` endpoint to router
5. Update `capabilities.rs`:
   - Add dev tools to `list_tools()`
   - Add dev tools to `call_tool()` routing
6. Add stats recording
7. Test deployment (requires CKB node)

**Deliverable:** All 8 dev tools + file upload working

### Phase 5: Search Implementation

**Goal:** Add tool and resource search

**Tasks:**
1. Create `src/search/mod.rs`
2. Create `src/search/engine.rs`:
   - Implement search algorithm
   - Keyword matching with scoring
   - Category filtering
3. Add `find_tools` tool definition and handler
4. Add `find_resources` tool definition and handler
5. Update `list_tools()` to include search tools
6. Test search with various queries

**Deliverable:** Working search for tools and resources

### Phase 6: Prompts Implementation

**Goal:** Add 4 workflow prompts

**Tasks:**
1. Create `src/prompts/mod.rs`
2. Create `src/prompts/definitions.rs`:
   - Define 4 prompts
   - Implement message generation with embedded resources
3. Update `capabilities.rs`:
   - Implement `list_prompts()`
   - Implement `get_prompt()`
4. Add stats recording for prompt usage
5. Update stats.rs for prompt tracking
6. Test each prompt

**Deliverable:** All 4 prompts working

### Phase 7: Testing

**Goal:** Comprehensive test coverage

**Tasks:**
1. Update test-common for new server
2. Create integration tests:
   - Server startup and health
   - MCP handshake
   - Tool listing and calling (sample from each category)
   - Resource listing and reading
   - Prompt listing and getting
   - Search functionality
   - Stats recording and retrieval
   - Feature flags
3. Manual testing with Claude Code
4. Performance testing (response times)
5. Fix any issues discovered

**Deliverable:** All tests passing, server verified working

### Phase 8: Cleanup

**Goal:** Remove old servers, update documentation

**Tasks:**
1. Remove from workspace Cargo.toml:
   - `crates/ckb-rpc-server`
   - `crates/ckb-docs-server`
   - `crates/ckb-tools-server`
2. Delete directories:
   - `rm -rf crates/ckb-rpc-server`
   - `rm -rf crates/ckb-docs-server`
   - `rm -rf crates/ckb-tools-server`
3. Update CLAUDE.md:
   - New server instructions
   - New port (3112)
   - New CLI arguments
   - Updated testing instructions
4. Update .claude/CLAUDE.md:
   - New server commands for devnet/testnet
5. Delete this spec file
6. Final build and test
7. Git commit

**Deliverable:** Clean codebase with single unified server

---

## 15. Testing Requirements

### Unit Tests

| Module | Tests |
|--------|-------|
| `search/engine.rs` | Keyword matching, scoring, category filter |
| `docs/loader.rs` | Description extraction, resource loading |
| `prompts/definitions.rs` | Message generation |

### Integration Tests

| Test | Description |
|------|-------------|
| `test_server_health` | Health endpoint returns OK |
| `test_mcp_initialize` | Protocol handshake succeeds |
| `test_tools_list` | Lists all tools with correct structure |
| `test_tool_call_rpc` | Sample RPC tool works |
| `test_tool_call_dev` | Sample dev tool works |
| `test_resources_list` | Lists all resources |
| `test_resource_read` | Reads resource content |
| `test_prompts_list` | Lists all prompts |
| `test_prompt_get` | Returns prompt messages |
| `test_search_tools` | Search returns relevant results |
| `test_search_resources` | Search returns relevant results |
| `test_stats_recording` | Stats increment on usage |
| `test_feature_docs_only` | Only docs features enabled |
| `test_feature_rpc_only` | Only RPC features enabled |

### Manual Testing

1. Start server: `cargo run -p ckb-ai-mcp -- --port 3112 --ckb-rpc http://192.168.0.73:28114`
2. Verify health: `curl http://localhost:3112/health`
3. Test in Claude Code
4. Verify all tools accessible
5. Verify all resources readable
6. Verify prompts work
7. Verify search works
8. Check stats endpoint

---

## 16. Cleanup Tasks

### Files to Delete After Migration

- [ ] `crates/ckb-rpc-server/` (entire directory)
- [ ] `crates/ckb-docs-server/` (entire directory)
- [ ] `crates/ckb-tools-server/` (entire directory)
- [ ] `TEMP_SPEC_ckb-ai-mcp-migration.md` (this file)

### Files to Update

- [ ] `Cargo.toml` (workspace members)
- [ ] `CLAUDE.md` (project instructions)
- [ ] `.claude/CLAUDE.md` (local config)
- [ ] `README.md` (if exists)

### Git Commits

Create logical commits for each phase:
1. "Add ckb-ai-mcp server skeleton with rmcp transport."
2. "Migrate RPC tools to unified server."
3. "Migrate documentation resources to unified server."
4. "Migrate development tools to unified server."
5. "Add tool and resource search functionality."
6. "Add workflow prompts."
7. "Add integration tests for unified server."
8. "Remove legacy servers and update documentation."

---

## Appendix A: Old to New Tool Name Mapping (Complete)

```
# RPC Tools (51)
get_block                    → rpc_get_block
get_block_by_number          → rpc_get_block_by_number
get_header                   → rpc_get_header
get_header_by_number         → rpc_get_header_by_number
get_transaction              → rpc_get_transaction
get_block_hash               → rpc_get_block_hash
get_tip_header               → rpc_get_tip_header
get_tip_block_number         → rpc_get_tip_block_number
get_current_epoch            → rpc_get_current_epoch
get_epoch_by_number          → rpc_get_epoch_by_number
get_live_cell                → rpc_get_live_cell
get_fork_block               → rpc_get_fork_block
get_indexer_tip              → rpc_get_indexer_tip
get_cells                    → rpc_search_cells
get_transactions             → rpc_search_transactions
get_cells_capacity           → rpc_get_cells_capacity
send_transaction             → rpc_submit_transaction
test_tx_pool_accept          → rpc_test_transaction
local_node_info              → rpc_get_node_info
sync_state                   → rpc_get_sync_state
get_peers                    → rpc_get_peers
tx_pool_info                 → rpc_get_pool_info
tx_pool_ready                → rpc_get_pool_ready
get_raw_tx_pool              → rpc_get_pool_transactions
get_pool_tx_detail_info      → rpc_get_pool_tx_detail
get_blockchain_info          → rpc_get_blockchain_info
get_consensus                → rpc_get_consensus
get_deployments_info         → rpc_get_deployments
estimate_cycles              → rpc_estimate_cycles
estimate_fee_rate            → rpc_estimate_fee_rate
calculate_dao_maximum_withdraw → rpc_calculate_dao_withdraw
get_block_economic_state     → rpc_get_block_economics
get_block_median_time        → rpc_get_block_median_time
get_block_filter             → rpc_get_block_filter
get_transaction_proof        → rpc_get_transaction_proof
verify_transaction_proof     → rpc_verify_transaction_proof

# Development Tools (8)
DeployCellData               → dev_deploy_cell
GetAddressBalance            → dev_get_balance
GetDefaultAccountInfo        → dev_get_account_info
GenerateLockInfo             → dev_generate_lock_info
GetLockInfoFromAddress       → dev_get_lock_from_address
GetChainType                 → dev_get_chain_type
GetGenesisHash               → dev_get_genesis_hash
RequestTestnetFunds          → dev_request_faucet

# New Search Tools (2)
(new)                        → find_tools
(new)                        → find_resources
```

---

## Appendix B: Old to New Resource URI Mapping

```
# Pattern: ckb-dev-context://{old-path} → ckb-dev-context://{new-category}/{new-name}

# start (from getting-started/)
getting-started/developer-resources-and-tooling → start/developer-resources
getting-started/offckb-development-workflow     → start/offckb-workflow
getting-started/tool-recommendations            → start/tool-recommendations

# concepts (from concepts/, concepts-for-coding/)
concepts/cell-model                    → concepts/cell-model
concepts/advanced-cell-concepts        → concepts/advanced-cells
concepts/transaction-structure         → concepts/transaction-structure
concepts/molecule-serialization        → concepts/molecule-serialization
concepts/script-groups-and-execution   → concepts/script-groups
concepts/ckb-syscalls-and-sources      → concepts/syscalls
concepts/ckb-network-history           → concepts/network-history
concepts/header-dependencies-and-time-access → concepts/header-deps
concepts/lock-value-relationships      → concepts/lock-value-relationships
concepts-for-coding/cell-lifecycle     → concepts/cell-lifecycle
concepts-for-coding/transaction-lifecycle → concepts/transaction-lifecycle

# (remaining mappings follow same pattern...)
```

---

## Appendix C: Environment Variables

| Variable | Purpose | Default |
|----------|---------|---------|
| `RUST_LOG` | Log level filter | `info` |
| `CKB_RPC_URL` | CKB node URL (for tests) | - |

---

*End of specification. Delete this file after migration is complete.*
