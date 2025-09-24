//! Authentication middleware for the proxy server

use crate::error::Result;
use hyper::{Request, Response, Body, StatusCode};

/// Authentication middleware
pub struct AuthMiddleware {
    api_key: Option<String>,
}

impl AuthMiddleware {
    pub fn new(api_key: Option<String>) -> Self {
        Self { api_key }
    }
    
    pub fn authenticate(&self, req: &Request<Body>) -> Result<bool> {
        if let Some(expected_key) = &self.api_key {
            if let Some(auth_header) = req.headers().get("authorization") {
                if let Ok(auth_value) = auth_header.to_str() {
                    if auth_value.starts_with("Bearer ") {
                        let provided_key = &auth_value[7..]; // Remove "Bearer " prefix
                        return Ok(provided_key == expected_key);
                    }
                }
            }
            Ok(false)
        } else {
            Ok(true) // No authentication required
        }
    }
    
    pub fn create_unauthorized_response(&self) -> Response<Body> {
        Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .header("content-type", "application/json")
            .body(Body::from(r#"{"error": "Unauthorized"}"#))
            .unwrap()
    }
}
