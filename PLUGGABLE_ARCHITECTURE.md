# Pluggable HTTP Proxy Architecture

This document explains the new pluggable architecture for the Rust Forward Proxy, which allows easy switching between different HTTP library implementations.

## Overview

The proxy now supports multiple HTTP implementations that can be easily swapped:

- **Pingora** - CloudFlare's high-performance proxy framework (default)
- **Hyper** - Rust's foundational HTTP library 
- **Reqwest** - High-level HTTP client library

All implementations share the same interface, making it trivial to switch between them based on your needs.

## Architecture

### Core Abstractions

The pluggable architecture is built around these key components:

```rust
// Core trait that all implementations must follow
pub trait HttpProxyCore: Send + Sync {
    async fn initialize(&mut self, config: &ProxyConfig, impl_config: &ProxyImplConfig) -> ProxyResult<()>;
    async fn start(&self, listen_addr: SocketAddr) -> ProxyResult<()>;
    async fn process_request(&self, request: ProxyRequest) -> ProxyResult<ProxyResponse>;
    async fn handle_health_check(&self) -> ProxyResult<ProxyResponse>;
    async fn get_metrics(&self) -> ProxyResult<ProxyMetrics>;
    async fn shutdown(&self) -> ProxyResult<()>;
    fn implementation_name(&self) -> &'static str;
}

// Factory for creating implementations
pub struct ProxyFactory;

// Manager that handles middleware and request processing
pub struct ProxyManager;
```

### Available Implementations

#### 1. Pingora Implementation
```rust
use rust_forward_proxy::{ProxyManager, ProxyImplementation};

let mut proxy = ProxyManager::new(ProxyImplementation::Pingora);
```

**Features:**
- Built for high performance and scalability
- Supports HTTP/1.1 and HTTP/2
- Advanced connection pooling
- Built-in load balancing capabilities
- CloudFlare battle-tested

**Best for:** Production deployments requiring maximum performance

#### 2. Hyper Implementation
```rust
let mut proxy = ProxyManager::new(ProxyImplementation::Hyper);
```

**Features:**
- Lower-level control over HTTP handling
- Excellent performance
- Mature and stable
- Wide ecosystem support

**Best for:** Custom proxy logic, tight control over HTTP behavior

#### 3. Reqwest Implementation
```rust
let mut proxy = ProxyManager::new(ProxyImplementation::Reqwest);
```

**Features:**
- High-level, easy to use
- Built-in support for many features
- Good for prototyping
- Actual HTTP client functionality

**Best for:** Development, testing, quick prototypes

## Usage Examples

### Basic Usage

```rust
use rust_forward_proxy::{
    ProxyManager, ProxyImplementation, ProxyConfig
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = ProxyConfig::load_config()?;
    
    // Create proxy manager with desired implementation
    let mut proxy = ProxyManager::new(ProxyImplementation::Pingora);
    
    // Initialize
    proxy.initialize(&config).await?;
    
    // Start the proxy
    proxy.start(config.listen_addr).await?;
    
    Ok(())
}
```

### Switching Implementations

You can switch implementations in several ways:

#### 1. Environment Variable
```bash
# Use Pingora (default)
PROXY_IMPL=pingora cargo run --bin rust-forward-proxy-pluggable

# Use Hyper
PROXY_IMPL=hyper cargo run --bin rust-forward-proxy-pluggable

# Use Reqwest
PROXY_IMPL=reqwest cargo run --bin rust-forward-proxy-pluggable
```

#### 2. Configuration File
```yaml
# config.yml
proxy_implementation:
  implementation: pingora  # or "hyper" or "reqwest"
  max_connections: 1000
  connection_timeout_ms: 10000
  request_timeout_ms: 30000
  enable_http2: true
  enable_connection_pooling: true
```

#### 3. Programmatically
```rust
// Runtime switching
let implementation = match std::env::var("PERFORMANCE_MODE") {
    Ok(mode) if mode == "high" => ProxyImplementation::Pingora,
    Ok(mode) if mode == "dev" => ProxyImplementation::Reqwest,
    _ => ProxyImplementation::Hyper,
};

let mut proxy = ProxyManager::new(implementation);
```

### Custom Middleware

The architecture supports middleware for request/response processing:

```rust
use rust_forward_proxy::proxy::core::{ProxyMiddleware, ProxyRequest, ProxyResponse};

struct CustomLoggingMiddleware;

#[async_trait]
impl ProxyMiddleware for CustomLoggingMiddleware {
    async fn process_request(&self, request: &mut ProxyRequest) -> ProxyResult<()> {
        println!("Processing request: {} {}", request.method, request.uri);
        Ok(())
    }
    
    async fn process_response(
        &self, 
        response: &mut ProxyResponse, 
        request: &ProxyRequest
    ) -> ProxyResult<()> {
        println!("Response: {} for {}", response.status_code, request.uri);
        Ok(())
    }
}

// Add middleware to proxy
let mut proxy = ProxyManager::new(ProxyImplementation::Pingora);
proxy.add_middleware(Box::new(CustomLoggingMiddleware));
```

### Processing Individual Requests

