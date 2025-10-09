# CKB MCP Development Context

## Project Overview

This is a Rust workspace providing Model Context Protocol (MCP) servers for Nervos CKB blockchain development. The workspace contains three specialized MCP servers that help AI assistants build CKB smart contracts and applications.

## Architecture

```
ckb-mcp/
├── crates/
│   ├── shared/              # Common types, errors, and utilities
│   ├── ckb-rpc-server/      # Blockchain query tools (port 8001)
│   ├── ckb-docs-server/     # Documentation resources (port 8002)
│   └── ckb-tools-server/    # Development tools (port 8003)
├── docs/                    # CKB development documentation
└── Cargo.toml              # Workspace configuration
```

### Server Responsibilities

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

### Testing

```bash
# Run all tests
cargo test

# Run tests for specific server
cargo test -p ckb-rpc-server
cargo test -p ckb-docs-server
cargo test -p ckb-tools-server

# Run tests with logging
RUST_LOG=debug cargo test
```

**IMPORTANT: Test Timing Expectations**
- **ckb-rpc-server** tests: Fast (< 5 seconds total).
- **ckb-docs-server** tests: Fast (< 5 seconds total).
- **ckb-tools-server** tests: Slow - deployment tests wait up to 60 seconds per test for blockchain transaction confirmation. The full test suite can take several minutes. This is expected behavior as these tests deploy actual cells to the blockchain and verify confirmation.

### Building and Running

**IMPORTANT: Server Management**
- Servers are **automatically managed** in a separate process/window.
- Servers auto-restart when code changes are detected.
- **DO NOT** manually start, stop, or restart servers during development.
- Servers are accessible at ports 8001, 8002, 8003 at all times.
- **Only build** to validate compilation and eliminate errors/warnings.

```bash
# Build for validation only (servers auto-managed)
cargo build --workspace --release

# These commands are handled automatically - DO NOT RUN:
# cargo run --bin ckb-rpc-server
# cargo run --bin ckb-docs-server
# cargo run --bin ckb-tools-server
```

### CLI Parameters

If you need to run servers manually for debugging, use `--help` to see available parameters:

```bash
# View parameters for each server
cargo run --bin ckb-rpc-server -- --help
cargo run --bin ckb-docs-server -- --help
cargo run --bin ckb-tools-server -- --help
```

**Key parameters:**
- All servers: `--host`, `--port`, `--log-level`
- **ckb-rpc-server**: `--ckb-rpc` (CKB node URL)
- **ckb-docs-server**: `--docs-path` (custom docs directory)
- **ckb-tools-server**: `--ckb-rpc`, `--private-key` (transaction signing key)

### Debugging

- Servers run on ports 8001, 8002, 8003 respectively.
- Use `RUST_LOG=debug` for detailed logging.
- MCP communication uses HTTP transport only.
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