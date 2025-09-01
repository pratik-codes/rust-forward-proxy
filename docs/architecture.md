# Rust Forward Proxy Architecture

This document provides a detailed explanation of the current architecture of the Rust Forward Proxy server.

## Overview

The Rust Forward Proxy is designed as a high-performance HTTP/HTTPS forward proxy server with a focus on:

- **Performance**: Utilizing Rust's memory safety and performance characteristics along with asynchronous programming via Tokio and Hyper
- **Clean Architecture**: Modular design following DRY principles with utility separation
- **Full Tunneling Support**: Complete HTTP interception and HTTPS tunneling via CONNECT method
- **Production Logging**: Two-tier logging system (clean INFO, verbose DEBUG)
- **Maintainability**: Well-organized codebase with clear separation of concerns

## System Architecture

```
┌─────────────────┐     ┌─────────────────────────┐     ┌──────────────────────┐
│                 │     │                         │     │                      │
│   HTTP Client   │────▶│    Forward Proxy        │────▶│   Upstream Server    │
│   (Browser)     │     │                         │     │                      │
│                 │◀────│  ┌─────────────────────┐ │◀────│                      │
└─────────────────┘     │  │   HTTP Requests     │ │     └──────────────────────┘
                        │  │  (Full Intercept)   │ │              
┌─────────────────┐     │  └─────────────────────┘ │     ┌──────────────────────┐
│                 │     │                         │═════│                      │
│   HTTPS Client  │═════│  ┌─────────────────────┐ │═════│   HTTPS Server       │
│   (Browser)     │     │  │  CONNECT Tunneling  │ │     │                      │
│                 │═════│  │   (Raw TCP Pass)    │ │═════│                      │
└─────────────────┘     │  └─────────────────────┘ │     └──────────────────────┘
                        └─────────────────────────┘              
                                      │
                                      ▼
                               ┌──────────────┐
                               │   Logging    │
                               │  INFO/DEBUG  │
                               └──────────────┘
```

## Current Architecture Components

### 1. Core Server (`src/proxy/server.rs`) - 402 lines

**Main Handler Functions:**
- `handle_request()` - Main request dispatcher (clean and lean)
- `handle_connect_request()` - CONNECT tunneling with hyper upgrade mechanism
- `handle_http_request()` - HTTP request processing with full interception
- `extract_request_data()` - Request data extraction and processing
- `handle_regular_request()` - Upstream forwarding logic

**Key Features:**
- **CONNECT Tunneling**: Uses `hyper::upgrade::on()` for proper HTTPS tunneling
- **Bidirectional Data Copy**: `tunnel_bidirectional()` for raw TCP forwarding
- **Clean Request Flow**: Simplified main handler with extracted utility functions

### 2. Utility Modules (`src/utils/`) - 366 lines total

#### HTTP Utilities (`src/utils/http.rs`) - 197 lines
```rust
// CONNECT handling
parse_connect_target() -> Result<(String, u16), String>
build_error_response(status: StatusCode, message: &str) -> Response<Body>

// Request processing  
extract_headers(req_headers: &HeaderMap, request_data: &mut RequestData)
extract_cookies_to_request_data(req_headers: &HeaderMap, request_data: &mut RequestData)
should_extract_body(req_headers: &HeaderMap, method: &str) -> (bool, Option<String>)
extract_body(body: Body, request_data: &mut RequestData)

// Request building
build_forwarding_request(request_data: &RequestData) -> Result<Request<Body>>
```

#### Logging Utilities (`src/utils/logging.rs`) - 111 lines
```rust
// Request logging
log_incoming_request(method: &str, uri: &str, remote_addr: &SocketAddr)
log_forwarding_request(request_data: &RequestData)

// CONNECT logging
log_connect_request(uri: &str)
log_connect_success(host: &str, port: u16, connect_time: u128)
log_connect_failure(host: &str, port: u16, connect_time: u128, error: &str)

// HTTP logging  
log_http_success(method: &str, path: &str, status: StatusCode, total_time: u128)
log_http_failure(method: &str, path: &str, total_time: u128, error: &anyhow::Error)

// Transaction logging
create_connect_transaction(request_data, response_data, error) -> ProxyLog
```

#### URL/Time Utilities (`src/utils/url.rs`, `src/utils/time.rs`) - 47 lines
- URL parsing and manipulation functions
- Time-related utility functions

### 3. Data Models (`src/models/mod.rs`) - 137 lines

