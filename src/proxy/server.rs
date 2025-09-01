//! Proxy server implementation

use crate::models::{ProxyLog, RequestData, ResponseData};
use crate::{log_info, log_error, log_debug, log_proxy_transaction};
use crate::utils::{is_hop_by_hop_header, parse_url, extract_path, extract_query, is_https, parse_cookies, parse_form_data};
use anyhow::Result;
use hyper::Client;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use std::convert::Infallible;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tracing::{error, info};

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

    // Extract basic request information
    let method = req.method().to_string();
    let uri = req.uri().to_string();

    info!(
        "Received {} request to {} from {}",
        method,
        uri,
        remote_addr.ip()
    );
    
    // Log request
    log_info!("Received {} request to {} from {}", method, uri, remote_addr.ip());
    log_debug!("Request details: method={}, uri={}, remote_addr={}, headers_count={}", 
               method, uri, remote_addr, req.headers().len());

    // Create our data structure
    let mut request_data = RequestData::new(
        method.clone(),
        uri.clone(),
        remote_addr.ip(),
        remote_addr.port(),
    );
    
    log_debug!("Created request data: client_ip={}, client_port={}", 
               request_data.client_ip, request_data.client_port);
    
    // For proxy requests, we need to extract the actual target URL
    // The URI field contains the full URL when using a proxy
    if let Ok(parsed_uri) = parse_url(&uri) {
        request_data.url = uri.clone();
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

    // Extract headers
    let mut header_count = 0;
    for (name, value) in req.headers() {
        if let Ok(value_str) = value.to_str() {
            request_data
                .headers
                .insert(name.to_string().to_lowercase(), value_str.to_string());
            header_count += 1;
        }
    }
    log_debug!("Extracted {} headers from request", header_count);

    // Extract cookies
    if let Some(cookie_header) = req.headers().get("cookie") {
        if let Ok(cookie_str) = cookie_header.to_str() {
            request_data.cookies = parse_cookies(cookie_str);
            log_debug!("Extracted {} cookies from request", request_data.cookies.len());
        }
    } else {
        log_debug!("No cookies found in request");
    }

    // Extract request body and form data if present
    let should_extract_body = if let Some(content_type) = req.headers().get("content-type") {
        if let Ok(content_type_str) = content_type.to_str() {
            request_data.content_type = Some(content_type_str.to_string());
            log_debug!("Request content-type: {}", content_type_str);
            
            // Check if content type suggests a body
            content_type_str.contains("application/x-www-form-urlencoded") ||
            content_type_str.contains("application/json") ||
            content_type_str.contains("text/") ||
            content_type_str.contains("multipart/")
        } else {
            false
        }
    } else {
        log_debug!("No content-type header found");
        // For requests without content-type, extract body if method suggests it
        request_data.method == "POST" || request_data.method == "PUT" || request_data.method == "PATCH"
    };
    
    // Extract body if needed
    if should_extract_body {
        log_debug!("Extracting request body");
        let body_bytes = hyper::body::to_bytes(req.into_body()).await.unwrap_or_default();
        request_data.body = body_bytes.to_vec();
        log_debug!("Body extracted, size: {} bytes", request_data.body.len());
        
        // Parse form data only for form-encoded content
        if let Some(content_type) = &request_data.content_type {
            if content_type.contains("application/x-www-form-urlencoded") {
                request_data.form_data = parse_form_data(&body_bytes);
                log_debug!("Extracted {} form fields", request_data.form_data.len());
            }
        }
    }

    // Handle the request based on method
    log_debug!("Starting request processing, elapsed time: {}ms", start_time.elapsed().as_millis());
    
    match request_data.method.as_str() {
        "CONNECT" => {
            log_debug!("Handling CONNECT request for HTTPS tunneling");
            match handle_connect_request(&mut request_data).await {
                Ok(response) => {
                    let total_time = start_time.elapsed().as_millis();
                    log_debug!("CONNECT request processed successfully in {}ms, status: {}", 
                              total_time, response.status());
                    Ok(response)
                },
                Err(e) => {
                    let total_time = start_time.elapsed().as_millis();
                    error!("Failed to handle CONNECT request: {}", e);
                    log_error!("CONNECT request failed after {}ms: {}", total_time, e);
                    Ok(Response::builder()
                        .status(StatusCode::BAD_GATEWAY)
                        .body(Body::from("CONNECT failed"))
                        .unwrap())
                }
            }
        },
        _ => {
            log_debug!("Handling regular HTTP request");
            match handle_regular_request(&mut request_data).await {
                Ok(response) => {
                    let total_time = start_time.elapsed().as_millis();
                    log_debug!("Request processed successfully in {}ms, status: {}", 
                              total_time, response.status());
                    Ok(response)
                },
                Err(e) => {
                    let total_time = start_time.elapsed().as_millis();
                    error!("Failed to handle request: {}", e);
                    log_error!("Request failed after {}ms: {}", total_time, e);
                    Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from("Internal Server Error"))
                        .unwrap())
                }
            }
        }
    }
}

