//! Proxy server implementation

use crate::models::{ProxyLog, RequestData, ResponseData};
use crate::{log_info, log_error, log_debug, log_proxy_transaction};
use crate::utils::{is_hop_by_hop_header, parse_url, extract_path, extract_query, is_https, parse_connect_target, build_error_response, extract_headers, extract_cookies_to_request_data, should_extract_body, extract_body, build_forwarding_request, log_incoming_request, log_connect_request, log_connect_success, log_connect_failure, create_connect_transaction, log_http_success, log_http_failure, log_forwarding_request};
use crate::tls::{generate_self_signed_cert, generate_domain_cert_with_ca, create_server_config, CertificateManager};
use anyhow::Result;
use hyper::Client;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use std::convert::Infallible;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio_rustls::TlsAcceptor;
use tracing::{error, info, debug, warn};
use hyper::upgrade::{Upgraded, on};
use futures::future::try_join;
use serde_json::json;
use std::sync::Arc;
use bytes::Bytes;


pub struct ProxyServer {
    listen_addr: SocketAddr,
    https_interception: bool,
    cert_manager: Arc<CertificateManager>,
}

impl ProxyServer {
    pub fn new(listen_addr: SocketAddr) -> Self {
        Self { 
            listen_addr,
            https_interception: false, // Default to false for backward compatibility
            cert_manager: Arc::new(CertificateManager::new()),
        }
    }
    
    pub fn with_https_interception(listen_addr: SocketAddr, enable_interception: bool) -> Self {
        let cert_manager = Arc::new(CertificateManager::new());
        info!("🔐 Certificate cache initialized: {}", cert_manager.cache_info());
        
        Self {
            listen_addr,
            https_interception: enable_interception,
            cert_manager,
        }
    }

