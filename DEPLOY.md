# CKB MCP Deployment Guide

## Description

Comprehensive deployment documentation for CKB MCP servers using Docker containers. Covers staging and production environments with automated updates via Shepherd. Essential for DevOps teams deploying CKB blockchain development tools.

## Overview

This project uses Docker containers for deployment with automatic updates. All three MCP servers (RPC, Docs, Tools) run in a single container managed by Supervisor.

- **Staging Environment**: Auto-deploys from `develop` branch
- **Production Environment**: Auto-deploys from tagged releases
- **Container Registry**: GitHub Container Registry (ghcr.io)
- **Auto-Updates**: Shepherd for Docker Compose-aware updates

## Prerequisites

- Docker and Docker Compose installed
- Access to deploy servers (staging/production)
- GitHub Container Registry access

## Docker Images

Images are automatically built and published to:
- **Development**: `ghcr.io/sonami-tech/ckb-mcp:dev-latest` (from `develop` branch)
- **Production**: `ghcr.io/sonami-tech/ckb-mcp:latest` (from version tags)

## Server Architecture

Single container running three servers:
- **Port 8001**: CKB RPC Server (blockchain queries)
- **Port 8002**: CKB Docs Server (documentation)
- **Port 8003**: CKB Tools Server (development tools)

Health checks monitor all three endpoints.

## Local Development

Build and run locally:

```bash
# Build Docker image
docker build -t ckb-mcp:local .

# Run with Docker Compose
docker compose up -d

# Check health
curl http://localhost:8001/health
curl http://localhost:8002/health
curl http://localhost:8003/health
```

### Configuration

All deployments support the following environment variable:

- **CKB_RPC_URL**: CKB node RPC endpoint (default: `http://127.0.0.1:8114`)

Override via environment variable or `.env` file:

```bash
# Using environment variable
CKB_RPC_URL=http://your-ckb-node:8114 docker compose up -d

# Using .env file
echo "CKB_RPC_URL=http://your-ckb-node:8114" > .env
docker compose up -d
```

## Staging Deployment

Staging server automatically deploys development builds from the `develop` branch.

### Initial Setup

1. Clone repository on staging server:
```bash
git clone https://github.com/sonami-tech/ckb-mcp.git /opt/ckb-mcp
cd /opt/ckb-mcp
```

2. Start staging services:
```bash
docker compose -f docker/docker-compose.staging.yml up -d
```

3. Start Shepherd auto-updater:
```bash
docker compose -f docker/docker-compose.shepherd.yml up -d
```

### Custom CKB Node Configuration

To connect to a different CKB node:

```bash
# Create environment override
echo "CKB_RPC_URL=http://testnet.ckb.org:8114" > /opt/ckb-mcp/.env

# Restart services
docker compose -f docker/docker-compose.staging.yml down
docker compose -f docker/docker-compose.staging.yml up -d
```

### Verification

```bash
# Check container status
docker compose -f docker/docker-compose.staging.yml ps

# Check health endpoints
curl http://staging-server:8001/health
curl http://staging-server:8002/health
curl http://staging-server:8003/health

# Check Shepherd logs
docker logs ckb-mcp-shepherd
```

## Production Deployment

Production server automatically deploys stable releases from version tags.

### Initial Setup

1. Clone repository on production server:
```bash
git clone https://github.com/sonami-tech/ckb-mcp.git /opt/ckb-mcp
cd /opt/ckb-mcp
```

2. Start production services:
```bash
docker compose -f docker/docker-compose.production.yml up -d
```

3. Start Shepherd auto-updater:
```bash
docker compose -f docker/docker-compose.shepherd.yml up -d
```

### Custom CKB Node Configuration

To connect to a different CKB node:

```bash
# Create environment override
echo "CKB_RPC_URL=http://mainnet.ckb.org:8114" > /opt/ckb-mcp/.env

# Restart services
docker compose -f docker/docker-compose.production.yml down
docker compose -f docker/docker-compose.production.yml up -d
```

### Verification

```bash
# Check container status
docker compose -f docker/docker-compose.production.yml ps

# Check health endpoints
curl http://production-server:8001/health
curl http://production-server:8002/health
curl http://production-server:8003/health
```

## Automatic Updates

Shepherd monitors for new Docker images and automatically updates containers:

- **Check Interval**: Every 5 minutes
- **Update Trigger**: New image tags in registry
- **Cleanup**: Automatically removes old images
- **Zero Downtime**: Rolling updates preserve service availability

### Shepherd Configuration

Shepherd only updates containers with the `shepherd.enable=true` label, which is set in both staging and production compose files.

## Manual Operations

### Force Update

```bash
# Staging
docker compose -f docker/docker-compose.staging.yml pull
docker compose -f docker/docker-compose.staging.yml up -d

# Production
docker compose -f docker/docker-compose.production.yml pull
docker compose -f docker/docker-compose.production.yml up -d
```

### View Logs

```bash
# All services
docker compose -f docker/docker-compose.staging.yml logs -f

# Specific service
docker logs ckb-mcp-staging -f

# Shepherd logs
docker logs ckb-mcp-shepherd -f
```

### Stop Services

```bash
# Stop application
docker compose -f docker/docker-compose.staging.yml down

# Stop Shepherd
docker compose -f docker/docker-compose.shepherd.yml down
```

## Release Process

### Development Release

1. Push changes to `develop` branch
2. GitHub Actions builds and pushes `dev-latest` image
3. Shepherd detects new image and updates staging server

### Production Release

1. Create and push version tag:
```bash
git tag v1.0.0
git push origin v1.0.0
```

2. GitHub Actions builds and pushes `latest` image
3. Shepherd detects new image and updates production server

## Monitoring

### Health Checks

All containers include health checks that verify all three servers are responding:

```bash
# Manual health check
curl -f http://localhost:8001/health && \
curl -f http://localhost:8002/health && \
curl -f http://localhost:8003/health
```

### Log Monitoring

Application logs are forwarded to Docker's logging system. Configure log aggregation as needed for your infrastructure.

## Troubleshooting

### Container Won't Start

1. Check Docker logs:
```bash
docker logs ckb-mcp-staging
```

2. Verify image exists:
```bash
docker images | grep ckb-mcp
```

3. Check resource usage:
```bash
docker stats
```

### Health Check Failures

1. Check individual server status:
```bash
docker exec ckb-mcp-staging supervisorctl status
```

2. Check server logs:
```bash
docker exec ckb-mcp-staging supervisorctl tail -f ckb-rpc-server
docker exec ckb-mcp-staging supervisorctl tail -f ckb-docs-server
docker exec ckb-mcp-staging supervisorctl tail -f ckb-tools-server
```

### Shepherd Not Updating

1. Check Shepherd configuration:
```bash
docker logs ckb-mcp-shepherd
```

2. Verify container labels:
```bash
docker inspect ckb-mcp-staging | grep shepherd
```

3. Manual image pull test:
```bash
docker pull ghcr.io/sonami-tech/ckb-mcp:dev-latest
```

## Security Considerations

- Container runs with minimal privileges
- No sensitive data in images
- Registry access via GitHub tokens
- Health checks use internal network only
- Supervisor manages process isolation

## File Structure

```
ckb-mcp/
├── Dockerfile                           # Multi-stage build configuration
├── .dockerignore                        # Exclude files from build context
├── docker/
│   ├── supervisord.conf                 # Process manager configuration
│   ├── docker-compose.staging.yml       # Staging environment
│   ├── docker-compose.production.yml    # Production environment
│   └── docker-compose.shepherd.yml      # Auto-updater configuration
└── docker-compose.yml                   # Local development
```