**Core Data Structures:**
```rust
pub struct RequestData {
    // HTTP request metadata
    method: String,
    url: String, 
    path: String,
    query_string: Option<String>,
    headers: HashMap<String, String>,
    cookies: HashMap<String, String>,
    body: Vec<u8>,
    form_data: HashMap<String, String>,
    
    // Connection metadata
    client_ip: IpAddr,
    client_port: u16,
    timestamp: DateTime<Utc>,
    is_https: bool,
    // ... other fields
}

pub struct ResponseData {
    status_code: u16,
    status_text: String,  
    headers: Vec<(String, String)>,
    body: Vec<u8>,
    response_time_ms: u64,
    // ... other fields
}

pub struct ProxyLog {
    request: RequestData,
    response: Option<ResponseData>,
    error: Option<String>,
}
```

### 4. Logging System (`src/logging/mod.rs`) - 296 lines

**Two-Tier Logging Architecture:**

#### INFO Level (Production)
```bash
📥 GET http://httpbin.org/get from 127.0.0.1
🔄 Forwarding GET to upstream  
📤 Upstream response: 200 (156ms)
✅ GET /get → 200 OK (158ms)
##################################

🔐 CONNECT tunnel to example.com:443
✅ Tunnel established to example.com:443 (45ms)
🔌 Tunnel completed for example.com:443
##################################
```

#### DEBUG Level (Development)  
```bash
🔍 REQUEST DETAILS:
  Method: GET
  URI: http://httpbin.org/get
  Remote: 127.0.0.1:54321
  Headers: 12

🔄 FORWARDING REQUEST:
  Method: GET
  URL: http://httpbin.org/get  
  Headers: 10
  Body Size: 0 bytes

📋 HTTP TRANSACTION:
{
  "request": { ... full RequestData struct ... },
  "response": { ... full ResponseData struct ... }
}
```

## Data Flow Architecture

### HTTP Request Flow
```
1. Client Request → handle_request()
2. log_incoming_request() → Clean INFO log
3. extract_request_data() → Parse headers, cookies, body
4. handle_http_request() → Process with full interception  
5. build_forwarding_request() → Construct upstream request
6. Forward to upstream → Get response
7. log_http_success() → Clean completion log
8. Return response to client
```

### HTTPS CONNECT Flow  
```
1. Client CONNECT → handle_request()
2. log_connect_request() → Log tunnel establishment
3. handle_connect_request() → Parse target host:port
4. TcpStream::connect() → Establish upstream connection
5. handle_connect_tunnel() → Return 200 OK + spawn tunnel task
6. hyper::upgrade::on() → Upgrade connection 
7. tunnel_bidirectional() → Raw TCP forwarding
8. log_connect_success() → Log tunnel completion
```

## Modular Architecture Benefits

### 1. **Clean Separation of Concerns**
- **Server Logic**: Pure request handling and flow control
- **HTTP Utilities**: Reusable HTTP processing functions  
- **Logging Utilities**: Consistent logging across the application
- **Data Models**: Clear data structure definitions

### 2. **DRY Principle Implementation**
- **No Code Duplication**: Common patterns extracted into utility functions
- **Reusable Components**: Utilities can be imported anywhere
- **Single Source of Truth**: Each function has one clear responsibility

### 3. **Testability & Maintainability**
- **Unit Testing**: Individual utility functions can be tested in isolation
- **Clear Dependencies**: Easy to trace function dependencies
- **Focused Changes**: Modifications affect only relevant modules

### 4. **Performance Characteristics**
- **Async Throughout**: Full Tokio/Hyper async stack
- **Zero-Copy Where Possible**: Minimal data copying in request/response handling
- **Connection Reuse**: HTTP client reuses connections for efficiency
- **Streaming**: Large request/response bodies handled efficiently

## Extension Points

The current architecture supports extension in several areas:

1. **Middleware System**: Can be added to `src/proxy/middleware/`
2. **Authentication**: Auth utilities can be added to `src/utils/`
3. **Rate Limiting**: Rate limiting logic can be integrated into request flow
4. **Connection Pooling**: Upstream connection management can be enhanced
5. **Protocol Support**: WebSocket tunneling, HTTP/2 support can be added

## Security Considerations

- **HTTPS Tunneling**: Raw TCP passthrough maintains end-to-end encryption
- **Request Validation**: Input validation in utility functions
- **Error Handling**: Safe error propagation without information leakage
- **Memory Safety**: Rust's ownership system prevents common security issues