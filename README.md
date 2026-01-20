# CKB MCP

A unified Model Context Protocol (MCP) server for Nervos CKB development.

## Current Development Status

| Server | Status | Description |
|--------|--------|-------------|
| **ckb-ai-mcp** | **Alpha** | Unified MCP server with 36 RPC tools, 8 dev tools, 86 docs resources, and 4 workflow prompts. |

⚠️ **Note**: This server is under active development. Expect breaking changes and incomplete functionality.

## Overview

This workspace provides a unified MCP server to help AI assistants build Nervos CKB smart contracts and applications:

- **RPC Tools**: Query CKB blockchain data (blocks, transactions, cells, epochs).
- **Dev Tools**: Deploy cells, manage addresses, request faucet funds.
- **Documentation**: Access 86 CKB development resources and guides.
- **Workflow Prompts**: Guided workflows for script creation, deployment, and transfers.

## Architecture

The server runs as an HTTP server with Streamable HTTP transport (MCP protocol 2025-03-26), compatible with Claude Code and other MCP clients.

```
ckb-mcp/
├── crates/
│   ├── shared/              # Common types and utilities
│   ├── test-common/         # Test utilities
│   └── ckb-ai-mcp/          # Unified MCP server (port 3112)
├── docs/                    # CKB development documentation
├── resources/               # External resource references
└── Cargo.toml              # Workspace configuration
```

## Quick Start

### Prerequisites

- Rust 1.75+ (stable).
- CKB node access (local or remote).
- Docker (optional, for containerized deployment).

### Build and Run

```bash
# Build the server.
cargo build --release

# Run the unified server (default port 3112).
./target/release/ckb-ai-mcp --ckb-rpc http://127.0.0.1:8114

# Or run in docs-only mode (no CKB node required).
./target/release/ckb-ai-mcp --docs-only

# Development: Auto-rebuild on changes.
cargo watch -x "build -p ckb-ai-mcp"
```

### Server Options

```bash
ckb-ai-mcp [OPTIONS]
  -p, --port <PORT>            Port [default: 3112]
      --host <HOST>            Host [default: 0.0.0.0]
      --ckb-rpc <CKB_RPC>      CKB node RPC URL [default: http://127.0.0.1:8114]
      --private-key <KEY>      Private key for signing transactions
      --docs-path <PATH>       Custom docs directory
      --stats-db <PATH>        Path to stats database
      --docs-only              Run in docs-only mode (no CKB node required)
      --rpc-only               Run with RPC tools only
      --tools-only             Run with dev tools only
      --no-prompts             Disable workflow prompts
      --log-level <LEVEL>      Log level [default: info]
```

## MCP Integration

### Claude Code (Project-Scoped)

Add the MCP server to your current project using the Claude Code CLI:

```bash
# Add the unified CKB MCP server
claude mcp add --scope project --transport http ckb-ai http://localhost:3112/mcp
```

### Verify MCP Configuration

```bash
# List configured MCP servers
claude mcp list

# Test server connectivity
curl http://localhost:3112/health

# Remove server if needed
claude mcp remove --scope project ckb-ai
```

### Using the MCP Server

Once configured, Claude Code can access CKB development resources:

**Query blockchain data:**
```
"What's the current tip block number on CKB?"
"Show me the transaction details for hash 0x123..."
```

**Access documentation:**
```
"How do I create a lock script in CKB?"
"Show me examples of UDT token creation"
"What are the CKB syscalls available?"
```

**Deploy and manage cells:**
```
"Deploy this contract to the blockchain"
"What's my account balance?"
"Request testnet funds"
```

The server provides context-aware assistance for CKB development workflows.

## Server Features

### RPC Tools (36 tools)

Query CKB blockchain data:

- **Chain Methods**: `get_block`, `get_block_by_number`, `get_header`, `get_transaction`, `get_tip_header`, `get_tip_block_number`, `get_current_epoch`, etc.
- **Indexer Methods**: `get_indexer_tip`, `get_cells`, `get_transactions`, `get_cells_capacity`.
- **Network Methods**: `local_node_info`.

### Dev Tools (8 tools)

Deploy and manage cells:

- `DeployCellData` - Deploy a cell with hex-encoded data.
- `GetAddressBalance` - Get CKB balance for an address.
- `GetChainType` - Get chain type (mainnet/testnet/devnet).
- `GetGenesisHash` - Get genesis block hash.
- `GenerateLockInfo` - Generate lock script info from private key.
- `GetLockInfoFromAddress` - Extract lock info from CKB address.
- `RequestTestnetFunds` - Request testnet funds from faucet.
- `GetDefaultAccountInfo` - Get configured account details and balance.

### Documentation Resources (86 resources)

Served via `ckb://docs/` URI scheme:

