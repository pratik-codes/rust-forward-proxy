# ⚙️ Configuration Reference

Complete configuration guide for the Rust Forward Proxy with all environment variables, settings, and deployment options.

## 🎯 Overview

The proxy supports flexible configuration through:
- **🌍 Environment Variables** - Primary configuration method
- **📋 Makefile Commands** - Predefined configurations for common scenarios  
- **🐳 Docker Environment** - Container-specific settings
- **🔧 CLI Arguments** - Command-line overrides

## 🌍 Environment Variables

### **Basic Proxy Configuration**
```bash
# Server Binding
PROXY_LISTEN_ADDR=127.0.0.1:8080        # HTTP proxy address
HTTPS_LISTEN_ADDR=127.0.0.1:8443        # HTTPS proxy address (when TLS enabled)

# Request Handling
PROXY_REQUEST_TIMEOUT=30                 # Request timeout (seconds)
PROXY_MAX_BODY_SIZE=1048576             # Max request body size (bytes, default: 1MB)

# Upstream Configuration
UPSTREAM_URL=http://httpbin.org          # Default upstream for testing
UPSTREAM_CONNECT_TIMEOUT=5               # Upstream connection timeout (seconds)
UPSTREAM_KEEP_ALIVE_TIMEOUT=60          # Keep-alive timeout (seconds)
```

### **TLS & HTTPS Configuration**
```bash
# TLS Server
TLS_ENABLED=false                        # Enable HTTPS proxy server (port 8443)
TLS_INTERCEPTION_ENABLED=true            # Enable HTTPS traffic decryption

# Certificate Paths
TLS_CERT_PATH=certs/proxy.crt           # TLS certificate for proxy server
TLS_KEY_PATH=certs/proxy.key            # TLS private key for proxy server
TLS_AUTO_GENERATE_CERT=true             # Auto-generate cert if missing

# Certificate Authority (for HTTPS interception)
TLS_CA_CERT_PATH=ca-certs/rootCA.crt    # CA certificate for signing domain certs
TLS_CA_KEY_PATH=ca-certs/rootCA.key     # CA private key for signing domain certs

# Certificate Generation
TLS_CERT_ORGANIZATION="Rust Forward Proxy"  # Organization name in certificates
TLS_CERT_COMMON_NAME="proxy.local"      # Common name for generated certificates
TLS_CERT_VALIDITY_DAYS=365              # Certificate validity period

# TLS Security
TLS_MIN_VERSION=1.2                     # Minimum TLS version (1.2 or 1.3)
TLS_SKIP_UPSTREAM_CERT_VERIFY=false     # Skip upstream certificate verification
```

### **Certificate Caching**
```bash
# Cache Configuration
CERTIFICATE_TTL_HOURS=24                # Certificate cache TTL (hours)
MAX_CACHED_CERTIFICATES=1000            # Memory cache size limit
CACHE_KEY_PREFIX="proxy:cert:"          # Redis key prefix for certificates

# Redis Configuration
REDIS_URL=redis://redis:6379            # Redis connection URL
REDIS_PASSWORD=your_secure_password      # Redis authentication
```

### **Logging Configuration**
```bash
# Log Levels
RUST_LOG=info                           # info, debug, trace, warn, error
RUST_LOG=debug                          # Verbose development logging
RUST_LOG=trace                          # Very detailed tracing

# Log Filtering (examples)
RUST_LOG="rust_forward_proxy=debug"     # Debug only this crate
RUST_LOG="debug,hyper=info"             # Debug most, info for hyper
RUST_LOG="warn,rust_forward_proxy=info" # Warn globally, info for proxy
```

## 📋 Makefile Configuration Presets

### **Development Configurations**
```bash
# Basic development (HTTP proxy + HTTPS tunneling)
make dev
# Environment: PROXY_LISTEN_ADDR=127.0.0.1:8080, HTTPS_INTERCEPTION_ENABLED=true

# Development with Securly CA certificates
make dev-securly
# Environment: TLS_CA_CERT_PATH=ca-certs/securly_ca.crt, TLS_CA_KEY_PATH=ca-certs/securly_ca.key

# Development with Docker (Redis caching)
make dev-docker
# Environment: Redis available, certificate caching enabled

# Custom certificate mode
CERT=securly make dev
# Environment: Uses securly CA certificates
```

### **Production Configurations**
```bash
# Local production mode
make prod
# Environment: Release build, INFO logging, optimized settings

# Production with Docker + Redis
make prod-docker  
# Environment: Multi-container, Redis caching, production logging

# Production deployment (pull latest images)
make prod-docker-deploy
# Environment: Latest images, production-ready configuration
```

