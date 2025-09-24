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
    
    /// TLS configuration for HTTPS interception
    pub tls: TlsConfig,
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

/// TLS configuration for HTTPS interception
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Enable TLS server (HTTPS proxy)
    pub enabled: bool,
    
    /// HTTPS listening address (separate from HTTP)
    pub https_listen_addr: SocketAddr,
    
    /// Path to TLS certificate file (.pem or .crt)
    pub cert_path: String,
    
    /// Path to TLS private key file (.pem or .key)
    pub key_path: String,
    
    /// Enable HTTPS interception (decrypt and re-encrypt)
    pub interception_enabled: bool,
    
    /// Generate self-signed certificate if cert files don't exist
    pub auto_generate_cert: bool,
    
    /// Certificate organization name for auto-generated certs
    pub cert_organization: String,
    
    /// Certificate common name (hostname) for auto-generated certs
    pub cert_common_name: String,
    
    /// Certificate validity period in days
    pub cert_validity_days: u32,
    
    /// Minimum TLS version (1.2 or 1.3)
    pub min_tls_version: String,
    
    /// Skip upstream certificate verification (for testing)
    pub skip_upstream_cert_verify: bool,
    
    /// Path to root CA certificate file for trust store
    pub root_ca_cert_path: Option<String>,
    
    /// Path to CA certificate for signing domain certificates
    pub ca_cert_path: Option<String>,
    
    /// Path to CA private key for signing domain certificates
    pub ca_key_path: Option<String>,
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
            tls: TlsConfig::default(),
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

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default for backward compatibility
            https_listen_addr: "127.0.0.1:8443".parse().unwrap(),
            cert_path: "certs/proxy.crt".to_string(),
            key_path: "certs/proxy.key".to_string(),
            interception_enabled: true, // Enable interception when TLS is enabled
            auto_generate_cert: true, // Auto-generate for development
            cert_organization: "Rust Forward Proxy".to_string(),
            cert_common_name: "proxy.local".to_string(),
            cert_validity_days: 365,
            min_tls_version: "1.2".to_string(),
            skip_upstream_cert_verify: false, // Verify upstream certs by default
            root_ca_cert_path: Some("ca-certs/securly_ca.crt".to_string()),
            ca_cert_path: Some("ca-certs/rootCA.crt".to_string()),
            ca_key_path: Some("ca-certs/rootCA.key".to_string()),
        }
    }
}
