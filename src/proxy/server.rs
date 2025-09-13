//! Proxy server implementation

use crate::models::{ProxyLog, RequestData, ResponseData};
use crate::{log_info, log_error, log_debug, log_proxy_transaction};
use crate::utils::{is_hop_by_hop_header, parse_url, extract_path, extract_query, is_https, parse_connect_target, build_error_response, extract_headers, extract_cookies_to_request_data, should_extract_body, extract_body, build_forwarding_request, log_incoming_request, log_connect_request, log_connect_success, log_connect_failure, create_connect_transaction, log_http_success, log_http_failure, log_forwarding_request};
use anyhow::Result;
use hyper::Client;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use std::convert::Infallible;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tracing::{error, info};
use hyper::upgrade::{Upgraded, on};
use futures::future::try_join;
use serde_json::json;


pub struct ProxyServer {
    listen_addr: SocketAddr,
}

impl ProxyServer {
    pub fn new(listen_addr: SocketAddr) -> Self {
        Self { listen_addr }
    }

    /// Start the proxy server
    pub async fn start(self) -> Result<()> {
        info!("Starting proxy server on {}", self.listen_addr);
        
        // Log server startup
        log_info!("Proxy server starting on {}", self.listen_addr);
        log_debug!("Server configuration: listen_addr={}", self.listen_addr);

        let make_svc = make_service_fn(|conn: &hyper::server::conn::AddrStream| {
            let remote_addr = conn.remote_addr();
            log_debug!("New connection from: {}", remote_addr);

            async move { 
                Ok::<_, Infallible>(service_fn(move |req| {
                    handle_request(req, remote_addr)
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
async fn handle_request(
    req: Request<Body>,
    remote_addr: SocketAddr,
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
    
    // Special handling for CONNECT requests - don't intercept, just tunnel
    if method == "CONNECT" {
        return handle_connect_request(req, request_data, start_time).await;
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

/// Handle CONNECT tunnel with proper upgrade mechanism
async fn handle_connect_tunnel(
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

// ============================================================================
// MAIN HANDLER FUNCTIONS
// ============================================================================

/// Handle CONNECT request for HTTPS tunneling
async fn handle_connect_request(
    req: Request<Body>,
    mut request_data: RequestData,
    start_time: std::time::Instant,
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
    
    log_debug!("CONNECT tunneling to {}:{}", host, port);
    
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
            handle_connect_tunnel(req, upstream_stream, &host, port).await
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


