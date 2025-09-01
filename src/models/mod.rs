use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;

// Core data extracted from an HTTP request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestData {
    // Basic HTTP information
    pub method: String,
    pub url: String,
    pub path: String,
    pub query_string: Option<String>,
    pub http_version: String,

    // Client information
    pub client_ip: IpAddr,
    pub client_port: u16,

    // Timing
    pub timestamp: DateTime<Utc>,
    pub duration_ms: Option<u64>,

    // Headers and content
    pub headers: HashMap<String, String>,
    pub cookies: HashMap<String, String>,
    pub content_type: Option<String>,
    pub content_length: u64,
    pub user_agent: Option<String>,
    pub referer: Option<String>,
    pub host: Option<String>,

    // Body content
    pub body: Vec<u8>,
    pub form_data: HashMap<String, String>, // For form submissions

    // Protocol information
    pub is_https: bool,
    pub protocol: String, // "HTTP/1.1", "HTTP/2", etc.
}

// Response data we'll collect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseData {
    pub status_code: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub content_type: Option<String>,
    pub content_length: u64,
    pub response_time_ms: u64,
    pub body: Vec<u8>,
}

impl ResponseData {
    pub fn new(
        status_code: u16,
        status_text: String,
        content_type: String,
        body: Vec<u8>,
        response_time_ms: u64,
    ) -> Self {
        Self {
            status_code,
            status_text,
            headers: HashMap::new(),
            content_type: Some(content_type),
            content_length: body.len() as u64,
            response_time_ms,
            body,
        }
    }
}

// Complete request-response pair for logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyLog {
    pub request: RequestData,
    pub response: Option<ResponseData>,
    pub error: Option<String>,
}

impl RequestData {
    // Create a new RequestData from basic components
    pub fn new(method: String, url: String, client_ip: IpAddr, client_port: u16) -> Self {
        Self {
            method,
            url: url.clone(),
            path: Self::extract_path(&url),
            query_string: Self::extract_query(&url),
            http_version: "HTTP/1.1".to_string(),
            client_ip,
            client_port,
            timestamp: Utc::now(),
            duration_ms: None,
            headers: HashMap::new(),
            cookies: HashMap::new(),
            content_type: None,
            content_length: 0,
            user_agent: None,
            referer: None,
            host: None,
            body: Vec::new(),
            form_data: HashMap::new(),
            is_https: url.starts_with("https://"),
            protocol: "HTTP/1.1".to_string(),
        }
    }

    // Extract path from URL
    fn extract_path(url: &str) -> String {
        if let Ok(parsed) = url::Url::parse(url) {
            parsed.path().to_string()
        } else {
            "/".to_string()
        }
    }

    // Extract query string from URL
    fn extract_query(url: &str) -> Option<String> {
        if let Ok(parsed) = url::Url::parse(url) {
            parsed.query().map(|q| q.to_string())
        } else {
            None
        }
    }
}