### **Testing Configurations**
```bash
# Test local proxy
make test-local
# Requires: Proxy running on 127.0.0.1:8080

# Test HTTPS interception
make test-intercept
# Requires: Proxy running with HTTPS interception

# Test Docker deployment
make test-docker
# Requires: Docker containers running

# Test all functionality
make test-all
# Runs: test-local, test-intercept, test-docker, test-redis
```

## 🔧 Configuration Scenarios

### **Scenario 1: Basic HTTP Proxy**
Perfect for HTTP traffic interception and HTTPS tunneling.

```bash
# Environment Variables
export PROXY_LISTEN_ADDR=127.0.0.1:8080
export HTTPS_INTERCEPTION_ENABLED=false
export TLS_ENABLED=false
export RUST_LOG=info

# Start proxy
make dev
```

**What you get:**
- ✅ HTTP request/response interception
- ✅ HTTPS CONNECT tunneling (encrypted passthrough)
- ✅ Clean production logging
- ❌ No HTTPS content visibility

### **Scenario 2: HTTPS Interception (Development)**
Complete HTTPS traffic visibility with certificate generation.

```bash
# Setup certificates
make setup-ca

# Environment Variables  
export PROXY_LISTEN_ADDR=127.0.0.1:8080
export HTTPS_INTERCEPTION_ENABLED=true
export TLS_CA_CERT_PATH=ca-certs/rootCA.crt
export TLS_CA_KEY_PATH=ca-certs/rootCA.key
export RUST_LOG=info

# Start proxy
make dev
```

**What you get:**
- ✅ Complete HTTP interception
- ✅ **HTTPS content decryption and logging**
- ✅ Dynamic certificate generation
- ✅ Certificate caching (memory)

### **Scenario 3: Securly Enterprise Mode**
For environments with Securly CA infrastructure.

```bash
# Place Securly certificates
# securly_ca.crt and securly_ca.key in ca-certs/

# Environment Variables
export CERT=securly
export TLS_CA_CERT_PATH=ca-certs/securly_ca.crt
export TLS_CA_KEY_PATH=ca-certs/securly_ca.key
export RUST_LOG=info

# Start proxy
make dev-securly
```

**What you get:**
- ✅ Securly CA certificate integration
- ✅ Enterprise-compatible certificates
- ⚠️ Fallback to self-signed if private key unavailable

### **Scenario 4: Production Deployment**
High-performance production setup with Redis caching.

```bash
# Docker Environment (.env file)
PROXY_PORT=8080
PROXY_LISTEN_ADDR=0.0.0.0:8080
REDIS_URL=redis://redis:6379
REDIS_PASSWORD=your_secure_password
RUST_LOG=info
TLS_ENABLED=false
HTTPS_INTERCEPTION_ENABLED=true

# Start production deployment
make prod-docker
```

**What you get:**
- ✅ Multi-container deployment
- ✅ Redis certificate caching (shared)
- ✅ Production logging and monitoring
- ✅ High availability and scalability

### **Scenario 5: Full TLS Mode (HTTPS Proxy Server)**
Dedicated HTTPS proxy server with TLS termination.

```bash
# Environment Variables
export TLS_ENABLED=true
export HTTPS_LISTEN_ADDR=127.0.0.1:8443
export TLS_CERT_PATH=certs/proxy.crt
export TLS_KEY_PATH=certs/proxy.key
export TLS_AUTO_GENERATE_CERT=true
export TLS_INTERCEPTION_ENABLED=true

# Start dual HTTP/HTTPS servers
cargo run --bin rust-forward-proxy
```

**What you get:**
- ✅ HTTP proxy (port 8080)
- ✅ HTTPS proxy (port 8443) with TLS termination
- ✅ Certificate generation for proxy server
- ✅ Complete HTTPS interception capability

## 🐳 Docker Configuration

### **Docker Compose Environment**
```yaml
# docker-compose.yml
version: '3.8'
services:
  rust-proxy:
    environment:
      - PROXY_LISTEN_ADDR=0.0.0.0:8080
      - REDIS_URL=redis://redis:6379
      - RUST_LOG=info
      - HTTPS_INTERCEPTION_ENABLED=true
      - TLS_CA_CERT_PATH=ca-certs/rootCA.crt
      - TLS_CA_KEY_PATH=ca-certs/rootCA.key
    volumes:
      - ./ca-certs:/app/ca-certs:ro
      - ./logs:/app/logs
    ports:
      - "8080:8080"
      
  redis:
    image: redis:7-alpine
    environment:
      - REDIS_PASSWORD=${REDIS_PASSWORD}
    volumes:
      - redis_data:/data
```

