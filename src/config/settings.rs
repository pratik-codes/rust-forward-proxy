//! Proxy server configuration settings

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// Main configuration for the proxy server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// Server listening address
    pub listen_addr: SocketAddr,
    
    /// Log level configuration
    pub log_level: String,
    
    /// Upstream server configuration
    pub upstream: UpstreamConfig,
    
    /// Request timeout in seconds
    pub request_timeout: u64,
    
    /// Maximum request body size in bytes
    pub max_body_size: usize,
}

/// Upstream server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamConfig {
    /// Upstream server URL
    pub url: String,
    
    /// Connection timeout in seconds
    pub connect_timeout: u64,
    
    /// Keep-alive timeout in seconds
    pub keep_alive_timeout: u64,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1:8080".parse().unwrap(),
            log_level: "info".to_string(),
            upstream: UpstreamConfig::default(),
            request_timeout: 30,
            max_body_size: 1024 * 1024, // 1MB
        }
    }
}

impl Default for UpstreamConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:3000".to_string(),
            connect_timeout: 5,
            keep_alive_timeout: 60,
        }
    }
}
