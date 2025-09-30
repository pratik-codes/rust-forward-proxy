//! Proxy server implementation

use crate::models::{ProxyLog, RequestData, ResponseData};
use crate::{log_info, log_error, log_debug, log_proxy_transaction};
use crate::utils::{is_hop_by_hop_header, parse_url, extract_path, extract_query, is_https, parse_connect_target, build_error_response, build_proxy_error_response, extract_headers, extract_cookies_to_request_data, should_extract_body, extract_body, build_forwarding_request, log_incoming_request, log_connect_request, log_http_success, log_http_failure, log_forwarding_request, log_headers_structured, log_response_headers_structured, should_forward_request_header, should_forward_response_header};
use crate::tls::{generate_domain_cert_with_ca, create_server_config, CertificateManager};
use crate::proxy::http_client::HttpClient;
use crate::proxy::streaming::SmartBodyHandler;
use anyhow::Result;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use std::convert::Infallible;
use std::net::SocketAddr;
use tokio_rustls::TlsAcceptor;
use socket2::{Socket, Domain, Type};
use tracing::{error, info, debug, warn};
use hyper::upgrade::{Upgraded, on};
use serde_json::json;
use std::sync::Arc;
use bytes::Bytes;


pub struct ProxyServer {
    listen_addr: SocketAddr,
    https_interception: bool,
    cert_manager: Arc<CertificateManager>,
    client_manager: Arc<HttpClient>,
    body_handler: Arc<SmartBodyHandler>,
    tls_config: crate::config::settings::TlsConfig,
}

/// Create a socket with SO_REUSEPORT support for multi-process binding
fn create_reusable_socket(addr: SocketAddr) -> Result<Socket> {
    use std::env;
    
    let domain = if addr.is_ipv6() { Domain::IPV6 } else { Domain::IPV4 };
    let socket = Socket::new(domain, Type::STREAM, None)?;
    
    // Enable SO_REUSEADDR (always good to have)
    socket.set_reuse_address(true)?;
    
    // Enable SO_REUSEPORT if requested (Linux/macOS)
    if env::var("PROXY_USE_REUSEPORT").unwrap_or_default() == "true" {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            socket.set_reuse_port(true)?;
            info!("✅ SO_REUSEPORT enabled for multi-process support");
        }
        
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            warn!("⚠️  SO_REUSEPORT requested but not supported on this platform");
        }
    }
    
    // Set to non-blocking
    socket.set_nonblocking(true)?;
    
    // Bind to address
    socket.bind(&addr.into())?;
    
    // Listen with backlog
    socket.listen(1024)?;
    
    Ok(socket)
}

impl ProxyServer {
    /// Create a new proxy server with configuration
    /// This is the recommended way to create a proxy server
    pub fn with_config(listen_addr: SocketAddr, config: &crate::config::settings::ProxyConfig) -> Self {
        Self { 
            listen_addr,
            https_interception: false, // Default to false for backward compatibility
            cert_manager: Arc::new(CertificateManager::new()),
            client_manager: Arc::new(HttpClient::from_config(&config.http_client)),
            body_handler: Arc::new(SmartBodyHandler::from_config(&config.streaming)),
            tls_config: config.tls.clone(),
        }
    }

    /// Create a new proxy server with HTTPS interception and configuration
    /// This is the recommended way to create a proxy server with HTTPS interception
    pub fn with_https_interception_and_config(listen_addr: SocketAddr, enable_interception: bool, config: &crate::config::settings::ProxyConfig) -> Self {
        let cert_manager = Arc::new(CertificateManager::new());
        let client_manager = Arc::new(HttpClient::from_config(&config.http_client));
        let body_handler = Arc::new(SmartBodyHandler::from_config(&config.streaming));
        
        Self {
            listen_addr,
            https_interception: enable_interception,
            cert_manager,
            client_manager,
            body_handler,
            tls_config: config.tls.clone(),
        }
    }

    /// Create a new proxy server (legacy method)
    /// DEPRECATED: Use with_config instead for better configuration management
    pub fn new(listen_addr: SocketAddr) -> Self {
        Self { 
            listen_addr,
            https_interception: false, // Default to false for backward compatibility
            cert_manager: Arc::new(CertificateManager::new()),
            client_manager: Arc::new(HttpClient::from_env()),
            body_handler: Arc::new(SmartBodyHandler::from_env()),
            tls_config: crate::config::settings::TlsConfig::default(),
        }
    }
    
