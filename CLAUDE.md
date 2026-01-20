# CKB MCP Development Context

## Project Overview

This is a Rust workspace providing Model Context Protocol (MCP) servers for Nervos CKB blockchain development. The workspace contains a unified MCP server and legacy individual servers.

## Architecture

```
ckb-mcp/
├── crates/
│   ├── shared/              # Common types, errors, and utilities
│   ├── ckb-ai-mcp/          # Unified MCP server (port 3112) - RECOMMENDED
│   ├── ckb-rpc-server/      # Legacy: Blockchain query tools (port 8001)
│   ├── ckb-docs-server/     # Legacy: Documentation resources (port 8002)
│   └── ckb-tools-server/    # Legacy: Development tools (port 8003)
├── docs/                    # CKB development documentation
└── Cargo.toml              # Workspace configuration
```

### ckb-ai-mcp (Unified Server)

The recommended server that combines all functionality in a single process using MCP protocol 2025-03-26 with Streamable HTTP transport:

- **36 RPC Tools** (`rpc_*`): Query blockchain data, transactions, cells, headers, blocks
- **8 Dev Tools** (`dev_*`): Deploy cells, manage addresses, request faucet funds
- **2 Search Tools**: Search available tools and documentation resources
- **86 Documentation Resources**: CKB concepts, patterns, API references
- **4 Workflow Prompts**: Guided workflows for script creation, deployment, queries, transfers
- **File Upload Endpoint**: POST /deploy/file for large binary deployments

### Legacy Servers

The individual servers remain available for specific use cases:

- **ckb-rpc-server**: Query live CKB blockchain data, transaction details, cell information.
- **ckb-docs-server**: Serve CKB development documentation, patterns, and API references.
- **ckb-tools-server**: Deploy cells, manage addresses and balances, generate lock info, request testnet funds.

## Development Guidelines

### Code Style

- Follow standard Rust conventions and use `cargo fmt`.
- Use structured error handling with `thiserror` and `anyhow`.
- Implement comprehensive logging with `tracing`.
- Write unit tests for all core functionality.
- Document public APIs with rustdoc comments.
- **CRITICAL**: Treat compiler warnings as errors. Always fix all warnings before committing code. Warnings indicate potential issues that must be resolved.

### Testing

**REQUIRED**: Tests need the `CKB_RPC_URL` environment variable set to the CKB node URL. This should match the URL used when starting the servers.

**REQUIRED**: This project uses `cargo-nextest` instead of `cargo test` for guaranteed sequential test execution.

```bash
# Install cargo-nextest (one-time setup)
cargo install cargo-nextest

# Set the CKB RPC URL (required for tests)
export CKB_RPC_URL=http://127.0.0.1:8114  # For local mainnet node
# or use a remote devnet/testnet node
export CKB_RPC_URL=http://your-node-ip:8114     # Mainnet (port 8114)
export CKB_RPC_URL=http://your-node-ip:18114    # Testnet (port 18114)
export CKB_RPC_URL=http://your-node-ip:28114    # Devnet (port 28114)

# Run all tests (uses CKB_RPC_URL from environment)
cargo nextest run

# Or specify URL inline for a single test run
CKB_RPC_URL=http://your-node-ip:18114 cargo nextest run

# Run tests for specific server with custom node
CKB_RPC_URL=http://your-node-ip:18114 cargo nextest run -p ckb-rpc-server
CKB_RPC_URL=http://your-node-ip:18114 cargo nextest run -p ckb-docs-server
CKB_RPC_URL=http://your-node-ip:18114 cargo nextest run -p ckb-tools-server

# Run tests with logging
CKB_RPC_URL=http://your-node-ip:18114 RUST_LOG=debug cargo nextest run
```

**CRITICAL: Test Independence and Isolation**

When testing MCP servers, tests must maintain strict independence from the system under test:

- **Setup**: Use direct CKB RPC client calls (NOT MCP server) to prepare test conditions and gather initial state.
- **Execute**: Call the MCP server endpoint being tested.
- **Verify**: Use direct CKB RPC client calls (NOT MCP server) to confirm results and validate outcomes.

**Why this matters:**
- Prevents circular dependencies (MCP verifying MCP).
- Avoids cascading failures (if MCP query breaks, all tests fail).
- Ensures accuracy (MCP bug could affect both creation and verification).

**Example:**
- ✅ **Correct**: Use direct `reqwest` calls to CKB RPC to verify transaction commitment after MCP creates it.
- ❌ **Incorrect**: Use MCP's own transaction query endpoint to verify MCP's transaction creation.

