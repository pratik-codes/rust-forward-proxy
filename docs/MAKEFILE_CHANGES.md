# Makefile Restructuring Changes

This document describes the changes made to restructure the Makefile for better organization and intuitive command naming.

## Summary

The Makefile has been restructured to provide cleaner, more intuitive commands:
- **`make dev`** - Local development with HTTPS interception (replaces `dev-local-intercept`)
- **`make dev-docker`** - Docker development environment (replaces `dev`)
- **`make prod`** - Local production server (new)
- **`make prod-docker`** - Docker production environment (replaces `prod`)

## Main Changes

### 1. **Development Commands Restructuring**

#### Before:
```makefile
make dev                  # Started Docker environment
make dev-local           # Local without interception
make dev-local-intercept # Local with interception
```

#### After:
```makefile
make dev                 # Local with HTTPS interception (most common use case)
make dev-docker          # Docker development environment
make dev-docker-detached # Docker development in background
```

### 2. **Production Commands Restructuring**

#### Before:
```makefile
make prod                # Started Docker production
make prod-deploy         # Docker deployment
```

#### After:
```makefile
make prod                # Local production server
make prod-docker         # Docker production environment
make prod-docker-deploy  # Docker production deployment
```

### 3. **Enhanced Logging Integration**

All commands now reflect the new logging level changes:
- **`make dev`**: INFO level by default (CONNECT hidden, HTTP/HTTPS visible)
- **`RUST_LOG=debug make dev`**: DEBUG level (all requests visible)
- **`make prod`**: INFO level with release build

### 4. **Cleaned Up Command Categories**

#### New Help Structure:
```
Main Commands:
  dev                   Start local development with HTTPS interception
  dev-docker            Start development environment with Docker
  prod                  Start local production server
  prod-docker           Start production environment with Docker

Setup & Testing:
  setup                 Initial setup commands
  test                  Test commands
  
Docker Commands:
  docker-*              All Docker-specific operations
  build/pull            Docker build operations

Help & Utilities:
  help-*                Help commands for different topics
  cache-*               Certificate cache operations
  clean                 Cleanup operations
```

## Command Mapping

| Old Command | New Command | Description |
|-------------|-------------|-------------|
| `make dev` | `make dev-docker` | Docker development |
| `make dev-local-intercept` | `make dev` | Local with interception |
| `make dev-local` | *Removed* | Basic local (not needed) |
| `make prod` | `make prod-docker` | Docker production |
| *New* | `make prod` | Local production |
| `make browser-help` | `make help-browser` | Browser setup help |
| `make cache-help` | `make help-cache` | Cache help |
| *New* | `make help-logging` | Logging help |
| `make up/start` | *Removed* | Simplified aliases |

## New Features

### 1. **Integrated Logging Configuration**
```bash
# Production-like logging (CONNECT hidden)
make dev

# Development logging (all requests visible)
RUST_LOG=debug make dev

# Trace logging (very verbose)
RUST_LOG=trace make dev
```

### 2. **Enhanced Command Descriptions**
Each command now shows:
- Log level behavior
- What requests are visible
- Test commands for verification
- Environment configuration

### 3. **Better Help System**
```bash
make help           # Main help
make help-browser   # Browser setup guide
make help-cache     # Certificate caching info
make help-logging   # Logging configuration
```

## Usage Examples

### Development Workflow
```bash
# Start local development (most common)
make dev

# Start with debug logging
RUST_LOG=debug make dev

# Test the proxy
make test
```

### Docker Development
```bash
# Start Docker environment
make dev-docker

# Test Docker proxy
make test-docker

# View Docker logs
make docker-logs
```

### Production Deployment
```bash
# Local production testing
make prod

# Docker production deployment
make prod-docker

# Health check
make health
```

### Certificate Setup
```bash
# Generate CA certificate
make setup-ca

# Get browser setup instructions
make help-browser

# Clear certificate cache
make cache-clear-redis
```

## Removed Commands

The following commands were removed to simplify the interface:
- `dev-local` - Basic local development without interception (not commonly needed)
- `dev-rebuild` - Use `make dev-docker` instead
- `up`, `start` - Aliases that caused confusion
- `stop`, `down`, `restart`, `logs` - Renamed to `docker-*` variants for clarity

## Benefits

1. **Intuitive Naming**: `make dev` does what developers expect
2. **Logging Integration**: Commands reflect the new logging behavior
3. **Clear Separation**: Local vs Docker commands are distinct
4. **Better Organization**: Related commands are grouped logically
5. **Enhanced Help**: Multiple help commands for different topics
6. **Simplified Workflow**: Most common use cases have the shortest commands

## Migration Guide

### For Existing Users

**If you were using:**
- `make dev` → Use `make dev-docker`
- `make dev-local-intercept` → Use `make dev`
- `make prod` → Use `make prod-docker`
- `make browser-help` → Use `make help-browser`

### New Users

Start with:
```bash
make setup      # Initial setup
make dev        # Start development
make test       # Test proxy
```

---

*Last updated: September 24, 2025*
*Makefile restructured for better usability and integration with logging changes*