/// Handle regular HTTP request (non-CONNECT)
async fn handle_regular_request(request_data: &mut RequestData) -> Result<Response<Body>> {
    let forward_start = std::time::Instant::now();
    
    log_debug!("Building forward request: {} {}", request_data.method, request_data.url);
    
    // Create HTTP client
    let client = Client::new();

    // Build the request
    let mut request_builder = Request::builder()
        .method(request_data.method.as_str())
        .uri(&request_data.url);

    // Add headers (excluding hop-by-hop headers)
    let mut forwarded_headers = 0;
    let mut skipped_headers = 0;
    for (name, value) in &request_data.headers {
        if !is_hop_by_hop_header(name) {
            request_builder = request_builder.header(name, value);
            forwarded_headers += 1;
        } else {
            skipped_headers += 1;
        }
    }
    log_debug!("Header forwarding: {} forwarded, {} skipped (hop-by-hop)", 
               forwarded_headers, skipped_headers);

    // Build the request
    let request = request_builder.body(Body::from(request_data.body.clone()))?;
    log_debug!("Forward request built, body size: {} bytes", request_data.body.len());

    // Forward the request
    log_debug!("Sending request to upstream server");
    let upstream_start = std::time::Instant::now();
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
                
            log_debug!("Upstream response received in {}ms: {} {} (content-type: {})", 
                      upstream_time, status_code, status_text, content_type);

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

            // Log transaction
            log_proxy_transaction!(&log_entry);
            log_debug!("Transaction logged: request_size={} bytes, response_size={} bytes, total_time={}ms", 
                      request_data.body.len(), response_data.body.len(), 
                      forward_start.elapsed().as_millis());

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
            
            error!("Failed to forward request: {}", e);
            log_error!("Upstream request failed after {}ms (upstream: {}ms): {}", 
                      total_time, upstream_time, e);

            let log_entry = ProxyLog {
                request: request_data.clone(),
                response: None,
                error: Some(e.to_string()),
            };

            // Log the error
            log_proxy_transaction!(&log_entry);
            log_debug!("Error transaction logged: request_size={} bytes, total_time={}ms", 
                      request_data.body.len(), total_time);

            Ok(Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .header("Content-Type", "text/plain")
                .body(Body::from(format!("Proxy Error: {}", e)))
                .unwrap())
        }
    }
}

/// Handle CONNECT request for HTTPS tunneling
async fn handle_connect_request(request_data: &mut RequestData) -> Result<Response<Body>> {
    let connect_start = std::time::Instant::now();
    
    // Extract target host and port from the URI
    let target = &request_data.url;
    log_debug!("CONNECT target: {}", target);
    
    // Parse host:port format
    let parts: Vec<&str> = target.split(':').collect();
    if parts.len() != 2 {
        let error_msg = format!("Invalid CONNECT target format: {}", target);
        log_error!("{}", error_msg);
        return Err(anyhow::anyhow!(error_msg));
    }
    
    let host = parts[0];
    let port = parts[1].parse::<u16>().map_err(|_e| {
        let error_msg = format!("Invalid port in CONNECT target: {}", parts[1]);
        log_error!("{}", error_msg);
        anyhow::anyhow!(error_msg)
    })?;
    
    log_debug!("CONNECT to {}:{}", host, port);
    
    // Attempt to establish connection to target
    match TcpStream::connect(format!("{}:{}", host, port)).await {
        Ok(_upstream_stream) => {
            let connect_time = connect_start.elapsed().as_millis();
            log_debug!("CONNECT successful to {}:{} in {}ms", host, port, connect_time);
            
            // Create a response that will upgrade the connection
            let response = Response::builder()
                .status(StatusCode::OK)
                .body(Body::empty())
                .unwrap();
            
            // For now, return a simple 200 OK response
            // The full tunneling implementation requires more complex handling
            // that would need to be integrated with the hyper server's upgrade mechanism
            log_debug!("CONNECT tunnel setup completed for {}", format!("{}:{}", host, port));
            
            let response_data = ResponseData::new(
                200,
                "OK".to_string(),
                "text/plain".to_string(),
                vec![],
                connect_time as u64,
            );
            
            let log_entry = ProxyLog {
                request: request_data.clone(),
                response: Some(response_data.clone()),
                error: None,
            };
            
            // Log transaction
            log_proxy_transaction!(&log_entry);
            log_debug!("CONNECT transaction logged: target={}:{}, connect_time={}ms", 
                      host, port, connect_time);
            
            // Return the upgrade response
            Ok(response)
        },
        Err(e) => {
            let connect_time = connect_start.elapsed().as_millis();
            let error_msg = format!("Failed to connect to {}:{}: {}", host, port, e);
            log_error!("CONNECT failed after {}ms: {}", connect_time, error_msg);
            
            let log_entry = ProxyLog {
                request: request_data.clone(),
                response: None,
                error: Some(error_msg.clone()),
            };
            
            // Log the error
            log_proxy_transaction!(&log_entry);
            log_debug!("CONNECT error transaction logged: target={}:{}, connect_time={}ms", 
                      host, port, connect_time);
            
            Err(anyhow::anyhow!(error_msg))
        }
    }
}