The MCP server is the **subject under test**, not a **test fixture**. All test setup and verification must use independent CKB RPC calls.

**Test Execution Phases**

Tests are organized into four sequential phases, enforced by alphabetical naming and nextest's `test-threads = 1` configuration:

1. **Phase 1 (test_00_server_running)**: Verify MCP server HTTP health endpoint is accessible.
   - On failure: Nextest stops, all tests abort.

2. **Phase 2 (test_01_ckb_rpc_available)**: Verify direct CKB RPC connectivity (independent of MCP).
   - Method: Direct `reqwest` call to CKB RPC (e.g., `get_tip_block_number`).
   - On failure: Nextest stops, all tests abort.

3. **Phase 3 (test_02_collect_shared_data)**: Collect shared blockchain data via direct CKB RPC.
   - Gathers: Chain type, genesis block hash, genesis block data.
   - Storage: `SharedTestData` static structure in `shared/tests/common/mod.rs`.
   - On failure: Nextest stops, all tests abort.

4. **Phase 4 (test_03_* and beyond)**: Individual feature tests.
   - Setup: Read from `SharedTestData::get_or_panic()` instead of querying.
   - Verification: Use direct CKB RPC calls (NOT MCP).
   - On failure: Test fails, nextest continues with remaining tests.

**Test Ordering**: Nextest runs tests alphabetically with `test-threads = 1`, ensuring phases 1-3 complete before individual tests run.

**Shared Test Data**: Tests access pre-collected blockchain data via:
```rust
let shared_data = SharedTestData::get_or_panic();
let genesis_hash = &shared_data.genesis_hash;
let chain_type = &shared_data.chain_type;
let genesis_block = &shared_data.genesis_block;
```

**IMPORTANT: Test Timing Expectations**
- **ckb-rpc-server** tests: Fast (< 5 seconds total).
- **ckb-docs-server** tests: Fast (< 5 seconds total).
- **ckb-tools-server** tests: Slow - deployment tests wait up to 60 seconds per test for blockchain transaction confirmation. The full test suite can take several minutes. This is expected behavior as these tests deploy actual cells to the blockchain and verify confirmation.

### Building and Running

**⚠️ CRITICAL: Server Management - MUST ASK USER FIRST ⚠️**

**BEFORE running any tests, building servers, or starting/stopping servers, you MUST ask the user:**

"Are the MCP servers (ports 8001-8003) auto-managed in another window, or should I manage them manually?"

Wait for the user's response before proceeding. Do not assume or guess.

Once confirmed, follow the appropriate workflow:

1. **Auto-managed in another window**: Servers auto-restart when code changes are detected. You should NOT manually start/stop them. However, if a request fails or times out, automatically retry at least once as the server may still be compiling.
2. **Manually managed by you**: After rebuilding, you must start/stop servers as needed.

**Auto-managed workflow:**
```bash
# Only build to validate compilation
cargo build --workspace --release

# Servers restart automatically - DO NOT manually start/stop
# If requests fail/timeout, retry automatically (server may be compiling)
```

**Manual management workflow:**
```bash
# Build and restart servers after code changes
cargo build --workspace --release

# RECOMMENDED: Start unified server
./target/release/ckb-ai-mcp --host 0.0.0.0 --port 3112 --ckb-rpc <node-url>

# Or start unified server in docs-only mode (no CKB node required)
./target/release/ckb-ai-mcp --docs-only --port 3112

# Legacy servers (for specific use cases):
./target/release/ckb-rpc-server --host 0.0.0.0 --port 8001 --ckb-rpc <node-url>
./target/release/ckb-docs-server --host 0.0.0.0 --port 8002
./target/release/ckb-tools-server --host 0.0.0.0 --port 8003 --ckb-rpc <node-url>
```

### CLI Parameters

If you need to run servers manually for debugging, use `--help` to see available parameters:

```bash
# View parameters for each server
cargo run --bin ckb-ai-mcp -- --help       # Unified server
cargo run --bin ckb-rpc-server -- --help   # Legacy
cargo run --bin ckb-docs-server -- --help  # Legacy
cargo run --bin ckb-tools-server -- --help # Legacy
```

