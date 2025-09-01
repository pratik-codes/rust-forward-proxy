# Rust Forward Proxy Architecture

This document provides a detailed explanation of the architecture of the Rust Forward Proxy server.

## Overview

The Rust Forward Proxy is designed as a high-performance HTTP/HTTPS forward proxy server with a focus on:

- **Performance**: Utilizing Rust's memory safety and performance characteristics along with asynchronous programming via Tokio
- **Extensibility**: Modular architecture with middleware support
- **Robustness**: Comprehensive error handling and logging
- **Production-readiness**: Structured logging, configurable timeouts, and connection management

## System Architecture

```
┌────────────────┐     ┌─────────────────┐     ┌────────────────────┐
│                │     │                 │     │                    │
│  HTTP Client   │────▶│  Proxy Server   │────▶│  Upstream Server   │
│  (Browser)     │     │                 │     │                    │
│                │◀────│                 │◀────│                    │
└────────────────┘     └─────────────────┘     └────────────────────┘
                               │
                               │
                               ▼
                       ┌─────────────────┐
                       │     Logging     │
                       └─────────────────┘
```

## Core Components

### 1. Proxy Server (`src/proxy/server.rs`)

The `ProxyServer` struct is the main entry point for the proxy server:

- It binds to a socket address and listens for incoming connections
- For each connection, it creates a new service function to handle requests
- The `handle_request` function processes each HTTP request
- The `handle_regular_request` function forwards requests to upstream servers

### 2. Request Processing Pipeline

```
┌────────────────┐     ┌─────────────────┐     ┌────────────────────┐     ┌─────────────────┐
│                │     │                 │     │                    │     │                 │
│ Parse Request  │────▶│ Apply Middleware│────▶│  Forward Request   │────▶│ Process Response│
│                │     │                 │     │                    │     │                 │
└────────────────┘     └─────────────────┘     └────────────────────┘     └─────────────────┘
```

1. **Parse Request**: Extract information from the incoming request
2. **Apply Middleware**: Authentication, rate limiting, logging, etc.
3. **Forward Request**: Send the request to the upstream server
4. **Process Response**: Return the response to the client

### 3. Configuration (`src/config/settings.rs`)

The `ProxyConfig` struct defines the configuration for the proxy server:

- `listen_addr`: The address the server listens on
- `log_level`: The logging level
- `upstream`: Configuration for upstream servers
- `request_timeout`: Maximum time to wait for a response
- `max_body_size`: Maximum size of request/response bodies

### 4. Middleware System (`src/proxy/middleware/`)

The middleware system allows you to intercept and modify requests and responses:

- **Authentication**: Verify API keys or other credentials
- **Rate Limiting**: Prevent abuse by limiting requests
- **Logging**: Record request/response information

### 5. Upstream Management (`src/proxy/upstream/`)

The upstream management system handles connections to backend servers:

- **Client**: Sends requests to upstream servers
- **Connection Pool**: Reuses connections for performance
- **Health Checker**: Monitors the health of upstream servers

### 6. Logging System (`src/logging/mod.rs`)

The logging system provides comprehensive logging capabilities:

- Structured logging via the `tracing` crate
- Environment-based configuration
- Console and file output options

## Data Flow

1. **Request Ingress**
   - Client sends HTTP request to proxy server
   - Proxy server parses the request and creates a `RequestData` struct

2. **Middleware Processing**
   - Request passes through middleware chain
   - Each middleware can modify the request or abort processing

3. **Upstream Forwarding**
   - Proxy server forwards the request to the upstream server
   - Connection pool is used to manage connections

4. **Response Processing**
   - Upstream server sends response
   - Proxy server processes the response and creates a `ResponseData` struct

5. **Response Egress**
   - Response is sent back to client
   - Transaction is logged

## Error Handling

The error handling system is defined in `src/error/mod.rs`:

- Custom `Error` enum with various error types
- Integration with `thiserror` for error messages
- Proper error propagation throughout the codebase

## Asynchronous Design

The proxy server is built on top of Tokio for asynchronous I/O:

- Uses `hyper` for HTTP server/client functionality
- Implements non-blocking I/O for high concurrency
- Uses `tokio::time` for timeouts

## Thread Safety

The proxy server is designed to be thread-safe:

- Uses `Arc` and `Mutex` for shared state
- Implements `Clone` for shared components
- Uses `tokio::sync` primitives for asynchronous synchronization

## Extension Points

The proxy server can be extended in several ways:

1. **New Middleware**: Implement custom middleware for additional functionality
2. **Custom Logging**: Extend the logging system for specific requirements
3. **Alternative Backends**: Support different types of upstream servers
4. **Protocol Support**: Add support for additional protocols (WebSockets, HTTP/2, etc.)

## Security Considerations

The proxy server includes several security features:

- **TLS Support**: Via the `rustls` and `tokio-rustls` crates
- **Authentication**: API key verification in the auth middleware
- **Rate Limiting**: Prevent abuse through the rate limiting middleware