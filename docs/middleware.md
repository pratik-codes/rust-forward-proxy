# Middleware System

This document provides detailed information about the middleware system in the Rust Forward Proxy and how it integrates with the current modular architecture.

## Overview

The Rust Forward Proxy uses a flexible middleware system that can intercept and process HTTP requests and responses. The middleware framework is designed to be extensible and follows Rust's ownership patterns for safe, concurrent operation.

## Current Architecture Integration

### Middleware in the Request Pipeline

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────────┐
│  Incoming       │───▶│    Middleware    │───▶│   Request Handler   │
│  Request        │    │    Pipeline      │    │                     │
└─────────────────┘    └──────────────────┘    └─────────────────────┘
                                │
                                ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────────┐
│  Outgoing       │◀───│   Response       │◀───│   Upstream          │
│  Response       │    │   Processing     │    │   Server            │
└─────────────────┘    └──────────────────┘    └─────────────────────┘
```

### Integration with Current Codebase

The middleware system integrates seamlessly with the current modular architecture:

- **Server Logic** (`proxy/server.rs`) - Main request routing and middleware orchestration
- **HTTP Utilities** (`utils/http.rs`) - Reusable functions for request/response processing
- **Logging Utilities** (`utils/logging.rs`) - Consistent logging across middleware
- **Data Models** (`models/mod.rs`) - Shared data structures for request/response

## Available Middleware

### 1. Authentication Middleware (`middleware/auth.rs`)

**Purpose**: Provides API key-based authentication for incoming requests.

#### Current Implementation

```rust
pub struct AuthMiddleware {
    required_api_key: Option<String>,
}

impl AuthMiddleware {
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            required_api_key: api_key,
        }
    }

    pub fn authenticate(&self, request: &Request<Body>) -> Result<bool> {
        // Bearer token authentication via Authorization header
    }
}
```

#### Features

- **Bearer Token Authentication**: Validates `Authorization: Bearer <token>` headers
- **Optional Authentication**: Can be disabled by setting `api_key` to `None`
- **Secure Validation**: Constant-time comparison to prevent timing attacks
- **Error Handling**: Proper error responses for authentication failures

#### Usage Example

```rust
use crate::utils::{log_incoming_request, build_error_response};

let auth_middleware = AuthMiddleware::new(Some("your-secret-key".to_string()));

// In request handler
match auth_middleware.authenticate(&request) {
    Ok(true) => {
        log_incoming_request(&method, &uri, &remote_addr);
        // Continue with request processing
    }
    Ok(false) => {
        return Ok(build_error_response(StatusCode::UNAUTHORIZED, "Authentication failed"));
    }
    Err(e) => {
        log_error!("Auth middleware error: {}", e);
        return Ok(build_error_response(StatusCode::INTERNAL_SERVER_ERROR, "Auth error"));
    }
}
```

### 2. Logging Middleware (`middleware/logging.rs`)

**Purpose**: Provides request/response logging capabilities.

#### Integration with New Logging System

The logging middleware now integrates with the new utility-based logging system:

```rust
use crate::utils::{
    log_incoming_request, 
    log_http_success, 
    log_http_failure,
    create_connect_transaction
};

pub struct LoggingMiddleware {
    pub enable_request_logging: bool,
    pub enable_response_logging: bool,
    pub log_level: LogLevel,
}

impl LoggingMiddleware {
    pub fn log_request(&self, request_data: &RequestData, remote_addr: &SocketAddr) {
        if self.enable_request_logging {
            log_incoming_request(&request_data.method, &request_data.url, remote_addr);
        }
    }

    pub fn log_response(&self, request_data: &RequestData, response_data: &ResponseData, duration: u128) {
        if self.enable_response_logging {
            log_http_success(&request_data.method, &request_data.path, 
                           StatusCode::from_u16(response_data.status_code).unwrap(), duration);
        }
    }
}
```

#### Features

- **Two-Tier Logging**: Supports both clean INFO and verbose DEBUG logging
- **Selective Logging**: Can enable/disable request and response logging independently
- **Performance Monitoring**: Tracks request duration and response times
- **Structured Data**: Integrates with the `ProxyLog` transaction system

### 3. Rate Limiting Middleware (`middleware/rate_limit.rs`)

**Purpose**: Provides IP-based rate limiting to prevent abuse.

#### Current Implementation

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct RateLimitMiddleware {
    requests_per_minute: u32,
    request_counts: Arc<Mutex<HashMap<IpAddr, (u32, Instant)>>>,
}

impl RateLimitMiddleware {
    pub fn new(requests_per_minute: u32) -> Self {
        Self {
            requests_per_minute,
            request_counts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn check_rate_limit(&self, client_ip: IpAddr) -> bool {
        // Implementation checks request count per IP per minute
    }
}
```

#### Features

- **IP-Based Rate Limiting**: Tracks requests per IP address
- **Configurable Limits**: Customizable requests per minute
- **Memory Efficient**: Automatic cleanup of old entries
- **Thread Safe**: Uses `Arc<Mutex<>>` for concurrent access

#### Usage Example

