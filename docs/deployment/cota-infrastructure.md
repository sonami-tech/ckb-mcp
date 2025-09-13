## Description

Comprehensive deployment guide for CoTA (Compact Token Aggregator) infrastructure covering aggregator server setup, registry configuration, and API endpoint deployment. Provides step-by-step instructions for setting up CoTA services, configuring database connections, managing token metadata, and establishing cross-chain bridges. Includes Docker deployment examples, monitoring setup, scaling considerations, and integration patterns for NFT marketplaces and wallet applications.

## Related Resources

- [ckb-dev-context://protocols/cota-protocol](ckb-dev-context://protocols/cota-protocol) - Layer-1.5 account-based token management solution using Sparse Merkle Trees
- [ckb-dev-context://patterns/cota-nft-development](ckb-dev-context://patterns/cota-nft-development) - Build cost-effective NFT applications using CoTA protocol
- [ckb-dev-context://api-reference/cota-sdk-examples](ckb-dev-context://api-reference/cota-sdk-examples) - CoTA SDK JavaScript implementation guide with production-ready examples

## Complete CoTA Infrastructure Stack

CoTA protocol requires a comprehensive infrastructure stack for production deployment. This guide covers all components needed for a fully operational CoTA system.

## Architecture Overview

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Client dApp   │    │   CoTA SDK JS    │    │  CKB Frontend   │
└─────────────────┘    └──────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
         ┌───────────────────────┼───────────────────────┐
         │                       │                       │
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│Registry Aggreg. │    │  CoTA Aggregator │    │   CKB Indexer   │
└─────────────────┘    └──────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌──────────────────┐
                    │   CoTA Syncer    │
                    └──────────────────┘
                                 │
         ┌───────────────────────┼───────────────────────┐
         │                       │                       │
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│     MySQL       │    │    CKB Node      │    │   RocksDB       │
│   Database      │    │                  │    │   (SMT Store)   │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## Core Components

### 1. CKB Node
The foundation layer providing blockchain data and transaction processing.

**Configuration:**
```toml
# ckb.toml
[rpc]
listen_address = "0.0.0.0:8114"
modules = ["Net", "Pool", "Miner", "Chain", "Stats", "Subscription", "Experiment", "Indexer", "Debug"]

[network]
listen_addresses = ["/ip4/0.0.0.0/tcp/8115"]
public_addresses = ["/ip4/YOUR_PUBLIC_IP/tcp/8115"]

[store]
path = "data"

[indexer]
index_tx_pool = true
```

**Docker Deployment:**
```yaml
# docker-compose.yml - CKB Node
ckb-node:
  image: nervos/ckb:v0.117.0
  container_name: ckb-node
  ports:
    - "8114:8114"
    - "8115:8115"
  volumes:
    - ./ckb-data:/data
    - ./ckb.toml:/etc/ckb.toml
  command: run --config /etc/ckb.toml
  restart: unless-stopped
```

### 2. CKB Indexer
Provides efficient cell and transaction querying capabilities.

**Configuration:**
```yaml
ckb-indexer:
  image: nervos/ckb-indexer:v0.5.0
  container_name: ckb-indexer
  environment:
    - CKB_NODE_RPC_URL=http://ckb-node:8114
  ports:
    - "8116:8116"
  volumes:
    - ./indexer-data:/data
  command: >
    --listen-uri http://0.0.0.0:8116
    --ckb-uri http://ckb-node:8114
    --db-path /data
  depends_on:
    - ckb-node
  restart: unless-stopped
```

### 3. MySQL Database
Stores indexed CoTA data for efficient querying.

**Database Schema Setup:**
```sql
-- Core CoTA tables
CREATE DATABASE cota_db;
USE cota_db;

-- Registry information
CREATE TABLE registry_entries (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    lock_hash VARCHAR(66) UNIQUE NOT NULL,
    block_number BIGINT NOT NULL,
    tx_hash VARCHAR(66) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_lock_hash (lock_hash),
    INDEX idx_block_number (block_number)
);

-- CoTA collections
CREATE TABLE cota_entries (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    cota_id VARCHAR(42) NOT NULL,
    issuer_lock_hash VARCHAR(66) NOT NULL,
    total BIGINT NOT NULL DEFAULT 0,
    issued BIGINT NOT NULL DEFAULT 0,
    block_number BIGINT NOT NULL,
    tx_hash VARCHAR(66) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_cota_id (cota_id),
    INDEX idx_issuer (issuer_lock_hash),
    INDEX idx_block_number (block_number)
);

-- NFT holdings and transfers
CREATE TABLE nft_entries (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    cota_id VARCHAR(42) NOT NULL,
    token_index BIGINT NOT NULL,
    lock_hash VARCHAR(66) NOT NULL,
    state TINYINT NOT NULL DEFAULT 0,
    characteristic VARCHAR(42) NOT NULL,
    tx_hash VARCHAR(66) NOT NULL,
    block_number BIGINT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE KEY uk_cota_token_lock (cota_id, token_index, lock_hash),
    INDEX idx_lock_hash (lock_hash),
    INDEX idx_cota_id (cota_id),
    INDEX idx_block_number (block_number)
);
```

**Docker Configuration:**
```yaml
mysql:
  image: mysql:8.0
  container_name: cota-mysql
  environment:
    MYSQL_ROOT_PASSWORD: secure_password
    MYSQL_DATABASE: cota_db
    MYSQL_USER: cota_user
    MYSQL_PASSWORD: cota_password
  ports:
    - "3306:3306"
  volumes:
    - ./mysql-data:/var/lib/mysql
    - ./mysql-init:/docker-entrypoint-initdb.d
  restart: unless-stopped
```

### 4. CoTA Syncer
Indexes CoTA-specific data from CKB blockchain transactions.

**Environment Configuration:**
```bash
# .env file for CoTA Syncer
DATABASE_URL=mysql://cota_user:cota_password@mysql:3306/cota_db
CKB_NODE_URL=http://ckb-node:8114
CKB_INDEXER_URL=http://ckb-indexer:8116
IS_MAINNET=false
RUST_LOG=info
MAX_POOL_SIZE=20
SYNC_THREADS=4
```

**Docker Deployment:**
```yaml
cota-syncer:
  image: nervinalabs/cota-syncer:latest
  container_name: cota-syncer
  environment:
    - DATABASE_URL=mysql://cota_user:cota_password@mysql:3306/cota_db
    - CKB_NODE_URL=http://ckb-node:8114
    - CKB_INDEXER_URL=http://ckb-indexer:8116
    - IS_MAINNET=false
    - RUST_LOG=info
  depends_on:
    - mysql
    - ckb-node
    - ckb-indexer
  restart: unless-stopped
```

### 5. CoTA Registry Aggregator
Manages global registry SMT for user registration.

**Configuration:**
```bash
# Registry Aggregator Environment
DATABASE_URL=mysql://cota_user:cota_password@mysql:3306/cota_db
CKB_INDEXER_URL=http://ckb-indexer:8116
IS_MAINNET=false
RUST_LOG=info
PORT=3050
```

**Docker Setup:**
```yaml
registry-aggregator:
  image: nervinalabs/cota-registry-aggregator:latest
  container_name: cota-registry-aggregator
  environment:
    - DATABASE_URL=mysql://cota_user:cota_password@mysql:3306/cota_db
    - CKB_INDEXER_URL=http://ckb-indexer:8116
    - IS_MAINNET=false
    - RUST_LOG=info
  ports:
    - "3050:3050"
  volumes:
    - ./registry-smt-data:/app/store.db
  depends_on:
    - mysql
    - ckb-indexer
    - cota-syncer
  restart: unless-stopped
```

### 6. CoTA Aggregator
Provides SMT generation and querying for CoTA operations.

**Configuration:**
```bash
# CoTA Aggregator Environment
DATABASE_URL=mysql://cota_user:cota_password@mysql:3306/cota_db
CKB_NODE_URL=http://ckb-node:8114
IS_MAINNET=false
RUST_LOG=info
PORT=3030
MAX_POOL_SIZE=20
THREADS=3
```

**Docker Setup:**
```yaml
cota-aggregator:
  image: nervinalabs/cota-aggregator:latest
  container_name: cota-aggregator
  environment:
    - DATABASE_URL=mysql://cota_user:cota_password@mysql:3306/cota_db
    - CKB_NODE_URL=http://ckb-node:8114
    - IS_MAINNET=false
    - RUST_LOG=info
    - MAX_POOL_SIZE=20
    - THREADS=3
  ports:
    - "3030:3030"
  volumes:
    - ./cota-smt-data:/app/store.db
  depends_on:
    - mysql
    - ckb-node
    - cota-syncer
  restart: unless-stopped
```

## Complete Docker Compose Configuration

```yaml
version: '3.8'

services:
  ckb-node:
    image: nervos/ckb:v0.117.0
    container_name: ckb-node
    ports:
      - "8114:8114"
      - "8115:8115"
    volumes:
      - ./ckb-data:/data
      - ./ckb.toml:/etc/ckb.toml
    command: run --config /etc/ckb.toml
    restart: unless-stopped

  ckb-indexer:
    image: nervos/ckb-indexer:v0.5.0
    container_name: ckb-indexer
    environment:
      - CKB_NODE_RPC_URL=http://ckb-node:8114
    ports:
      - "8116:8116"
    volumes:
      - ./indexer-data:/data
    command: >
      --listen-uri http://0.0.0.0:8116
      --ckb-uri http://ckb-node:8114
      --db-path /data
    depends_on:
      - ckb-node
    restart: unless-stopped

  mysql:
    image: mysql:8.0
    container_name: cota-mysql
    environment:
      MYSQL_ROOT_PASSWORD: secure_password
      MYSQL_DATABASE: cota_db
      MYSQL_USER: cota_user
      MYSQL_PASSWORD: cota_password
    ports:
      - "3306:3306"
    volumes:
      - ./mysql-data:/var/lib/mysql
      - ./mysql-init:/docker-entrypoint-initdb.d
    restart: unless-stopped

  cota-syncer:
    image: nervinalabs/cota-syncer:latest
    container_name: cota-syncer
    environment:
      - DATABASE_URL=mysql://cota_user:cota_password@mysql:3306/cota_db
      - CKB_NODE_URL=http://ckb-node:8114
      - CKB_INDEXER_URL=http://ckb-indexer:8116
      - IS_MAINNET=false
      - RUST_LOG=info
    depends_on:
      - mysql
      - ckb-node
      - ckb-indexer
    restart: unless-stopped

  registry-aggregator:
    image: nervinalabs/cota-registry-aggregator:latest
    container_name: cota-registry-aggregator
    environment:
      - DATABASE_URL=mysql://cota_user:cota_password@mysql:3306/cota_db
      - CKB_INDEXER_URL=http://ckb-indexer:8116
      - IS_MAINNET=false
      - RUST_LOG=info
    ports:
      - "3050:3050"
    volumes:
      - ./registry-smt-data:/app/store.db
    depends_on:
      - mysql
      - ckb-indexer
      - cota-syncer
    restart: unless-stopped

  cota-aggregator:
    image: nervinalabs/cota-aggregator:latest
    container_name: cota-aggregator
    environment:
      - DATABASE_URL=mysql://cota_user:cota_password@mysql:3306/cota_db
      - CKB_NODE_URL=http://ckb-node:8114
      - IS_MAINNET=false
      - RUST_LOG=info
      - MAX_POOL_SIZE=20
      - THREADS=3
    ports:
      - "3030:3030"
    volumes:
      - ./cota-smt-data:/app/store.db
    depends_on:
      - mysql
      - ckb-node
      - cota-syncer
    restart: unless-stopped

  # Optional: nginx reverse proxy
  nginx:
    image: nginx:alpine
    container_name: cota-nginx
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
      - ./ssl:/etc/nginx/ssl
    depends_on:
      - cota-aggregator
      - registry-aggregator
    restart: unless-stopped

volumes:
  ckb-data:
  indexer-data:
  mysql-data:
  registry-smt-data:
  cota-smt-data:
```

## Production Deployment Guide

### 1. System Requirements

**Minimum Requirements:**
- **CPU**: 4 cores
- **RAM**: 16GB 
- **Storage**: 500GB SSD (growing ~10GB/month)
- **Network**: 100Mbps bandwidth

**Recommended Production:**
- **CPU**: 8+ cores
- **RAM**: 32GB+
- **Storage**: 1TB+ NVMe SSD
- **Network**: 1Gbps bandwidth

### 2. Deployment Steps

```bash
# 1. Clone infrastructure setup
git clone https://github.com/your-org/cota-infrastructure
cd cota-infrastructure

# 2. Configure environment
cp .env.example .env
# Edit .env with your specific settings

# 3. Initialize directories
mkdir -p {ckb-data,indexer-data,mysql-data,registry-smt-data,cota-smt-data}

# 4. Start infrastructure
docker-compose up -d

# 5. Verify services
docker-compose ps
docker-compose logs -f
```

### 3. Health Monitoring

**Monitoring Script:**
```bash
#!/bin/bash
# health-check.sh

check_service() {
    local service=$1
    local port=$2
    local host=${3:-localhost}
    
    if curl -s "http://$host:$port" > /dev/null; then
        echo "✅ $service is healthy"
        return 0
    else
        echo "❌ $service is down"
        return 1
    fi
}

echo "🔍 Checking CoTA infrastructure health..."

check_service "CKB Node" 8114
check_service "CKB Indexer" 8116  
check_service "Registry Aggregator" 3050
check_service "CoTA Aggregator" 3030

# Check database connectivity
if mysql -h localhost -u cota_user -pcota_password -e "SELECT 1" > /dev/null 2>&1; then
    echo "✅ MySQL is healthy"
else
    echo "❌ MySQL is down"
fi

echo "Health check completed."
```

### 4. Backup and Recovery

**Database Backup:**
```bash
#!/bin/bash
# backup.sh
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/backups/cota"

mkdir -p $BACKUP_DIR

# Backup MySQL database
docker exec cota-mysql mysqldump -u root -psecure_password cota_db > $BACKUP_DIR/cota_db_$DATE.sql

# Backup SMT data  
tar -czf $BACKUP_DIR/smt_data_$DATE.tar.gz registry-smt-data/ cota-smt-data/

# Cleanup old backups (keep 7 days)
find $BACKUP_DIR -name "*.sql" -mtime +7 -delete
find $BACKUP_DIR -name "*.tar.gz" -mtime +7 -delete

echo "Backup completed: $DATE"
```

**Recovery Process:**
```bash
#!/bin/bash
# restore.sh
BACKUP_FILE=$1

if [ -z "$BACKUP_FILE" ]; then
    echo "Usage: $0 <backup_file.sql>"
    exit 1
fi

# Stop services
docker-compose down

# Restore database
docker-compose up -d mysql
sleep 30  # Wait for MySQL to start
cat $BACKUP_FILE | docker exec -i cota-mysql mysql -u root -psecure_password cota_db

# Restart all services
docker-compose up -d

echo "Recovery completed"
```

## Performance Optimization

### 1. Database Tuning

```sql
-- MySQL optimization for CoTA workload
SET GLOBAL innodb_buffer_pool_size = 8G;
SET GLOBAL innodb_log_file_size = 512M;
SET GLOBAL innodb_flush_log_at_trx_commit = 2;
SET GLOBAL query_cache_size = 256M;
SET GLOBAL max_connections = 200;

-- Indexing optimization
CREATE INDEX idx_nft_entries_composite ON nft_entries(cota_id, lock_hash, block_number);
CREATE INDEX idx_registry_block_time ON registry_entries(block_number, created_at);
```

### 2. Aggregator Scaling

```bash
# Scale aggregator replicas
docker-compose up -d --scale cota-aggregator=3

# Load balancer configuration (nginx.conf)
upstream cota_aggregators {
    server cota-aggregator-1:3030;
    server cota-aggregator-2:3030;  
    server cota-aggregator-3:3030;
}

server {
    listen 80;
    location /aggregator {
        proxy_pass http://cota_aggregators;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

### 3. Caching Layer

```yaml
# Add Redis for caching
redis:
  image: redis:7-alpine
  container_name: cota-redis
  ports:
    - "6379:6379"
  volumes:
    - ./redis-data:/data
  restart: unless-stopped
```

## Security Configuration

### 1. Network Security

```yaml
# docker-compose.yml with networks
networks:
  cota-internal:
    driver: bridge
    internal: true
  cota-external:
    driver: bridge

services:
  # Only expose necessary ports externally
  nginx:
    networks:
      - cota-external
      - cota-internal
  
  cota-aggregator:
    networks:
      - cota-internal
    # Remove external port exposure
```

### 2. SSL/TLS Configuration

```nginx
# nginx.conf with SSL
server {
    listen 443 ssl http2;
    server_name your-domain.com;
    
    ssl_certificate /etc/nginx/ssl/cert.pem;
    ssl_certificate_key /etc/nginx/ssl/key.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    
    location /aggregator {
        proxy_pass http://cota-aggregator:3030;
        proxy_ssl_verify off;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
    
    location /registry-aggregator {
        proxy_pass http://registry-aggregator:3050;
        proxy_ssl_verify off;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

### 3. Environment Security

```bash
# Use Docker secrets for sensitive data
echo "secure_password" | docker secret create mysql_root_password -
echo "cota_password" | docker secret create mysql_cota_password -

# Update docker-compose.yml
mysql:
  secrets:
    - mysql_root_password
    - mysql_cota_password
  environment:
    MYSQL_ROOT_PASSWORD_FILE: /run/secrets/mysql_root_password
    MYSQL_PASSWORD_FILE: /run/secrets/mysql_cota_password
```

## Maintenance Operations

### 1. Log Management

```bash
# Log rotation configuration
cat > /etc/logrotate.d/docker-cota << EOF
/var/lib/docker/containers/*/*.log {
    rotate 7
    daily
    compress
    missingok
    delaycompress
    copytruncate
    maxsize 100M
}
EOF
```

### 2. Update Procedures

```bash
#!/bin/bash
# update.sh
echo "🔄 Updating CoTA infrastructure..."

# Pull latest images
docker-compose pull

# Rolling update with zero downtime
docker-compose up -d --no-deps cota-aggregator
sleep 30
docker-compose up -d --no-deps registry-aggregator
sleep 30
docker-compose up -d --no-deps cota-syncer

echo "✅ Update completed"
```

This comprehensive infrastructure setup provides a production-ready CoTA deployment with high availability, security, and monitoring capabilities. The modular architecture allows for scaling individual components based on load requirements.