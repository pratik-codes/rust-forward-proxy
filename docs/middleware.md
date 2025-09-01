# Middleware System

This document provides detailed information about the middleware system in the Rust Forward Proxy.

## Overview

Middleware are components that intercept HTTP requests and responses in the processing pipeline. They can perform various tasks such as authentication, logging, and rate limiting. The middleware system follows a chainable pattern where each middleware is executed in sequence.

## Available Middleware

The Rust Forward Proxy includes several built-in middleware:

### 1. Authentication Middleware (`auth.rs`)

Provides API key-based authentication for requests.

#### Features

- Bearer token authentication via the `Authorization` header
- Optional authentication (can be disabled)
- Returns a 401 Unauthorized response when authentication fails

#### Usage

```rust
// Create an authentication middleware with a required API key
let auth_middleware = AuthMiddleware::new(Some("your-api-key".to_string()));

// Check if a request is authenticated
match auth_middleware.authenticate(&request) {
    Ok(true) => {
        // Request is authenticated, continue processing
    }
    Ok(false) => {
        // Authentication failed, return unauthorized response
        return auth_middleware.create_unauthorized_response();
    }
    Err(_) => {
        // Error during authentication, handle error
    }
}
```

#### Configuration

The authentication middleware can be configured in the following ways:

- **API Key**: Set to `Some(api_key)` to enable authentication, or `None` to disable it
- **Token Format**: Currently supports Bearer tokens in the format `Bearer your-api-key`

### 2. Rate Limiting Middleware (`rate_limit.rs`)

Prevents abuse by limiting the number of requests from a single client within a specified time window.

#### Features

- Configurable request limits and time windows
- Client identification based on IP address or custom identifiers
- Thread-safe request counting using Mutex

#### Usage

```rust
use std::time::Duration;

// Create a rate limiting middleware
// Allow 100 requests per client in a 60 second window
let rate_limiter = RateLimitMiddleware::new(100, Duration::from_secs(60));

// Check if a request is within rate limits
match rate_limiter.check_rate_limit("client_identifier") {
    Ok(true) => {
        // Request is within rate limits, continue processing
    }
    Ok(false) => {
        // Rate limit exceeded, return 429 Too Many Requests
        return Response::builder()
            .status(StatusCode::TOO_MANY_REQUESTS)
            .body(Body::from("Rate limit exceeded"))
            .unwrap();
    }
    Err(_) => {
        // Error during rate limit check, handle error
    }
}
```

#### Configuration

The rate limiting middleware can be configured with:

- **Max Requests**: The maximum number of requests allowed within the time window
- **Time Window**: The duration of the rate limiting window (e.g., 60 seconds)

### 3. Logging Middleware (`logging.rs`)

Records detailed information about requests and responses.

#### Features

- Request method, URI, and headers logging
- Response status code and timing information
- Asynchronous logging to avoid blocking the request pipeline

#### Usage

```rust
// Create a logging middleware
let logging_middleware = LoggingMiddleware::new();

// Log an incoming request
logging_middleware.log_request(&request).await;

// Record the start time
let start_time = std::time::Instant::now();

// Process the request and get a response
// ...

// Log the response with duration
let duration = start_time.elapsed();
logging_middleware.log_response(&response, duration).await;
```

## Creating Custom Middleware

You can create your own middleware by following these steps:

1. Create a new struct to represent your middleware
2. Implement the necessary methods for request and response processing
3. Integrate your middleware into the request processing pipeline

### Example: Custom Header Middleware

```rust
pub struct CustomHeaderMiddleware {
    header_name: String,
    header_value: String,
}

impl CustomHeaderMiddleware {
    pub fn new(header_name: String, header_value: String) -> Self {
        Self {
            header_name,
            header_value,
        }
    }
    
    pub fn add_header(&self, request: &mut Request<Body>) {
        request.headers_mut().insert(
            HeaderName::from_str(&self.header_name).unwrap(),
            HeaderValue::from_str(&self.header_value).unwrap(),
        );
    }
}
```

## Middleware Chain

While the current implementation doesn't have a formal middleware chain, you can create one by calling each middleware in sequence in the request handler:

```rust
async fn process_request(req: Request<Body>) -> Result<Response<Body>> {
    // Apply authentication middleware
    if !auth_middleware.authenticate(&req)? {
        return Ok(auth_middleware.create_unauthorized_response());
    }
    
    // Apply rate limiting middleware
    if !rate_limit_middleware.check_rate_limit(&client_id)? {
        return Ok(Response::builder()
            .status(StatusCode::TOO_MANY_REQUESTS)
            .body(Body::from("Rate limit exceeded"))
            .unwrap());
    }
    
    // Log the request
    logging_middleware.log_request(&req).await;
    
    // Forward the request to the upstream server
    // ...
    
    Ok(response)
}
```

## Best Practices

1. **Middleware Order**: Consider the order in which middleware are applied. Authentication should typically come before other middleware.

2. **Error Handling**: Each middleware should handle its own errors gracefully and decide whether to continue the request processing or abort it.

3. **Performance**: Middleware should be efficient and avoid blocking operations. Use asynchronous code where appropriate.

4. **State Management**: If your middleware needs to maintain state, ensure it's thread-safe using appropriate synchronization primitives.

5. **Configuration**: Make your middleware configurable so it can be adapted to different scenarios.

## Future Improvements

1. **Formal Middleware Chain**: Implement a more structured middleware chain with proper ordering and error propagation.

2. **Middleware Registry**: Create a registry for dynamically registering and configuring middleware.

3. **Conditional Middleware**: Allow middleware to be applied conditionally based on request attributes.

4. **Metrics Collection**: Add middleware for collecting performance metrics.

5. **Request/Response Modification**: Enhance middleware to modify both requests and responses.