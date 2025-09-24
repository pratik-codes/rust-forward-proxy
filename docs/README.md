# 📚 Rust Forward Proxy Documentation

Comprehensive documentation for the high-performance HTTP/HTTPS forward proxy with TLS interception capabilities.

## 🚀 Quick Start

**New to the project?** Start here:

1. **[Main README](../README.md)** - Project overview and quick start
2. **[Setup Guide](SETUP.md)** - Complete setup in 5 minutes
3. **[Browser Setup](BROWSER_SETUP.md)** - Configure HTTPS interception

## 📖 Documentation Structure

### **🚀 Getting Started**
- **[Setup Guide](SETUP.md)** - Complete installation and configuration guide
  - Basic HTTP proxy setup
  - HTTPS interception with certificate generation
  - Browser configuration for certificate trust
  - Docker deployment options

- **[Browser Setup](BROWSER_SETUP.md)** - Detailed browser configuration
  - Certificate installation for Chrome, Firefox, Edge, Safari
  - Proxy configuration steps
  - Troubleshooting browser issues

### **🏗️ Architecture & Implementation**
- **[Architecture Overview](architecture.md)** - System design and implementation
  - Project structure and file organization
  - Data flow diagrams for HTTP/HTTPS traffic
  - Core components and their responsibilities
  - Performance characteristics and optimization

### **🔒 Certificate Management**
- **[Certificate System](CERTIFICATES.md)** - Complete certificate management guide
  - Certificate generation and CA signing
  - Multiple certificate modes (rootCA, Securly)
  - High-performance caching system (Memory/Redis)
  - CLI tools for certificate management
  - Browser integration and troubleshooting

### **⚙️ Configuration & Deployment**
- **[Configuration Reference](CONFIGURATION.md)** - Complete configuration guide
  - Environment variables reference
  - Makefile commands and presets
  - Docker and production settings
  - Performance tuning and security options

- **[Deployment Guide](deployment.md)** - Production deployment instructions
  - Local development deployment
  - Docker containerization
  - Cloud deployment (AWS, GCP)
  - Kubernetes configuration
  - CI/CD integration

### **🛠️ Advanced Topics**
- **[Middleware Documentation](middleware.md)** - Extensible middleware system
  - Authentication middleware examples
  - Rate limiting implementation
  - Custom middleware development

- **[Upstream Management](upstream.md)** - Backend connection handling
  - Connection pooling and optimization
  - Health checking and failover
  - Load balancing strategies

- **[Performance Guide](performance.md)** - Optimization and benchmarking
  - Performance characteristics and benchmarks
  - Optimization techniques
  - Monitoring and metrics

## 🎯 Use Case Guides

### **🔍 Development & Testing**
```bash
# Quick HTTP proxy for API development
make dev

# HTTPS interception for security testing
make setup-ca && make dev
# Configure browser and browse to https://httpbin.org/get
```

### **🏢 Enterprise Integration**
```bash
# Securly CA mode for enterprise environments
CERT=securly make dev

# Production deployment with Redis caching
make prod-docker
```

### **🚀 Production Deployment**
```bash
# High-performance production setup
make prod-docker-deploy

# Custom configuration
cp env.example .env
# Edit .env with your settings
make prod-docker
```

## 📊 Feature Matrix

| Feature | Basic HTTP | HTTPS Tunneling | HTTPS Interception | Production |
|---------|------------|-----------------|-------------------|------------|
| **HTTP Request Logging** | ✅ | ✅ | ✅ | ✅ |
| **HTTPS CONNECT Tunneling** | ❌ | ✅ | ✅ | ✅ |
| **HTTPS Content Visibility** | ❌ | ❌ | ✅ | ✅ |
| **Certificate Generation** | ❌ | ❌ | ✅ | ✅ |
| **Certificate Caching** | ❌ | ❌ | Memory | Redis |
| **Browser Trust** | N/A | N/A | Manual | Automated |
| **Performance** | High | High | Very High | Ultra High |
| **Use Case** | Development | Testing | Security Analysis | Production |

## 🔧 Common Tasks

