# CKB Tools Server

MCP server providing CKB development tools including cell deployment, address management, and testnet faucet access.

## Endpoints

### MCP Endpoint (`POST /mcp`)

Standard MCP protocol endpoint for AI tool integration.

**Available Tools:**
- `DeployCellData` - Deploy hex-encoded data up to 1KB
- `GetAddressBalance` - Query CKB balance for an address
- `GetChainType` - Get connected chain type (mainnet/testnet/devnet)
- `GetGenesisHash` - Get genesis block hash
- `GenerateLockInfo` - Generate lock script from private key
- `GetLockInfoFromAddress` - Extract lock info from address
- `RequestTestnetFunds` - Request funds from testnet faucet
- `GetDefaultAccountInfo` - Get info about server's configured account

### File Deployment Endpoint (`POST /deploy/file`)

HTTP multipart form endpoint for deploying files larger than 1KB.

**Usage:**
```bash
curl -F 'file=@/path/to/file' http://localhost:8003/deploy/file
```

**Response:**
```json
{
  "tx_hash": "0x...",
  "output_index": 0,
  "data_size": 12345,
  "capacity": 1234500000000
}
```

### Health Check (`GET /health`)

Returns `OK` if server is running.

### Server Info (`GET /`)

Returns server metadata and available endpoints.

## Security Model

### Private Key Configuration

The server uses a private key for signing deployment transactions. Specify via `--private-key` flag or use the built-in test key for development.

**WARNING**: The default test key is publicly known. Never use it with real funds.

### Endpoint Security

Both deployment methods (MCP tool and HTTP endpoint) use the same private key:

- **MCP (DeployCellData)**: Limited to 1KB, suitable for small inline data
- **HTTP (/deploy/file)**: No size limit, accepts multipart form uploads

The HTTP endpoint is intentionally unauthenticated for local development simplicity.

**Production Recommendations:**
- Bind to localhost only (`--host 127.0.0.1`)
- Use a reverse proxy with authentication for remote access
- Never expose to untrusted networks without authentication
- All deployments are logged for audit purposes

### Why Two Endpoints?

AI assistants cannot reliably generate large hex strings due to context limitations. The HTTP endpoint accepts raw binary data, bypassing hex encoding overhead and context constraints.

## Running

```bash
# Development (uses test key - DO NOT use with real funds)
./ckb-tools-server --port 8003 --ckb-rpc http://127.0.0.1:8114

# Production (with secure private key)
./ckb-tools-server --port 8003 --ckb-rpc http://your-node:8114 --private-key YOUR_PRIVATE_KEY
```

See `--help` for all options.
