use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use crate::utils::{parse_url, extract_path, extract_query, is_https};

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
        // Extract URL components using centralized utility functions
        let (path, query_string, is_https_url) = if url.contains(':') && !url.starts_with("http://") && !url.starts_with("https://") {
            // CONNECT request format (host:port)
            ("".to_string(), None, true)
        } else {
            // Regular URL
            if let Ok(parsed_url) = parse_url(&url) {
                (
                    extract_path(&parsed_url),
                    extract_query(&parsed_url),
                    is_https(&parsed_url)
                )
            } else {
                ("/".to_string(), None, url.starts_with("https://"))
            }
        };

        Self {
            method,
            url: url.clone(),
            path,
            query_string,
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
            is_https: is_https_url,
            protocol: "HTTP/1.1".to_string(),
        }
    }
}
