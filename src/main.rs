//! Main entry point for the Rust Forward Proxy

use rust_forward_proxy::{
    init_logger_with_config,
    log_info,
    ProxyConfig,
    ForwardProxy,
};
use pingora::prelude::*;
// use std::sync::Arc;




fn main() {
    // Load configuration from YAML file or fallback to environment variables
    let config = ProxyConfig::load_config()
        .unwrap_or_else(|e| {
            eprintln!("Failed to load configuration: {}", e);
            std::process::exit(1);
        });

    // Initialize production-grade logging with configuration
    init_logger_with_config(&config.log_level, config.logging.enable_file_logging);

    log_info!("Starting Pingora-based Forward Proxy Server");
    
    // Log startup information
    log_info!("Proxy server starting on {}", config.listen_addr);
    if config.tls.enabled {
        log_info!("HTTPS proxy server starting on {}", config.tls.https_listen_addr);
        log_info!("TLS interception: {}", if config.tls.interception_enabled { "ENABLED" } else { "DISABLED" });
    }
    log_info!("Console-only logging enabled");

    // Create pingora server
    let mut my_server = Server::new(Some(Opt::parse_args())).unwrap();
    my_server.bootstrap();

    // Create the forward proxy with configuration
    let forward_proxy = ForwardProxy::new(config.clone());

    // Create HTTP proxy service
    let mut proxy_service = pingora::proxy::http_proxy_service(
        &my_server.configuration, 
        forward_proxy
    );
    
    // AddP traff TCP listener for HTTic
    proxy_service.add_tcp(&config.listen_addr.to_string());
    
    log_info!("🚀 Starting Pingora-based proxy server");
    log_info!("Test with: curl -x http://{} http://httpbin.org/get", config.listen_addr);
    
    if config.tls.interception_enabled {
        log_info!("🔍 HTTPS interception enabled - CONNECT requests to port 443 will be intercepted");
        log_info!("⚠️  Clients will see certificate warnings (normal for self-signed certs)");
    }

    // Add the proxy service to the server
    my_server.add_service(proxy_service);

    // Start the server (this blocks)
    my_server.run_forever();
}
