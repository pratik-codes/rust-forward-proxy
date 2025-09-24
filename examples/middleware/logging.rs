//! Logging middleware for the proxy server

use crate::log_info;
use hyper::{Request, Response, Body};

/// Middleware for logging requests and responses
pub struct LoggingMiddleware;

impl LoggingMiddleware {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn log_request(&self, req: &Request<Body>) {
        let method = req.method().to_string();
        let uri = req.uri().to_string();
        log_info!("Request: {} {}", method, uri);
    }
    
    pub async fn log_response(&self, res: &Response<Body>, duration: std::time::Duration) {
        let status = res.status();
        let duration_ms = duration.as_millis();
        log_info!("Response: {} ({}ms)", status, duration_ms);
    }
}
