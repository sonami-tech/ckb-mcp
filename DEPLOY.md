# CKB MCP Deployment Guide

## Description

Deployment documentation for the CKB MCP server using Docker containers. Covers staging and production environments with automated updates via Watchtower.

## Overview

This project uses Docker containers for deployment with automatic updates. The unified MCP server runs in a single container.

- **Staging Environment**: Auto-deploys from `develop` branch
- **Production Environment**: Auto-deploys from tagged releases
- **Container Registry**: GitHub Container Registry (ghcr.io)
- **Auto-Updates**: Watchtower for Docker Compose container updates

## Docker File Structure

This project includes multiple Docker configuration files for different environments:

### Core Docker Files

- **`Dockerfile`**: Multi-stage build configuration
  - Builder stage: Compiles Rust binary with dependencies
  - Runtime stage: Debian bookworm-slim with the unified server
  - Includes health check for the server

- **`.dockerignore`**: Excludes unnecessary files from build context
  - Skips target/, .git/, node_modules/, etc.
  - Reduces build time and image size

### Docker Compose Files

- **`docker-compose.yml`**: Local development environment
  - Builds from local Dockerfile
  - Uses `RUST_LOG=debug` for detailed logging

- **`docker/docker-compose.staging.yml`**: Staging deployment
  - Uses `ghcr.io/sonami-tech/ckb-mcp:dev-latest` image
  - Auto-deploys from `develop` branch pushes
  - Includes Watchtower labels for auto-updates
  - Uses `RUST_LOG=info` for production-like logging

- **`docker/docker-compose.production.yml`**: Production deployment
  - Uses `ghcr.io/sonami-tech/ckb-mcp:latest` image
  - Deploys from version tags (v1.0.0, etc.)
  - Production-grade configuration and monitoring

- **`docker/docker-compose.watchtower.yml`**: Auto-updater service
  - Monitors Docker images for updates every 5 minutes
  - Automatically pulls and deploys new images
  - Works with both staging and production containers

### Environment Configuration