**ckb-ai-mcp parameters (unified server):**
- `--host`, `--port` (default: 3112), `--log-level`
- `--ckb-rpc` (CKB node URL, default: http://127.0.0.1:8114)
- `--private-key` (transaction signing key)
- `--docs-path` (custom docs directory)
- `--stats-db` (path to stats database)
- Feature flags: `--docs-only`, `--rpc-only`, `--tools-only`, `--no-prompts`

**Legacy server parameters:**
- All servers: `--host`, `--port`, `--log-level`
- **ckb-rpc-server**: `--ckb-rpc` (CKB node URL)
- **ckb-docs-server**: `--docs-path` (custom docs directory)
- **ckb-tools-server**: `--ckb-rpc`, `--private-key` (transaction signing key)

### Debugging

- Unified server runs on port 3112 (default).
- Legacy servers run on ports 8001, 8002, 8003 respectively.
- Use `RUST_LOG=debug` for detailed logging.
- Unified server uses Streamable HTTP transport (MCP 2025-03-26).
- Legacy servers use HTTP transport only (MCP 2024-11-05).
- Test endpoints manually with curl or use MCP client tools.

## Dependencies

### Core Dependencies

- **tokio**: Async runtime.
- **axum**: HTTP server framework.
- **serde/serde_json**: Serialization.
- **tracing**: Structured logging.
- **clap**: CLI argument parsing.
- **reqwest**: HTTP client for CKB RPC calls.

### CKB-Specific

- CKB node connection for RPC queries.
- Modern CKB development tooling and SDKs.
- Molecule serialization for CKB data types.

## Documentation Integration

The `docs/` directory contains comprehensive CKB development documentation:

- **concepts/**: Core CKB concepts.
  - Cell Model fundamentals and advanced patterns.
  - Transaction structure and lifecycle.
  - Molecule serialization type system.
- **patterns/**: Development patterns and best practices.
  - Lock and type script development.
  - Token creation (UDT/sUDT/xUDT patterns).
  - Omnilock cross-chain integration.
  - Molecule schema development patterns.
  - CoTA NFT development patterns.
  - Cell lifecycle and operation detection.
- **api-reference/**: API examples and quick references.
  - CKB syscalls reference.
  - CCC SDK integration patterns.
  - Molecule API examples and usage.
  - Omnilock API reference and examples.
  - CoTA SDK examples.
- **protocols/**: Protocol specifications.
  - CoTA NFT protocol.
  - Omnilock universal lock protocol.
  - Spore digital objects protocol.
  - RGB++ asset protocol.
- **troubleshooting/**: Common errors and debugging guides.

Documentation is served via the ckb-docs-server with URI scheme `ckb-dev-context://`

### Documentation Format Requirements

**IMPORTANT**: All markdown documentation files MUST include a `## Description` section immediately after the main title. This section should:

- Be placed right after the `# Title` heading
- Contain a description under 1,024 characters
- Serve dual purpose: document introduction AND MCP resource description
- Use action-oriented language highlighting practical value
- Summarize key topics, code examples, and use cases
- **AVOID verbose phrases**: No "comprehensive", "provides", "this guide covers", "essential for", "learn", "discover", "master"
- **Use direct, concise language**: Start with topic directly, avoid transition phrases
- **Redundancy is acceptable**: Description can repeat content details since it describes what's available through MCP

**AI-Optimized Documentation Guidelines**:
- **Target audience**: AI assistants exclusively, not human readers
- **Concise style**: Remove verbose transitions, explanatory padding, and redundant qualifiers
- **Direct statements**: Replace "This guide covers X" with "X" 
- **Information density**: Maximize technical content, minimize prose

Example format:
```markdown
# Document Title

## Description

Token creation patterns for CKB blockchain. Production-ready Rust code for fungible tokens with owner-controlled minting/burning. Token amount encoding (u128 as 16 bytes), conservation validation logic, and multi-cell token operations.

## Next Section
...
```

The ckb-docs-server automatically extracts these descriptions for the MCP resource listing, making documentation discoverable and providing context to AI assistants.

## Common Tasks

### Adding New Documentation

1. Create markdown file in appropriate `docs/` subdirectory.
2. **REQUIRED**: Add a `## Description` section immediately after the main title (see Documentation Format Requirements above).
3. **CRITICAL**: Add URI mapping to `crates/ckb-docs-server/src/docs.rs` in the `resource_mappings` array.
   - Format: `("ckb-dev-context://path/filename", "path/filename.md")`
   - Without this step, the new documentation will NOT be available through the MCP server
4. **RECOMMENDED**: Verify description format using the validation script:
   ```bash
   python3 utils/verify_descriptions.py --verbose
   ```
5. Documentation is automatically reloaded by the auto-managed server.
6. The description will be automatically extracted and shown in MCP resource listings.

**⚠️ IMPORTANT**: Always add newly created documents to the MCP server mapping. Documentation files are only accessible to AI assistants when properly registered in the `resource_mappings` array in `docs.rs`.

### Adding New RPC Endpoints  

1. Define request/response types in `shared/src/types.rs`.
2. Implement handler in respective server's `handlers/` module.
3. Register route in server's main routing.
4. Add comprehensive error handling.

### Contract Development Integration

1. Use ckb-tools-server for contract operations.
2. Use modern CKB development tools and frameworks.
3. Follow CKB development best practices.
4. Test on both testnet and mainnet configurations.

## Error Handling

All servers use structured error types:

```rust
// Shared error types
pub enum CkbMcpError {
    RpcError(String),
    SerializationError(String),
    NotFound(String),
    Internal(String),
}
```

Always provide meaningful error messages and proper HTTP status codes.

## Security Considerations

- Validate all external inputs (RPC responses, file paths).
- Use secure defaults for network configurations.
- Log security-relevant events appropriately.
- Handle sensitive data (private keys, seeds) carefully.
- Never commit secrets to version control.
- **IMPORTANT**: The ckb-tools-server includes a default test private key for development convenience. This key should **NEVER** be used in production. Always provide a secure `--private-key` parameter when deploying to production environments.

### Deployment Endpoint Security Model

The ckb-tools-server provides two methods for deploying cell data:

1. **MCP Tool (DeployCellData)**: Limited to 1KB inline hex data. Intended for small deployments within AI context limits.
2. **HTTP Endpoint (POST /deploy/file)**: Multipart form upload for files of any size. Accessed via curl or similar tools.

**Trust Model**: Both endpoints use the same private key configured at server startup. The HTTP endpoint is intentionally unauthenticated for local development simplicity. When deploying to production or shared environments:

- Run the server on localhost only, or behind an authenticated reverse proxy.
- Never expose the `/deploy/file` endpoint to untrusted networks without authentication.
- The server logs all deployment transactions for audit purposes.

**Why HTTP Instead of MCP for Large Files**: AI assistants cannot reliably generate or transmit large hex strings. The MCP protocol adds context overhead that compounds this limitation. The HTTP endpoint accepts raw binary data, avoiding hex encoding entirely.

## Performance Notes

- Use async/await for all I/O operations.
- Implement connection pooling for CKB RPC clients.
- Cache frequently accessed documentation.
- Consider rate limiting for production deployments.
- Profile memory usage with large documentation sets.

## Troubleshooting

### Common Issues

1. **CKB Node Connection**: Ensure CKB node is running and accessible.
2. **Port Conflicts**: Check ports 8001-8003 are available.
3. **Documentation Loading**: Verify file paths and permissions.
4. **MCP Protocol**: Validate HTTP headers and JSON-RPC format.

### Debug Commands

```bash
# Check server health
curl http://localhost:8001/health
curl http://localhost:8002/health
curl http://localhost:8003/health

# List available resources
curl http://localhost:8002/resources

# Test RPC connectivity
curl -X POST http://localhost:8001/rpc \
  -H "Content-Type: application/json" \
  -d '{"method": "get_tip_header", "params": [], "id": 1}'

# Test tools server (examples require parameters - see MCP tools/list for schemas)
curl -X POST http://localhost:8003/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"GetDefaultAccountInfo","arguments":{}}}'
```

## Utility Scripts

The `utils/` directory contains maintenance and validation scripts:

### Documentation Verification

```bash
# Verify all documentation has proper Description sections
python3 utils/verify_descriptions.py

# Show detailed verification output
python3 utils/verify_descriptions.py --verbose
```

This script ensures all documentation files maintain the required Description format for MCP server integration. Run before committing documentation changes.

## Contributing

1. Follow existing code patterns and styles.
2. Add tests for new functionality.
3. Update documentation for user-facing changes.
4. **Validate documentation**: Run `python3 utils/verify_descriptions.py` before committing doc changes.
5. Use descriptive commit messages.
6. Ensure all tests pass before submitting changes.
7. **Create git commits**: Make a git commit for every major group of changes (e.g., feature implementation, documentation updates, bug fixes). This helps maintain clear project history and facilitates easier rollbacks if needed.

## Resources

- [CKB Developer Docs](https://docs.nervos.org/)
- [Capsule Framework](https://github.com/nervosnetwork/capsule)
- [CKB-SDK](https://github.com/nervosnetwork/ckb-sdk-rust)
- [Molecule Serialization](https://github.com/nervosnetwork/molecule)