//! Logging utility functions for proxy operations

use crate::models::{ProxyLog, RequestData, ResponseData};
use crate::{log_debug, log_proxy_transaction};
use crate::logging::process_info;
use hyper::StatusCode;
use std::net::SocketAddr;
use tracing::info;

/// Log incoming request information
pub fn log_incoming_request(method: &str, uri: &str, remote_addr: &SocketAddr) {
    // Log based on request type
    if method == "CONNECT" {
        // CONNECT requests at DEBUG level
        log_debug!("🔐 {} {} from {}", method, uri, remote_addr.ip());
        log_debug!("🔍 CONNECT DETAILS:\n  Method: {}\n  URI: {}\n  Remote: {}", 
                   method, uri, remote_addr);
    } else {
        // HTTP/HTTPS requests at INFO level with process info
        info!("{} 📥 {} {} from {}", process_info(), method, uri, remote_addr.ip());
        log_debug!("{} 🔍 REQUEST DETAILS:\n  Method: {}\n  URI: {}\n  Remote: {}", 
                   process_info(), method, uri, remote_addr);
    }
}

/// Log CONNECT request details
pub fn log_connect_request(uri: &str) {
    // CONNECT requests at DEBUG level
    log_debug!("🔐 CONNECT to {} (will intercept)", uri);
    
    // Verbose DEBUG log for CONNECT details
    log_debug!("🔐 CONNECT REQUEST:\n  Target: {}\n  Will intercept HTTPS traffic for full visibility", uri);
}

/// Log successful CONNECT tunnel establishment
pub fn log_connect_success(host: &str, port: u16, connect_time: u128) {
    // CONNECT success at DEBUG level
    log_debug!("✅ Tunnel established to {}:{} ({}ms)", host, port, connect_time);
    
    // Verbose DEBUG log with full details
    log_debug!("✅ CONNECT SUCCESS:\n  Target: {}:{}\n  Connect Time: {}ms\n  Setting up bidirectional tunnel", 
              host, port, connect_time);
}

/// Log failed CONNECT attempt
pub fn log_connect_failure(host: &str, port: u16, connect_time: u128, error: &str) {
    // CONNECT failure at DEBUG level
    log_debug!("❌ CONNECT failed to {}:{} ({}ms): {}", host, port, connect_time, error);
    
    // Verbose DEBUG log with full details
    log_debug!("❌ CONNECT FAILURE:\n  Target: {}:{}\n  Time: {}ms\n  Error: {}", 
              host, port, connect_time, error);
}

/// Create and log CONNECT transaction
pub fn create_connect_transaction(
    request_data: &RequestData, 
    response_data: Option<ResponseData>, 
    error: Option<String>
) -> ProxyLog {
    let log_entry = ProxyLog {
        request: request_data.clone(),
        response: response_data,
        error,
    };
    
    // DEBUG: Log full transaction details
    if log_entry.error.is_some() {
        log_debug!("📋 CONNECT ERROR TRANSACTION:\n{:#?}", log_entry);
    } else {
        log_debug!("📋 CONNECT TRANSACTION:\n{:#?}", log_entry);
    }
    
    // Log the transaction
    log_proxy_transaction!(&log_entry);
    
    log_entry
}

/// Log HTTP request success
pub fn log_http_success(method: &str, path: &str, status: StatusCode, total_time: u128) {
    let current_pid = std::process::id();
    // Clean INFO log for successful HTTP request with process info
    info!("✅ PID {} completed {} {} → {} ({}ms)", current_pid, method, 
          path.chars().take(50).collect::<String>(), 
          status, total_time);
    
    // Verbose DEBUG log
    log_debug!("✅ HTTP SUCCESS (PID {}):\n  Method: {}\n  Path: {}\n  Status: {}\n  Time: {}ms", 
              current_pid, method, path, status, total_time);
    
    // Separator for INFO mode
    info!("PID {} ##################################\n", current_pid);
}

/// Log HTTP request failure
pub fn log_http_failure(method: &str, path: &str, total_time: u128, error: &anyhow::Error) {
    let current_pid = std::process::id();
    // Clean INFO log for failed HTTP request
    info!("❌ PID {} failed {} {} → ERROR ({}ms): {}", current_pid, method, 
          path.chars().take(50).collect::<String>(), 
          total_time, error);
    
    // Verbose DEBUG log
    log_debug!("❌ HTTP FAILURE (PID {}):\n  Method: {}\n  Path: {}\n  Time: {}ms\n  Error: {}", 
              current_pid, method, path, total_time, error);
    
    // Separator for INFO mode
    info!("PID {} ##################################\n", current_pid);
}

/// Log forwarding request details
pub fn log_forwarding_request(request_data: &RequestData) {
    // Clean INFO log for request forwarding
    info!("🔄 Forwarding {} to upstream", request_data.method);
    
    // Verbose DEBUG log
    log_debug!("🔄 FORWARDING REQUEST:\n  Method: {}\n  URL: {}\n  Headers: {}\n  Body Size: {} bytes", 
               request_data.method, request_data.url, request_data.headers.len(), request_data.body.len());
}
