//! Main entry point for the Rust Forward Proxy

use rust_forward_proxy::{
    init_logger_with_env,
    log_info,
    ProxyServer,
    ProxyConfig,
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
    log_info!("Console-only logging enabled");

    // Create and start server
    let server = ProxyServer::new(config.listen_addr);

    log_info!("Test with: curl -x http://127.0.0.1:8080 http://httpbin.org/get");

    server.start().await?;

    Ok(())
}