### **Certificate Setup**
```bash
# Generate root CA for browser installation
make setup-ca

# Validate existing certificates  
cargo run --bin rust-forward-proxy-cli cert validate \
  --cert-path ca-certs/rootCA.crt \
  --key-path ca-certs/rootCA.key

# Generate custom certificates
cargo run --bin rust-forward-proxy-cli cert generate \
  --organization "My Company" \
  --common-name "My Proxy" \
  --cert-path custom.crt \
  --key-path custom.key
```

### **Testing & Validation**
```bash
# Test HTTP proxy functionality
make test-local

# Test HTTPS interception
make test-intercept

# Test complete deployment
make test-all

# Performance testing
curl -x http://127.0.0.1:8080 -w "@curl-format.txt" \
  https://httpbin.org/get
```

### **Monitoring & Debugging**
```bash
# Monitor certificate cache performance
grep -E "(cached certificate|Generating new)" logs/proxy.log

# Debug TLS handshake issues
RUST_LOG=debug,rustls=debug make dev

# Check Redis certificate cache
redis-cli --scan --pattern "proxy:cert:*"
```

## 🔍 Troubleshooting Quick Reference

### **Common Issues**

| Problem | Solution | Documentation |
|---------|----------|---------------|
| **Proxy not responding** | Check if running on correct port | [Setup Guide](SETUP.md#testing-your-setup) |
| **Certificate warnings** | Install rootCA.crt in browser | [Browser Setup](BROWSER_SETUP.md) |
| **HTTPS interception not working** | Verify CA certificate installation | [Certificates](CERTIFICATES.md#browser-integration) |
| **Performance issues** | Check certificate caching | [Configuration](CONFIGURATION.md#performance-configuration) |
| **Docker container issues** | Check Redis connectivity | [Deployment](deployment.md#docker-deployment) |

### **Debug Commands**
```bash
# Maximum verbosity logging
RUST_LOG=trace make dev

# Test proxy connectivity
curl -x http://127.0.0.1:8080 http://httpbin.org/get

# Test HTTPS with certificate bypass
curl -x http://127.0.0.1:8080 https://httpbin.org/get --proxy-insecure

# Check certificate cache status
make cache-clear-redis  # Clear Redis cache
```

## 📈 Performance Expectations

### **Throughput**
- **HTTP Requests**: 1000+ requests/second
- **HTTPS Tunneling**: 500+ connections/second  
- **HTTPS Interception**: 100+ new domains/second
- **Certificate Caching**: 10,000+ cached lookups/second

### **Latency**
- **HTTP Proxy**: +1-2ms overhead
- **HTTPS Tunneling**: +2-5ms overhead
- **HTTPS Interception (first request)**: +5-10ms
- **HTTPS Interception (cached)**: +1-2ms

## 🛡️ Security Considerations

- **📋 Certificate Security**: Install only trusted CA certificates
- **🔒 Private Key Storage**: Secure CA private keys with proper permissions
- **⏰ Certificate Rotation**: Regular CA certificate renewal (annually)
- **🚨 Monitoring**: Monitor certificate generation and usage patterns

## 🤝 Contributing to Documentation

Found an issue or want to improve the documentation?

1. **Small fixes**: Edit files directly and submit PR
2. **New sections**: Follow the existing structure and style
3. **Code examples**: Test all examples before including
4. **Screenshots**: Use high-resolution images with annotations

### **Documentation Style Guide**
- **Emoji headers**: Use relevant emojis for section headers
- **Code blocks**: Include language specification and comments
- **Step-by-step**: Number complex procedures
- **Cross-references**: Link to related documentation sections

---

## 🎉 Ready to Get Started?

### **For Developers**
Start with the [Setup Guide](SETUP.md) to get the proxy running in minutes.

### **For Security Engineers**  
Check out [HTTPS Interception](CERTIFICATES.md) for complete traffic visibility.

### **For DevOps Engineers**
Review the [Deployment Guide](deployment.md) for production deployment patterns.

### **For Enterprise Users**
See [Configuration](CONFIGURATION.md) for Securly CA integration and advanced settings.

**Questions? Check our [troubleshooting sections](SETUP.md#troubleshooting) or review the comprehensive guides above!**