    /// Start the proxy server
    pub async fn start(self) -> Result<()> {
        info!("Starting proxy server on {}", self.listen_addr);
        
        // Log server startup
        log_info!("Proxy server starting on {}", self.listen_addr);
        log_debug!("Server configuration: listen_addr={}", self.listen_addr);
        
        if self.https_interception {
            log_info!("🔍 HTTPS interception mode: ENABLED - all HTTPS content will be logged!");
        } else {
            log_info!("🔒 HTTPS interception mode: DISABLED - HTTPS will be tunneled");
        }

        let https_interception = self.https_interception;
        let cert_manager = Arc::clone(&self.cert_manager);
        let make_svc = make_service_fn(move |conn: &hyper::server::conn::AddrStream| {
            let remote_addr = conn.remote_addr();
            let cert_manager = Arc::clone(&cert_manager);
            log_debug!("New connection from: {}", remote_addr);

            async move { 
                Ok::<_, Infallible>(service_fn(move |req| {
                    let cert_manager = Arc::clone(&cert_manager);
                    handle_request(req, remote_addr, https_interception, cert_manager)
                })) 
            }
        });

        log_debug!("Creating server with service factory");
        let server = Server::bind(&self.listen_addr).serve(make_svc);
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
) -> Result<Response<Body>, Infallible> {
    let start_time = std::time::Instant::now();
    let method = req.method().to_string();
    let uri = req.uri().to_string();

    // Log incoming request
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
    
    // Handle CONNECT requests - either intercept HTTPS or tunnel
    if method == "CONNECT" {
        return handle_connect_request(req, request_data, start_time, https_interception, cert_manager).await;
    } else {
        // Extract and process regular HTTP request data
        extract_request_data(&mut request_data, &uri, req).await;
        
        // Handle regular HTTP requests with full interception
        handle_http_request(request_data, method, start_time).await
    }
}

/// Handle health check endpoint locally
async fn handle_health_check(
    method: String,
    start_time: std::time::Instant,
) -> Result<Response<Body>, Infallible> {
    let elapsed_time = start_time.elapsed().as_millis();
    
    if method == "GET" {
        // Create health response
        let health_data = json!({
            "status": "healthy",
            "service": "rust-forward-proxy",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "uptime_ms": elapsed_time,
            "version": env!("CARGO_PKG_VERSION")
        });
        
        let response = Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "application/json")
            .header("cache-control", "no-cache")
            .body(Body::from(health_data.to_string()))
            .unwrap();
            
        log_info!("✅ GET /health → 200 OK ({}ms)", elapsed_time);
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

/// Handle traditional CONNECT tunneling (no interception)
async fn handle_connect_tunnel_only(
    req: Request<Body>,
    request_data: RequestData,
    host: String,
    port: u16,
    start_time: std::time::Instant,
) -> Result<Response<Body>, Infallible> {
    // Attempt to establish connection to target
    match TcpStream::connect(format!("{}:{}", host, port)).await {
        Ok(upstream_stream) => {
            let connect_time = start_time.elapsed().as_millis();
            log_connect_success(&host, port, connect_time);
            
            // Create response data and log transaction
            let response_data = ResponseData::new(
                200,
                "OK".to_string(),
                "tunnel".to_string(),
                vec![],
                connect_time as u64,
            );
            
            create_connect_transaction(&request_data, Some(response_data), None);
            
            // Handle the upgrade and tunnel the data
            handle_connect_tunnel_upgrade(req, upstream_stream, &host, port).await
        },
        Err(e) => {
            let connect_time = start_time.elapsed().as_millis();
            let error_msg = format!("Failed to connect to {}:{}: {}", host, port, e);
            
            log_connect_failure(&host, port, connect_time, &error_msg);
            create_connect_transaction(&request_data, None, Some(error_msg));
            
            Ok(build_error_response(StatusCode::BAD_GATEWAY, "CONNECT failed"))
        }
    }
}

/// Handle CONNECT tunnel with proper upgrade mechanism
async fn handle_connect_tunnel_upgrade(
    req: Request<Body>,
    upstream_stream: TcpStream,
    host: &str,
    port: u16,
) -> Result<Response<Body>, Infallible> {
    log_debug!("Setting up CONNECT tunnel for {}:{}", host, port);
    
    // Create a response that signals the tunnel is ready
    let response = Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap();
    
    // Clone the host and port for the async block
    let host_clone = host.to_string();
    let port_clone = port;
    
    // Spawn a task to handle the actual data tunneling
    tokio::spawn(async move {
        // Wait for the connection to be upgraded
        match on(req).await {
            Ok(upgraded) => {
                log_debug!("CONNECT tunnel upgraded successfully for {}:{}", host_clone, port_clone);
                
                // Start bidirectional data copying
                if let Err(e) = tunnel_bidirectional(upgraded, upstream_stream).await {
                    info!("🔌 Tunnel closed for {}:{}: {}", host_clone, port_clone, e);
                    log_debug!("🔌 TUNNEL CLOSED:\n  Target: {}:{}\n  Reason: {}", host_clone, port_clone, e);
                } else {
                    info!("🔌 Tunnel completed for {}:{}", host_clone, port_clone);
                    log_debug!("🔌 TUNNEL COMPLETED:\n  Target: {}:{}\n  Clean closure", host_clone, port_clone);
                }
            }
            Err(e) => {
                log_error!("Failed to upgrade CONNECT tunnel for {}:{}: {}", host_clone, port_clone, e);
            }
        }
    });
    
    Ok(response)
}

/// Perform bidirectional data copying between client and upstream
async fn tunnel_bidirectional(
    client: Upgraded, 
    mut upstream: TcpStream
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (mut client_read, mut client_write) = tokio::io::split(client);
    let (mut upstream_read, mut upstream_write) = upstream.split();
    
    // Create bidirectional copying tasks
    let client_to_upstream = async {
        tokio::io::copy(&mut client_read, &mut upstream_write).await?;
        Ok::<_, std::io::Error>(())
    };
    
    let upstream_to_client = async {
        tokio::io::copy(&mut upstream_read, &mut client_write).await?;
        Ok::<_, std::io::Error>(())
    };
    
    // Run both directions concurrently
    try_join(client_to_upstream, upstream_to_client).await?;
    
    Ok(())
}

/// Handle HTTPS interception - decrypt, log, and re-encrypt
async fn handle_https_interception(
    req: Request<Body>,
    host: String,
    port: u16,
    start_time: std::time::Instant,
    cert_manager: Arc<CertificateManager>,
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
            let ca_cert_path = "ca-certs/rootCA.crt";
            let ca_key_path = "ca-certs/rootCA.key";
            
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
            let ca_cert_path = "ca-certs/rootCA.crt";
            let ca_key_path = "ca-certs/rootCA.key";
            
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
        cert_data.cert,
        cert_data.key,
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
    
    // Create a response that signals the tunnel is ready
    let response = Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap();
    
    // Clone variables for the async block
    let host_clone = host.clone();
    let port_clone = port;
    
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
                        if let Err(e) = handle_intercepted_https_connection(tls_stream, host_clone.clone(), port_clone).await {
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
) -> Result<()> {
    info!("🌐 Processing decrypted HTTPS traffic for {}:{}", host, port);
    
    // Clone host for use in service and logging
    let host_for_service = host.clone();
    let host_for_logging = host.clone();
    
    // Create HTTP service for handling decrypted requests
    let service = hyper::service::service_fn(move |req: Request<Body>| {
        let host_clone = host_for_service.clone();
        let port_clone = port;
        async move {
            handle_intercepted_request(req, host_clone, port_clone).await
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
    
    info!("🔍 INTERCEPTED HTTPS: {} {} (decrypted from {}:{})", method, full_url, host, port);
    
    // Log request headers
    info!("📋 Request Headers:");
    for (name, value) in &headers {
        if let Ok(value_str) = value.to_str() {
            info!("  {}: {}", name, value_str);
        }
    }
    
    // Extract and log the request body
    let body_bytes = hyper::body::to_bytes(req.into_body()).await
        .map_err(|e| {
            error!("Failed to read request body: {}", e);
            e
        })?;
    
    if !body_bytes.is_empty() {
        info!("📄 Request Body ({} bytes):", body_bytes.len());
        if let Ok(body_str) = std::str::from_utf8(&body_bytes) {
            info!("{}", body_str);
        } else {
            info!("  [Binary data - {} bytes]", body_bytes.len());
        }
    }
    
    info!("🔄 Forwarding intercepted {} request to {}:{}", method, host, port);
    
    // Forward the request to the real server over HTTPS
    match forward_intercepted_request_direct(method.clone(), uri, headers, body_bytes, &host, port).await {
        Ok(response) => {
            let total_time = start_time.elapsed().as_millis();
            info!("✅ INTERCEPTED {} {} → {} ({} ms)", method, path, response.status(), total_time);
            info!("##################################");
            Ok(response)
        }
        Err(e) => {
            let total_time = start_time.elapsed().as_millis();
            error!("❌ INTERCEPTED {} {} → ERROR: {} ({} ms)", method, path, e, total_time);
            
            let error_response = Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .header("Content-Type", "text/plain")
                .body(Body::from(format!("Interception Error: {}", e)))
                .unwrap();
            
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
) -> Result<Response<Body>> {
    // Create HTTPS client 
    let https = hyper_rustls::HttpsConnectorBuilder::new()
        .with_native_roots()
        .https_or_http()
        .enable_http1()
        .build();
    
    let client = Client::builder().build::<_, hyper::Body>(https);
    
    // Build the target URL
    let target_url = format!("https://{}:{}{}", host, port, uri.path_and_query().map_or("", |pq| pq.as_str()));
    
    info!("🌐 Forwarding to: {}", target_url);
    
    // Build the request
    let mut request_builder = Request::builder()
        .method(method)
        .uri(&target_url);
    
    // Add headers (skip hop-by-hop headers)
    for (name, value) in &headers {
        if !is_hop_by_hop_header(name.as_str()) && name != "host" {
            request_builder = request_builder.header(name, value);
        }
    }
    
    // Set the correct host header
    request_builder = request_builder.header("host", host);
    
    let request = request_builder.body(Body::from(body_bytes))?;
    
    info!("📡 Sending request to upstream server...");
    
    // Forward the request
    let response = client.request(request).await?;
    
    // Get response details for logging
    let status = response.status();
    let response_headers = response.headers().clone();
    
    info!("📤 Upstream HTTPS response: {} ", status);
    
    // Log response headers
    info!("📋 Response Headers:");
    for (name, value) in &response_headers {
        if let Ok(value_str) = value.to_str() {
            info!("  {}: {}", name, value_str);
        }
    }
    
    // Convert response body
    let body_bytes = hyper::body::to_bytes(response.into_body()).await?;
    
    info!("📄 Response Body ({} bytes):", body_bytes.len());
    if !body_bytes.is_empty() {
        if let Ok(body_str) = std::str::from_utf8(&body_bytes) {
            // Only log first 1000 chars to avoid spam
            let display_body = if body_str.len() > 1000 {
                format!("{}...[truncated]", &body_str[..1000])
            } else {
                body_str.to_string()
            };
            info!("{}", display_body);
        } else {
            info!("  [Binary response data - {} bytes]", body_bytes.len());
        }
    }
    
    // Build response to send back to client
    let mut response_builder = Response::builder().status(status);
    
    // Add response headers (excluding hop-by-hop headers)
    for (name, value) in &response_headers {
        if !is_hop_by_hop_header(name.as_str()) {
            response_builder = response_builder.header(name, value);
        }
    }
    
    // Return the response body
    Ok(response_builder
        .body(Body::from(body_bytes))
        .unwrap())
}

// ============================================================================
// MAIN HANDLER FUNCTIONS
// ============================================================================

/// Handle CONNECT request - either intercept HTTPS or tunnel
async fn handle_connect_request(
    req: Request<Body>,
    mut request_data: RequestData,
    start_time: std::time::Instant,
    https_interception: bool,
    cert_manager: Arc<CertificateManager>,
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
    
    // Decide whether to intercept or tunnel based on port and configuration
    if https_interception && port == 443 {
        log_info!("🔍 CONNECT {}:{} - INTERCEPTING (will decrypt and log HTTPS)", host, port);
        handle_https_interception(req, host, port, start_time, cert_manager).await
    } else {
        log_debug!("🔒 CONNECT {}:{} - TUNNELING (pass-through)", host, port);
        handle_connect_tunnel_only(req, request_data, host, port, start_time).await
    }
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
) -> Result<Response<Body>, Infallible> {
    log_debug!("🔍 Processing HTTP request with full interception");
    
            match handle_regular_request(&mut request_data).await {
                Ok(response) => {
                    let total_time = start_time.elapsed().as_millis();
            log_http_success(&method, &request_data.path, response.status(), total_time);
                    Ok(response)
                },
                Err(e) => {
                    let total_time = start_time.elapsed().as_millis();
            log_http_failure(&method, &request_data.path, total_time, &e);
            Ok(build_error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error"))
        }
    }
}

/// Handle regular HTTP request (non-CONNECT)
async fn handle_regular_request(request_data: &mut RequestData) -> Result<Response<Body>> {
    let forward_start = std::time::Instant::now();
    
    log_forwarding_request(request_data);
    
    // Create HTTP client and build request
    let client = Client::new();
    let request = build_forwarding_request(request_data)?;
    
    // Forward the request to upstream
    let upstream_start = std::time::Instant::now();
    log_debug!("Sending request to upstream server");
    match client.request(request).await {
        Ok(response) => {
            let upstream_time = upstream_start.elapsed().as_millis();
            let status_code = response.status().as_u16();
            let status_text = response.status().to_string();
            let content_type = response
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_string();
                
            // Clean INFO log for upstream response
            info!("📤 Upstream response: {} ({}ms)", status_code, upstream_time);
                
            // Verbose DEBUG log
            log_debug!("📤 UPSTREAM RESPONSE:\n  Status: {} {}\n  Content-Type: {}\n  Time: {}ms", 
                      status_code, status_text, content_type, upstream_time);

            // Extract headers before consuming the response
            let mut response_headers = Vec::new();
            let mut skipped_response_headers = 0;
            for (name, value) in response.headers() {
                let name_str = name.as_str();
                if !is_hop_by_hop_header(name_str) {
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
                upstream_time as u64, // Use the actual upstream response time
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
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from("Failed to build response"))
                        .unwrap()
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

            Ok(Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .header("Content-Type", "text/plain")
                .body(Body::from(format!("Proxy Error: {}", e)))
                .unwrap())
        }
    }
}