### **Environment File (.env)**
```bash
# Basic Configuration
PROXY_PORT=8080
PROXY_LISTEN_ADDR=0.0.0.0:8080

# Redis Configuration
REDIS_URL=redis://redis:6379
REDIS_PASSWORD=your_secure_redis_password_here

# Logging
RUST_LOG=info

# TLS Settings
TLS_ENABLED=false
HTTPS_INTERCEPTION_ENABLED=true
TLS_CA_CERT_PATH=ca-certs/rootCA.crt
TLS_CA_KEY_PATH=ca-certs/rootCA.key

# Certificate Generation
TLS_AUTO_GENERATE_CERT=true
TLS_CERT_ORGANIZATION=Rust Forward Proxy
```

### **Production Docker Settings**
```bash
# Production environment variables for Docker
COMPOSE_PROJECT_NAME=rust-proxy-prod
PROXY_LISTEN_ADDR=0.0.0.0:8080
REDIS_URL=redis://redis:6379
REDIS_PASSWORD=$(openssl rand -base64 32)
RUST_LOG=info
HTTPS_INTERCEPTION_ENABLED=true
MAX_CACHED_CERTIFICATES=10000
CERTIFICATE_TTL_HOURS=24
```

## 🎮 CLI Configuration

### **Server Management**
```bash
# Start server with CLI configuration
cargo run --bin rust-forward-proxy-cli server \
  --listen-addr "127.0.0.1:8080" \
  --log-level "info" \
  --enable-https-interception \
  --ca-cert-path "ca-certs/rootCA.crt" \
  --ca-key-path "ca-certs/rootCA.key"

# Start with TLS enabled
cargo run --bin rust-forward-proxy-cli server \
  --listen-addr "127.0.0.1:8080" \
  --enable-tls \
  --https-listen-addr "127.0.0.1:8443" \
  --cert-path "certs/proxy.crt" \
  --key-path "certs/proxy.key" \
  --auto-generate-cert
```

### **Certificate Management**
```bash
# Generate root CA
cargo run --bin rust-forward-proxy-cli cert generate \
  --organization "My Company" \
  --common-name "My Root CA" \
  --cert-path "ca-certs/custom-ca.crt" \
  --key-path "ca-certs/custom-ca.key" \
  --validity-days 3650

# Validate certificates
cargo run --bin rust-forward-proxy-cli cert validate \
  --cert-path "ca-certs/rootCA.crt" \
  --key-path "ca-certs/rootCA.key"
```

## 📊 Performance Configuration

### **Connection Pooling**
```bash
# HTTP client optimization
UPSTREAM_CONNECT_TIMEOUT=5              # Fast connection establishment
UPSTREAM_KEEP_ALIVE_TIMEOUT=60          # Connection reuse
PROXY_REQUEST_TIMEOUT=30                # Request timeout

# Connection limits (in code configuration)
MAX_CONCURRENT_CONNECTIONS=1000         # Concurrent connection limit
CONNECTION_POOL_SIZE=100                # HTTP client pool size
```

### **Memory Management**
```bash
# Request body handling
PROXY_MAX_BODY_SIZE=1048576             # 1MB max body size
BODY_EXTRACT_THRESHOLD=65536            # 64KB extract vs stream threshold

# Certificate caching
MAX_CACHED_CERTIFICATES=1000            # Memory cache limit
CERTIFICATE_TTL_HOURS=24                # Cache TTL
```

### **Redis Optimization**
```bash
# Redis connection settings
REDIS_URL=redis://redis:6379/0          # Database 0
REDIS_POOL_SIZE=10                      # Connection pool size
REDIS_CONNECTION_TIMEOUT=5              # Connection timeout (seconds)
REDIS_COMMAND_TIMEOUT=10                # Command timeout (seconds)
```

## 🔍 Monitoring Configuration

### **Health Check Settings**
```bash
# Health endpoint configuration (always enabled)
GET /health
# Returns: {"status": "healthy", "uptime": "...", ...}

# Health check from load balancer
curl -f http://127.0.0.1:8080/health || exit 1
```

### **Logging Configuration Examples**
```bash
# Production logging (clean, structured)
RUST_LOG=info

# Development logging (verbose)
RUST_LOG=debug

# Trace specific modules
RUST_LOG="info,rust_forward_proxy::proxy=debug"

# Quiet external crates
RUST_LOG="warn,rust_forward_proxy=info"

# File + console logging (future enhancement)
LOG_TO_FILE=true
LOG_FILE_PATH=logs/proxy.log
LOG_FILE_ROTATION=daily
```

