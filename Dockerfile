# Multi-stage build for CKB MCP server
# Stage 1: Build
FROM rust:1.95-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the source code
COPY . .

# Build the final binary
RUN cargo build --release -p ckb-ai-mcp

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder stage
COPY --from=builder /app/target/release/ckb-ai-mcp /usr/local/bin/

# Copy application resources
COPY --from=builder /app/docs /app/docs

# Create data directory for stats persistence
RUN mkdir -p /app/data

# Expose port for the unified server
EXPOSE 3112

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:3112/health || exit 1

# Start server
CMD ["/bin/sh", "-c", "exec /usr/local/bin/ckb-ai-mcp --host 0.0.0.0 --port 3112 --ckb-rpc ${CKB_RPC_URL:-http://127.0.0.1:8114} --docs-path /app/docs --stats-db /app/data/ckb-ai-mcp-stats.redb"]
