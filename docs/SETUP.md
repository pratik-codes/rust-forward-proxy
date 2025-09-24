# 🚀 Complete Setup Guide

Get the Rust Forward Proxy running in minutes with full HTTP/HTTPS interception capabilities.

## ⚡ Quick Start (2 minutes)

### **1. Install & Run**
```bash
# Clone and start basic HTTP proxy
git clone <your-repo-url>
cd rust-forward-proxy
make dev

# Test immediately
curl -x http://127.0.0.1:8080 http://httpbin.org/get
```

### **2. Enable HTTPS Interception**
```bash
# Generate root CA certificate
make setup-ca

# Configure browser proxy: 127.0.0.1:8080
# Install ca-certs/rootCA.crt in browser trust store
# Browse to https://httpbin.org/get
# Check proxy logs - complete HTTPS content visible!
```

## 🎯 Setup Options

### **Option 1: Basic HTTP Proxy** ⚡
Perfect for HTTP traffic interception and HTTPS tunneling.

```bash
make dev
```

**What you get:**
- ✅ Full HTTP request/response interception
- ✅ HTTPS CONNECT tunneling (encrypted passthrough)
- ✅ Production-grade logging
- ✅ Health monitoring

**Test it:**
```bash
# HTTP interception (see full content)
curl -x http://127.0.0.1:8080 http://httpbin.org/get

# HTTPS tunneling (encrypted passthrough)
curl -x http://127.0.0.1:8080 https://httpbin.org/get
```

### **Option 2: HTTPS Interception** 🔍
Decrypt and inspect HTTPS traffic content.

```bash
# Setup root certificate
make setup-ca

# Start proxy with interception
make dev

# Configure browser (see Browser Setup section)
```

**What you get:**
- ✅ Everything from Option 1
- ✅ **HTTPS content interception** - see encrypted data!
- ✅ Certificate generation and caching
- ✅ Complete request/response visibility

### **Option 3: Securly CA Mode** 🏢
Use Securly CA certificates for enterprise environments.

```bash
# Place Securly CA files in ca-certs/
# securly_ca.crt and securly_ca.key (if available)

# Start with Securly CA
CERT=securly make dev
# or
make dev-securly
```

### **Option 4: Production Deployment** 🚀
Redis caching, Docker containerization, production logging.

```bash
# Docker with Redis
make prod-docker

# Local production
make prod
```

## 🌐 Browser Setup for HTTPS Interception

### **Step 1: Generate Root CA**
```bash
make setup-ca
```

This creates:
- `ca-certs/rootCA.crt` - Root certificate (install in browser)
- `ca-certs/rootCA.key` - Private key (keep secure)

### **Step 2: Install Certificate in Browser**

#### **Chrome/Edge (macOS)**
1. Open the certificate: `open ca-certs/rootCA.crt`
2. macOS Keychain will open
3. Add certificate to keychain
4. Find "Rust Proxy Root CA" in Keychain Access
5. Double-click → Expand "Trust" section
6. Set "When using this certificate" to **"Always Trust"**
7. Save changes

#### **Chrome/Edge (Windows/Linux)**
1. Chrome → Settings → Privacy and Security → Security → Manage certificates
2. Go to "Trusted Root Certification Authorities" tab
3. Click "Import" → Select `ca-certs/rootCA.crt`
4. Place in "Trusted Root Certification Authorities" store
5. Restart browser

#### **Firefox**
1. Firefox → Settings → Privacy & Security → Certificates → View Certificates
2. Go to "Authorities" tab
3. Click "Import" → Select `ca-certs/rootCA.crt`
4. Check "Trust this CA to identify websites"
5. OK and restart Firefox

### **Step 3: Configure Browser Proxy**

#### **Chrome/Edge**
1. Settings → Advanced → System → "Open your computer's proxy settings"
2. Manual proxy setup:
   - **HTTP Proxy:** `127.0.0.1:8080`
   - **HTTPS Proxy:** `127.0.0.1:8080`
   - Check "Use proxy for all protocols"

#### **Firefox**
1. Settings → Network Settings → "Settings" button
2. Select "Manual proxy configuration"
3. **HTTP Proxy:** `127.0.0.1` **Port:** `8080`
4. **SSL Proxy:** `127.0.0.1` **Port:** `8080`
5. Check "Use this proxy server for all protocols"

### **Step 4: Test HTTPS Interception**
1. Browse to https://httpbin.org/get
2. Check proxy console logs
3. You should see **complete HTTPS request/response content**!

```bash
✅ INTERCEPTED HTTPS: GET https://httpbin.org/get
📋 Request Headers:
  user-agent: Mozilla/5.0...
  cookie: session=abc123...
📤 Upstream HTTPS response: 200 OK  
📄 Response Body: {"args": {}, "headers": {...}}
```

## 🔧 Configuration Options

### **Environment Variables**
```bash
# Basic Configuration
PROXY_LISTEN_ADDR=127.0.0.1:8080    # HTTP proxy address
HTTPS_LISTEN_ADDR=127.0.0.1:8443    # HTTPS proxy address (when TLS enabled)

# TLS Configuration  
TLS_ENABLED=false                    # Enable HTTPS server
TLS_INTERCEPTION_ENABLED=true       # Decrypt HTTPS traffic
TLS_CA_CERT_PATH=ca-certs/rootCA.crt
TLS_CA_KEY_PATH=ca-certs/rootCA.key

# Logging
RUST_LOG=info                        # Production: clean logs
RUST_LOG=debug                       # Development: verbose logs

# Redis (Certificate Caching)
REDIS_URL=redis://redis:6379
```