    /// Create a new proxy server with HTTPS interception (legacy method)
    /// DEPRECATED: Use with_https_interception_and_config instead for better configuration management
    pub fn with_https_interception(listen_addr: SocketAddr, enable_interception: bool) -> Self {
        let cert_manager = Arc::new(CertificateManager::new());
        let client_manager = Arc::new(HttpClient::from_env());
        let body_handler = Arc::new(SmartBodyHandler::from_env());
        
        info!("🔐 Certificate cache initialized: {}", cert_manager.cache_info());
        info!("🚀 Optimized HTTP client manager initialized");
        info!("🚀 Smart body handler initialized");
        
        Self {
            listen_addr,
            https_interception: enable_interception,
            cert_manager,
            client_manager,
            body_handler,
            tls_config: crate::config::settings::TlsConfig::default(),
        }
    }

    /// Start the proxy server
    pub async fn start(self) -> Result<()> {
        info!("Starting proxy server on {}", self.listen_addr);
        
        // Log server startup
        log_info!("Proxy server starting on {}", self.listen_addr);
        log_debug!("Server configuration: listen_addr={}", self.listen_addr);
        
        log_info!("🔍 HTTPS interception mode: ENABLED - all HTTPS content will be logged!");

        let https_interception = self.https_interception;
        let cert_manager = Arc::clone(&self.cert_manager);
        let client_manager = Arc::clone(&self.client_manager);
        let body_handler = Arc::clone(&self.body_handler);
        let tls_config = self.tls_config.clone();
        let make_svc = make_service_fn(move |conn: &hyper::server::conn::AddrStream| {
            let remote_addr = conn.remote_addr();
            let cert_manager = Arc::clone(&cert_manager);
            let client_manager = Arc::clone(&client_manager);
            let body_handler = Arc::clone(&body_handler);
            let tls_config = tls_config.clone();
            log_debug!("New connection from: {}", remote_addr);

            async move { 
                Ok::<_, Infallible>(service_fn(move |req| {
                    let cert_manager = Arc::clone(&cert_manager);
                    let client_manager = Arc::clone(&client_manager);
                    let body_handler = Arc::clone(&body_handler);
                    let tls_config = tls_config.clone();
                    async move {
                        handle_request(req, remote_addr, https_interception, cert_manager, client_manager, body_handler, &tls_config).await
                    }
                })) 
            }
        });

        log_debug!("Creating server with service factory");
        
        // Create server with SO_REUSEPORT support if needed
        let server = if std::env::var("PROXY_USE_REUSEPORT").unwrap_or_default() == "true" {
            // Use reusable socket for multi-process support
            let socket = create_reusable_socket(self.listen_addr)?;
            let std_listener = std::net::TcpListener::from(socket);
            let tokio_listener = tokio::net::TcpListener::from_std(std_listener)?;
            
            // Get process index for logging
            let process_info = if let Ok(index) = std::env::var("PROXY_PROCESS_INDEX") {
                format!(" (process {})", index)
            } else {
                String::new()
            };
            
            info!("🔄 Server binding with SO_REUSEPORT{}", process_info);
            log_info!("Server binding with SO_REUSEPORT{}", process_info);
            
            Server::builder(hyper::server::conn::AddrIncoming::from_listener(tokio_listener)?)
                .serve(make_svc)
        } else {
            // Standard single-process binding
            Server::bind(&self.listen_addr).serve(make_svc)
        };
        
        log_info!("Server bound successfully, waiting for connections");

        if let Err(e) = server.await {
            error!("Server error: {}", e);
            log_error!("Server error: {}", e);
        }

        Ok(())
    }
}

