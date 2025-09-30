//! Proxy server configuration settings

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::Path;
use anyhow::{Context, Result};

/// Main configuration for the proxy server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// Use privileged ports (80/443) when true, regular ports (8080/8443) when false
    #[serde(default)]
    pub use_privileged_ports: bool,
    
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
    
    /// Logging configuration
    pub logging: LoggingConfig,
    
    /// HTTP client configuration
    pub http_client: HttpClientConfig,
    
    /// Streaming configuration
    pub streaming: StreamingConfig,
    
    /// Runtime configuration
    pub runtime: RuntimeConfig,
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

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Enable file logging (default: true)
    pub enable_file_logging: bool,
}

/// HTTP client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpClientConfig {
    /// Maximum idle connections per host
    pub max_idle_per_host: u32,
    
    /// Idle timeout in seconds
    pub idle_timeout_secs: u64,
    
    /// Connection timeout in seconds
    pub connect_timeout_secs: u64,
    
    /// Enable HTTP/2 support
    pub enable_http2: bool,
    
    /// HTTP/2 stream window size
    pub http2_stream_window_size: u32,
    
    /// HTTP/2 connection window size
    pub http2_connection_window_size: u32,
    
    /// HTTP/2 keepalive interval in seconds
    pub http2_keepalive_interval_secs: u64,
    
    /// HTTP/2 keepalive timeout in seconds
    pub http2_keepalive_timeout_secs: u64,
    
    /// HTTP/2 maximum concurrent streams
    pub http2_max_concurrent_streams: u32,
    
    /// Enable TCP keepalive
    pub tcp_keepalive: bool,
    
    /// TCP keepalive interval in seconds
    pub tcp_keepalive_interval_secs: u64,
}

/// Streaming configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConfig {
    /// Maximum log body size in bytes
    pub max_log_body_size: usize,
    
    /// Maximum partial log size in bytes
    pub max_partial_log_size: usize,
    
    /// Enable response streaming
    pub enable_response_streaming: bool,
    
    /// Enable request streaming
    pub enable_request_streaming: bool,
}

/// Runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Runtime mode: "single_threaded", "multi_threaded", or "multi_process"
    pub mode: String,
    
    /// Number of worker threads for multi-threaded mode (0 = auto-detect CPU cores)
    pub worker_threads: Option<usize>,
    
    /// Number of processes for multi-process mode
    pub process_count: Option<usize>,
    
    /// Current process index (0-based) - set at runtime
    #[serde(skip)]
    pub process_index: Option<usize>,
    
    /// Use SO_REUSEPORT for multi-process mode (Linux/macOS)
    pub use_reuseport: bool,
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
    
    /// Certificate storage backend: "cache" (in-memory) or "redis" 
    pub certificate_storage: String,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            use_privileged_ports: false, // Default to non-privileged ports
            listen_addr: "127.0.0.1:8080".parse().unwrap(),
            log_level: "info".to_string(),
            upstream: UpstreamConfig::default(),
            redis: RedisConfig::default(),
            request_timeout: 30,
            max_body_size: 1024 * 1024, // 1MB
            tls: TlsConfig::default(),
            logging: LoggingConfig::default(),
            http_client: HttpClientConfig::default(),
            streaming: StreamingConfig::default(),
            runtime: RuntimeConfig::default(),
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
            url: "redis://redis:6379".to_string(),
            pool_size: 10,
            connection_timeout: 5,
            command_timeout: 10,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            enable_file_logging: true,
        }
    }
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            max_idle_per_host: 50,
            idle_timeout_secs: 90,
            connect_timeout_secs: 10,
            enable_http2: true,
            http2_stream_window_size: 2097152, // 2MB
            http2_connection_window_size: 8388608, // 8MB
            http2_keepalive_interval_secs: 30,
            http2_keepalive_timeout_secs: 10,
            http2_max_concurrent_streams: 100,
            tcp_keepalive: true,
            tcp_keepalive_interval_secs: 30,
        }
    }
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            max_log_body_size: 1048576, // 1MB
            max_partial_log_size: 1024, // 1KB
            enable_response_streaming: true,
            enable_request_streaming: false,
        }
    }
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            mode: "multi_threaded".to_string(), // Default to multi-threaded for backward compatibility
            worker_threads: None, // Auto-detect CPU cores
            process_count: None, // Default to single process
            process_index: None, // Set at runtime
            use_reuseport: true, // Enable SO_REUSEPORT by default on supported systems
        }
    }
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default for backward compatibility
            https_listen_addr: "127.0.0.1:443".parse().unwrap(),
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
            certificate_storage: "cache".to_string(), // Default to cache storage
        }
    }
}