You can also process individual requests programmatically:

```rust
use rust_forward_proxy::proxy::core::ProxyRequest;
use std::collections::HashMap;

let request = ProxyRequest {
    method: "GET".to_string(),
    uri: "https://httpbin.org/get".to_string(),
    headers: HashMap::new(),
    body: bytes::Bytes::new(),
    client_addr: "127.0.0.1:12345".parse().unwrap(),
    is_connect: false,
};

let response = proxy.process_request(request).await?;
println!("Status: {}", response.status_code);
```

## Implementation Comparison

| Feature | Pingora | Hyper | Reqwest |
|---------|---------|--------|---------|
| Performance | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| Ease of Use | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐⭐ |
| Production Ready | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| Customization | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ |
| Memory Usage | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| HTTP/2 Support | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| Load Balancing | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐ |

## Configuration Options

### ProxyImplConfig Options

```rust
pub struct ProxyImplConfig {
    /// Which implementation to use
    pub implementation: ProxyImplementation,
    /// Maximum concurrent connections
    pub max_connections: usize,
    /// Connection timeout in milliseconds
    pub connection_timeout_ms: u64,
    /// Request timeout in milliseconds
    pub request_timeout_ms: u64,
    /// Enable HTTP/2 support
    pub enable_http2: bool,
    /// Enable connection pooling
    pub enable_connection_pooling: bool,
}
```

### Default Values
```rust
ProxyImplConfig {
    implementation: ProxyImplementation::Pingora,
    max_connections: 1000,
    connection_timeout_ms: 10000,
    request_timeout_ms: 30000,
    enable_http2: true,
    enable_connection_pooling: true,
}
```

## Running Different Implementations

### Development Mode (Reqwest)
```bash
# Good for development and testing
PROXY_IMPL=reqwest cargo run --bin rust-forward-proxy-pluggable
```

### Production Mode (Pingora)
```bash
# Best performance for production
PROXY_IMPL=pingora cargo run --bin rust-forward-proxy-pluggable
```

### Compatibility Mode (Hyper)
```bash
# Good balance of performance and compatibility
PROXY_IMPL=hyper cargo run --bin rust-forward-proxy-pluggable
```

## Metrics and Monitoring

All implementations provide consistent metrics:

```rust
let metrics = proxy.get_metrics().await?;
println!("Implementation: {}", metrics.implementation);
println!("Total requests: {}", metrics.total_requests);
println!("Success rate: {:.2}%", 
    metrics.successful_requests as f64 / metrics.total_requests as f64 * 100.0
);
```

## Adding New Implementations

To add support for a new HTTP library:

1. **Implement the `HttpProxyCore` trait:**
```rust
pub struct MyCustomProxy;

#[async_trait]
impl HttpProxyCore for MyCustomProxy {
    async fn initialize(&mut self, config: &ProxyConfig, impl_config: &ProxyImplConfig) -> ProxyResult<()> {
        // Initialize your implementation
        Ok(())
    }
    
    async fn process_request(&self, request: ProxyRequest) -> ProxyResult<ProxyResponse> {
        // Handle the request with your HTTP library
        todo!()
    }
    
    fn implementation_name(&self) -> &'static str {
        "my_custom"
    }
    
    // ... implement other required methods
}
```

2. **Add to the factory:**
```rust
// Add to ProxyImplementation enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProxyImplementation {
    Pingora,
    Hyper,
    Reqwest,
    MyCustom,  // Add your implementation
}

// Add to factory
impl ProxyFactory {
    pub fn create_proxy(implementation: ProxyImplementation) -> Box<dyn HttpProxyCore> {
        match implementation {
            ProxyImplementation::MyCustom => {
                Box::new(MyCustomProxy::new())
            }
            // ... other implementations
        }
    }
}
```

3. **Update module structure:**
```rust
// src/proxy/mod.rs
pub mod my_custom_impl;
```

## Benefits of the Pluggable Architecture

1. **Flexibility**: Easy to switch between different HTTP implementations
2. **Testing**: Test with different implementations to find the best fit
3. **Performance Tuning**: Choose the best implementation for your use case
4. **Future-Proofing**: Easy to add new HTTP libraries as they emerge
5. **Development**: Use simpler implementations for development, optimized ones for production
6. **Consistency**: All implementations share the same interface and configuration
7. **Middleware Support**: Add custom processing logic that works with any implementation

## Troubleshooting

### Common Issues

**Compilation Errors with Pingora:**
- Ensure you have `cmake` installed: `brew install cmake` (macOS) or `apt-get install cmake` (Ubuntu)
- Pingora requires specific system dependencies

**Performance Issues:**
- Use Pingora for highest performance
- Check your `max_connections` setting
- Enable HTTP/2 if supported by clients

**Development Issues:**
- Use Reqwest for easier debugging
- Enable detailed logging with `RUST_LOG=debug`

### Debug Mode

Enable debug logging to see which implementation is being used:

```bash
RUST_LOG=debug PROXY_IMPL=pingora cargo run --bin rust-forward-proxy-pluggable
```

This will show detailed logs about implementation selection, initialization, and request processing.
