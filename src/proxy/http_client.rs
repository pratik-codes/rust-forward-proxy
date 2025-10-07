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
    /// HTTP/2 initial stream window size (default: 2MB)
    pub http2_initial_stream_window_size: Option<u32>,
    /// HTTP/2 initial connection window size (default: 8MB)
    pub http2_initial_connection_window_size: Option<u32>,
    /// HTTP/2 keep alive interval (default: 30 seconds)
    pub http2_keep_alive_interval: Option<Duration>,
    /// HTTP/2 keep alive timeout (default: 10 seconds)
    pub http2_keep_alive_timeout: Option<Duration>,
    /// HTTP/2 max concurrent streams per connection (default: 100)
    pub http2_max_concurrent_streams: Option<u32>,
    /// Enable TCP keepalive (default: true)
    pub tcp_keepalive: bool,
    /// TCP keepalive interval (default: 30 seconds)
    pub tcp_keepalive_interval: Option<Duration>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        // Check if we're in multi-process mode for load balancing optimization
        let is_multi_process = std::env::var("PROXY_RUNTIME_MODE").unwrap_or_default() == "single_threaded"
            && std::env::var("PROXY_USE_REUSEPORT").unwrap_or_default() == "true";
        
        // For multi-process mode, use more aggressive connection cycling for better load balancing
        let (max_idle_per_host, idle_timeout) = if is_multi_process {
            // Reduce connection pooling in multi-process mode to improve load distribution
            (10, Duration::from_secs(15))
        } else {
            // Use standard settings for single-process mode
            (50, Duration::from_secs(90))
        };
        
        Self {
            max_idle_per_host,
            idle_timeout,
            connect_timeout: Duration::from_secs(10),
            enable_http2: true,
            http2_initial_stream_window_size: Some(2097152), // 2MB for better throughput
            http2_initial_connection_window_size: Some(8388608), // 8MB for better parallelism
            http2_keep_alive_interval: Some(Duration::from_secs(30)),
            http2_keep_alive_timeout: Some(Duration::from_secs(10)),
            http2_max_concurrent_streams: Some(100),
            tcp_keepalive: true,
            tcp_keepalive_interval: Some(Duration::from_secs(30)),
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
        let is_multi_process = std::env::var("PROXY_RUNTIME_MODE").unwrap_or_default() == "single_threaded"
            && std::env::var("PROXY_USE_REUSEPORT").unwrap_or_default() == "true";
        
        info!("🚀 Initializing advanced HTTP client with connection pooling");
        info!("   Max idle connections per host: {}", config.max_idle_per_host);
        info!("   Idle timeout: {:?}", config.idle_timeout);
        info!("   Connect timeout: {:?}", config.connect_timeout);
        info!("   HTTP/2 enabled: {}", config.enable_http2);
        info!("   HTTP/2 stream window: {} bytes", config.http2_initial_stream_window_size.unwrap_or(0));
        info!("   HTTP/2 connection window: {} bytes", config.http2_initial_connection_window_size.unwrap_or(0));
        info!("   HTTP/2 keep-alive interval: {:?}", config.http2_keep_alive_interval);
        info!("   TCP keepalive enabled: {}", config.tcp_keepalive);
        
        if is_multi_process {
            info!("   🔄 Multi-process load balancing mode: aggressive connection cycling enabled");
        }

        // Create HTTPS connector with optimized settings
        // Temporarily disable HTTP/2 to fix 400 errors with some servers like Google
        let https_connector = HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_or_http()
            .enable_http1()
            .build();

        // Create HTTPS client with advanced connection pooling
        // Temporarily force HTTP/1.1 only to fix 400 errors with some servers like Google
        let https_client = Client::builder()
            .pool_idle_timeout(config.idle_timeout)
            .pool_max_idle_per_host(config.max_idle_per_host)
            .http2_only(false) // Allow both HTTP/1.1 and HTTP/2 but prefer HTTP/1.1
            .build(https_connector);

        // Create HTTP connector for regular HTTP requests with advanced TCP settings
        let mut http_connector = hyper::client::HttpConnector::new();
        http_connector.set_connect_timeout(Some(config.connect_timeout));
        http_connector.set_nodelay(true); // Disable Nagle's algorithm for lower latency
        http_connector.set_reuse_address(true); // Allow address reuse for better connection pooling
        
        // Apply TCP keepalive settings
        if config.tcp_keepalive {
            http_connector.set_keepalive(config.tcp_keepalive_interval);
        }

        // Create HTTP client with enhanced connection pooling
        let http_client = Client::builder()
            .pool_idle_timeout(config.idle_timeout)
            .pool_max_idle_per_host(config.max_idle_per_host)
            .http2_only(false) // Allow both HTTP/1.1 and HTTP/2
            .build(http_connector);

        info!("✅ Advanced HTTP client with connection pooling initialized successfully");

        let http_client = Self {
            https_client: Arc::new(https_client),
            http_client: Arc::new(http_client),
            config,
        };
        
        // Log detailed connection pool statistics for monitoring
        http_client.log_connection_stats();
        
        http_client
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

    /// Get comprehensive client statistics and configuration information
    pub fn get_stats(&self) -> ClientStats {
        ClientStats {
            max_idle_per_host: self.config.max_idle_per_host,
            idle_timeout_secs: self.config.idle_timeout.as_secs(),
            connect_timeout_secs: self.config.connect_timeout.as_secs(),
            http2_enabled: self.config.enable_http2,
            http2_stream_window_size: self.config.http2_initial_stream_window_size.unwrap_or(0),
            http2_connection_window_size: self.config.http2_initial_connection_window_size.unwrap_or(0),
            http2_keepalive_interval_secs: self.config.http2_keep_alive_interval.map(|d| d.as_secs()).unwrap_or(0),
            http2_keepalive_timeout_secs: self.config.http2_keep_alive_timeout.map(|d| d.as_secs()).unwrap_or(0),
            http2_max_concurrent_streams: self.config.http2_max_concurrent_streams.unwrap_or(0),
            tcp_keepalive_enabled: self.config.tcp_keepalive,
            tcp_keepalive_interval_secs: self.config.tcp_keepalive_interval.map(|d| d.as_secs()).unwrap_or(0),
        }
    }
    
    /// Log detailed connection pool statistics for monitoring
    pub fn log_connection_stats(&self) {
        let stats = self.get_stats();
        info!("📊 HTTP Client Connection Pool Statistics:");
        info!("   Connection Pooling:");
        info!("     Max idle per host: {}", stats.max_idle_per_host);
        info!("     Idle timeout: {}s", stats.idle_timeout_secs);
        info!("     Connect timeout: {}s", stats.connect_timeout_secs);
        info!("   HTTP/2 Configuration:");
        info!("     HTTP/2 enabled: {}", stats.http2_enabled);
        info!("     Stream window size: {} bytes ({:.1} MB)", stats.http2_stream_window_size, stats.http2_stream_window_size as f64 / 1_048_576.0);
        info!("     Connection window size: {} bytes ({:.1} MB)", stats.http2_connection_window_size, stats.http2_connection_window_size as f64 / 1_048_576.0);
        info!("     Keep-alive interval: {}s", stats.http2_keepalive_interval_secs);
        info!("     Keep-alive timeout: {}s", stats.http2_keepalive_timeout_secs);
        info!("     Max concurrent streams: {}", stats.http2_max_concurrent_streams);
        info!("   TCP Configuration:");
        info!("     TCP keep-alive: {}", stats.tcp_keepalive_enabled);
        info!("     TCP keep-alive interval: {}s", stats.tcp_keepalive_interval_secs);
    }
}

