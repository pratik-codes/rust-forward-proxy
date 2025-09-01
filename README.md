# Rust Forward Proxy

A high-performance HTTP/HTTPS forward proxy server written in Rust.

## Features

- HTTP/HTTPS proxy support
- Request/response logging
- Thread-safe architecture
- Production-grade logging system

## Documentation

- [Codebase Documentation](./docs/documentation.md)
- [Usage Guide](./docs/usage.md)

## Logging System

This project uses a production-grade logging setup with the following features:

### Dependencies
- `log` crate for basic logging macros
- `tracing` for structured logging and async support
- `tracing-subscriber` for log formatting and output

### Initialization

The logging system is initialized in `main.rs` using one of these methods:

```rust
// Initialize with environment variable support (recommended)
init_logger_with_env();

// Initialize with custom level
init_logger_with_level(Level::DEBUG);

// Initialize with default settings
init_logger();
```

### Environment Configuration

Set the `RUST_LOG` environment variable to control log levels:

```bash
# Set log level for the entire application
export RUST_LOG=info

# Set log level for specific modules
export RUST_LOG=rust_forward_proxy=debug,tracing=warn

# Run with debug logging
RUST_LOG=debug cargo run
```

### Usage

#### Using Log Macros (Recommended)

```rust
use crate::{log_info, log_error, log_debug, log_warning, log_trace};

// Log different levels
log_info!("Server started on port 8080");
log_error!("Failed to connect to upstream server");
log_debug!("Processing request: {}", request_id);
log_warning!("High memory usage detected");
log_trace!("Entering function: handle_request");

// Log proxy transactions
log_proxy_transaction!(&proxy_log_entry);
```

#### Using Tracing Macros

```rust
use tracing::{info, error, debug, warn, trace};

info!("Server started on port 8080");
error!("Failed to connect to upstream server");
debug!("Processing request: {}", request_id);
warn!("High memory usage detected");
trace!("Entering function: handle_request");
```

#### Using Log Crate Macros

```rust
use log::{info, error, debug, warn, trace};

info!("Server started on port 8080");
error!("Failed to connect to upstream server");
debug!("Processing request: {}", request_id);
warn!("High memory usage detected");
trace!("Entering function: handle_request");
```

### Log Output

The logging system provides rich console output including:
- Timestamps
- Log levels with color coding
- Thread IDs and names
- File names and line numbers
- Structured JSON output for transactions

Example output:
```
2024-01-15T10:30:45.123Z INFO  [thread_id=1] rust_forward_proxy::main: Starting Forward Proxy Tutorial
2024-01-15T10:30:45.124Z INFO  [thread_id=1] rust_forward_proxy::main: Forward Proxy Server starting up
2024-01-15T10:30:45.125Z INFO  [thread_id=1] rust_forward_proxy::server: Proxy server starting on 127.0.0.1:8080
```

## Installation

```bash
git clone <repository-url>
cd rust-forward-proxy
cargo build --release
```

## Usage

```bash
# Run with default settings
cargo run

# Run with custom log level
RUST_LOG=debug cargo run

# Test the proxy
curl -x http://127.0.0.1:8080 http://httpbin.org/get
```

## Configuration

The proxy server listens on `127.0.0.1:8080` by default. You can modify the address in `src/main.rs`.

## License

MIT License
