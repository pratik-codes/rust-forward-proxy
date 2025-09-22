//! Main entry point for the Rust Forward Proxy

use rust_forward_proxy::{
    init_logger_with_env,
    log_info,
    ProxyServer,
    ProxyConfig,
    tls::start_dual_servers,
};

/// Load configuration from environment variables with fallback to defaults
fn load_config_from_env() -> ProxyConfig {
    use std::env;
    
    let mut config = ProxyConfig::default();
    
    // Override with environment variables if present
    if let Ok(addr_str) = env::var("PROXY_LISTEN_ADDR") {
        if let Ok(addr) = addr_str.parse() {
            config.listen_addr = addr;
        }
    }
    
    if let Ok(log_level) = env::var("RUST_LOG") {
        config.log_level = log_level;
    }
    
    if let Ok(timeout) = env::var("PROXY_REQUEST_TIMEOUT") {
        if let Ok(timeout_val) = timeout.parse() {
            config.request_timeout = timeout_val;
        }
    }
    
    if let Ok(max_size) = env::var("PROXY_MAX_BODY_SIZE") {
        if let Ok(size_val) = max_size.parse() {
            config.max_body_size = size_val;
        }
    }
    
    if let Ok(upstream_url) = env::var("UPSTREAM_URL") {
        config.upstream.url = upstream_url;
    }
    
    if let Ok(connect_timeout) = env::var("UPSTREAM_CONNECT_TIMEOUT") {
        if let Ok(timeout_val) = connect_timeout.parse() {
            config.upstream.connect_timeout = timeout_val;
        }
    }
    
    if let Ok(keep_alive_timeout) = env::var("UPSTREAM_KEEP_ALIVE_TIMEOUT") {
        if let Ok(timeout_val) = keep_alive_timeout.parse() {
            config.upstream.keep_alive_timeout = timeout_val;
        }
    }
    
    // TLS configuration from environment
    if let Ok(tls_enabled) = env::var("TLS_ENABLED") {
        config.tls.enabled = tls_enabled.parse().unwrap_or(false);
    }
    
    if let Ok(https_addr) = env::var("HTTPS_LISTEN_ADDR") {
        if let Ok(addr) = https_addr.parse() {
            config.tls.https_listen_addr = addr;
        }
    }
    
    if let Ok(cert_path) = env::var("TLS_CERT_PATH") {
        config.tls.cert_path = cert_path;
    }
    
    if let Ok(key_path) = env::var("TLS_KEY_PATH") {
        config.tls.key_path = key_path;
    }
    
    if let Ok(auto_gen) = env::var("TLS_AUTO_GENERATE_CERT") {
        config.tls.auto_generate_cert = auto_gen.parse().unwrap_or(true);
    }
    
    if let Ok(interception) = env::var("TLS_INTERCEPTION_ENABLED") {
        config.tls.interception_enabled = interception.parse().unwrap_or(true);
    }
    
    config
}



#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize production-grade logging
    init_logger_with_env();

    log_info!("Starting Forward Proxy Server");

    // Load configuration from environment variables and defaults
    let config = load_config_from_env();
    
    // Log startup information
    log_info!("Proxy server starting on {}", config.listen_addr);
    if config.tls.enabled {
        log_info!("HTTPS proxy server starting on {}", config.tls.https_listen_addr);
        log_info!("TLS interception: {}", if config.tls.interception_enabled { "ENABLED" } else { "DISABLED" });
    }
    log_info!("Console-only logging enabled");

    if config.tls.enabled {
        // Start both HTTP and HTTPS servers
        log_info!("🚀 Starting dual HTTP/HTTPS proxy servers");
        log_info!("Test HTTP: curl -x http://127.0.0.1:8080 http://httpbin.org/get");
        log_info!("Test HTTPS: curl -x https://127.0.0.1:8443 https://httpbin.org/get");
        
        start_dual_servers(config).await?;
    } else {
        // Start only HTTP server
        log_info!("🌐 Starting HTTP-only proxy server (TLS disabled)");
        log_info!("Test with: curl -x http://127.0.0.1:8080 http://httpbin.org/get");
        
        // Check if HTTPS interception is enabled via environment variable
        let https_interception = std::env::var("HTTPS_INTERCEPTION_ENABLED")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);
        
        if https_interception {
            log_info!("🔍 HTTPS interception enabled - CONNECT requests to port 443 will be intercepted");
            log_info!("⚠️  Clients will see certificate warnings (normal for self-signed certs)");
        }
        
        let server = ProxyServer::with_https_interception(config.listen_addr, true);
        server.start().await?;
    }

    Ok(())
}
