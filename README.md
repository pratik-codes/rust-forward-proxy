# Rust Forward Proxy

A high-performance HTTP/HTTPS forward proxy server written in Rust with full tunneling support and comprehensive logging.

## Features

- **Full HTTP/HTTPS Proxy Support** - Intercepts HTTP requests, tunnels HTTPS via CONNECT
- **High Performance** - Built with Hyper and Tokio for async networking
- **Production Logging** - Clean INFO logs for monitoring, verbose DEBUG for development
- **Modular Architecture** - Clean separation of concerns with utility modules

## Quick Start

```bash
# Install dependencies
cargo build

# Run the proxy (INFO logging)
cargo run

# Run with verbose logging
RUST_LOG=debug cargo run

# Test HTTP request
curl -x http://127.0.0.1:8080 http://httpbin.org/get

# Test HTTPS request  
curl -x http://127.0.0.1:8080 https://httpbin.org/get
```

The proxy runs on `127.0.0.1:8080` by default.

## Documentation

### 📚 **Comprehensive Guides**
- **[Architecture Overview](./docs/architecture.md)** - System design and component structure
- **[Usage Guide](./docs/usage.md)** - How to use the proxy with examples
- **[Deployment Guide](./docs/deployment.md)** - Production deployment instructions
- **[Performance Guide](./docs/performance.md)** - Optimization and benchmarking

### 🔧 **Technical References**
- **[API Documentation](./docs/documentation.md)** - Codebase structure and modules
- **[Middleware Documentation](./docs/middleware.md)** - Authentication, rate limiting, logging
- **[Upstream Documentation](./docs/upstream.md)** - Connection pooling and health checks

## How It Works

1. **HTTP Requests** → Fully intercepted, logged, and forwarded to upstream servers
2. **HTTPS Requests** → CONNECT method establishes encrypted tunnel, raw TCP forwarding
3. **Logging** → All transactions logged with clean INFO output and detailed DEBUG data

## Configuration

Set environment variables:

```bash
# Logging level
export RUST_LOG=info          # Clean logs for production
export RUST_LOG=debug         # Verbose logs for development

# Custom port (modify src/main.rs)
```

## License

MIT License