## ⚠️ Security Configuration

### **TLS Security Settings**
```bash
# TLS version enforcement
TLS_MIN_VERSION=1.2                     # Minimum TLS 1.2
TLS_CIPHER_SUITES=modern                # Modern cipher suites only

# Certificate validation
TLS_SKIP_UPSTREAM_CERT_VERIFY=false     # Always verify upstream certificates
TLS_CERT_CHAIN_VALIDATION=true          # Validate certificate chains

# Certificate rotation
TLS_CERT_VALIDITY_DAYS=90               # Shorter validity for security
TLS_AUTO_CERT_RENEWAL=true              # Auto-renew certificates
```

### **Access Control**
```bash
# Authentication (future enhancement)
AUTH_ENABLED=false                      # Enable authentication
AUTH_API_KEY=your_api_key_here          # API key authentication
AUTH_JWT_SECRET=your_jwt_secret         # JWT token secret

# Rate limiting (future enhancement)
RATE_LIMIT_ENABLED=false                # Enable rate limiting
RATE_LIMIT_REQUESTS_PER_MINUTE=100      # Requests per minute limit
RATE_LIMIT_BY_IP=true                   # Rate limit by client IP
```

## 🚨 Troubleshooting Configuration

### **Debug Configuration**
```bash
# Maximum verbosity
export RUST_LOG=trace
export HYPER_LOG=trace
export TOKIO_LOG=trace

# Performance debugging
export RUST_LOG="debug,rust_forward_proxy::tls::cache=trace"

# TLS debugging
export RUST_LOG="debug,rustls=debug,tokio_rustls=debug"

# Connection debugging
export RUST_LOG="debug,hyper=debug"
```

### **Development Overrides**
```bash
# Skip certificate verification (testing only)
export TLS_SKIP_UPSTREAM_CERT_VERIFY=true

# Allow self-signed certificates
export TLS_ACCEPT_INVALID_CERTS=true

# Disable certificate caching
export CERTIFICATE_TTL_HOURS=0

# Force certificate regeneration
export TLS_FORCE_CERT_REGENERATION=true
```

### **Error Recovery Configuration**
```bash
# Graceful degradation
FALLBACK_TO_SELF_SIGNED=true           # Use self-signed if CA fails
CONTINUE_WITHOUT_CACHE=true            # Continue if Redis unavailable
RETRY_FAILED_CONNECTIONS=3             # Retry failed upstream connections
```

## 🎯 Configuration Validation

### **Validate Current Configuration**
```bash
# Check environment variables
env | grep -E "(PROXY|TLS|REDIS|RUST_LOG)"

# Test configuration
make test-local

# Validate certificates
cargo run --bin rust-forward-proxy-cli cert validate \
  --cert-path "${TLS_CA_CERT_PATH}" \
  --key-path "${TLS_CA_KEY_PATH}"
```

### **Configuration Templates**

#### **Development Template (.env.dev)**
```bash
PROXY_LISTEN_ADDR=127.0.0.1:8080
HTTPS_INTERCEPTION_ENABLED=true
TLS_CA_CERT_PATH=ca-certs/rootCA.crt
TLS_CA_KEY_PATH=ca-certs/rootCA.key
RUST_LOG=debug
TLS_AUTO_GENERATE_CERT=true
```

#### **Production Template (.env.prod)**
```bash
PROXY_LISTEN_ADDR=0.0.0.0:8080
REDIS_URL=redis://redis:6379
REDIS_PASSWORD=secure_password_here
HTTPS_INTERCEPTION_ENABLED=true
TLS_CA_CERT_PATH=ca-certs/rootCA.crt
TLS_CA_KEY_PATH=ca-certs/rootCA.key
RUST_LOG=info
MAX_CACHED_CERTIFICATES=10000
CERTIFICATE_TTL_HOURS=24
```

---

## 🚀 Summary

The configuration system provides:

### **✅ Flexible Configuration**
- **Environment variables** for all settings
- **Makefile presets** for common scenarios
- **CLI overrides** for testing and development
- **Docker integration** for containerized deployments

### **✅ Security & Performance**
- **TLS configuration** for modern security
- **Certificate management** with multiple CA support
- **Caching optimization** for high performance
- **Connection pooling** for efficiency

### **✅ Operational Excellence**
- **Health monitoring** endpoints
- **Structured logging** with multiple levels
- **Error recovery** and graceful degradation
- **Comprehensive validation** tools

**Ready to configure your proxy? Start with our [Setup Guide](SETUP.md) for step-by-step instructions!**
