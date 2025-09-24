# Logging Level Changes

This document describes the changes made to the logging system to differentiate between HTTP/HTTPS requests and CONNECT requests based on log levels.

## Summary

The logging system has been modified so that:
- **HTTP and HTTPS requests** are logged at **INFO level** (visible in production)
- **CONNECT requests** are logged at **DEBUG level** (hidden in production, visible in development)

## Changes Made

### 1. Modified `src/utils/logging.rs`

#### `log_incoming_request()` Function
**Before:**
```rust
pub fn log_incoming_request(method: &str, uri: &str, remote_addr: &SocketAddr) {
    info!("📥 {} {} from {}", method, uri, remote_addr.ip());
    log_debug!("🔍 REQUEST DETAILS:\n  Method: {}\n  URI: {}\n  Remote: {}", 
               method, uri, remote_addr);
}
```

**After:**
```rust
pub fn log_incoming_request(method: &str, uri: &str, remote_addr: &SocketAddr) {
    if method == "CONNECT" {
        // CONNECT requests at DEBUG level
        log_debug!("🔐 {} {} from {}", method, uri, remote_addr.ip());
        log_debug!("🔍 CONNECT DETAILS:\n  Method: {}\n  URI: {}\n  Remote: {}", 
                   method, uri, remote_addr);
    } else {
        // HTTP/HTTPS requests at INFO level
        info!("📥 {} {} from {}", method, uri, remote_addr.ip());
        log_debug!("🔍 REQUEST DETAILS:\n  Method: {}\n  URI: {}\n  Remote: {}", 
                   method, uri, remote_addr);
    }
}
```

#### `log_connect_request()` Function
**Before:**
```rust
pub fn log_connect_request(uri: &str) {
    info!("🔐 CONNECT to {} (will intercept)", uri);
    log_debug!("🔐 CONNECT REQUEST:\n  Target: {}\n  Will intercept HTTPS traffic for full visibility", uri);
}
```

**After:**
```rust
pub fn log_connect_request(uri: &str) {
    log_debug!("🔐 CONNECT to {} (will intercept)", uri);
    log_debug!("🔐 CONNECT REQUEST:\n  Target: {}\n  Will intercept HTTPS traffic for full visibility", uri);
}
```

#### `log_connect_success()` Function
**Before:**
```rust
pub fn log_connect_success(host: &str, port: u16, connect_time: u128) {
    info!("✅ Tunnel established to {}:{} ({}ms)", host, port, connect_time);
    log_debug!("✅ CONNECT SUCCESS:\n  Target: {}:{}\n  Connect Time: {}ms\n  Setting up bidirectional tunnel", 
              host, port, connect_time);
}
```

**After:**
```rust
pub fn log_connect_success(host: &str, port: u16, connect_time: u128) {
    log_debug!("✅ Tunnel established to {}:{} ({}ms)", host, port, connect_time);
    log_debug!("✅ CONNECT SUCCESS:\n  Target: {}:{}\n  Connect Time: {}ms\n  Setting up bidirectional tunnel", 
              host, port, connect_time);
}
```

#### `log_connect_failure()` Function
**Before:**
```rust
pub fn log_connect_failure(host: &str, port: u16, connect_time: u128, error: &str) {
    info!("❌ CONNECT failed to {}:{} ({}ms): {}", host, port, connect_time, error);
    log_debug!("❌ CONNECT FAILURE:\n  Target: {}:{}\n  Time: {}ms\n  Error: {}", 
              host, port, connect_time, error);
}
```

**After:**
```rust
pub fn log_connect_failure(host: &str, port: u16, connect_time: u128, error: &str) {
    log_debug!("❌ CONNECT failed to {}:{} ({}ms): {}", host, port, connect_time, error);
    log_debug!("❌ CONNECT FAILURE:\n  Target: {}:{}\n  Time: {}ms\n  Error: {}", 
              host, port, connect_time, error);
}
```

