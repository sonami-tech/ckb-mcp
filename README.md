# CKB MCP Rust Workspace

A collection of Model Context Protocol (MCP) servers for Nervos CKB development, built in Rust.

## Overview

This workspace provides multiple specialized MCP servers to help AI assistants build Nervos CKB smart contracts and applications:

- **ckb-rpc-server**: Query CKB blockchain data via RPC.
- **ckb-docs-server**: Access CKB development documentation and resources.
- **ckb-tools-server**: Generate, compile, test, and deploy CKB contracts.

## Architecture

Each server runs as an independent HTTP server with JSON-RPC transport, compatible with Claude Code and other MCP clients.

```
ckb-mcp/
├── crates/
│   ├── shared/              # Common types and utilities
│   ├── ckb-rpc-server/      # Blockchain query tools (port 8001)
│   ├── ckb-docs-server/     # Documentation resources (port 8002)
│   └── ckb-tools-server/    # Development tools (port 8003)
├── docs/                    # CKB development documentation
├── resources/               # External resource references
└── Cargo.toml              # Workspace configuration
```

## Quick Start

### Prerequisites

- Rust 1.70+.
- CKB node running (for RPC server).
- CKB development tools (for tools server).

### Build and Run

```bash
# Build all servers
cargo build --release

# Development: Auto-rebuild and run on changes
cargo watch -x "build --workspace" -i "crates/*/Cargo.toml" -s 'parallel --line-buffer ::: "target/debug/ckb-docs-server" "target/debug/ckb-rpc-server --ckb-rpc http://192.168.0.73:18114" "target/debug/ckb-tools-server"'

# Simple run (starts on ports 8001, 8002, 8003)
cargo run --bin ckb-rpc-server & \
cargo run --bin ckb-docs-server & \
cargo run --bin ckb-tools-server & \
wait
```

### Run Individual Servers

```bash
# CKB RPC Server (port 8001)
cargo run --bin ckb-rpc-server

# CKB Docs Server (port 8002)  
cargo run --bin ckb-docs-server

# CKB Tools Server (port 8003)
cargo run --bin ckb-tools-server
```

## MCP Integration

### Claude Code (Project-Scoped)

Add the MCP servers to your current project using the Claude Code CLI:

```bash
# Add CKB RPC server for blockchain queries
claude mcp add --scope project --transport http ckb-rpc http://localhost:8001/mcp

# Add CKB Docs server for development documentation  
claude mcp add --scope project --transport http ckb-docs http://localhost:8002/mcp

# Add CKB Tools server for contract development
claude mcp add --scope project --transport http ckb-tools http://localhost:8003/mcp
```

### Verify MCP Configuration

```bash
# List configured MCP servers
claude mcp list

# Test server connectivity
curl http://localhost:8001/health
curl http://localhost:8002/health  
curl http://localhost:8003/health

# Remove servers if needed
claude mcp remove --scope project ckb-rpc
claude mcp remove --scope project ckb-docs
claude mcp remove --scope project ckb-tools
```

### Using the MCP Servers

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

**Generate and manage contracts:**
```
"Create a new lock script template"
"Compile my CKB contract and run tests"
"Generate a contract template for NFT development"
```

The servers provide context-aware assistance for CKB development workflows.

## Server Details

### CKB RPC Server

**Purpose**: Query CKB blockchain data

**Chain Methods**:
- `get_block` - Get block by hash.
- `get_block_by_number` - Get block by number.
- `get_header` - Get block header by hash.
- `get_header_by_number` - Get block header by number.
- `get_transaction` - Get transaction by hash.
- `get_block_hash` - Get block hash by number.
- `get_tip_header` - Get tip block header.
- `get_live_cell` - Get live cell by outpoint.
- `get_tip_block_number` - Get tip block number.
- `get_current_epoch` - Get current epoch information.
- `get_epoch_by_number` - Get epoch by number.

**Indexer Methods**:
- `get_indexer_tip` - Get indexer tip.
- `get_cells` - Search for cells by criteria.
- `get_transactions` - Search for transactions by criteria.
- `get_cells_capacity` - Get total capacity of cells by search criteria.

**Network Methods**:
- `local_node_info` - Get local node information.

**Usage**:
```bash
ckb-rpc-server [OPTIONS]
  -p, --port <PORT>           Port [default: 8001]
      --ckb-rpc <URL>         CKB RPC URL [default: http://127.0.0.1:8114]
```

### CKB Docs Server

**Purpose**: Provide development resources and documentation