All compose files support these environment variables:
- **`CKB_RPC_URL`**: CKB node RPC endpoint (default: http://127.0.0.1:8114)
- **`RUST_LOG`**: Logging level (debug/info/warn/error)
- **`TZ`**: Timezone (set to America/Los_Angeles for US West)

Override via `.env` files or environment variables:
```bash
# Example .env file
CKB_RPC_URL=http://your-ckb-node:8114
RUST_LOG=debug
```

## Prerequisites

- Docker and Docker Compose installed
- Access to deploy servers (staging/production)
- GitHub Container Registry access

## Docker Images

Images are automatically built and published to:
- **Development**: `ghcr.io/sonami-tech/ckb-mcp:dev-latest` (from `develop` branch)
- **Production**: `ghcr.io/sonami-tech/ckb-mcp:latest` (from version tags)

## Server Architecture

Single container running the unified MCP server:
- **Port 3112**: Unified MCP Server (RPC, Docs, Tools, Prompts)

Health check monitors the `/health` endpoint.

## Local Development

Build and run locally:

```bash
# Build Docker image
docker build -t ckb-mcp:local .

# Run with Docker Compose
docker compose up -d

# Check health
curl http://localhost:3112/health
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

3. Start Watchtower auto-updater:
```bash
docker compose -f docker/docker-compose.watchtower.yml up -d
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

# Check health endpoint
curl http://staging-server:3112/health

# Check Watchtower logs
docker logs ckb-mcp-watchtower
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

3. Start Watchtower auto-updater:
```bash
docker compose -f docker/docker-compose.watchtower.yml up -d
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

# Check health endpoint
curl http://production-server:3112/health
```

## Automatic Updates

Watchtower monitors for new Docker images and automatically updates containers:

- **Check Interval**: Every 5 minutes
- **Update Trigger**: New image tags in registry
- **Cleanup**: Automatically removes old images
- **Zero Downtime**: Rolling updates preserve service availability

### Watchtower Configuration

Watchtower only updates containers with the `com.centurylinklabs.watchtower.enable=true` label, which is set in both staging and production compose files.

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

# Watchtower logs
docker logs ckb-mcp-watchtower -f
```

### Stop Services

```bash
# Stop application
docker compose -f docker/docker-compose.staging.yml down

# Stop Watchtower
docker compose -f docker/docker-compose.watchtower.yml down
```

## Release Process

### Development Release

1. Push changes to `develop` branch
2. GitHub Actions builds and pushes `dev-latest` image
3. Watchtower detects new image and updates staging server

### Production Release

1. Create and push version tag:
```bash
git tag v1.0.0
git push origin v1.0.0
```

2. GitHub Actions builds and pushes `latest` image
3. Watchtower detects new image and updates production server

## Monitoring

### Health Checks

Container includes health check that verifies the server is responding:

```bash
# Manual health check
curl -f http://localhost:3112/health
```

### Log Monitoring

Application logs are forwarded to Docker's logging system. Configure log aggregation as needed for your infrastructure.

## Monitoring Auto-Updates

### Understanding Watchtower Behavior

Watchtower provides clear logging about its update activities:

**Normal Operation (No Updates Available):**
```
time="2025-08-19T21:45:27-07:00" level=info msg="Checking containers for updated images"
time="2025-08-19T21:45:28-07:00" level=debug msg="Pulling image for container /ckb-mcp-staging"
time="2025-08-19T21:45:30-07:00" level=info msg="Container /ckb-mcp-staging is up to date"
```

**Translation:**
- ✅ Watchtower is checking for updates
- ✅ Debug logging shows registry pulls and comparisons
- ✅ "up to date" means no update needed

**When an Update Happens:**
```
time="2025-08-19T22:05:27-07:00" level=info msg="Found new image for container /ckb-mcp-staging"
time="2025-08-19T22:05:30-07:00" level=info msg="Stopping container /ckb-mcp-staging"
time="2025-08-19T22:05:35-07:00" level=info msg="Creating new container /ckb-mcp-staging"
time="2025-08-19T22:05:40-07:00" level=info msg="Starting container /ckb-mcp-staging"
```

### Verifying Container Currency

Check if your container is running the latest available image:

```bash
# Pull latest to ensure local registry is current
docker pull ghcr.io/sonami-tech/ckb-mcp:dev-latest

# Compare running container vs available image
echo "Running: $(docker inspect ckb-mcp-staging --format='{{.Image}}' | cut -c8-19)"
echo "Available: $(docker images ghcr.io/sonami-tech/ckb-mcp:dev-latest --format='{{.ID}}')"

# If IDs match, container is current
# If different, update is pending or needed
```

### Testing the Update Pipeline

To verify Watchtower is working:

1. **Make a small change** and push to develop branch
2. **Wait 5-10 minutes** for GitHub Actions to build new image
3. **Watch Watchtower logs** for update activity:
   ```bash
   docker logs -f ckb-mcp-watchtower
   ```
4. **Verify container restart** with `docker ps` (look for recent "Up X minutes")

### Troubleshooting Watchtower

**Watchtower Not Updating:**
```bash
# Check if Watchtower can see labeled containers
docker exec ckb-mcp-watchtower docker ps --filter "label=com.centurylinklabs.watchtower.enable=true"

# Should show your staging container
```

**Force Watchtower to Check:**
```bash
# Restart Watchtower to trigger immediate check
docker restart ckb-mcp-watchtower
docker logs -f ckb-mcp-watchtower
```

**Manual Update (if needed):**
```bash
# Pull and restart manually
docker compose -f docker/docker-compose.staging.yml pull
docker compose -f docker/docker-compose.staging.yml up -d
```

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

1. Check server status:
```bash
curl -v http://localhost:3112/health
```

2. Check server logs:
```bash
docker logs ckb-mcp-staging
```

### Watchtower Not Updating

1. Check Watchtower configuration:
```bash
docker logs ckb-mcp-watchtower
```

2. Verify container labels:
```bash
docker inspect ckb-mcp-staging | grep watchtower
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

## File Structure

```
ckb-mcp/
├── Dockerfile                           # Multi-stage build configuration
├── .dockerignore                        # Exclude files from build context
├── docker/
│   ├── docker-compose.staging.yml       # Staging environment
│   ├── docker-compose.production.yml    # Production environment
│   └── docker-compose.watchtower.yml    # Auto-updater configuration
└── docker-compose.yml                   # Local development
```
