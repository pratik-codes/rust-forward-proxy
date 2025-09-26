//! Main entry point demonstrating the pluggable proxy architecture

use rust_forward_proxy::{
    init_logger_with_config,
    log_info,
    ProxyConfig,
};
use rust_forward_proxy::proxy::{ProxyManager, ProxyImplementation, ProxyImplConfig};
use std::env;

fn main() {
    // Load configuration from YAML file or fallback to environment variables
    let config = ProxyConfig::load_config()
        .unwrap_or_else(|e| {
            eprintln!("Failed to load configuration: {}", e);
            std::process::exit(1);
        });

    // Initialize production-grade logging with configuration
    init_logger_with_config(&config.log_level, config.logging.enable_file_logging);

    log_info!("🚀 Starting Pluggable Forward Proxy Server");
    
    // Determine which implementation to use
    let implementation = determine_implementation();
    
    log_info!("📦 Using proxy implementation: {:?}", implementation);
    log_info!("💡 You can change this by setting PROXY_IMPL environment variable");
    log_info!("   Supported values: pingora, hyper, reqwest");
    
    // Create proxy manager with selected implementation
    let mut proxy_manager = ProxyManager::new(implementation.clone());
    
    // Initialize the proxy
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        if let Err(e) = proxy_manager.initialize(&config).await {
            eprintln!("Failed to initialize proxy: {}", e);
            std::process::exit(1);
        }
        
        log_info!("✅ Proxy manager initialized with {} implementation", 
                 proxy_manager.implementation_name());
        
        // Start the proxy server
        log_info!("🌐 Starting proxy server on {}", config.listen_addr);
        
        // For demonstration, let's also show how to process individual requests
        demonstrate_request_processing(&proxy_manager).await;
        
        // In a real implementation, you would call:
        // proxy_manager.start(config.listen_addr).await.unwrap();
        
        log_info!("🛑 Proxy demonstration completed");
    });
}

/// Determine which proxy implementation to use based on environment or config
fn determine_implementation() -> ProxyImplementation {
    // Check environment variable first
    if let Ok(impl_name) = env::var("PROXY_IMPL") {
        match impl_name.to_lowercase().as_str() {
            "pingora" => return ProxyImplementation::Pingora,
            "hyper" => return ProxyImplementation::Hyper,
            "reqwest" => return ProxyImplementation::Reqwest,
            _ => {
                eprintln!("⚠️  Unknown proxy implementation '{}', falling back to default", impl_name);
            }
        }
    }
    
    // Default to Pingora for best performance
    ProxyImplementation::Pingora
}

/// Demonstrate how to process requests with different implementations
async fn demonstrate_request_processing(proxy_manager: &ProxyManager) {
    use rust_forward_proxy::proxy::core::ProxyRequest;
    use std::collections::HashMap;
    use std::net::SocketAddr;
    
    log_info!("🧪 Demonstrating request processing capabilities...");
    
    // Create a sample request
    let sample_request = ProxyRequest {
        method: "GET".to_string(),
        uri: "https://httpbin.org/get".to_string(),
        headers: {
            let mut headers = HashMap::new();
            headers.insert("user-agent".to_string(), "rust-forward-proxy-demo".to_string());
            headers.insert("accept".to_string(), "application/json".to_string());
            headers
        },
        body: bytes::Bytes::new(),
        client_addr: "127.0.0.1:12345".parse::<SocketAddr>().unwrap(),
        is_connect: false,
    };
    
    // Process the request
    match proxy_manager.process_request(sample_request).await {
        Ok(response) => {
            log_info!("✅ Sample request processed successfully!");
            log_info!("   Status: {}", response.status_code);
            log_info!("   Implementation: {}", 
                     response.headers.get("x-proxy-impl").unwrap_or(&"unknown".to_string()));
            log_info!("   Response size: {} bytes", response.body.len());
        }
        Err(e) => {
            log_info!("❌ Sample request failed: {}", e);
        }
    }
    
    // Demonstrate health check
    match proxy_manager.get_metrics().await {
        Ok(metrics) => {
            log_info!("📊 Proxy metrics:");
            log_info!("   Implementation: {}", metrics.implementation);
            log_info!("   Total requests: {}", metrics.total_requests);
            log_info!("   Successful: {}", metrics.successful_requests);
            log_info!("   Failed: {}", metrics.failed_requests);
            log_info!("   Average response time: {:.2}ms", metrics.average_response_time_ms);
            log_info!("   Uptime: {}s", metrics.uptime_seconds);
        }
        Err(e) => {
            log_info!("❌ Failed to get metrics: {}", e);
        }
    }
}
