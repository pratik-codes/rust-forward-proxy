//! Main entry point for the Rust Forward Proxy

use rust_forward_proxy::{
    init_logger_with_config,
    log_info,
    ProxyServer,
    ProxyConfig,
    tls::start_dual_servers,
};




#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration from YAML file or fallback to environment variables
    let config = ProxyConfig::load_config()
        .map_err(|e| anyhow::anyhow!("Failed to load configuration: {}", e))?;

    // Initialize production-grade logging with configuration
    init_logger_with_config(&config.log_level, config.logging.enable_file_logging);

    log_info!("Starting Forward Proxy Server");
    
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
        log_info!("Test HTTP: curl -x http://{} http://httpbin.org/get", config.listen_addr);
        log_info!("Test HTTPS: curl -x https://{} https://httpbin.org/get", config.tls.https_listen_addr);
        
        start_dual_servers(config).await?;
    } else {
        // Start only HTTP server
        log_info!("🌐 Starting HTTP-only proxy server (TLS disabled)");
        log_info!("Test with: curl -x http://{} http://httpbin.org/get", config.listen_addr);
        
        if config.tls.interception_enabled {
            tracing::debug!("🔍 HTTPS interception enabled - CONNECT requests to port 443 will be intercepted");
            log_info!("⚠️  Clients will see certificate warnings (normal for self-signed certs)");
        }
        
        let server = ProxyServer::with_https_interception_and_config(config.listen_addr, true, &config);
        server.start().await?;
    }

    Ok(())
}
