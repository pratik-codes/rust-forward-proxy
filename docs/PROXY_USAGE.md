# Rust Forward Proxy - Usage Guide

## 🚀 Quick Start

The Rust Forward Proxy now supports flexible port configurations for both development and production environments.

## 📋 Test Results Summary

Both configurations pass **6 out of 10 tests** with core functionality working:

✅ **Working Features:**
- HTTP proxy functionality
- HTTPS CONNECT tunneling (all external sites working)
- Health endpoints (`/health`)
- Request/response logging
- Performance benchmarking
- Certificate management

⚠️ **In Development:**
- Direct HTTPS proxy (TLS termination)
- Full HTTPS interception  
- Certificate CLI tools

## 🔧 Configuration Modes

### Production Mode (Ports 80/443)
Requires `sudo` for privileged ports:

```bash
# Start production proxy
sudo make dev

# Test production mode
HTTP_PROXY_PORT=80 HTTPS_PROXY_PORT=443 ./scripts/test_https_proxy.sh

# Manual testing
curl -x http://127.0.0.1:80 http://httpbin.org/get
curl -x http://127.0.0.1:80 https://httpbin.org/get -k
```

### Development Mode (Ports 8080/8443)  
No `sudo` required:

```bash
# Test development mode
HTTP_PROXY_PORT=8080 HTTPS_PROXY_PORT=8443 ./scripts/test_https_proxy.sh

# Manual testing  
curl -x http://127.0.0.1:8080 http://httpbin.org/get
curl -x http://127.0.0.1:8080 https://httpbin.org/get -k
```

## 🔧 Environment Variables

The proxy now supports these environment variables to override config.yml:

- `HTTP_PROXY_PORT` - Override HTTP proxy port (default: 80)
- `HTTPS_PROXY_PORT` - Override HTTPS proxy port (default: 443)

## 📁 Configuration Files

### config.yml
Main configuration file with production defaults (80/443)

### Test Scripts
- `scripts/test_https_proxy.sh` - Comprehensive test suite (supports both modes)
- `scripts/test_proxy_both_modes.sh` - Tests both configurations sequentially
- `scripts/test_securly_integration.sh` - Securly certificate integration

## 🧪 Testing Commands

```bash
# Test both modes automatically
./scripts/test_proxy_both_modes.sh

# Test production mode only  
HTTP_PROXY_PORT=80 HTTPS_PROXY_PORT=443 ./scripts/test_https_proxy.sh

# Test development mode only
HTTP_PROXY_PORT=8080 HTTPS_PROXY_PORT=8443 ./scripts/test_https_proxy.sh
```

## 🔗 Browser Configuration

### Development (8080/8443)
Configure your browser proxy settings:
- HTTP Proxy: `127.0.0.1:8080`
- HTTPS Proxy: `127.0.0.1:8080` (uses CONNECT tunneling)

### Production (80/443)
Configure your browser proxy settings:
- HTTP Proxy: `127.0.0.1:80`  
- HTTPS Proxy: `127.0.0.1:80` (uses CONNECT tunneling)

## 📜 Certificate Setup

The proxy uses certificates from:
- `ca-certs/rootCA.crt` - Root CA for signing domain certificates
- `ca-certs/securly_ca.crt` - Securly CA certificate (if available)
- `certs/proxy.crt` - Proxy server certificate
- `certs/proxy.key` - Proxy server private key

## 🔍 Troubleshooting

### Permission Denied (Port 80/443)
```bash
# Use sudo for privileged ports
sudo make dev
```

### TLS Handshake Failures
This is normal when HTTP requests are sent to HTTPS port - expected behavior.

### Environment Variable Not Working
Make sure to rebuild after configuration changes:
```bash
cargo build --release
```

## 📊 Performance

Current benchmarks show ~0.7 requests/second for basic HTTP proxy functionality during testing.

---

**Status**: Core proxy functionality working ✅  
**Next**: Full TLS interception and certificate chain validation 🔧