### 2. Modified `src/proxy/server.rs`

#### `handle_connect_request()` Function
**Before:**
```rust
log_info!("🔍 CONNECT {}:{} - INTERCEPTING (will decrypt and log HTTPS)", host, port);
```

**After:**
```rust
log_debug!("🔍 CONNECT {}:{} - INTERCEPTING (will decrypt and log HTTPS)", host, port);
```

### 3. Modified `src/main.rs`

#### HTTPS Interception Logging
**Before:**
```rust
log_info!("🔍 HTTPS interception enabled - CONNECT requests to port 443 will be intercepted");
```

**After:**
```rust
tracing::debug!("🔍 HTTPS interception enabled - CONNECT requests to port 443 will be intercepted");
```

## Behavior Changes

### Production Mode (INFO Level)
```bash
export RUST_LOG=info
cargo run
```

**Visible:**
- ✅ HTTP requests: `📥 GET http://example.com from 127.0.0.1`
- ✅ HTTPS intercepted content: `🔍 INTERCEPTED HTTPS: GET https://example.com`
- ✅ HTTP responses: `✅ GET /path → 200 (150ms)`
- ✅ Upstream responses: `📤 Upstream response: 200 (120ms)`

**Hidden:**
- ❌ CONNECT requests: `🔐 CONNECT example.com:443 from 127.0.0.1`
- ❌ Tunnel establishment: `✅ Tunnel established to example.com:443 (50ms)`
- ❌ CONNECT failures: `❌ CONNECT failed to example.com:443 (1000ms): Connection timeout`

### Development Mode (DEBUG Level)
```bash
export RUST_LOG=debug
cargo run
```

**Visible:**
- ✅ All HTTP/HTTPS requests and responses (as above)
- ✅ All CONNECT requests and tunnel operations
- ✅ Detailed request/response information
- ✅ Internal proxy operations

## Log Level Matrix

| Request Type | INFO Level | DEBUG Level |
|--------------|------------|-------------|
| HTTP GET/POST/PUT/DELETE | ✅ Visible | ✅ Visible |
| HTTPS Intercepted Content | ✅ Visible | ✅ Visible |
| CONNECT Requests | ❌ Hidden | ✅ Visible |
| CONNECT Tunnel Setup | ❌ Hidden | ✅ Visible |
| CONNECT Failures | ❌ Hidden | ✅ Visible |
| Request Details | ❌ Hidden | ✅ Visible |
| Response Headers | ❌ Hidden | ✅ Visible |

## Use Cases

### Production Environment
```bash
# Clean logs showing only actual HTTP/HTTPS traffic
export RUST_LOG=info
cargo run
```
- Perfect for monitoring actual web traffic
- CONNECT noise is filtered out
- Focus on the content being proxied

### Development Environment
```bash
# Verbose logs showing all proxy operations
export RUST_LOG=debug
cargo run
```
- See everything including tunnel establishment
- Debug CONNECT failures
- Understand complete proxy flow

### Module-Specific Logging
```bash
# Fine-grained control
export RUST_LOG=rust_forward_proxy=debug,hyper=info,tokio=warn
cargo run
```

## Testing

Run the test script to verify the changes:
```bash
./scripts/test_logging_levels.sh
```

This script demonstrates the difference between INFO and DEBUG level logging and provides examples of what will be visible at each level.

## Migration Notes

### For Existing Users
- **No Breaking Changes**: Existing functionality remains the same
- **Behavioral Change**: CONNECT requests are now less verbose at INFO level
- **Recommended**: Use DEBUG level during development, INFO level in production

### Configuration Examples
```bash
# Production (clean logs)
export RUST_LOG=info

# Development (verbose logs)  
export RUST_LOG=debug

# Custom (proxy debug, others info)
export RUST_LOG=rust_forward_proxy=debug,hyper=info
```

---

*Last updated: September 24, 2025*
*Changes implemented to improve production log clarity while maintaining development visibility*