impl ProxyConfig {
    /// Load configuration from a YAML file
    pub fn from_yaml_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = std::fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read config file: {}", path.as_ref().display()))?;
        
        let mut config: ProxyConfig = serde_yaml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {}", path.as_ref().display()))?;
        
        // Apply port configuration based on use_privileged_ports
        config.apply_port_configuration();
        
        Ok(config)
    }
    
    /// Load configuration from YAML file with environment variable overrides
    pub fn load_config() -> Result<Self> {
        let config_path = "config.yml";
        
        let mut config = if Path::new(&config_path).exists() {
            Self::from_yaml_file(&config_path)?
        } else {
            return Err(anyhow::anyhow!("Config file '{}' not found. Please ensure config.yml exists in the project root.", config_path));
        };
        
        // Override with environment variables for development/testing (these take precedence over automatic port configuration)
        if let Ok(http_port) = std::env::var("HTTP_PROXY_PORT") {
            if let Ok(port) = http_port.parse::<u16>() {
                config.listen_addr = format!("127.0.0.1:{}", port).parse().unwrap();
            }
        }
        
        if let Ok(https_port) = std::env::var("HTTPS_PROXY_PORT") {
            if let Ok(port) = https_port.parse::<u16>() {
                config.tls.https_listen_addr = format!("127.0.0.1:{}", port).parse().unwrap();
            }
        }
        
        Ok(config)
    }
    
    /// Apply port configuration based on use_privileged_ports setting
    /// This will override the ports in the configuration based on the use_privileged_ports flag
    pub fn apply_port_configuration(&mut self) {
        if self.use_privileged_ports {
            // Use privileged ports (80/443) - requires sudo
            self.listen_addr = "127.0.0.1:80".parse().unwrap();
            self.tls.https_listen_addr = "127.0.0.1:443".parse().unwrap();
        } else {
            // Use regular ports (8080/8443) - no sudo required  
            self.listen_addr = "127.0.0.1:8080".parse().unwrap();
            self.tls.https_listen_addr = "127.0.0.1:8443".parse().unwrap();
        }
    }
    
    /// Legacy function to load configuration from environment variables
    /// This is kept for backward compatibility
    pub fn from_env_vars() -> Self {
        let mut config = Self::default();
        
        // Load basic proxy settings
        if let Ok(addr_str) = std::env::var("PROXY_LISTEN_ADDR") {
            if let Ok(addr) = addr_str.parse() {
                config.listen_addr = addr;
            }
        }
        
        if let Ok(log_level) = std::env::var("RUST_LOG") {
            config.log_level = log_level;
        }
        
        if let Ok(timeout) = std::env::var("PROXY_REQUEST_TIMEOUT") {
            if let Ok(timeout) = timeout.parse() {
                config.request_timeout = timeout;
            }
        }
        
        if let Ok(max_size) = std::env::var("PROXY_MAX_BODY_SIZE") {
            if let Ok(max_size) = max_size.parse() {
                config.max_body_size = max_size;
            }
        }
        
        // Load upstream settings
        if let Ok(upstream_url) = std::env::var("UPSTREAM_URL") {
            config.upstream.url = upstream_url;
        }
        
        if let Ok(connect_timeout) = std::env::var("UPSTREAM_CONNECT_TIMEOUT") {
            if let Ok(timeout) = connect_timeout.parse() {
                config.upstream.connect_timeout = timeout;
            }
        }
        
        if let Ok(keep_alive_timeout) = std::env::var("UPSTREAM_KEEP_ALIVE_TIMEOUT") {
            if let Ok(timeout) = keep_alive_timeout.parse() {
                config.upstream.keep_alive_timeout = timeout;
            }
        }
        
        // Load Redis settings
        if let Ok(redis_url) = std::env::var("REDIS_URL") {
            config.redis.url = redis_url;
        }
        
        if let Ok(pool_size) = std::env::var("REDIS_POOL_SIZE") {
            if let Ok(size) = pool_size.parse() {
                config.redis.pool_size = size;
            }
        }
        
        if let Ok(timeout) = std::env::var("REDIS_CONNECTION_TIMEOUT") {
            if let Ok(timeout) = timeout.parse() {
                config.redis.connection_timeout = timeout;
            }
        }
        
        if let Ok(timeout) = std::env::var("REDIS_COMMAND_TIMEOUT") {
            if let Ok(timeout) = timeout.parse() {
                config.redis.command_timeout = timeout;
            }
        }
        
        // Load TLS settings
        if let Ok(tls_enabled) = std::env::var("TLS_ENABLED") {
            config.tls.enabled = tls_enabled.to_lowercase() == "true";
        }
        
        if let Ok(https_addr) = std::env::var("HTTPS_LISTEN_ADDR") {
            if let Ok(addr) = https_addr.parse() {
                config.tls.https_listen_addr = addr;
            }
        }
        
        if let Ok(cert_path) = std::env::var("TLS_CERT_PATH") {
            config.tls.cert_path = cert_path;
        }
        
        if let Ok(key_path) = std::env::var("TLS_KEY_PATH") {
            config.tls.key_path = key_path;
        }
        
        if let Ok(auto_gen) = std::env::var("TLS_AUTO_GENERATE_CERT") {
            config.tls.auto_generate_cert = auto_gen.to_lowercase() == "true";
        }
        
        if let Ok(interception) = std::env::var("TLS_INTERCEPTION_ENABLED") {
            config.tls.interception_enabled = interception.to_lowercase() == "true";
        }
        
        if let Ok(org) = std::env::var("TLS_CERT_ORGANIZATION") {
            config.tls.cert_organization = org;
        }
        
        if let Ok(cn) = std::env::var("TLS_CERT_COMMON_NAME") {
            config.tls.cert_common_name = cn;
        }
        
        if let Ok(days) = std::env::var("TLS_CERT_VALIDITY_DAYS") {
            if let Ok(days) = days.parse() {
                config.tls.cert_validity_days = days;
            }
        }
        
        if let Ok(version) = std::env::var("TLS_MIN_TLS_VERSION") {
            config.tls.min_tls_version = version;
        }
        
        if let Ok(skip) = std::env::var("TLS_SKIP_UPSTREAM_CERT_VERIFY") {
            config.tls.skip_upstream_cert_verify = skip.to_lowercase() == "true";
        }
        
        if let Ok(ca_cert) = std::env::var("TLS_ROOT_CA_CERT_PATH") {
            config.tls.root_ca_cert_path = Some(ca_cert);
        }
        
        if let Ok(ca_cert) = std::env::var("TLS_CA_CERT_PATH") {
            config.tls.ca_cert_path = Some(ca_cert);
        }
        
        if let Ok(ca_key) = std::env::var("TLS_CA_KEY_PATH") {
            config.tls.ca_key_path = Some(ca_key);
        }
        
        // Load logging settings
        if let Ok(enable_file_logging) = std::env::var("PROXY_ENABLE_FILE_LOGGING") {
            config.logging.enable_file_logging = enable_file_logging.to_lowercase() == "true";
        }
        
        // Load HTTP client settings
        if let Ok(max_idle) = std::env::var("PROXY_MAX_IDLE_PER_HOST") {
            if let Ok(max_idle) = max_idle.parse() {
                config.http_client.max_idle_per_host = max_idle;
            }
        }
        
        if let Ok(timeout) = std::env::var("PROXY_IDLE_TIMEOUT_SECS") {
            if let Ok(timeout) = timeout.parse() {
                config.http_client.idle_timeout_secs = timeout;
            }
        }
        
        if let Ok(timeout) = std::env::var("PROXY_CONNECT_TIMEOUT_SECS") {
            if let Ok(timeout) = timeout.parse() {
                config.http_client.connect_timeout_secs = timeout;
            }
        }
        
        if let Ok(enable_http2) = std::env::var("PROXY_ENABLE_HTTP2") {
            config.http_client.enable_http2 = enable_http2.to_lowercase() == "true";
        }
        
        if let Ok(window_size) = std::env::var("PROXY_HTTP2_STREAM_WINDOW_SIZE") {
            if let Ok(size) = window_size.parse() {
                config.http_client.http2_stream_window_size = size;
            }
        }
        
        if let Ok(window_size) = std::env::var("PROXY_HTTP2_CONNECTION_WINDOW_SIZE") {
            if let Ok(size) = window_size.parse() {
                config.http_client.http2_connection_window_size = size;
            }
        }
        
        if let Ok(interval) = std::env::var("PROXY_HTTP2_KEEPALIVE_INTERVAL_SECS") {
            if let Ok(interval) = interval.parse() {
                config.http_client.http2_keepalive_interval_secs = interval;
            }
        }
        
        if let Ok(timeout) = std::env::var("PROXY_HTTP2_KEEPALIVE_TIMEOUT_SECS") {
            if let Ok(timeout) = timeout.parse() {
                config.http_client.http2_keepalive_timeout_secs = timeout;
            }
        }
        
        if let Ok(streams) = std::env::var("PROXY_HTTP2_MAX_CONCURRENT_STREAMS") {
            if let Ok(streams) = streams.parse() {
                config.http_client.http2_max_concurrent_streams = streams;
            }
        }
        
        if let Ok(keepalive) = std::env::var("PROXY_TCP_KEEPALIVE") {
            config.http_client.tcp_keepalive = keepalive.to_lowercase() == "true";
        }
        
        if let Ok(interval) = std::env::var("PROXY_TCP_KEEPALIVE_INTERVAL_SECS") {
            if let Ok(interval) = interval.parse() {
                config.http_client.tcp_keepalive_interval_secs = interval;
            }
        }
        
        // Load streaming settings
        if let Ok(max_size) = std::env::var("PROXY_MAX_LOG_BODY_SIZE") {
            if let Ok(size) = max_size.parse() {
                config.streaming.max_log_body_size = size;
            }
        }
        
        if let Ok(max_size) = std::env::var("PROXY_MAX_PARTIAL_LOG_SIZE") {
            if let Ok(size) = max_size.parse() {
                config.streaming.max_partial_log_size = size;
            }
        }
        
        if let Ok(enable) = std::env::var("PROXY_ENABLE_RESPONSE_STREAMING") {
            config.streaming.enable_response_streaming = enable.to_lowercase() == "true";
        }
        
        if let Ok(enable) = std::env::var("PROXY_ENABLE_REQUEST_STREAMING") {
            config.streaming.enable_request_streaming = enable.to_lowercase() == "true";
        }
        
        // Load runtime settings
        if let Ok(mode) = std::env::var("PROXY_RUNTIME_MODE") {
            config.runtime.mode = mode;
        }
        
        if let Ok(threads) = std::env::var("PROXY_WORKER_THREADS") {
            if let Ok(threads) = threads.parse() {
                config.runtime.worker_threads = Some(threads);
            }
        }
        
        if let Ok(processes) = std::env::var("PROXY_PROCESS_COUNT") {
            if let Ok(processes) = processes.parse() {
                config.runtime.process_count = Some(processes);
            }
        }
        
        if let Ok(reuseport) = std::env::var("PROXY_USE_REUSEPORT") {
            config.runtime.use_reuseport = reuseport.to_lowercase() == "true";
        }
        
        config
    }
}
