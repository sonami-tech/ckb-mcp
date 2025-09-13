## Description

Comprehensive integration guide for SSRI (Server-Side Rich Information) server enabling off-chain CKB script execution and enhanced blockchain data access. Covers server setup, VM environment configuration, enhanced syscall APIs, and method-based script interaction patterns. Provides examples of complex queries, data processing, and integration with CKB applications without on-chain execution costs. Essential tool for building data-intensive CKB applications.

## Related Resources

- [ckb-dev-context://protocols/ssri](ckb-dev-context://protocols/ssri) - Extension protocol enabling CKB scripts to provide rich information through off-chain execution
- [ckb-dev-context://patterns/ssri-implementation](ckb-dev-context://patterns/ssri-implementation) - Implementation guide for Script-Sourced Rich Information in CKB smart contracts
- [ckb-dev-context://api-reference/ccc-sdk-ssri](ckb-dev-context://api-reference/ccc-sdk-ssri) - Guide to Script-Sourced Rich Information framework in the CCC SDK

The SSRI Server provides off-chain execution capabilities for CKB scripts, enabling rich information queries and complex computations without on-chain gas costs.

## Overview

The SSRI Server:
- Executes CKB scripts in an isolated VM environment
- Provides enhanced syscalls for blockchain data access
- Supports multiple execution contexts (Code, Script, Cell, Transaction)
- Enables method-based script interaction

## Server Configuration

### Basic Setup

```toml
# config.toml
[server]
host = "127.0.0.1"
port = 9090

[ckb]
url = "https://testnet.ckb.dev/rpc"
# For mainnet: url = "https://mainnet.ckb.dev/rpc"

[ssri]
max_cycles = 70_000_000
max_output_size = 262144  # 256KB
```

### Docker Deployment

```dockerfile
FROM rust:1.75-alpine as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM alpine:latest
RUN apk add --no-cache ca-certificates
COPY --from=builder /app/target/release/ssri-server /usr/local/bin/
COPY config.toml /etc/ssri/
CMD ["ssri-server", "-c", "/etc/ssri/config.toml"]
```

## API Endpoints

### Execute Script Level Code

Execute raw script code with minimal context:

```rust
// POST /run_script_level_code
#[derive(Serialize, Deserialize)]
struct CodeRequest {
    code: String,        // Hex-encoded script code
    method: String,      // Method signature (e.g., "UDT.name")
    args: Vec<String>,   // Method arguments
}

// Example request
let response = client.post("/run_script_level_code")
    .json(&CodeRequest {
        code: hex::encode(&script_code),
        method: "SSRI.version".to_string(),
        args: vec![],
    })
    .send()
    .await?;
```

### Execute Script Level Script

Execute with script structure context:

```rust
// POST /run_script_level_script
#[derive(Serialize, Deserialize)]
struct ScriptRequest {
    script: Script,      // CKB script structure
    method: String,      // Method signature
    args: Vec<String>,   // Method arguments
}

// Example: Query token metadata
let response = client.post("/run_script_level_script")
    .json(&ScriptRequest {
        script: udt_script.clone(),
        method: "UDT.symbol".to_string(),
        args: vec![],
    })
    .send()
    .await?;
```

### Execute Script Level Cell

Execute with full cell context:

```rust
// POST /run_script_level_cell
#[derive(Serialize, Deserialize)]
struct CellRequest {
    out_point: OutPoint,  // Cell location
    method: String,       // Method signature
    args: Vec<String>,    // Method arguments
}

// Example: Query token balance
let response = client.post("/run_script_level_cell")
    .json(&CellRequest {
        out_point: cell_out_point,
        method: "UDT.balance".to_string(),
        args: vec![],
    })
    .send()
    .await?;
```

### Execute Script Level Transaction

Execute with full transaction context:

```rust
// POST /run_script_level_tx
#[derive(Serialize, Deserialize)]
struct TransactionRequest {
    tx: Transaction,      // Full transaction
    index: u32,           // Script group index
    method: String,       // Method signature
    args: Vec<String>,    // Method arguments
}
```

## Client Integration

### Rust Client Example

```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct SsriClient {
    client: Client,
    base_url: String,
}

impl SsriClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
        }
    }
    
    pub async fn execute_method(
        &self,
        script: &Script,
        method: &str,
    ) -> Result<Vec<u8>, Error> {
        let request = ScriptRequest {
            script: script.clone(),
            method: method.to_string(),
            args: vec![],
        };
        
        let response = self.client
            .post(&format!("{}/run_script_level_script", self.base_url))
            .json(&request)
            .send()
            .await?;
            
        if response.status().is_success() {
            let result: ExecuteResult = response.json().await?;
            hex::decode(result.output).map_err(Into::into)
        } else {
            Err(Error::ServerError(response.text().await?))
        }
    }
}
```

### JavaScript/TypeScript Client

```typescript
class SsriClient {
    constructor(private baseUrl: string) {}
    
    async executeMethod(
        script: Script,
        method: string,
        args: string[] = []
    ): Promise<Uint8Array> {
        const response = await fetch(
            `${this.baseUrl}/run_script_level_script`,
            {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    script,
                    method,
                    args,
                }),
            }
        );
        
        if (!response.ok) {
            throw new Error(`SSRI error: ${await response.text()}`);
        }
        
        const result = await response.json();
        return hexToBytes(result.output);
    }
}
```

## Enhanced Syscalls

### Find Live Cells

```rust
// Find cells by type script
let mut out_point = [0u8; 36];
match ssri_syscalls::find_out_point_by_type(&mut out_point, &type_script) {
    Ok(_) => {
        // Found cell with matching type script
        let cell_data = ssri_syscalls::find_cell_data_by_out_point(&out_point)?;
        // Process cell data...
    }
    Err(_) => {
        // No matching cells found
    }
}
```

### Load Cell Information

```rust
// Load cell by outpoint
let mut cell_output = vec![0u8; 1024];
let len = ssri_syscalls::find_cell_by_out_point(
    &mut cell_output,
    &out_point
)?;

let cell = CellOutput::from_slice(&cell_output[..len])?;
```

## Use Cases

### Token Information Service

```rust
pub async fn get_token_info(
    ssri_client: &SsriClient,
    token_script: &Script,
) -> Result<TokenInfo, Error> {
    // Execute multiple methods in parallel
    let (name, symbol, decimals) = tokio::join!(
        ssri_client.execute_method(token_script, "UDT.name"),
        ssri_client.execute_method(token_script, "UDT.symbol"),
        ssri_client.execute_method(token_script, "UDT.decimals")
    );
    
    Ok(TokenInfo {
        name: String::from_utf8(parse_molecule_string(&name?))?,
        symbol: String::from_utf8(parse_molecule_string(&symbol?))?,
        decimals: decimals?[0],
    })
}
```

### Balance Query Service

```rust
pub async fn get_all_balances(
    ssri_client: &SsriClient,
    address: &Address,
) -> Result<Vec<Balance>, Error> {
    // Find all cells for address
    let cells = find_cells_by_lock(&address.into())?;
    
    let mut balances = Vec::new();
    
    for cell in cells {
        if let Some(type_script) = cell.output.type_() {
            // Query balance using SSRI
            let balance_bytes = ssri_client
                .execute_cell_method(&cell.out_point, "UDT.balance")
                .await?;
                
            let balance = u128::from_le_bytes(
                balance_bytes[..16].try_into()?
            );
            
            balances.push(Balance {
                token: type_script,
                amount: balance,
            });
        }
    }
    
    Ok(balances)
}
```

### Script Capability Discovery

```rust
pub async fn discover_capabilities(
    ssri_client: &SsriClient,
    script: &Script,
) -> Result<ScriptCapabilities, Error> {
    // Get all methods
    let methods_result = ssri_client
        .execute_method(script, "SSRI.get_methods")
        .await?;
        
    let methods = parse_method_array(&methods_result)?;
    
    // Check for specific capabilities
    let has_udt = methods.iter().any(|m| {
        m == &UDT_BALANCE || m == &UDT_NAME
    });
    
    let has_nft = methods.iter().any(|m| {
        m == &NFT_METADATA || m == &NFT_OWNER
    });
    
    Ok(ScriptCapabilities {
        methods,
        is_udt: has_udt,
        is_nft: has_nft,
        version: get_ssri_version(ssri_client, script).await?,
    })
}
```

## Error Handling

### Server Errors

```rust
#[derive(Debug, Error)]
pub enum SsriError {
    #[error("Script execution failed: {0}")]
    ExecutionError(String),
    
    #[error("Method not found: {0}")]
    MethodNotFound(String),
    
    #[error("Invalid context for method")]
    InvalidContext,
    
    #[error("Exceeded cycle limit")]
    CycleLimit,
    
    #[error("Output too large")]
    OutputLimit,
}
```

### Client-Side Validation

```rust
fn validate_ssri_response(response: &[u8], expected_type: &str) -> Result<(), Error> {
    match expected_type {
        "u8" => {
            if response.len() != 1 {
                return Err(Error::InvalidResponse);
            }
        }
        "u128" => {
            if response.len() != 16 {
                return Err(Error::InvalidResponse);
            }
        }
        "string" => {
            // Validate Molecule vector encoding
            if response.len() < 4 {
                return Err(Error::InvalidResponse);
            }
        }
        _ => {}
    }
    
    Ok(())
}
```

## Performance Optimization

### Caching Strategy

```rust
use lru::LruCache;
use std::sync::Mutex;

pub struct CachedSsriClient {
    client: SsriClient,
    cache: Mutex<LruCache<CacheKey, Vec<u8>>>,
}

impl CachedSsriClient {
    pub async fn execute_method(
        &self,
        script: &Script,
        method: &str,
    ) -> Result<Vec<u8>, Error> {
        let key = CacheKey {
            script_hash: script.calc_script_hash(),
            method: method.to_string(),
        };
        
        // Check cache
        if let Some(cached) = self.cache.lock().unwrap().get(&key) {
            return Ok(cached.clone());
        }
        
        // Execute and cache
        let result = self.client.execute_method(script, method).await?;
        self.cache.lock().unwrap().put(key, result.clone());
        
        Ok(result)
    }
}
```

### Batch Requests

```rust
pub async fn batch_execute(
    client: &SsriClient,
    requests: Vec<(Script, String)>,
) -> Result<Vec<Vec<u8>>, Error> {
    let futures: Vec<_> = requests
        .into_iter()
        .map(|(script, method)| {
            client.execute_method(&script, &method)
        })
        .collect();
        
    futures::future::try_join_all(futures).await
}
```

## Security Considerations

1. **Cycle Limits**: Always configure appropriate cycle limits
2. **Output Size**: Limit output size to prevent DoS
3. **Input Validation**: Validate all inputs before execution
4. **Network Security**: Use HTTPS in production
5. **Rate Limiting**: Implement rate limiting for public endpoints

## Monitoring

### Health Check Endpoint

```rust
// GET /health
#[derive(Serialize)]
struct HealthStatus {
    status: String,
    version: String,
    ckb_connected: bool,
    uptime_seconds: u64,
}
```

### Metrics Collection

```rust
#[derive(Debug)]
struct SsriMetrics {
    total_requests: AtomicU64,
    failed_requests: AtomicU64,
    average_execution_time_ms: AtomicU64,
    cache_hit_rate: AtomicU64,
}
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_udt_methods() {
        let server = TestSsriServer::new();
        let client = SsriClient::new(&server.url());
        
        let script = create_test_udt_script();
        
        // Test name method
        let name = client.execute_method(&script, "UDT.name").await.unwrap();
        assert_eq!(parse_string(&name), "Test Token");
        
        // Test symbol method
        let symbol = client.execute_method(&script, "UDT.symbol").await.unwrap();
        assert_eq!(parse_string(&symbol), "TEST");
    }
}
```

### Integration Tests

```bash
#!/bin/bash
# test.sh

# Start SSRI server
./ssri-server -c config.test.toml &
SERVER_PID=$!

# Wait for server to start
sleep 2

# Run test queries
curl -X POST http://localhost:9090/run_script_level_code \
  -H "Content-Type: application/json" \
  -d '{
    "code": "'$SCRIPT_CODE'",
    "method": "SSRI.version",
    "args": []
  }'

# Cleanup
kill $SERVER_PID
```