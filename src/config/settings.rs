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
    
    /// Redis configuration
    pub redis: RedisConfig,
    
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

/// Redis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    /// Redis server URL (format: redis://[username:password@]host:port[/database])
    pub url: String,
    
    /// Redis connection pool size
    pub pool_size: u32,
    
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    
    /// Command timeout in seconds
    pub command_timeout: u64,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1:8080".parse().unwrap(),
            log_level: "info".to_string(),
            upstream: UpstreamConfig::default(),
            redis: RedisConfig::default(),
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

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://redis:6379".to_string()),
            pool_size: 10,
            connection_timeout: 5,
            command_timeout: 10,
        }
    }
}