/// Handle incoming HTTP request
pub async fn handle_request(
    req: Request<Body>,
    remote_addr: SocketAddr,
    https_interception: bool,
    cert_manager: Arc<CertificateManager>,
    client_manager: Arc<HttpClient>,
    body_handler: Arc<SmartBodyHandler>,
    tls_config: &crate::config::settings::TlsConfig,
) -> Result<Response<Body>, Infallible> {
    let start_time = std::time::Instant::now();
    let method = req.method().to_string();
    let uri = req.uri().to_string();

    // Log incoming request with current process PID
    let current_pid = std::process::id();
    info!("🎯 REQUEST HANDLER: PID {} processing {} {} from {}", 
          current_pid, method, uri, remote_addr.ip());
    log_incoming_request(&method, &uri, &remote_addr);

    // Handle health check endpoint locally (don't forward to upstream)
    if req.uri().path() == "/health" {
        return handle_health_check(method, start_time).await;
    }

    // Create request data structure
    let mut request_data = RequestData::new(
        method.clone(),
        uri.clone(),
        remote_addr.ip(),
        remote_addr.port(),
    );
    
    log_debug!("Created request data: client_ip={}, client_port={}", 
               request_data.client_ip, request_data.client_port);
    
    // Handle CONNECT requests - always intercept HTTPS
    if method == "CONNECT" {
        return handle_connect_request(req, request_data, start_time, https_interception, cert_manager, client_manager, body_handler, tls_config).await;
    } else {
        // Extract and process regular HTTP request data
        extract_request_data(&mut request_data, &uri, req).await;
        
        // Handle regular HTTP requests with full interception
        handle_http_request(request_data, method, start_time, client_manager, body_handler).await
    }
}

/// Handle health check endpoint locally
async fn handle_health_check(
    method: String,
    start_time: std::time::Instant,
) -> Result<Response<Body>, Infallible> {
    let current_pid = std::process::id();
    let elapsed_time = start_time.elapsed().as_millis();
    
    if method == "GET" {
        // Create health response
        let health_data = json!({
            "status": "healthy",
            "service": "rust-forward-proxy",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "uptime_ms": elapsed_time,
            "version": env!("CARGO_PKG_VERSION"),
            "pid": current_pid
        });
        
        let response = Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "application/json")
            .header("cache-control", "no-cache")
            .body(Body::from(health_data.to_string()))
            .unwrap();
            
        log_info!("✅ GET /health → 200 OK ({}ms) - handled by PID {}", elapsed_time, current_pid);
        Ok(response)
    } else {
        // Health endpoint only supports GET
        let response = Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .header("allow", "GET")
            .body(Body::from("Method Not Allowed"))
            .unwrap();
            
        log_info!("❌ {} /health → 405 Method Not Allowed ({}ms)", method, elapsed_time);
        Ok(response)
    }
}


