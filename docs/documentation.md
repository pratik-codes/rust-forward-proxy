# Rust Forward Proxy - Codebase Documentation

This document provides comprehensive documentation of the current Rust Forward Proxy codebase structure and implementation.

## Project Structure

The project follows a clean, modular architecture with clear separation of concerns:

```
rust-forward-proxy/
├── src/
│   ├── main.rs                     # Application entry point
│   ├── lib.rs                      # Library root and exports
│   ├── cli/                        # Command-line interface
│   ├── config/                     # Configuration management
│   │   ├── mod.rs
│   │   └── settings.rs
│   ├── error/                      # Error handling
│   │   └── mod.rs
│   ├── logging/                    # Logging system
│   │   └── mod.rs
│   ├── models/                     # Data structures
│   │   └── mod.rs
│   ├── proxy/                      # Core proxy logic
│   │   ├── server.rs              # Main server (402 lines)
│   │   ├── middleware/            # Middleware components
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs
│   │   │   ├── logging.rs
│   │   │   └── rate_limit.rs
│   │   └── upstream/              # Upstream management
│   │       ├── mod.rs
│   │       ├── client.rs
│   │       ├── connection_pool.rs
│   │       └── health_check.rs
│   └── utils/                     # Utility modules (366 lines total)
│       ├── mod.rs                 # Module exports (11 lines)
│       ├── http.rs                # HTTP utilities (197 lines)
│       ├── logging.rs             # Logging utilities (111 lines)
│       ├── time.rs                # Time utilities (24 lines)
│       └── url.rs                 # URL utilities (23 lines)
└── docs/                          # Documentation
    ├── README.md
    ├── architecture.md
    ├── usage.md
    ├── deployment.md
    ├── performance.md
    ├── middleware.md
    └── upstream.md
```

## Core Modules

### 1. `main.rs` - Application Entry Point

**Purpose**: Initializes and starts the proxy server

**Key Functions**:
```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging system with environment support
    init_logger_with_env();
    
    // Create and start proxy server
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let server = ProxyServer::new(addr);
    server.start().await
}
```

### 2. `lib.rs` - Library Root

**Purpose**: Defines module hierarchy and exports

**Exports**:
- Logging macros: `log_info!`, `log_error!`, `log_debug!`, `log_proxy_transaction!`
- Re-exports from modules for easy access

### 3. `proxy/server.rs` - Core Server Logic (402 lines)

**Purpose**: Main proxy server implementation with clean, modular handlers

The server handles two main types of requests:
- **HTTP Requests**: Fully intercepted, logged, and forwarded
- **CONNECT Requests**: Tunneled for HTTPS traffic using hyper upgrade mechanism

**Key Architecture Changes**:
- Moved utility functions to `utils/` modules following DRY principle
- Clean separation between server logic and utility functions
- Modular handler functions for different request types

### 4. `utils/http.rs` - HTTP Utilities (197 lines)

**Purpose**: Reusable HTTP processing functions moved from server.rs

**Key Functions**:
- `parse_connect_target()` - Parse CONNECT host:port targets
- `build_error_response()` - Create HTTP error responses
- `extract_headers()`, `extract_cookies_to_request_data()` - Request parsing
- `should_extract_body()`, `extract_body()` - Body processing
- `build_forwarding_request()` - Construct upstream requests
- `is_hop_by_hop_header()` - Header filtering for proxies

### 5. `utils/logging.rs` - Logging Utilities (111 lines)

**Purpose**: Consistent logging patterns extracted from server.rs

**Features**:
- Two-tier logging: Clean INFO logs, verbose DEBUG logs
- Specialized logging for HTTP vs CONNECT requests
- Transaction logging with structured data

**Key Functions**:
- `log_incoming_request()`, `log_forwarding_request()`
- `log_connect_success()`, `log_connect_failure()`
- `log_http_success()`, `log_http_failure()`
- `create_connect_transaction()`

### 6. `models/mod.rs` - Data Structures (137 lines)

**Purpose**: Core data models for requests, responses, and logging

**Key Structures**:
- `RequestData` - Complete HTTP request metadata and body
- `ResponseData` - HTTP response data and timing
- `ProxyLog` - Transaction container for logging

**Features**:
- Support for both HTTP and CONNECT request types
- Comprehensive metadata extraction
- JSON serialization for structured logging

### 7. `logging/mod.rs` - Logging System (296 lines)

**Purpose**: Production-grade logging setup with environment configuration

**Features**:
- Environment variable support (`RUST_LOG`)
- Integration with `tracing` and `log` crates
- Console output with structured formatting
- Bridge between log crate and tracing

### 8. Configuration & Middleware

**Configuration System**:
- `config/settings.rs` - Server and upstream configuration
- Environment-based configuration support

**Middleware Components**:
- `middleware/auth.rs` - Authentication middleware
- `middleware/logging.rs` - Request/response logging
- `middleware/rate_limit.rs` - Rate limiting by IP

**Upstream Management**:
- `upstream/client.rs` - HTTP client for upstream requests
- `upstream/connection_pool.rs` - Connection pooling
- `upstream/health_check.rs` - Health monitoring

## Current Implementation Status

### ✅ **Fully Implemented & Working**

1. **HTTP Request Interception**
   - Complete request/response logging
   - Header extraction and processing
   - Body extraction for POST/PUT/PATCH
   - Cookie parsing and form data handling

2. **HTTPS CONNECT Tunneling**
   - Proper CONNECT method handling
   - Hyper upgrade mechanism for tunneling
   - Bidirectional TCP data forwarding
   - Raw encrypted traffic passthrough

3. **Modular Architecture**
   - Clean separation of concerns
   - DRY principle implementation
   - Utility functions properly organized
   - Reusable components across modules

4. **Production Logging**
   - Two-tier logging system (INFO/DEBUG)
   - Environment-based configuration
   - Structured transaction logging
   - Clean console output

### 🔄 **Framework Ready for Extension**

1. **Middleware System** - Framework exists, ready for custom middleware
2. **Connection Pooling** - Structure in place, can be enhanced
3. **Health Checking** - Basic framework for upstream monitoring
4. **Authentication** - Middleware structure ready for auth logic

### 📊 **Code Organization**

- **Server Logic**: 402 lines (down from 623) - focused on core functionality
- **HTTP Utilities**: 197 lines - reusable HTTP processing functions
- **Logging Utilities**: 111 lines - consistent logging patterns
- **Total Utils**: 366 lines - well-organized utility modules

This architecture provides a solid foundation for a production-ready HTTP/HTTPS forward proxy with excellent maintainability and extensibility.