*Core Concepts*:
- Cell model fundamentals and advanced patterns.
- Transaction structure and lifecycle.
- Molecule serialization and type system.
- CKB syscalls and sources.
- Script groups and execution.

*Development Patterns*:
- Lock and type script development (minimal templates).
- Token creation (UDT, sUDT, xUDT patterns).
- Omnilock cross-chain integration.
- CoTA and Spore NFT development.
- DAO staking and iCKB liquidity.
- File storage with CKBFS.
- Rust and C contract development.

*API References*:
- CKB syscalls quick reference.
- CCC SDK patterns (including cross-chain and SSRI).
- Molecule, Omnilock, Spore, CoTA, iCKB SDK examples.
- Well-known hashes and constants.

*Protocols*:
- CoTA, Omnilock, Spore, iCKB, RGB++, CKBFS protocols.
- CoBuild, Open Transaction, SSRI, xUDT specifications.

*Troubleshooting*:
- Common script errors and debugging.
- Framework-specific error guides (Omnilock, xUDT, Spore, iCKB).
- Transaction building errors.

### Workflow Prompts (4 prompts)

Guided workflows for common tasks:

- Script creation workflow.
- Deployment workflow.
- Query workflow.
- Transfer workflow.

### File Upload Endpoint

For deploying large binaries that exceed MCP context limits:

```bash
# Upload and deploy a contract binary
curl -X POST http://localhost:3112/deploy/file \
  -F "file=@my-contract.wasm"
```

## Development

### Project Structure

- **shared/**: Common types, error handling, and MCP utilities.
- **test-common/**: Shared test utilities and fixtures.
- **ckb-ai-mcp/**: Unified MCP server implementation.

### Adding New Tools

1. Define tool schema in the appropriate module.
2. Implement tool logic in the provider.
3. Register the tool in the tools list.
4. Add handler for tool calls.

### Testing

**REQUIRED**:
- Install `cargo-nextest`: `cargo install cargo-nextest`
- Set `CKB_RPC_URL` environment variable to your CKB node URL

```bash
# Install cargo-nextest (one-time setup)
cargo install cargo-nextest

# Set the CKB RPC URL (required for running tests)
export CKB_RPC_URL=http://127.0.0.1:8114  # For local mainnet node
# or use a remote devnet/testnet node
export CKB_RPC_URL=http://your-node-ip:8114     # Mainnet (port 8114)
export CKB_RPC_URL=http://your-node-ip:18114    # Testnet (port 18114)
export CKB_RPC_URL=http://your-node-ip:28114    # Devnet (port 28114)

# Run all tests
cargo nextest run

# Specify URL inline for a single test run
CKB_RPC_URL=http://your-node-ip:18114 cargo nextest run

# Test specific package
CKB_RPC_URL=http://your-node-ip:18114 cargo nextest run -p ckb-ai-mcp

# Run tests with logging
CKB_RPC_URL=http://your-node-ip:18114 RUST_LOG=debug cargo nextest run
```

**Note**: This project uses `cargo-nextest` instead of `cargo test` for guaranteed sequential test execution.

### Utilities

The `utils/` directory contains maintenance scripts:

```bash
# Verify documentation descriptions are properly formatted
python3 utils/verify_descriptions.py

# Show detailed verification output
python3 utils/verify_descriptions.py --verbose
```

See `utils/README.md` for complete utility documentation.

## Configuration

### Environment Variables

- `RUST_LOG`: Logging level (debug, info, warn, error).
- `CKB_RPC_URL`: **Required for tests**. CKB node RPC endpoint. Examples:
  - `http://127.0.0.1:8114` - Local mainnet node
  - `http://your-node-ip:18114` - Remote testnet node
  - `http://your-node-ip:28114` - Remote devnet node

### Documentation Structure

The documentation system includes:

- **Core Documentation** (`docs/`): Comprehensive CKB development guides.
- **External Resources** (`resources/`): Reference implementations and examples from CKB ecosystem projects.
- **MCP Integration**: All docs served via ckb-ai-mcp with URI scheme `ckb://docs/`.

To add new documentation:
1. Create markdown file in appropriate `docs/` subdirectory.
2. **REQUIRED**: Add a `## Description` section after the main title (see CLAUDE.md for format requirements).
3. Add URI mapping to `crates/ckb-ai-mcp/src/docs.rs`.
4. Verify format: `python3 utils/verify_descriptions.py`
5. Restart server to load new resources.

## Deployment

For production deployment using Docker containers, see **[DEPLOY.md](DEPLOY.md)** for comprehensive deployment documentation including:

- Docker container setup
- Staging and production environments
- Automatic updates with Watchtower
- Health monitoring and troubleshooting

## License

MIT
