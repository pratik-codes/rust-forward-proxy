//! Main entry point for the Rust Forward Proxy

use rust_forward_proxy::{
    init_logger_with_env,
    log_info,
    ProxyServer,
    ProxyConfig,
};



#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize production-grade logging
    init_logger_with_env();

    log_info!("Starting Forward Proxy Server");

    // Load configuration
    let config = ProxyConfig::default();
    
    // Log startup information
    log_info!("Proxy server starting on {}", config.listen_addr);
    log_info!("Console-only logging enabled");

    // Create and start server
    let server = ProxyServer::new(config.listen_addr);

    log_info!("Test with: curl -x http://127.0.0.1:8080 http://httpbin.org/get");

    server.start().await?;

    Ok(())
}
