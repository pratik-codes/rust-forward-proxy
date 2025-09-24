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

/// Check if a request header should be forwarded to upstream server
/// This function consolidates all header filtering logic for requests
pub fn should_forward_request_header(name: &str) -> bool {
    let name_lower = name.to_lowercase();
    
    // Skip hop-by-hop headers and problematic headers that might cause 400 errors
    !is_hop_by_hop_header(&name_lower) 
        && name_lower != "host"                   // Will be set explicitly by proxy
        && name_lower != "content-length"         // Will be set explicitly by proxy to avoid duplicates
        && name_lower != "x-forwarded-for"        // Proxy will handle forwarding headers
        && name_lower != "x-forwarded-proto"      // Proxy will handle forwarding headers
        && name_lower != "x-real-ip"              // Proxy will handle real IP
        && name_lower != "rtt"                    // Network timing hint
        && name_lower != "downlink"               // Network speed hint  
        && name_lower != "priority"               // Browser priority hint
        && name_lower != "ect"                    // Effective connection type hint
        && !name_lower.starts_with("x-browser-") // Chrome browser headers
        && !name_lower.starts_with("sec-ch-ua")  // Chrome Client Hints
        && !name_lower.starts_with("sec-ch-prefers") // Chrome preference hints
        && name_lower != "x-client-data"          // Chrome telemetry data
        && name_lower != "sec-fetch-dest"         // Browser security hint
        && name_lower != "sec-fetch-mode"         // Browser security hint
        && name_lower != "sec-fetch-site"         // Browser security hint
        && name_lower != "sec-fetch-user"         // Browser security hint
        && name_lower != "upgrade-insecure-requests" // Browser security hint
        && name_lower != "sec-fetch-storage-access" // Browser security hint
}

