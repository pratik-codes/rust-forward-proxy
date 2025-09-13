# Docker Deployment Guide

This guide explains how to deploy the Rust Forward Proxy using Docker Compose with Redis integration.

## Quick Start

1. **Copy the environment template:**
   ```bash
   cp env.example .env
   ```

2. **Edit the `.env` file with your configurations:**
   ```bash
   # Required: Set a secure Redis password
   REDIS_PASSWORD=your_very_secure_password_here
   
   # Optional: Customize other settings
   PROXY_PORT=8080
   REDIS_PORT=6379
   ```

3. **Start the services:**
   ```bash
   docker-compose up -d
   ```

4. **Verify the services are running:**
   ```bash
   docker-compose ps
   docker-compose logs rust-proxy
   docker-compose logs redis
   ```

5. **Test the proxy:**
   ```bash
   curl -x http://localhost:8080 http://httpbin.org/get
   ```

## Environment Configuration

### Redis Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `REDIS_PASSWORD` | (empty) | Redis authentication password |
| `REDIS_USERNAME` | `default` | Redis username (Redis 6+) |
| `REDIS_PORT` | `6379` | Redis port mapping |
| `REDIS_DB` | `0` | Redis database number |

### Proxy Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `PROXY_PORT` | `8080` | Port to expose the proxy service |
| `LOG_LEVEL` | `info` | Logging level (trace, debug, info, warn, error) |
| `REQUEST_TIMEOUT` | `30` | Request timeout in seconds |
| `MAX_BODY_SIZE` | `1048576` | Maximum request body size in bytes |

### Upstream Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `UPSTREAM_URL` | `http://httpbin.org` | Default upstream server |
| `UPSTREAM_CONNECT_TIMEOUT` | `5` | Connection timeout in seconds |
| `UPSTREAM_KEEP_ALIVE_TIMEOUT` | `60` | Keep-alive timeout in seconds |

## Production Deployment

### Security Considerations

1. **Always set a strong Redis password:**
   ```env
   REDIS_PASSWORD=your_very_secure_password_with_special_chars!123
   ```

2. **Use Redis AUTH (Redis 6+ with username):**
   ```env
   REDIS_USERNAME=proxy_user
   REDIS_PASSWORD=secure_password_here
   ```

3. **Limit network exposure:**
   ```yaml
   # In docker-compose.yml, comment out ports for Redis if not needed externally
   # ports:
   #   - "${REDIS_PORT:-6379}:6379"
   ```

### Scaling and Performance

1. **Increase Redis memory limits:**
   ```yaml
   redis:
     deploy:
       resources:
         limits:
           memory: 512M
         reservations:
           memory: 256M
   ```

2. **Configure Redis persistence:**
   The Redis container is configured with both RDB and AOF persistence.

### Monitoring and Logging

1. **View logs:**
   ```bash
   # All services
   docker-compose logs -f
   
   # Specific service
   docker-compose logs -f rust-proxy
   docker-compose logs -f redis
   ```

2. **Monitor Redis:**
   ```bash
   # Connect to Redis CLI
   docker-compose exec redis redis-cli
   
   # With authentication
   docker-compose exec redis redis-cli -a your_password
   ```

## Swapping Redis Credentials

To swap Redis credentials for different environments:

### Method 1: Environment Files

1. Create environment-specific files:
   ```bash
   # Development
   cp env.example .env.dev
   
   # Staging
   cp env.example .env.staging
   
   # Production
   cp env.example .env.prod
   ```

2. Edit each file with appropriate credentials:
   ```bash
   # .env.prod
   REDIS_PASSWORD=production_secure_password
   REDIS_USERNAME=prod_user
   ```

3. Deploy with specific environment:
   ```bash
   # Copy the desired environment
   cp .env.prod .env
   docker-compose up -d
   ```

### Method 2: Direct Override

```bash
# Set credentials directly
export REDIS_PASSWORD=new_password
export REDIS_USERNAME=new_user
docker-compose up -d
```

### Method 3: External Redis

To connect to an external Redis instance:

```env
# In .env file
REDIS_URL=redis://username:password@external-redis-host:6379/0
```

Then modify `docker-compose.yml` to remove the Redis service and update the proxy's `REDIS_URL` environment variable.

## Troubleshooting

### Common Issues

1. **Connection refused to Redis:**
   ```bash
   # Check if Redis is running
   docker-compose ps redis
   
   # Check Redis logs
   docker-compose logs redis
   ```

2. **Authentication errors:**
   - Verify `REDIS_PASSWORD` is set correctly
   - Check Redis logs for authentication attempts

3. **Permission errors:**
   ```bash
   # Fix volume permissions
   sudo chown -R 999:999 ./logs
   ```

### Health Checks

Both services include health checks:

```bash
# Check service health
docker-compose ps

# Manual health check
curl -f http://localhost:8080/health
```

## Backup and Recovery

### Redis Data Backup

```bash
# Create backup
docker-compose exec redis redis-cli BGSAVE

# Copy backup file
docker cp rust-proxy-redis:/data/dump.rdb ./backup/

# Restore from backup
docker cp ./backup/dump.rdb rust-proxy-redis:/data/
docker-compose restart redis
```

## Development

For development with live reloading:

```bash
# Use override file
docker-compose -f docker-compose.yml -f docker-compose.dev.yml up
```

Create `docker-compose.dev.yml`:
```yaml
version: '3.8'
services:
  rust-proxy:
    volumes:
      - .:/app
    environment:
      - RUST_LOG=debug
```
