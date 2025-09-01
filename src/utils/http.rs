//! HTTP utility functions

use hyper::{HeaderMap, header::{HeaderName, HeaderValue}};
use std::collections::HashMap;
use tracing::debug;

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
