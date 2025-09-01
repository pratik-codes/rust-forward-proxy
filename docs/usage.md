# Usage Guide

This document explains how to use the Rust Forward Proxy with comprehensive examples and configuration options.

## Quick Start

### Running the Proxy

```bash
# Start with default INFO logging
cargo run

# Start with verbose DEBUG logging  
RUST_LOG=debug cargo run

# Start with custom address (modify src/main.rs)
```

The proxy server starts on `127.0.0.1:8080` by default.

### Testing the Proxy

```bash
# Test HTTP request
curl -x http://127.0.0.1:8080 http://httpbin.org/get

# Test HTTPS request (tunneling)
curl -x http://127.0.0.1:8080 https://httpbin.org/get

# Test with verbose output
curl -x http://127.0.0.1:8080 -v https://www.google.com
```

## Browser Configuration

Configure your browser to use the proxy for both HTTP and HTTPS traffic.

### Mozilla Firefox

1. Go to **Settings** → **General** → **Network Settings** → **Settings**
2. Select **Manual proxy configuration**
3. Set **HTTP Proxy**: `127.0.0.1` Port: `8080`
4. Check **"Use this proxy server for all protocols"**
5. Click **OK**

### Google Chrome

Chrome uses system proxy settings:

#### Windows
1. **Settings** → **Network & Internet** → **Proxy**
2. Enable **"Use a proxy server"**
3. Address: `127.0.0.1` Port: `8080`
4. Click **Save**

#### macOS  
1. **System Preferences** → **Network**
2. Select your network → **Advanced** → **Proxies**
3. Check **"Web Proxy (HTTP)"** and **"Secure Web Proxy (HTTPS)"**
4. Server: `127.0.0.1` Port: `8080`
5. Click **OK** → **Apply**

### Safari (macOS)
Safari uses system proxy settings (same as Chrome instructions above).

## Proxy Behavior

### HTTP Requests
- **Full Interception**: All HTTP requests are intercepted, logged, and forwarded
- **Header Processing**: Hop-by-hop headers are filtered
- **Body Extraction**: POST/PUT/PATCH bodies are extracted and logged
- **Response Processing**: Full response data captured and logged

### HTTPS Requests (CONNECT Method)
- **Tunnel Establishment**: Uses HTTP CONNECT method for HTTPS traffic
- **Raw TCP Forwarding**: Encrypted traffic passes through without decryption
- **Connection Logging**: Tunnel establishment and closure logged
- **No Content Interception**: HTTPS content remains encrypted end-to-end

## Logging Output

### INFO Level (Production Monitoring)
```bash
RUST_LOG=info cargo run
```

**Clean, single-line logs:**
```
📥 GET http://httpbin.org/get from 127.0.0.1
🔄 Forwarding GET to upstream
📤 Upstream response: 200 (156ms)
✅ GET /get → 200 OK (158ms)
##################################

📥 CONNECT httpbin.org:443 from 127.0.0.1  
🔐 CONNECT tunnel to httpbin.org:443
✅ Tunnel established to httpbin.org:443 (45ms)
🔌 Tunnel completed for httpbin.org:443
##################################
```

### DEBUG Level (Development & Troubleshooting)
```bash
RUST_LOG=debug cargo run
```

**Verbose, detailed logs:**
```
🔍 REQUEST DETAILS:
  Method: GET
  URI: http://httpbin.org/get
  Remote: 127.0.0.1:54321

🔄 FORWARDING REQUEST:
  Method: GET
  URL: http://httpbin.org/get
  Headers: 10
  Body Size: 0 bytes

📤 UPSTREAM RESPONSE:
  Status: 200 OK
  Content-Type: application/json
  Time: 156ms

📋 HTTP TRANSACTION:
{
  "request": {
    "method": "GET",
    "url": "http://httpbin.org/get",
    "headers": { ... },
    "client_ip": "127.0.0.1",
    "timestamp": "2025-01-01T12:00:00Z"
  },
  "response": {
    "status_code": 200,
    "headers": { ... },
    "response_time_ms": 156
  }
}
```

## Advanced Usage

### Testing Different Request Types

```bash
# GET request
curl -x http://127.0.0.1:8080 http://httpbin.org/get

# POST with JSON data
curl -x http://127.0.0.1:8080 -X POST http://httpbin.org/post \
  -H "Content-Type: application/json" \
  -d '{"key":"value"}'

# POST with form data  
curl -x http://127.0.0.1:8080 -X POST http://httpbin.org/post \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "name=test&value=123"

# Request with custom headers
curl -x http://127.0.0.1:8080 http://httpbin.org/headers \
  -H "X-Custom-Header: test-value" \
  -H "Authorization: Bearer token123"

# HTTPS requests (automatically tunneled)
curl -x http://127.0.0.1:8080 https://httpbin.org/get
curl -x http://127.0.0.1:8080 https://www.google.com
curl -x http://127.0.0.1:8080 https://api.github.com/user
```

### Browser Testing

Once configured, test with these URLs in your browser:
- `http://httpbin.org/get` - HTTP request (intercepted)
- `https://httpbin.org/get` - HTTPS request (tunneled)
- `https://www.google.com` - Real-world HTTPS (tunneled)

## Environment Configuration

### Logging Levels
```bash
# Minimal logging
RUST_LOG=error cargo run

# Standard production
RUST_LOG=info cargo run  

# Development debugging
RUST_LOG=debug cargo run

# Maximum verbosity
RUST_LOG=trace cargo run

# Module-specific logging
RUST_LOG=rust_forward_proxy=debug,hyper=info cargo run
```

### Custom Configuration

Modify `src/main.rs` to customize:
```rust
// Change listening address
let addr = SocketAddr::from(([0, 0, 0, 0], 3128)); // Listen on all interfaces, port 3128

// Change port only
let addr = SocketAddr::from(([127, 0, 0, 1], 9090)); // Port 9090
```

## Troubleshooting

### Common Issues

1. **"Connection refused"**
   - Ensure proxy is running: `cargo run`
   - Check correct address/port: `127.0.0.1:8080`

2. **HTTPS sites not loading**
   - Verify CONNECT tunneling is working
   - Check DEBUG logs for tunnel establishment
   - Ensure browser is configured for HTTPS proxy

3. **Slow performance**
   - Monitor DEBUG logs for response times
   - Check upstream server performance
   - Consider connection pooling enhancements

### Debugging Steps

1. **Enable DEBUG logging**: `RUST_LOG=debug cargo run`
2. **Check proxy logs** for request/response flow
3. **Test with curl** before browser testing
4. **Verify network connectivity** to upstream servers

### Log Analysis

Monitor these log patterns:
- `📥` - Incoming requests
- `🔄` - Request forwarding  
- `📤` - Upstream responses
- `✅` - Successful completions
- `❌` - Errors or failures
- `🔐` - CONNECT tunnel establishment
- `🔌` - Tunnel completion/closure

## Security Considerations

- **HTTPS Traffic**: Remains encrypted end-to-end through tunneling
- **HTTP Traffic**: Fully visible to proxy (as designed for monitoring)
- **Local Testing**: Default configuration binds to localhost only
- **Production Deployment**: See [Deployment Guide](./deployment.md) for security hardening