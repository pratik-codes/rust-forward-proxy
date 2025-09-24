# 🚀 Rust Forward Proxy

[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()

A **high-performance HTTP/HTTPS forward proxy server** written in Rust with advanced TLS interception, certificate management, and comprehensive logging capabilities.

## ✨ Features

### 🔒 **Complete HTTPS Interception**
- **TLS Termination & Re-encryption** - Full decrypt/inspect/re-encrypt capability
- **Certificate Generation** - Automatic domain certificate creation with CA signing
- **Certificate Caching** - Memory & Redis backends for 25-30x performance improvement
- **Multiple Certificate Modes** - Support for rootCA and Securly CA certificates

### 🌐 **Full Proxy Capabilities**
- **HTTP Request Interception** - Complete request/response logging and modification
- **HTTPS CONNECT Tunneling** - Standards-compliant tunnel for encrypted traffic
- **Dual Server Mode** - Simultaneous HTTP (8080) and HTTPS (8443) operation
- **Production Logging** - Clean INFO level for production, detailed DEBUG for development

### ⚡ **High Performance**
- **Async Architecture** - Built on Tokio/Hyper for maximum throughput
- **Connection Pooling** - Efficient upstream connection management
- **Smart Body Handling** - Optimized request/response body processing
- **Certificate Caching** - Sub-millisecond certificate retrieval

### 🔧 **Developer Experience**
- **Comprehensive CLI Tools** - Certificate generation, validation, and server management
- **Flexible Configuration** - Environment variables + configuration files
- **Docker Support** - Production-ready containerization with Redis
- **Extensive Documentation** - Complete guides for setup, deployment, and usage

## 🚀 Quick Start

### **Simple HTTP Proxy**
```bash
# Start basic HTTP proxy
make dev

# Test HTTP request
curl -x http://127.0.0.1:8080 http://httpbin.org/get

# Test HTTPS tunneling
curl -x http://127.0.0.1:8080 https://httpbin.org/get
```

### **HTTPS Interception (See Encrypted Content)**
```bash
# Setup root CA certificate for browser
make setup-ca

# Start HTTPS interception proxy
make dev

# Configure browser proxy: 127.0.0.1:8080
# Install rootCA.crt in browser (see BROWSER_SETUP.md)
# Browse to https://httpbin.org/get
# Check proxy logs - you'll see complete HTTPS content!
```

### **Production Deployment**
```bash
# Production with Docker + Redis caching
make prod-docker

# Local production mode
make prod
```

## 📁 Project Structure

```
rust-forward-proxy/
├── 📦 src/                         # Core implementation
│   ├── 🌐 proxy/                   # HTTP/HTTPS proxy logic
│   │   ├── server.rs              # Main server implementation
│   │   ├── http_client.rs         # Optimized upstream client
│   │   └── streaming.rs           # Smart body handling
│   ├── 🔒 tls/                     # TLS & certificate management
│   │   ├── server.rs              # HTTPS termination server
│   │   ├── cert_gen.rs            # Certificate generation
│   │   ├── cache.rs               # Certificate caching (Memory/Redis)
│   │   └── config.rs              # TLS configuration
│   ├── ⚙️ config/                  # Configuration management
│   ├── 📋 logging/                 # Production-grade logging
│   ├── 🛠️ utils/                   # HTTP/URL/Time utilities
│   ├── 🎮 cli/                     # Command-line interface
│   └── 📊 models/                  # Data structures
├── 📚 docs/                        # Comprehensive documentation
├── 🐳 docker-compose.yml           # Docker deployment
├── 📋 Makefile                     # Development commands
└── 🧪 scripts/                     # Testing & setup scripts
```

## 🎯 How It Works

### **HTTP Request Flow**
```
Client → [HTTP Proxy:8080] → [Full Interception] → [Log Everything] → Upstream
   ↖                                                                      ↙
    ↖                          [Response Logging]                       ↙
     ↖                                                                ↙
      ↖─────────────────── Clean Response ──────────────────────────↙
```

### **HTTPS Interception Flow**
```
Client → [HTTPS Proxy:8443] → [TLS Terminate] → [Decrypt] → [Log Content] → [Re-encrypt] → Upstream
   ↖                                                                                           ↙
    ↖                                    [Certificate Cache]                                 ↙
     ↖                                                                                     ↙
      ↖─────────────────────────── Encrypted Response ────────────────────────────────↙
```

### **Certificate Generation Flow**
```
Request for domain.com
        ↓
[Cache Check] → Hit: Return cached cert (0ms)
        ↓
     Miss: Generate new cert (5-10ms)
        ↓
[Sign with CA] → Cache for 24h → Return cert
```

## 🔧 Configuration

### **Certificate Modes**
```bash
# Default mode (uses rootCA)
make dev

# Securly CA mode
CERT=securly make dev
# or
make dev-securly
```

### **Environment Variables**
```bash
# Proxy Configuration
PROXY_LISTEN_ADDR=127.0.0.1:8080
HTTPS_LISTEN_ADDR=127.0.0.1:8443

# TLS Configuration
TLS_ENABLED=true
TLS_INTERCEPTION_ENABLED=true
TLS_CA_CERT_PATH=ca-certs/rootCA.crt
TLS_CA_KEY_PATH=ca-certs/rootCA.key

# Logging
RUST_LOG=info                    # Clean production logs
RUST_LOG=debug                   # Verbose development logs

# Redis (for certificate caching)
REDIS_URL=redis://redis:6379
```

## 📖 Documentation

### **🚀 Getting Started**
- **[Quick Setup Guide](docs/SETUP.md)** - Get running in 5 minutes
- **[Browser Configuration](docs/BROWSER_SETUP.md)** - Setup HTTPS interception
- **[Certificate Management](docs/CERTIFICATES.md)** - Complete certificate guide

### **🏗️ Architecture & Implementation**
- **[Architecture Overview](docs/ARCHITECTURE.md)** - System design and flow diagrams
- **[TLS Implementation](docs/CERTIFICATES.md)** - HTTPS termination and certificate handling
- **[Performance Optimization](docs/PERFORMANCE.md)** - Caching, pooling, and benchmarks

### **🚀 Deployment & Operations**
- **[Deployment Guide](docs/DEPLOYMENT.md)** - Docker, Kubernetes, cloud deployment
- **[Configuration Reference](docs/CONFIGURATION.md)** - Complete config documentation
- **[CLI Reference](docs/CONFIGURATION.md)** - CLI commands and configuration options

## 🧪 Testing

```bash
# Test basic functionality
make test

# Test HTTPS interception
make test-intercept

# Test Docker deployment
make test-docker

# Run all tests
make test-all
```

## 🎯 Use Cases

### **🔍 Development & Debugging**
- **API Development** - See exactly what your applications send/receive
- **Security Testing** - Analyze encrypted traffic for vulnerabilities
- **Network Debugging** - Troubleshoot mysterious network issues

### **🛡️ Security & Monitoring**
- **Traffic Analysis** - Monitor and log all HTTP/HTTPS traffic
- **Content Filtering** - Inspect and potentially modify requests/responses
- **Compliance Auditing** - Log all network communications

### **⚡ Performance Testing**
- **Load Testing** - Proxy traffic for performance analysis
- **Caching Analysis** - Understand application caching behavior
- **Bandwidth Monitoring** - Track data usage and patterns

## 🚀 Performance

### **Benchmarks**
- **HTTP Throughput**: 1000+ requests/second
- **HTTPS Latency**: +2-5ms overhead for interception
- **Certificate Generation**: 5-10ms first request, <1ms cached
- **Memory Usage**: ~10-50MB depending on load
- **Concurrent Connections**: 1000+ simultaneous HTTPS sessions

### **Certificate Caching Performance**
```
Without Caching: 25-30ms per HTTPS request
With Caching:     <1ms per HTTPS request
Performance Gain: 25-30x improvement
```

## 🤝 Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) and [Tokio](https://tokio.rs/)
- Uses [Hyper](https://hyper.rs/) for HTTP implementation
- TLS powered by [rustls](https://github.com/rustls/rustls)
- Certificate generation via [rcgen](https://github.com/est31/rcgen)

---

## 🎉 Ready to Start?

```bash
# Clone and run
git clone <your-repo-url>
cd rust-forward-proxy
make dev

# Start intercepting HTTP traffic in seconds!
curl -x http://127.0.0.1:8080 http://httpbin.org/get
```

**🔥 For HTTPS interception, see our [Browser Setup Guide](docs/BROWSER_SETUP.md) to configure certificate trust!**