**Resources** (served via `ckb-dev-context://` URI scheme):

*Core Concepts*:
- `concepts/cell-model` - CKB Cell Model fundamentals.
- `concepts/transaction-structure` - Transaction anatomy and validation.
- `concepts/molecule-serialization` - Molecule type system and encoding.
- `concepts/advanced-cell-concepts` - Advanced cell patterns.

*Development Patterns*:
- `patterns/minimal-lock-script` - Basic lock script development.
- `patterns/minimal-type-script` - Basic type script development.
- `patterns/simple-transfer` - Basic CKB transfers.
- `patterns/token-creation` - Custom token creation.
- `patterns/udt-tokens` - User Defined Tokens (sUDT/xUDT).
- `patterns/omnilock-development` - Cross-chain wallet integration.
- `patterns/molecule-schema-development` - Schema design patterns.
- `patterns/cota-nft-development` - NFT development with CoTA.
- `patterns/spore-development` - Spore SDK integration patterns.

*API References*:
- `api-reference/syscalls-quick-ref` - CKB syscalls reference.
- `api-reference/ccc-api-patterns` - CCC SDK usage patterns.
- `api-reference/molecule-api-examples` - Molecule API examples.
- `api-reference/omnilock-api-examples` - Omnilock integration examples.
- `api-reference/spore-sdk-examples` - Spore SDK API reference and examples.

*Protocols*:
- `protocols/cota-protocol` - CoTA NFT protocol specification.
- `protocols/omnilock-protocol` - Universal lock script protocol.
- `protocols/spore-protocol` - Spore protocol for digital objects.
- `protocols/spore-digital-objects` - Legacy Spore documentation.
- `protocols/rgb-plus-plus` - RGB++ asset protocol.

*Troubleshooting*:
- `troubleshooting/common-script-errors` - Script debugging guide.

**Usage**:
```bash
ckb-docs-server [OPTIONS]
  -p, --port <PORT>           Port [default: 8002]
      --docs-path <PATH>      Custom docs directory
```

### CKB Tools Server

**Purpose**: Development and build tools

**Tools**:
- `generate_contract` - Create contract boilerplate.
- `compile_contract` - Compile Rust contracts.
- `run_tests` - Execute contract tests.
- `deploy_contract` - Deploy to network.
- `format_code` - Code formatting.
- `create_project` - Create new contract project.

**Usage**:
```bash
ckb-tools-server [OPTIONS]
  -p, --port <PORT>           Port [default: 8003]
      --ckb-rpc <URL>         CKB RPC URL [default: http://127.0.0.1:8114]
      --workspace <PATH>      Target workspace directory
```

## Development

### Project Structure

- **shared/**: Common types, error handling, and MCP utilities.
- **ckb-rpc-server/**: RPC client and blockchain query handlers.
- **ckb-docs-server/**: Documentation provider and resource handlers.
- **ckb-tools-server/**: Development tools and code generation.

### Adding New Tools

1. Define tool schema in `handlers.rs`.
2. Implement tool logic in the appropriate provider module.
3. Add tool to the tools list in `handle_tools_list()`.
4. Add tool call handler in `handle_tools_call()`.

### Testing

```bash
# Run all tests
cargo test

# Test specific server
cargo test -p ckb-rpc-server

# Integration test with actual CKB node
INTEGRATION_TEST=1 cargo test
```

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
- `CKB_RPC_URL`: Default CKB node RPC endpoint.
- `INTEGRATION_TEST`: Enable integration tests.

### Documentation Structure

The documentation system includes:

- **Core Documentation** (`docs/`): Comprehensive CKB development guides.
- **External Resources** (`resources/`): Reference implementations and examples from CKB ecosystem projects.
- **MCP Integration**: All docs served via ckb-docs-server with URI scheme `ckb-dev-context://`.

To add new documentation:
1. Create markdown file in appropriate `docs/` subdirectory.
2. **REQUIRED**: Add a `## Description` section after the main title (see CLAUDE.md for format requirements).
3. Add URI mapping to `crates/ckb-docs-server/src/docs.rs`.
4. Verify format: `python3 utils/verify_descriptions.py`
5. Restart server to load new resources.

## Deployment

For production deployment using Docker containers, see **[DEPLOY.md](DEPLOY.md)** for comprehensive deployment documentation including:

- Docker container setup
- Staging and production environments
- Automatic updates with Watchtower
- Health monitoring and troubleshooting

## License

MIT License - see LICENSE file for details.