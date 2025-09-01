# Upstream Connection Management

This document provides detailed information about the upstream connection management system in the Rust Forward Proxy.

## Overview

The upstream connection management system is responsible for handling connections to upstream servers (the destination servers that clients are trying to access through the proxy). It provides mechanisms for establishing connections, reusing them efficiently, and monitoring their health.

## Components

The upstream connection management system consists of three main components:

1. **Upstream Client**: Handles sending requests to upstream servers
2. **Connection Pool**: Manages a pool of connections for reuse
3. **Health Checker**: Monitors the health of upstream servers

## 1. Upstream Client (`client.rs`)

The `UpstreamClient` is responsible for sending HTTP requests to upstream servers.

### Features

- **HTTP Client**: Built on hyper's HTTP client
- **Timeout Management**: Configurable request timeouts
- **Connection Settings**: Customizable connection parameters

### Implementation Details

```rust
pub struct UpstreamClient {
    client: Client<hyper::client::HttpConnector>,
    timeout: Duration,
}

impl UpstreamClient {
    pub fn new(timeout: Duration) -> Self {
        let client = Client::builder()
            .pool_idle_timeout(Duration::from_secs(30))
            .build(hyper::client::HttpConnector::new());
            
        Self { client, timeout }
    }
    
    pub async fn request(&self, req: Request<Body>) -> Result<Response<Body>> {
        let response = tokio::time::timeout(self.timeout, self.client.request(req)).await??;
        Ok(response)
    }
}
```

### Usage

```rust
// Create a client with a 10-second timeout
let client = UpstreamClient::new(Duration::from_secs(10));

// Send a request to an upstream server
let response = client.request(request).await?;
```

### Configuration Options

- **Timeout**: Maximum time to wait for a response from the upstream server
- **Pool Idle Timeout**: How long to keep idle connections in the pool
- **Connection Parameters**: Other hyper client configuration options

## 2. Connection Pool (`connection_pool.rs`)

The `ConnectionPool` manages a pool of connections to upstream servers for reuse, improving performance by avoiding the overhead of establishing new connections for each request.

### Features

- **Connection Reuse**: Maintains a pool of open connections
- **Connection Limits**: Configurable maximum number of connections
- **Thread Safety**: Safe for concurrent access

### Implementation Details

```rust
pub struct ConnectionPool {
    connections: Arc<Mutex<HashMap<String, Vec<()>>>>, // Placeholder for actual connections
    max_connections: usize,
}

impl ConnectionPool {
    pub fn new(max_connections: usize) -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            max_connections,
        }
    }
    
    pub async fn get_connection(&self, _host: &str) -> Option<()> {
        // Placeholder implementation
        Some(())
    }
    
    pub async fn return_connection(&self, _host: &str, _connection: ()) {
        // Placeholder implementation
    }
}
```

### Usage

```rust
// Create a connection pool with a maximum of 100 connections
let pool = ConnectionPool::new(100);

// Get a connection for a specific host
if let Some(conn) = pool.get_connection("example.com").await {
    // Use the connection
    // ...
    
    // Return the connection to the pool
    pool.return_connection("example.com", conn).await;
}
```

### Future Implementation

Note that the current implementation is a placeholder. A full implementation would:

1. Maintain actual connection objects
2. Implement connection timeouts
3. Handle connection errors
4. Provide metrics on connection usage
5. Implement connection cleanup

## 3. Health Checker (`health_check.rs`)

The `HealthChecker` monitors the health of upstream servers to avoid sending requests to unhealthy servers.

### Features

- **Health Status Tracking**: Maintains health status of upstream servers
- **Periodic Checks**: Performs regular health checks
- **Configurable Intervals**: Customizable check frequency

### Implementation Details

```rust
#[derive(Debug, Clone)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
    Unknown,
}

pub struct HealthChecker {
    check_interval: Duration,
}

impl HealthChecker {
    pub fn new(check_interval: Duration) -> Self {
        Self { check_interval }
    }
    
    pub async fn check_health(&self, _url: &str) -> Result<HealthStatus> {
        // Placeholder implementation
        Ok(HealthStatus::Healthy)
    }
    
    pub async fn start_health_checks(&self) {
        // Placeholder implementation
    }
}
```

### Usage

```rust
// Create a health checker that checks every 30 seconds
let health_checker = HealthChecker::new(Duration::from_secs(30));

// Check the health of an upstream server
let status = health_checker.check_health("http://example.com").await?;

// Start background health checks
health_checker.start_health_checks().await;
```

### Future Implementation

The current implementation is a placeholder. A full implementation would:

1. Send actual HTTP requests to upstream servers
2. Track response times and error rates
3. Update health status based on check results
4. Implement circuit breaker pattern
5. Provide health status reporting

## Integration

These components work together to provide efficient and reliable connections to upstream servers:

```
┌─────────────────────┐
│                     │
│   Proxy Server      │
│                     │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐     ┌─────────────────────┐
│                     │     │                     │
│   Upstream Client   │◀───▶│   Connection Pool   │
│                     │     │                     │
└──────────┬──────────┘     └─────────────────────┘
           │
           ▼
┌─────────────────────┐
│                     │
│   Health Checker    │
│                     │
└─────────────────────┘
```

1. The proxy server receives a client request
2. It uses the upstream client to forward the request
3. The upstream client gets a connection from the pool
4. The health checker ensures the upstream server is healthy
5. The request is sent to the upstream server
6. The connection is returned to the pool after use

## Configuration

The upstream connection management system can be configured in various ways:

### 1. Client Configuration

```rust
// Create a client with custom timeouts
let client = UpstreamClient::new(Duration::from_secs(5)); // 5-second timeout
```

### 2. Connection Pool Configuration

```rust
// Create a connection pool with custom limits
let pool = ConnectionPool::new(50); // Maximum 50 connections
```

### 3. Health Checker Configuration

```rust
// Create a health checker with custom interval
let checker = HealthChecker::new(Duration::from_secs(15)); // Check every 15 seconds
```

## Best Practices

1. **Connection Limits**: Set appropriate connection limits to avoid resource exhaustion.
2. **Timeouts**: Configure reasonable timeouts to prevent hanging connections.
3. **Health Check Frequency**: Balance the need for fresh health data with the overhead of health checks.
4. **Error Handling**: Implement robust error handling for connection failures.
5. **Metrics**: Collect and monitor metrics on connection usage and health.

## Future Improvements

1. **Connection Pooling**: Implement a more sophisticated connection pool.
2. **Protocol Support**: Add support for HTTP/2 and WebSockets.
3. **TLS Configuration**: Provide more options for TLS connections.
4. **Load Balancing**: Implement load balancing across multiple upstream servers.
5. **Circuit Breaking**: Add circuit breaker pattern to prevent cascading failures.
6. **Connection Draining**: Implement graceful connection draining during shutdown.