### **Certificate Modes**
```bash
# Default: rootCA certificates
make dev

# Securly CA certificates  
CERT=securly make dev

# Custom certificate paths
TLS_CA_CERT_PATH=/path/to/ca.crt TLS_CA_KEY_PATH=/path/to/ca.key make dev
```

### **Logging Levels**
```bash
# Clean production logs (default)
make dev

# Verbose development logs
RUST_LOG=debug make dev

# Trace level (very verbose)
RUST_LOG=trace make dev
```

## 🐳 Docker Setup

### **Development with Docker**
```bash
# Start with Redis caching
make dev-docker

# Background mode
make dev-docker-detached
```

### **Production with Docker**
```bash
# Production deployment
make prod-docker

# Custom environment
cp env.example .env
# Edit .env with your settings
make prod-docker
```

### **Docker Environment Variables**
```bash
# In .env file
PROXY_PORT=8080
PROXY_LISTEN_ADDR=0.0.0.0:8080
REDIS_URL=redis://redis:6379
REDIS_PASSWORD=your_secure_password
RUST_LOG=info
```

## 🧪 Testing Your Setup

### **Basic Functionality Tests**
```bash
# Test HTTP interception
curl -x http://127.0.0.1:8080 http://httpbin.org/get

# Test HTTPS tunneling
curl -x http://127.0.0.1:8080 https://httpbin.org/get

# Test POST with data
curl -x http://127.0.0.1:8080 -X POST http://httpbin.org/post \
  -H "Content-Type: application/json" \
  -d '{"test": "data"}'
```

### **HTTPS Interception Tests**
```bash
# Test HTTPS interception (requires browser setup)
curl -x http://127.0.0.1:8080 https://httpbin.org/get --proxy-insecure

# Test with various HTTPS sites
curl -x http://127.0.0.1:8080 https://api.github.com/users/octocat --proxy-insecure
```

### **Automated Test Scripts**
```bash
# Run all tests
make test-all

# Test local proxy
make test-local

# Test HTTPS interception
make test-intercept

# Test Docker deployment
make test-docker
```

## 🎯 Expected Results

### **HTTP Requests (Always Works)**
```
📥 GET http://httpbin.org/get from 127.0.0.1
🔄 Forwarding GET to upstream
📤 Upstream response: 200 (156ms)
✅ GET /get → 200 OK (158ms)
```

### **HTTPS Tunneling (Default)**
```
🔐 CONNECT tunnel to httpbin.org:443
✅ Tunnel established to httpbin.org:443 (45ms)
🔌 Tunnel completed for httpbin.org:443
```

### **HTTPS Interception (After Browser Setup)**
```
🔍 CONNECT httpbin.org:443 - INTERCEPTING
💾 Generating new certificate for httpbin.org (5ms)
🔒 Connection upgraded, starting TLS handshake
✅ TLS handshake successful
📥 INTERCEPTED HTTPS: GET https://httpbin.org/get
📋 Complete request/response details logged
```

## ⚠️ Troubleshooting

### **Proxy Not Working**
```bash
# Check if proxy is running
curl http://127.0.0.1:8080/health

# Check logs for errors
RUST_LOG=debug make dev
```

### **HTTPS Interception Issues**
1. **Certificate not trusted**: Install rootCA.crt in browser trust store
2. **Browser warnings**: Normal for some sites with certificate pinning
3. **Connection errors**: Restart browser after certificate installation

### **Performance Issues**
```bash
# Check Redis connection (for Docker)
make test-redis

# Monitor certificate cache performance
# Look for "Using cached certificate" vs "Generating new certificate"

# Clear certificate cache if needed
make cache-clear-redis
```

### **Remove Certificates**
```bash
# Remove from macOS Keychain
# Keychain Access → Find "Rust Proxy Root CA" → Delete

# Remove from browser
# Chrome: Settings → Security → Manage certificates → Delete
# Firefox: Settings → Certificates → Authorities → Delete
```

## 🚀 Next Steps

### **Development Use Cases**
- **API Testing**: Intercept your application's HTTP/HTTPS requests
- **Security Analysis**: Examine encrypted traffic for vulnerabilities  
- **Network Debugging**: Troubleshoot mysterious connection issues

### **Advanced Configuration**
- **[Certificate Management](CERTIFICATES.md)** - Custom CAs, certificate validation
- **[Performance Tuning](PERFORMANCE.md)** - Caching, connection pooling, optimization
- **[Deployment Guide](DEPLOYMENT.md)** - Production deployment patterns

### **Integration Examples**
- **CI/CD Testing**: Automated API testing with traffic interception
- **Load Testing**: Monitor application behavior under load
- **Security Auditing**: Log and analyze all network communications

---

## 🎉 Success!

You now have a powerful HTTP/HTTPS proxy that can:
- ✅ **Intercept all HTTP traffic** with complete visibility
- ✅ **Decrypt HTTPS traffic** for security analysis
- ✅ **Generate certificates dynamically** for any domain
- ✅ **Cache certificates** for high performance
- ✅ **Scale to production** with Docker and Redis

**Ready to explore encrypted traffic? Check out our [Browser Setup Guide](BROWSER_SETUP.md) for complete HTTPS interception!**
