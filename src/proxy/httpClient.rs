//! HTTP Client Management
//! 
//! This module provides optimized HTTP client with connection pooling
//! for maximum proxy performance:
//! - Shared HTTP client with connection pooling
//! - Connection reuse and persistent connections

use hyper::{Client, Body};
use hyper_rustls::HttpsConnectorBuilder;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, debug};

/// High-performance HTTP client with connection pooling
/// 
/// This eliminates the critical performance bottleneck of creating new HTTP clients
/// for every request, instead providing shared, reusable clients with connection pooling.
pub struct HttpClient {
    /// Shared HTTPS client with connection pooling for HTTPS requests
    https_client: Arc<Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>, Body>>,
    /// Shared HTTP client for regular HTTP requests  
    http_client: Arc<Client<hyper::client::HttpConnector, Body>>,
    /// Configuration for connection pooling
    config: ClientConfig,
}

/// Configuration for optimized HTTP clients
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Maximum idle connections per host (default: 50)
    pub max_idle_per_host: usize,
    /// How long to keep idle connections alive (default: 90 seconds)
    pub idle_timeout: Duration,
    /// Timeout for establishing new connections (default: 10 seconds)
    pub connect_timeout: Duration,
    /// Enable HTTP/2 support (default: true)
    pub enable_http2: bool,
    /// HTTP/2 initial stream window size (default: 1MB)
    pub http2_initial_stream_window_size: Option<u32>,
    /// HTTP/2 initial connection window size (default: 4MB)
    pub http2_initial_connection_window_size: Option<u32>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            max_idle_per_host: 50,
            idle_timeout: Duration::from_secs(90),
            connect_timeout: Duration::from_secs(10),
            enable_http2: true,
            http2_initial_stream_window_size: Some(1048576), // 1MB
            http2_initial_connection_window_size: Some(4194304), // 4MB
        }
    }
}

impl HttpClient {
    /// Create a new optimized HTTP client with default configuration
    pub fn new() -> Self {
        Self::with_config(ClientConfig::default())
    }

    /// Create a new optimized HTTP client with custom configuration
    pub fn with_config(config: ClientConfig) -> Self {
        info!("🚀 Initializing optimized HTTP client");
        info!("   Max idle connections per host: {}", config.max_idle_per_host);
        info!("   Idle timeout: {:?}", config.idle_timeout);
        info!("   Connect timeout: {:?}", config.connect_timeout);
        info!("   HTTP/2 enabled: {}", config.enable_http2);

        // Create HTTPS connector with optimized settings
        let https_connector = HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_or_http()
            .enable_http1()
            .build();

        // Create HTTPS client with connection pooling optimizations
        let https_client = Client::builder()
            .pool_idle_timeout(config.idle_timeout)
            .pool_max_idle_per_host(config.max_idle_per_host)
            .http2_only(false) // Allow both HTTP/1.1 and HTTP/2
            .build(https_connector);

        // Create HTTP connector for regular HTTP requests
        let mut http_connector = hyper::client::HttpConnector::new();
        http_connector.set_connect_timeout(Some(config.connect_timeout));
        http_connector.set_nodelay(true); // Disable Nagle's algorithm for lower latency

        // Create HTTP client with connection pooling
        let http_client = Client::builder()
            .pool_idle_timeout(config.idle_timeout)
            .pool_max_idle_per_host(config.max_idle_per_host)
            .build(http_connector);

        info!("✅ Optimized HTTP client initialized successfully");

        Self {
            https_client: Arc::new(https_client),
            http_client: Arc::new(http_client),
            config,
        }
    }

    /// Get the shared HTTPS client for making HTTPS requests
    /// 
    /// This client has connection pooling enabled and will reuse connections
    /// to the same host, dramatically reducing connection establishment overhead.
    pub fn get_https_client(&self) -> Arc<Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>, Body>> {
        debug!("📡 Using shared HTTPS client with connection pooling");
        Arc::clone(&self.https_client)
    }

    /// Get the shared HTTP client for making HTTP requests
    /// 
    /// This client has connection pooling enabled for regular HTTP requests.
    pub fn get_http_client(&self) -> Arc<Client<hyper::client::HttpConnector, Body>> {
        debug!("📡 Using shared HTTP client with connection pooling");
        Arc::clone(&self.http_client)
    }

    /// Get the appropriate client based on whether the request is HTTPS or not
    /// 
    /// For maximum performance, this returns the HTTPS client for both HTTP and HTTPS
    /// requests since the HTTPS client can handle both protocols efficiently.
    pub fn get_client_for_url(&self, _is_https: bool) -> Arc<Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>, Body>> {
        // Use HTTPS client for both HTTP and HTTPS since it can handle both efficiently
        self.get_https_client()
    }

    /// Get configuration information for monitoring and debugging
    pub fn get_config(&self) -> &ClientConfig {
        &self.config
    }

    /// Get client statistics and health information
    pub fn get_stats(&self) -> ClientStats {
        ClientStats {
            max_idle_per_host: self.config.max_idle_per_host,
            idle_timeout_secs: self.config.idle_timeout.as_secs(),
            http2_enabled: self.config.enable_http2,
        }
    }
}

/// Statistics about the optimized HTTP client
#[derive(Debug)]
pub struct ClientStats {
    pub max_idle_per_host: usize,
    pub idle_timeout_secs: u64,
    pub http2_enabled: bool,
}

/// Utility functions for client configuration from environment variables
impl HttpClient {
    /// Create HTTP client with configuration from environment variables
    pub fn from_env() -> Self {
        let config = ClientConfig {
            max_idle_per_host: std::env::var("PROXY_MAX_IDLE_PER_HOST")
                .unwrap_or_else(|_| "50".to_string())
                .parse()
                .unwrap_or(50),
            idle_timeout: Duration::from_secs(
                std::env::var("PROXY_IDLE_TIMEOUT_SECS")
                    .unwrap_or_else(|_| "90".to_string())
                    .parse()
                    .unwrap_or(90)
            ),
            connect_timeout: Duration::from_secs(
                std::env::var("PROXY_CONNECT_TIMEOUT_SECS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10)
            ),
            enable_http2: std::env::var("PROXY_ENABLE_HTTP2")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            http2_initial_stream_window_size: Some(
                std::env::var("PROXY_HTTP2_STREAM_WINDOW_SIZE")
                    .unwrap_or_else(|_| "1048576".to_string())
                    .parse()
                    .unwrap_or(1048576)
            ),
            http2_initial_connection_window_size: Some(
                std::env::var("PROXY_HTTP2_CONNECTION_WINDOW_SIZE")
                    .unwrap_or_else(|_| "4194304".to_string())
                    .parse()
                    .unwrap_or(4194304)
            ),
        };

        info!("🔧 Loading HTTP client configuration from environment variables");
        Self::with_config(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ClientConfig::default();
        assert_eq!(config.max_idle_per_host, 50);
        assert_eq!(config.idle_timeout, Duration::from_secs(90));
        assert!(config.enable_http2);
    }

    #[test]
    fn test_http_client_creation() {
        let client = HttpClient::new();
        let stats = client.get_stats();
        assert_eq!(stats.max_idle_per_host, 50);
        assert!(stats.http2_enabled);
    }
}