/// Check if a response header should be forwarded to client
/// For now, this only filters hop-by-hop headers, but can be extended
pub fn should_forward_response_header(name: &str) -> bool {
    !is_hop_by_hop_header(name)
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

/// Build error response with specific content type
pub fn build_error_response_with_type(status: StatusCode, message: &str, content_type: &str) -> Response<Body> {
    Response::builder()
        .status(status)
        .header("Content-Type", content_type)
        .body(Body::from(message.to_string()))
        .unwrap()
}

/// Build a formatted error response for proxy errors
pub fn build_proxy_error_response(message: &str) -> Response<Body> {
    build_error_response_with_type(
        StatusCode::BAD_GATEWAY,
        &format!("Proxy Error: {}", message),
        "text/plain"
    )
}

/// Build response with headers
pub fn build_response_with_headers(
    status: StatusCode,
    headers: &[(String, String)],
    body: Body,
) -> Result<Response<Body>, hyper::http::Error> {
    let mut response_builder = Response::builder().status(status);
    
    // Add provided headers
    for (name, value) in headers {
        response_builder = response_builder.header(name, value);
    }
    
    response_builder.body(body)
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

    // Add headers using comprehensive filtering logic
    let mut forwarded_headers = 0;
    let mut skipped_headers = 0;
    for (name, value) in &request_data.headers {
        if should_forward_request_header(name) {
            request_builder = request_builder.header(name, value);
            forwarded_headers += 1;
        } else {
            skipped_headers += 1;
        }
    }
    debug!("Header forwarding: {} forwarded, {} skipped (filtered)", 
               forwarded_headers, skipped_headers);

    // Build the request
    let request = request_builder.body(Body::from(request_data.body.clone()))?;
    debug!("Forward request built, body size: {} bytes", request_data.body.len());
    
    Ok(request)
}

/// Log headers in a structured JSON format
pub fn log_headers_structured(headers: &HeaderMap, title: &str) {
    use tracing::info;
    use serde_json::{json, Map, Value};
    
    if headers.is_empty() {
        info!("📋 {} (empty)", title);
        return;
    }
    
    let mut header_map = Map::new();
    let mut cookie_headers = Vec::new();
    
    for (name, value) in headers {
        if let Ok(value_str) = value.to_str() {
            let name_str = name.as_str();
            if name_str.to_lowercase() == "cookie" {
                // Separate cookies for special handling
                cookie_headers.push(value_str.to_string());
            } else {
                header_map.insert(name_str.to_string(), Value::String(value_str.to_string()));
            }
        }
    }
    
    // Log the structured headers
    info!("📋 {}:", title);
    if !header_map.is_empty() {
        let headers_json = json!(header_map);
        info!("{}", serde_json::to_string_pretty(&headers_json).unwrap_or_default());
    }
    
    // Log cookies separately in a structured format if present
    if !cookie_headers.is_empty() {
        log_cookies_structured(&cookie_headers);
    }
}

/// Log cookies in a structured JSON format
pub fn log_cookies_structured(cookie_headers: &[String]) {
    use tracing::info;
    use serde_json::{json, Map, Value};
    
    let mut all_cookies = Map::new();
    
    for cookie_header in cookie_headers {
        let cookies = parse_cookies(cookie_header);
        for (name, value) in cookies {
            all_cookies.insert(name, Value::String(value));
        }
    }
    
    if !all_cookies.is_empty() {
        info!("🍪 Request Cookies:");
        let cookies_json = json!(all_cookies);
        info!("{}", serde_json::to_string_pretty(&cookies_json).unwrap_or_default());
    }
}

/// Log response headers in a structured JSON format
pub fn log_response_headers_structured(headers: &HeaderMap) {
    use tracing::info;
    use serde_json::{json, Map, Value};
    
    if headers.is_empty() {
        info!("📋 Response Headers (empty)");
        return;
    }
    
    let mut header_map = Map::new();
    let mut set_cookie_headers = Vec::new();
    
    for (name, value) in headers {
        if let Ok(value_str) = value.to_str() {
            let name_str = name.as_str();
            if name_str.to_lowercase() == "set-cookie" {
                // Collect set-cookie headers for special handling
                set_cookie_headers.push(value_str.to_string());
            } else {
                header_map.insert(name_str.to_string(), Value::String(value_str.to_string()));
            }
        }
    }
    
    // Log the structured headers
    info!("📋 Response Headers:");
    if !header_map.is_empty() {
        let headers_json = json!(header_map);
        info!("{}", serde_json::to_string_pretty(&headers_json).unwrap_or_default());
    }
    
    // Log set-cookie headers separately in a structured format if present
    if !set_cookie_headers.is_empty() {
        log_set_cookies_structured(&set_cookie_headers);
    }
}

/// Log Set-Cookie headers in a structured format
pub fn log_set_cookies_structured(set_cookie_headers: &[String]) {
    use tracing::info;
    use serde_json::{json, Map, Value};
    
    if set_cookie_headers.is_empty() {
        return;
    }
    
    info!("🍪 Response Set-Cookie Headers:");
    let mut cookies_array = Vec::new();
    
    for cookie_str in set_cookie_headers {
        // Parse each set-cookie header (they can have attributes like path, domain, etc.)
        let parts: Vec<&str> = cookie_str.split(';').collect();
        if let Some(cookie_part) = parts.first() {
            if let Some(eq_pos) = cookie_part.find('=') {
                let name = cookie_part[..eq_pos].trim();
                let value = cookie_part[eq_pos + 1..].trim();
                
                let mut cookie_obj = Map::new();
                cookie_obj.insert("name".to_string(), Value::String(name.to_string()));
                cookie_obj.insert("value".to_string(), Value::String(value.to_string()));
                
                // Add attributes if any
                if parts.len() > 1 {
                    let mut attributes = Map::new();
                    for attr in &parts[1..] {
                        let attr = attr.trim();
                        if let Some(eq_pos) = attr.find('=') {
                            let attr_name = attr[..eq_pos].trim();
                            let attr_value = attr[eq_pos + 1..].trim();
                            attributes.insert(attr_name.to_string(), Value::String(attr_value.to_string()));
                        } else {
                            // Boolean attributes like HttpOnly, Secure
                            attributes.insert(attr.to_string(), Value::Bool(true));
                        }
                    }
                    if !attributes.is_empty() {
                        cookie_obj.insert("attributes".to_string(), Value::Object(attributes));
                    }
                }
                
                cookies_array.push(Value::Object(cookie_obj));
            }
        }
    }
    
    let cookies_json = json!(cookies_array);
    info!("{}", serde_json::to_string_pretty(&cookies_json).unwrap_or_default());
}