```rust
let rate_limiter = RateLimitMiddleware::new(60); // 60 requests per minute

// In request handler
if !rate_limiter.check_rate_limit(remote_addr.ip()) {
    log_http_failure(&method, &path, 0, &anyhow::anyhow!("Rate limit exceeded"));
    return Ok(build_error_response(StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded"));
}
```

## Implementing Custom Middleware

### Middleware Trait Pattern

```rust
use crate::models::{RequestData, ResponseData};
use crate::utils::{log_incoming_request, build_error_response};
use hyper::{Request, Response, Body, StatusCode};
use std::net::SocketAddr;

pub trait Middleware {
    async fn process_request(
        &self, 
        request: &mut Request<Body>, 
        request_data: &mut RequestData,
        remote_addr: &SocketAddr
    ) -> Result<Option<Response<Body>>, Box<dyn std::error::Error + Send + Sync>>;

    async fn process_response(
        &self,
        request_data: &RequestData,
        response: &mut Response<Body>,
        response_data: &mut ResponseData
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
```

### Example: Custom Header Middleware

```rust
pub struct HeaderMiddleware {
    required_headers: Vec<String>,
}

impl HeaderMiddleware {
    pub fn new(headers: Vec<String>) -> Self {
        Self { required_headers: headers }
    }
}

impl Middleware for HeaderMiddleware {
    async fn process_request(
        &self,
        request: &mut Request<Body>,
        request_data: &mut RequestData,
        remote_addr: &SocketAddr
    ) -> Result<Option<Response<Body>>, Box<dyn std::error::Error + Send + Sync>> {
        
        for required_header in &self.required_headers {
            if !request_data.headers.contains_key(required_header) {
                log_http_failure(&request_data.method, &request_data.path, 0, 
                               &anyhow::anyhow!("Missing required header: {}", required_header));
                
                return Ok(Some(build_error_response(
                    StatusCode::BAD_REQUEST, 
                    &format!("Missing required header: {}", required_header)
                )));
            }
        }
        
        Ok(None) // Continue processing
    }

    async fn process_response(
        &self,
        _request_data: &RequestData,
        response: &mut Response<Body>,
        _response_data: &mut ResponseData
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Add custom response headers
        response.headers_mut().insert("X-Processed-By", "Rust-Forward-Proxy".parse().unwrap());
        Ok(())
    }
}
```

## Middleware Integration in Server

### Request Pipeline Integration

The middleware system integrates with the current server architecture:

```rust
// In proxy/server.rs
async fn handle_request(
    req: Request<Body>,
    remote_addr: SocketAddr,
) -> Result<Response<Body>, Infallible> {
    let start_time = std::time::Instant::now();
    let method = req.method().to_string();
    let uri = req.uri().to_string();

    // Log incoming request using utility function
    log_incoming_request(&method, &uri, &remote_addr);

    // Create request data
    let mut request_data = RequestData::new(method.clone(), uri.clone(), remote_addr.ip(), remote_addr.port());
    
    // Apply middleware chain
    if let Some(middleware_response) = apply_middleware_chain(&mut req, &mut request_data, &remote_addr).await {
        return Ok(middleware_response);
    }
    
    // Continue with normal request processing...
}

async fn apply_middleware_chain(
    request: &mut Request<Body>,
    request_data: &mut RequestData,
    remote_addr: &SocketAddr
) -> Option<Response<Body>> {
    // Apply authentication middleware
    // Apply rate limiting middleware  
    // Apply custom middleware
    None // If all middleware pass
}
```

## Performance Considerations

### Middleware Efficiency

1. **Async Processing**: All middleware operations are async to prevent blocking
2. **Early Returns**: Middleware can short-circuit the pipeline for efficiency
3. **Shared State**: Uses `Arc<Mutex<>>` for thread-safe shared state
4. **Memory Management**: Automatic cleanup of old data in rate limiters

### Integration with Logging System

The middleware leverages the efficient logging utilities:

```rust
// Clean INFO logging
log_incoming_request(&method, &uri, &remote_addr);
log_http_success(&method, &path, status, duration);

// Detailed DEBUG logging when needed
log_debug!("Middleware processing: {:#?}", request_data);
```

## Configuration and Deployment

### Environment-Based Configuration

```rust
// In configuration
pub struct MiddlewareConfig {
    pub enable_auth: bool,
    pub api_key: Option<String>,
    pub rate_limit_rpm: u32,
    pub enable_request_logging: bool,
    pub enable_response_logging: bool,
}

impl Default for MiddlewareConfig {
    fn default() -> Self {
        Self {
            enable_auth: false,
            api_key: std::env::var("PROXY_API_KEY").ok(),
            rate_limit_rpm: 60,
            enable_request_logging: true,
            enable_response_logging: true,
        }
    }
}
```

### Production Deployment

For production deployments:

1. **Enable Authentication**: Set `PROXY_API_KEY` environment variable
2. **Configure Rate Limiting**: Adjust limits based on expected traffic
3. **Logging Configuration**: Use INFO level for production, DEBUG for troubleshooting
4. **Monitoring**: Leverage the structured logging for observability

The middleware system provides a solid foundation for extending the proxy with additional functionality while maintaining the clean, modular architecture of the current codebase.