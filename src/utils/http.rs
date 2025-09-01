//! HTTP utility functions

use hyper::{HeaderMap, header::{HeaderName, HeaderValue}, Body, Request, Response, StatusCode};
use std::collections::HashMap;
use tracing::debug;
use anyhow::Result;
use crate::models::RequestData;
use form_urlencoded;

/// Check if a header is a hop-by-hop header
pub fn is_hop_by_hop_header(name: &str) -> bool {
    let hop_by_hop_headers = [
        "connection",
        "keep-alive",
        "proxy-authenticate",
        "proxy-authorization",
        "te",
        "trailers",
        "transfer-encoding",
        "upgrade",
    ];
    
    hop_by_hop_headers.contains(&name.to_lowercase().as_str())
}

/// Convert HeaderMap to HashMap<String, String>
pub fn headers_to_map(headers: &HeaderMap) -> HashMap<String, String> {
    let mut map = HashMap::new();
    
    for (name, value) in headers {
        if let Ok(value_str) = value.to_str() {
            map.insert(name.to_string().to_lowercase(), value_str.to_string());
        }
    }
    
    map
}

/// Convert HashMap<String, String> to HeaderMap
pub fn map_to_headers(map: &HashMap<String, String>) -> HeaderMap {
    let mut headers = HeaderMap::new();
    
    for (name, value) in map {
        if let (Ok(name), Ok(value)) = (name.parse::<HeaderName>(), value.parse::<HeaderValue>()) {
            headers.insert(name, value);
        }
    }
    
    headers
}

/// Parse cookie header into key-value pairs
pub fn parse_cookies(cookie_header: &str) -> HashMap<String, String> {
    debug!("cookie_header: {}", cookie_header);

    let mut cookies = HashMap::new();

    for cookie in cookie_header.split(';') {
        let cookie = cookie.trim();
        if let Some(eq_pos) = cookie.find('=') {
            let name = cookie[..eq_pos].trim().to_string();
            let value = cookie[eq_pos + 1..].trim().to_string();
            cookies.insert(name, value);
        }
    }

    cookies
}

/// Parse URL-encoded form data
pub fn parse_form_data(body: &[u8]) -> HashMap<String, String> {
    let mut form_data = HashMap::new();

    debug!("form_data: {:?}", std::str::from_utf8(body));

    if let Ok(body_str) = std::str::from_utf8(body) {
        for (key, value) in form_urlencoded::parse(body_str.as_bytes()) {
            form_data.insert(key.to_string(), value.to_string());
        }
    }

    form_data
}

/// Parse host and port from CONNECT target
pub fn parse_connect_target(target: &str) -> Result<(String, u16), String> {
    let parts: Vec<&str> = target.split(':').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid CONNECT target format: {}", target));
    }
    
    let host = parts[0].to_string();
    let port = parts[1].parse::<u16>().unwrap_or(443);
    
    Ok((host, port))
}

/// Build error response
pub fn build_error_response(status: StatusCode, message: &str) -> Response<Body> {
    Response::builder()
        .status(status)
        .body(Body::from(message.to_string()))
        .unwrap()
}

/// Extract headers from request and populate RequestData
pub fn extract_headers(req_headers: &HeaderMap, request_data: &mut RequestData) {
    let mut header_count = 0;
    for (name, value) in req_headers {
        if let Ok(value_str) = value.to_str() {
            request_data
                .headers
                .insert(name.to_string().to_lowercase(), value_str.to_string());
            header_count += 1;
        }
    }
    debug!("Extracted {} headers from request", header_count);
}

/// Extract cookies from request and populate RequestData
pub fn extract_cookies_to_request_data(req_headers: &HeaderMap, request_data: &mut RequestData) {
    if let Some(cookie_header) = req_headers.get("cookie") {
        if let Ok(cookie_str) = cookie_header.to_str() {
            request_data.cookies = parse_cookies(cookie_str);
            debug!("Extracted {} cookies from request", request_data.cookies.len());
        }
    } else {
        debug!("No cookies found in request");
    }
}

/// Determine if request body should be extracted and return content type
pub fn should_extract_body(req_headers: &HeaderMap, method: &str) -> (bool, Option<String>) {
    if let Some(content_type) = req_headers.get("content-type") {
        if let Ok(content_type_str) = content_type.to_str() {
            debug!("Request content-type: {}", content_type_str);
            
            // Check if content type suggests a body
            let should_extract = content_type_str.contains("application/x-www-form-urlencoded") ||
                content_type_str.contains("application/json") ||
                content_type_str.contains("text/") ||
                content_type_str.contains("multipart/");
            
            (should_extract, Some(content_type_str.to_string()))
        } else {
            (false, None)
        }
    } else {
        debug!("No content-type header found");
        // For requests without content-type, extract body if method suggests it
        let should_extract = method == "POST" || method == "PUT" || method == "PATCH";
        (should_extract, None)
    }
}

/// Extract and process request body
pub async fn extract_body(body: Body, request_data: &mut RequestData) {
    debug!("Extracting request body");
    let body_bytes = hyper::body::to_bytes(body).await.unwrap_or_default();
    request_data.body = body_bytes.to_vec();
    debug!("Body extracted, size: {} bytes", request_data.body.len());
    
    // Parse form data only for form-encoded content
    if let Some(content_type) = &request_data.content_type {
        if content_type.contains("application/x-www-form-urlencoded") {
            request_data.form_data = parse_form_data(&body_bytes);
            debug!("Extracted {} form fields", request_data.form_data.len());
        }
    }
}

/// Build forwarding request with proper headers
pub fn build_forwarding_request(request_data: &RequestData) -> Result<Request<Body>> {
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
    debug!("Header forwarding: {} forwarded, {} skipped (hop-by-hop)", 
               forwarded_headers, skipped_headers);

    // Build the request
    let request = request_builder.body(Body::from(request_data.body.clone()))?;
    debug!("Forward request built, body size: {} bytes", request_data.body.len());
    
    Ok(request)
}