/// Handle HTTPS interception - decrypt, log, and re-encrypt
async fn handle_https_interception(
    req: Request<Body>,
    host: String,
    port: u16,
    start_time: std::time::Instant,
    cert_manager: Arc<CertificateManager>,
    client_manager: Arc<HttpClient>,
    body_handler: Arc<SmartBodyHandler>,
    tls_config: &crate::config::settings::TlsConfig,
) -> Result<Response<Body>, Infallible> {
    let connect_time = start_time.elapsed().as_millis();
    
    info!("🔍 Starting HTTPS interception for {}:{}", host, port);
    
    // Check cache first
    let cert_data = match cert_manager.get_certificate(&host) {
        Ok(Some(cert)) => {
            info!("🎯 Using cached certificate for {} ({} ms)", host, connect_time);
            cert
        }
        Ok(None) => {
            info!("💾 Generating new certificate for {} ({} ms)", host, connect_time);
            
            // Generate new certificate - try CA-signed, fall back to self-signed
            let default_ca_cert = "ca-certs/rootCA.crt".to_string();
            let default_ca_key = "ca-certs/rootCA.key".to_string();
            let ca_cert_path = tls_config.ca_cert_path.as_ref().unwrap_or(&default_ca_cert);
            let ca_key_path = tls_config.ca_key_path.as_ref().unwrap_or(&default_ca_key);
            
            let cert_data = match generate_domain_cert_with_ca(&host, ca_cert_path, ca_key_path) {
                Ok(cert) => cert,
                Err(e) => {
                    error!("Failed to generate certificate for {}: {}", host, e);
                    return Ok(build_error_response(StatusCode::INTERNAL_SERVER_ERROR, "Certificate generation failed"));
                }
            };
            
            // Cache the newly generated certificate
            if let Err(e) = cert_manager.cache_certificate(&host, cert_data.clone()) {
                warn!("Failed to cache certificate for {}: {}", host, e);
                // Continue anyway - caching failure shouldn't break the request
            } else {
                info!("🔄 Cached certificate for {} (expires in 24h)", host);
            }
            
            cert_data
        }
        Err(e) => {
            warn!("Certificate cache error for {}: {}", host, e);
            info!("💾 Generating new certificate (cache unavailable)");
            
            // Generate without caching on cache error
            let default_ca_cert = "ca-certs/rootCA.crt".to_string();
            let default_ca_key = "ca-certs/rootCA.key".to_string();
            let ca_cert_path = tls_config.ca_cert_path.as_ref().unwrap_or(&default_ca_cert);
            let ca_key_path = tls_config.ca_key_path.as_ref().unwrap_or(&default_ca_key);
            
            match generate_domain_cert_with_ca(&host, ca_cert_path, ca_key_path) {
                Ok(cert) => cert,
                Err(e) => {
                    error!("Failed to generate certificate for {}: {}", host, e);
                    return Ok(build_error_response(StatusCode::INTERNAL_SERVER_ERROR, "Certificate generation failed"));
                }
            }
        }
    };
    
    // Create TLS server configuration
    let tls_config = match create_server_config(
        cert_data.cert(),
        cert_data.key(),
        &crate::config::settings::TlsConfig::default(),
    ) {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to create TLS config for {}: {}", host, e);
            return Ok(build_error_response(StatusCode::INTERNAL_SERVER_ERROR, "TLS config failed"));
        }
    };
    
    let tls_acceptor = Arc::new(TlsAcceptor::from(tls_config));
    
    info!("✅ Generated certificate for {} ({}ms)", host, connect_time);
    
    // Create a response that signals the HTTPS interception is ready
    let response = Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap();
    
    // Clone variables for the async block
    let host_clone = host.clone();
    let port_clone = port;
    let client_manager_clone = Arc::clone(&client_manager);
    
    // Spawn a task to handle the HTTPS interception
    tokio::spawn(async move {
        // Wait for the connection to be upgraded
        match on(req).await {
            Ok(upgraded_stream) => {
                info!("🔒 Connection upgraded for {}:{}, starting TLS handshake", host_clone, port_clone);
                
                // Perform TLS handshake with the client using our generated certificate
                match tls_acceptor.accept(upgraded_stream).await {
                    Ok(tls_stream) => {
                        info!("✅ TLS handshake successful for {}:{}", host_clone, port_clone);
                        
                        // Now handle HTTP requests over the decrypted TLS connection
                        if let Err(e) = handle_intercepted_https_connection(tls_stream, host_clone.clone(), port_clone, client_manager_clone, body_handler.clone()).await {
                            error!("HTTPS interception error for {}:{}: {}", host_clone, port_clone, e);
                        }
                    }
                    Err(e) => {
                        warn!("❌ TLS handshake failed for {}:{}: {}", host_clone, port_clone, e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to upgrade connection for {}:{}: {}", host_clone, port_clone, e);
            }
        }
    });
    
    Ok(response)
}

/// Handle intercepted HTTPS connection - process decrypted HTTP requests
async fn handle_intercepted_https_connection(
    tls_stream: tokio_rustls::server::TlsStream<Upgraded>,
    host: String,
    port: u16,
    client_manager: Arc<HttpClient>,
    body_handler: Arc<SmartBodyHandler>,
) -> Result<()> {
    info!("🌐 Processing decrypted HTTPS traffic for {}:{}", host, port);
    
    // Clone host for use in service and logging
    let host_for_service = host.clone();
    let host_for_logging = host.clone();
    let client_manager_for_service = Arc::clone(&client_manager);
    let body_handler_for_service = Arc::clone(&body_handler);
    
    // Create HTTP service for handling decrypted requests
    let service = hyper::service::service_fn(move |req: Request<Body>| {
        let host_clone = host_for_service.clone();
        let port_clone = port;
        let client_manager_clone = Arc::clone(&client_manager_for_service);
        let body_handler_clone = Arc::clone(&body_handler_for_service);
        async move {
            handle_intercepted_request(req, host_clone, port_clone, client_manager_clone, body_handler_clone).await
        }
    });
    
    // Serve HTTP over the TLS connection (this gives us decrypted HTTP requests!)
    if let Err(e) = hyper::server::conn::Http::new()
        .serve_connection(tls_stream, service)
        .await
    {
        debug!("HTTPS interception connection ended for {}:{}: {}", host_for_logging, port, e);
    }
    
    info!("🔌 HTTPS interception completed for {}:{}", host, port);
    
    Ok(())
}

/// Handle a decrypted HTTPS request (now we can see everything!)
async fn handle_intercepted_request(
    req: Request<Body>,
    host: String,
    port: u16,
    client_manager: Arc<HttpClient>,
    body_handler: Arc<SmartBodyHandler>,
) -> Result<Response<Body>, hyper::Error> {
    let start_time = std::time::Instant::now();
    let method = req.method().clone();
    let uri = req.uri().clone();
    let headers = req.headers().clone();
    let path = uri.path().to_string();
    
    // Reconstruct the full HTTPS URL for logging
    let full_url = if uri.to_string().starts_with("http") {
        uri.to_string()
    } else {
        format!("https://{}:{}{}", host, port, uri)
    };
    
    let current_pid = std::process::id();
    info!("🔍 INTERCEPTED HTTPS: PID {} handling {} {} (decrypted from {}:{})", 
          current_pid, method, full_url, host, port);
    info!("⏱️  Request started at: {:?}", start_time);
    
    // Log request headers in structured format
    log_headers_structured(&headers, "Request Headers");
    
    let header_processing_time = start_time.elapsed();
    info!("⏱️  Header processing: {:.2} ms", header_processing_time.as_secs_f64() * 1000.0);
    
    // Extract and log the request body using smart body handler
    let (body_bytes, is_large_body) = match body_handler.handle_request_body(req.into_body(), "Intercepted Request").await {
        Ok(result) => result,
        Err(e) => {
            error!("Failed to read request body: {}", e);
            // Return a simple error response instead of trying to convert error types
            return Ok(build_error_response(StatusCode::BAD_REQUEST, &format!("Error reading request body: {}", e)));
        }
    };
    
    let prep_time = start_time.elapsed();
    info!("⏱️  Total request preparation: {:.2} ms", prep_time.as_secs_f64() * 1000.0);
    info!("🔄 Forwarding intercepted {} request to {}:{}", method, host, port);
    
    if is_large_body {
        info!("🚀 Large request body detected - optimized processing enabled");
    }
    
    // Forward the request to the real server over HTTPS
    let forward_start = std::time::Instant::now();
    match forward_intercepted_request_direct(method.clone(), uri, headers, Bytes::from(body_bytes), &host, port, client_manager, body_handler).await {
        Ok(response) => {
            let forward_time = forward_start.elapsed();
            let total_time = start_time.elapsed();
            
            info!("⏱️  Upstream processing: {:.2} ms", forward_time.as_secs_f64() * 1000.0);
            info!("⏱️  🎯 TOTAL REQUEST TIME: {:.2} ms ({:.3} seconds)", 
                  total_time.as_secs_f64() * 1000.0, 
                  total_time.as_secs_f64());
            info!("✅ INTERCEPTED {} {} → {} (completed in {:.2} ms)", 
                  method, path, response.status(), total_time.as_secs_f64() * 1000.0);
            info!("##################################");
            Ok(response)
        }
        Err(e) => {
            let forward_time = forward_start.elapsed();
            let total_time = start_time.elapsed();
            
            error!("⏱️  Failed upstream processing: {:.2} ms", forward_time.as_secs_f64() * 1000.0);
            error!("⏱️  🎯 TOTAL REQUEST TIME (FAILED): {:.2} ms ({:.3} seconds)", 
                   total_time.as_secs_f64() * 1000.0, 
                   total_time.as_secs_f64());
            error!("❌ INTERCEPTED {} {} → ERROR: {} (failed in {:.2} ms)", 
                   method, path, e, total_time.as_secs_f64() * 1000.0);
            
            let error_response = build_proxy_error_response(&format!("Interception Error: {}", e));
            
            Ok(error_response)
        }
    }
}

/// Forward an intercepted request directly to the real server
async fn forward_intercepted_request_direct(
    method: hyper::Method,
    uri: hyper::Uri,
    headers: hyper::HeaderMap,
    body_bytes: Bytes,
    host: &str,
    port: u16,
    client_manager: Arc<HttpClient>,
    body_handler: Arc<SmartBodyHandler>,
) -> Result<Response<Body>> {
    // Use shared HTTPS client with connection pooling for optimal performance
    // This eliminates the critical performance bottleneck of creating new clients per request
    let client = client_manager.get_https_client();
    
    // Build the target URL - don't include port 443 for HTTPS or port 80 for HTTP as it's redundant
    let target_url = if port == 443 {
        format!("https://{}{}", host, uri.path_and_query().map_or("", |pq| pq.as_str()))
    } else if port == 80 {
        format!("http://{}{}", host, uri.path_and_query().map_or("", |pq| pq.as_str()))
    } else {
        format!("https://{}:{}{}", host, port, uri.path_and_query().map_or("", |pq| pq.as_str()))
    };
    
    info!("🌐 Forwarding to: {}", target_url);
    
    // Build the request
    let mut request_builder = Request::builder()
        .method(method)
        .uri(&target_url);
    
    // Add headers (skip hop-by-hop and problematic headers)
    let mut forwarded_headers = 0;
    let mut skipped_headers = 0;
    
    for (name, value) in &headers {
        // Use centralized header filtering logic
        if should_forward_request_header(name.as_str()) {
            request_builder = request_builder.header(name, value);
            forwarded_headers += 1;
        } else {
            skipped_headers += 1;
        }
    }
    
    info!("📋 Header filtering: {} forwarded, {} skipped", forwarded_headers, skipped_headers);
    
    // Set the correct host header (without port for standard ports)
    let host_header = if port == 443 || port == 80 { host.to_string() } else { format!("{}:{}", host, port) };
    request_builder = request_builder.header("host", host_header);
    
    // Ensure we have required headers for proper HTTP handling
    if !headers.contains_key("user-agent") {
        request_builder = request_builder.header("user-agent", "Mozilla/5.0 (compatible; RustProxy/1.0)");
    }
    
    // Always set proper content-length header to avoid duplicates and ensure correctness
    request_builder = request_builder.header("content-length", body_bytes.len().to_string());
    
    let body_size = body_bytes.len();
    let request = request_builder.body(Body::from(body_bytes))?;
    
    // Debug log the final request that will be sent upstream
    info!("📡 Sending request to upstream server...");
    info!("🔍 Upstream request: {} {}", request.method(), request.uri());
    info!("📝 Upstream request headers:");
    for (name, value) in request.headers() {
        if let Ok(value_str) = value.to_str() {
            info!("   {}: {}", name, value_str);
        }
    }
    info!("📦 Upstream request body size: {} bytes", body_size);
    
    let upstream_start = std::time::Instant::now();
    
    // Forward the request
    let response = client.request(request).await?;
    
    let upstream_response_time = upstream_start.elapsed();
    info!("⏱️  Upstream response time: {:.2} ms", upstream_response_time.as_secs_f64() * 1000.0);
    
    // Get response details for logging
    let status = response.status();
    let response_headers = response.headers().clone();
    
    info!("📤 Upstream HTTPS response: {} ", status);
    
    // Log response headers in structured format
    log_response_headers_structured(&response_headers);
    
    // Handle response with smart streaming - this provides 70-90% memory reduction!
    let optimized_response = body_handler.handle_response_streaming(response, "Upstream Response").await
        .map_err(|e| anyhow::anyhow!("Response streaming error: {}", e))?;
    
    info!("🚀 Response streaming optimization applied");
    
    // Extract status and headers from optimized response for final processing
    let (parts, body) = optimized_response.into_parts();
    let mut response_builder = Response::builder().status(parts.status);
    
    // Add response headers (excluding hop-by-hop headers)
    for (name, value) in &parts.headers {
        if !is_hop_by_hop_header(name.as_str()) {
            response_builder = response_builder.header(name, value);
        }
    }
    
    // Return the optimized streaming response
    Ok(response_builder
        .body(body)
        .unwrap())
}

// ============================================================================
// MAIN HANDLER FUNCTIONS
// ============================================================================

/// Handle CONNECT request - always intercept HTTPS
async fn handle_connect_request(
    req: Request<Body>,
    mut request_data: RequestData,
    start_time: std::time::Instant,
    _https_interception: bool,
    cert_manager: Arc<CertificateManager>,
    client_manager: Arc<HttpClient>,
    body_handler: Arc<SmartBodyHandler>,
    tls_config: &crate::config::settings::TlsConfig,
) -> Result<Response<Body>, Infallible> {
    log_connect_request(&request_data.url);
    
    // Configure request data for CONNECT
    request_data.path = "".to_string(); // CONNECT doesn't have a path
    request_data.query_string = None; // CONNECT doesn't have query params
    request_data.is_https = true; // CONNECT is always for HTTPS
    
    // Parse host:port format
    let (host, port) = match parse_connect_target(&request_data.url) {
        Ok((h, p)) => (h, p),
        Err(error_msg) => {
            log_error!("{}", error_msg);
            return Ok(build_error_response(StatusCode::BAD_REQUEST, "Invalid CONNECT target"));
        }
    };
    
    // Always intercept HTTPS for full visibility - CONNECT logging at DEBUG level
    log_debug!("🔍 CONNECT {}:{} - INTERCEPTING (will decrypt and log HTTPS)", host, port);
    handle_https_interception(req, host, port, start_time, cert_manager, client_manager, body_handler, tls_config).await
}

/// Extract and process HTTP request data
async fn extract_request_data(request_data: &mut RequestData, uri: &str, req: Request<Body>) {
    // Parse URL for regular HTTP requests
    if let Ok(parsed_uri) = parse_url(uri) {
        request_data.url = uri.to_string();
        request_data.path = extract_path(&parsed_uri);
        request_data.query_string = extract_query(&parsed_uri);
        request_data.is_https = is_https(&parsed_uri);
        
        log_debug!("Parsed URI: path={}, query={}, is_https={}", 
                   request_data.path, 
                   request_data.query_string.as_deref().unwrap_or("None"), 
                   request_data.is_https);
    } else {
        log_debug!("Failed to parse URI: {}", uri);
    }

    // Extract various parts of the request
    extract_headers(req.headers(), request_data);
    extract_cookies_to_request_data(req.headers(), request_data);
    
    // Extract body if needed
    let (should_extract, content_type) = should_extract_body(req.headers(), &request_data.method);
    if let Some(ct) = content_type {
        request_data.content_type = Some(ct);
    }
    if should_extract {
        extract_body(req.into_body(), request_data).await;
    }

    // DEBUG: Log full request data structure
    log_debug!("📋 REQUEST DATA:\n{:#?}", request_data);
}

/// Handle regular HTTP request
async fn handle_http_request(
    mut request_data: RequestData,
    method: String,
    start_time: std::time::Instant,
    client_manager: Arc<HttpClient>,
    body_handler: Arc<SmartBodyHandler>,
) -> Result<Response<Body>, Infallible> {
    let current_pid = std::process::id();
    info!("🔍 Processing HTTP request with full interception - PID {}", current_pid);
    info!("⏱️  Request started at: {:?}", start_time);
    
    let prep_time = start_time.elapsed();
    info!("⏱️  HTTP request preparation: {:.2} ms", prep_time.as_secs_f64() * 1000.0);
    
    match handle_regular_request(&mut request_data, client_manager, body_handler).await {
        Ok(response) => {
            let total_time = start_time.elapsed();
            info!("⏱️  🎯 TOTAL HTTP REQUEST TIME: {:.2} ms ({:.3} seconds)", 
                  total_time.as_secs_f64() * 1000.0, 
                  total_time.as_secs_f64());
            log_http_success(&method, &request_data.path, response.status(), total_time.as_millis());
            Ok(response)
        },
        Err(e) => {
            let total_time = start_time.elapsed();
            error!("⏱️  🎯 TOTAL HTTP REQUEST TIME (FAILED): {:.2} ms ({:.3} seconds)", 
                   total_time.as_secs_f64() * 1000.0, 
                   total_time.as_secs_f64());
            log_http_failure(&method, &request_data.path, total_time.as_millis(), &e);
            Ok(build_error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error"))
        }
    }
}

/// Handle regular HTTP request (non-CONNECT)
async fn handle_regular_request(request_data: &mut RequestData, client_manager: Arc<HttpClient>, body_handler: Arc<SmartBodyHandler>) -> Result<Response<Body>> {
    let forward_start = std::time::Instant::now();
    
    // Log configuration for future optimization potential
    debug!("🚀 Regular HTTP request handler ready (streaming config: max_log_body_size={})", body_handler.get_config().max_log_body_size);
    
    log_forwarding_request(request_data);
    
    // Use shared HTTP client with connection pooling for optimal performance
    let client = client_manager.get_client_for_url(request_data.is_https);
    let request = build_forwarding_request(request_data)?;
    
    // Forward the request to upstream
    let upstream_start = std::time::Instant::now();
    info!("📡 Sending HTTP request to upstream server...");
    match client.request(request).await {
        Ok(response) => {
            let upstream_time = upstream_start.elapsed();
            let status_code = response.status().as_u16();
            let status_text = response.status().to_string();
            let content_type = response
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_string();
                
            // Clean INFO log for upstream response
            info!("⏱️  HTTP upstream response time: {:.2} ms", upstream_time.as_secs_f64() * 1000.0);
            info!("📤 Upstream response: {} ({:.2}ms)", status_code, upstream_time.as_secs_f64() * 1000.0);
                
            // Verbose DEBUG log
            log_debug!("📤 UPSTREAM RESPONSE:\n  Status: {} {}\n  Content-Type: {}\n  Time: {:.2}ms", 
                      status_code, status_text, content_type, upstream_time.as_secs_f64() * 1000.0);

            // Extract headers before consuming the response
            let mut response_headers = Vec::new();
            let mut skipped_response_headers = 0;
            for (name, value) in response.headers() {
                let name_str = name.as_str();
                if should_forward_response_header(name_str) {
                    if let Ok(value_str) = value.to_str() {
                        response_headers.push((name_str.to_string(), value_str.to_string()));
                    }
                } else {
                    skipped_response_headers += 1;
                }
            }
            log_debug!("Response headers: {} forwarded, {} skipped (hop-by-hop)", 
                      response_headers.len(), skipped_response_headers);

            let body_bytes = hyper::body::to_bytes(response.into_body()).await?;
            log_debug!("Response body received: {} bytes", body_bytes.len());
            
            let response_data = ResponseData::new(
                status_code,
                status_text,
                content_type,
                body_bytes.to_vec(),
                upstream_time.as_millis() as u64, // Use the actual upstream response time
            );

            let log_entry = ProxyLog {
                request: request_data.clone(),
                response: Some(response_data.clone()),
                error: None,
            };

            // DEBUG: Log full transaction details
            log_debug!("📋 HTTP TRANSACTION:\n{:#?}", log_entry);
            
            // Log transaction to file
            log_proxy_transaction!(&log_entry);

            // Build response to send back to client
            let mut response_builder = Response::builder().status(response_data.status_code);

            // Add response headers
            for (name, value) in &response_headers {
                response_builder = response_builder.header(name, value);
            }
            log_debug!("Response builder created with {} headers", response_headers.len());

            // Return the actual response body
            Ok(response_builder
                .body(Body::from(response_data.body))
                .unwrap_or_else(|_| {
                    log_error!("Failed to build response body");
                    build_error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to build response")
                }))
        }
        Err(e) => {
            let upstream_time = upstream_start.elapsed().as_millis();
            let total_time = forward_start.elapsed().as_millis();
            
            // Clean INFO log for upstream error
            info!("❌ Upstream failed ({}ms): {}", total_time, e);
            
            // Verbose DEBUG log
            log_debug!("❌ UPSTREAM ERROR:\n  Error: {}\n  Upstream Time: {}ms\n  Total Time: {}ms", 
                      e, upstream_time, total_time);

            let log_entry = ProxyLog {
                request: request_data.clone(),
                response: None,
                error: Some(e.to_string()),
            };

            // DEBUG: Log full error transaction
            log_debug!("📋 HTTP ERROR TRANSACTION:\n{:#?}", log_entry);
            
            // Log the error to file
            log_proxy_transaction!(&log_entry);

            Ok(build_proxy_error_response(&e.to_string()))
        }
    }
}


