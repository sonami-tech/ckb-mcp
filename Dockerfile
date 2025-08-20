# Multi-stage build for CKB MCP servers
# Stage 1: Build
FROM rust:1.88-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the source code
COPY . .

# Build the final binaries
RUN cargo build --release --workspace

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    supervisor \
    && rm -rf /var/lib/apt/lists/*

# Copy binaries from builder stage
COPY --from=builder /app/target/release/ckb-rpc-server /usr/local/bin/
COPY --from=builder /app/target/release/ckb-docs-server /usr/local/bin/
COPY --from=builder /app/target/release/ckb-tools-server /usr/local/bin/

# Copy application resources
COPY --from=builder /app/docs /app/docs
COPY --from=builder /app/crates/ckb-tools-server/templates /app/templates

# Copy supervisor configuration
COPY docker/supervisord.conf /etc/supervisor/conf.d/supervisord.conf

# Expose ports for all three servers
EXPOSE 8001 8002 8003

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:8001/health && \
        curl -f http://localhost:8002/health && \
        curl -f http://localhost:8003/health || exit 1

# Start supervisor
CMD ["/usr/bin/supervisord", "-c", "/etc/supervisor/conf.d/supervisord.conf"]