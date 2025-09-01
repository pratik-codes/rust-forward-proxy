# Rust Forward Proxy Documentation

This document provides a detailed explanation of the Rust Forward Proxy codebase.

## Project Structure

The project is organized into the following modules:

- **`main.rs`**: The entry point of the application.
- **`lib.rs`**: The main library crate.
- **`cli`**: Handles command-line argument parsing.
- **`config`**: Defines the configuration for the proxy server.
- **`error`**: Defines the custom error type for the application.
- **`logging`**: Contains the logging setup and utility functions.
- **`models`**: Defines the data structures for requests, responses, and logs.
- **`proxy`**: Contains the core proxy logic.
  - **`middleware`**: Contains middleware for the proxy server.
  - **`upstream`**: Contains modules for managing upstream servers.
- **`utils`**: Contains utility functions.

## Code Details

### `main.rs`

The `main.rs` file is the entry point for the `rust-forward-proxy` executable. It performs the following actions:

1.  **Initializes Logging**: It calls `init_logger_with_env()` to set up the logging framework. The log level is determined by the `RUST_LOG` environment variable.
2.  **Loads Configuration**: It loads the default `ProxyConfig`.
3.  **Starts the Server**: It creates a new `ProxyServer` instance and starts it. The server listens on the address specified in the configuration.

### `lib.rs`

This file serves as the root of the `rust_forward_proxy` library. It defines the module hierarchy and re-exports key components for easy access from other parts of the application.

### `cli/mod.rs`

This module uses the `clap` crate to parse command-line arguments. The `Cli` struct defines the expected arguments, and the `load_config` function creates a `ProxyConfig` from the parsed arguments.

### `config/settings.rs`

This file defines the `ProxyConfig` and `UpstreamConfig` structs, which hold the configuration for the proxy server. These structs are deserialized from a configuration file (e.g., a TOML or JSON file) and provide a typed way to access configuration settings.

### `error/mod.rs`

This module defines the `Error` enum, which represents all possible errors that can occur in the application. It uses the `thiserror` crate to derive the `std::error::Error` trait and provide descriptive error messages.

### `logging/mod.rs`

This module sets up the logging framework for the application using the `tracing` and `log` crates. It provides functions to initialize the logger with different configurations and to log messages at various levels.

### `models/mod.rs`

This module defines the data models used throughout the application:

-   **`RequestData`**: Represents an incoming HTTP request, including its method, URL, headers, and body.
-   **`ResponseData`**: Represents an outgoing HTTP response, including its status code, headers, and body.
-   **`ProxyLog`**: A container for a `RequestData` and an optional `ResponseData` or error, used for logging the entire transaction.

### `proxy/server.rs`

This is the core of the proxy server. The `ProxyServer` struct contains the listening address and a shared logger. The `start` method binds to the specified address and starts listening for incoming connections.

For each incoming request, the `handle_request` function is called. This function:

1.  Parses the incoming `hyper::Request`.
2.  Creates a `RequestData` struct.
3.  Forwards the request to the upstream server.
4.  Receives the response from the upstream server.
5.  Creates a `ResponseData` struct.
6.  Logs the request and response.
7.  Sends the response back to the client.

### `proxy/middleware/`

This directory contains middleware that can be used to inspect and modify requests and responses.

-   **`auth.rs`**: An example of an authentication middleware that checks for an API key in the `Authorization` header.
-   **`logging.rs`**: A middleware for logging requests and responses.
-   **`rate_limit.rs`**: A middleware for rate-limiting requests based on the client's IP address.

### `proxy/upstream/`

This directory contains modules for managing connections to upstream servers.

-   **`client.rs`**: An HTTP client for sending requests to upstream servers.
-   **`connection_pool.rs`**: A connection pool for reusing connections to upstream servers.
-   **`health_check.rs`**: A health checker for monitoring the health of upstream servers.

### `utils/`

This directory contains various utility functions.

-   **`http.rs`**: Functions for working with HTTP headers, such as checking for hop-by-hop headers.
-   **`url.rs`**: Functions for parsing and manipulating URLs.
-   **`time.rs`**: Functions for working with timestamps and durations.