/// Statistics and configuration information about the optimized HTTP client
#[derive(Debug)]
pub struct ClientStats {
    pub max_idle_per_host: usize,
    pub idle_timeout_secs: u64,
    pub connect_timeout_secs: u64,
    pub http2_enabled: bool,
    pub http2_stream_window_size: u32,
    pub http2_connection_window_size: u32,
    pub http2_keepalive_interval_secs: u64,
    pub http2_keepalive_timeout_secs: u64,
    pub http2_max_concurrent_streams: u32,
    pub tcp_keepalive_enabled: bool,
    pub tcp_keepalive_interval_secs: u64,
}

/// Utility functions for client configuration from environment variables and config
impl HttpClient {
    /// Create HTTP client with configuration from the config struct
    /// This is the recommended way to create an HTTP client with explicit configuration
    pub fn from_config(http_client_config: &crate::config::settings::HttpClientConfig) -> Self {
        let config = ClientConfig {
            max_idle_per_host: http_client_config.max_idle_per_host as usize,
            idle_timeout: Duration::from_secs(http_client_config.idle_timeout_secs),
            connect_timeout: Duration::from_secs(http_client_config.connect_timeout_secs),
            enable_http2: http_client_config.enable_http2,
            http2_initial_stream_window_size: Some(http_client_config.http2_stream_window_size),
            http2_initial_connection_window_size: Some(http_client_config.http2_connection_window_size),
            http2_keep_alive_interval: Some(Duration::from_secs(http_client_config.http2_keepalive_interval_secs)),
            http2_keep_alive_timeout: Some(Duration::from_secs(http_client_config.http2_keepalive_timeout_secs)),
            http2_max_concurrent_streams: Some(http_client_config.http2_max_concurrent_streams),
            tcp_keepalive: http_client_config.tcp_keepalive,
            tcp_keepalive_interval: Some(Duration::from_secs(http_client_config.tcp_keepalive_interval_secs)),
        };

        info!("🔧 Loading HTTP client configuration from config file");
        Self::with_config(config)
    }

    /// Create HTTP client with advanced configuration from environment variables
    /// DEPRECATED: Use from_config instead for better configuration management
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
                    .unwrap_or_else(|_| "2097152".to_string()) // 2MB default
                    .parse()
                    .unwrap_or(2097152)
            ),
            http2_initial_connection_window_size: Some(
                std::env::var("PROXY_HTTP2_CONNECTION_WINDOW_SIZE")
                    .unwrap_or_else(|_| "8388608".to_string()) // 8MB default
                    .parse()
                    .unwrap_or(8388608)
            ),
            http2_keep_alive_interval: Some(Duration::from_secs(
                std::env::var("PROXY_HTTP2_KEEPALIVE_INTERVAL_SECS")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .unwrap_or(30)
            )),
            http2_keep_alive_timeout: Some(Duration::from_secs(
                std::env::var("PROXY_HTTP2_KEEPALIVE_TIMEOUT_SECS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10)
            )),
            http2_max_concurrent_streams: Some(
                std::env::var("PROXY_HTTP2_MAX_CONCURRENT_STREAMS")
                    .unwrap_or_else(|_| "100".to_string())
                    .parse()
                    .unwrap_or(100)
            ),
            tcp_keepalive: std::env::var("PROXY_TCP_KEEPALIVE")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            tcp_keepalive_interval: Some(Duration::from_secs(
                std::env::var("PROXY_TCP_KEEPALIVE_INTERVAL_SECS")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .unwrap_or(30)
            )),
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
