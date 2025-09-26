//! Core proxy abstractions for pluggable HTTP proxy implementations
//! 
//! This module provides traits and interfaces that allow easy swapping between
//! different HTTP proxy implementations (pingora, hyper, reqwest, etc.)

use async_trait::async_trait;
use std::collections::HashMap;
use std::net::SocketAddr;
// use std::sync::Arc;
use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::models::{ProxyLog, RequestData, ResponseData};
use crate::config::settings::ProxyConfig;

/// Represents an HTTP request in a library-agnostic way
#[derive(Debug, Clone)]
pub struct ProxyRequest {
    pub method: String,
    pub uri: String,
    pub headers: HashMap<String, String>,
    pub body: Bytes,
    pub client_addr: SocketAddr,
    pub is_connect: bool,
}

/// Represents an HTTP response in a library-agnostic way
#[derive(Debug, Clone)]
pub struct ProxyResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: Bytes,
}

/// Error types for proxy operations
#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Request processing failed: {0}")]
    RequestFailed(String),
    
    #[error("Response processing failed: {0}")]
    ResponseFailed(String),
    
    #[error("TLS/SSL error: {0}")]
    TlsError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type for proxy operations
pub type ProxyResult<T> = Result<T, ProxyError>;

/// Configuration for proxy implementations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyImplConfig {
    /// Which proxy implementation to use
    pub implementation: ProxyImplementation,
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    /// Connection timeout in milliseconds
    pub connection_timeout_ms: u64,
    /// Request timeout in milliseconds  
    pub request_timeout_ms: u64,
    /// Enable HTTP/2 support
    pub enable_http2: bool,
    /// Enable connection pooling
    pub enable_connection_pooling: bool,
}

/// Available proxy implementations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProxyImplementation {
    /// Pingora-based implementation (CloudFlare's proxy framework)
    Pingora,
    /// Hyper-based implementation (Rust's HTTP library)
    Hyper,
    /// Reqwest-based implementation (High-level HTTP client)
    Reqwest,
}

impl Default for ProxyImplConfig {
    fn default() -> Self {
        Self {
            implementation: ProxyImplementation::Pingora,
            max_connections: 1000,
            connection_timeout_ms: 10000,
            request_timeout_ms: 30000,
            enable_http2: true,
            enable_connection_pooling: true,
        }
    }
}

/// Core trait for HTTP proxy implementations
/// 
/// This trait provides a library-agnostic interface for implementing HTTP proxies.
/// Different HTTP libraries (pingora, hyper, reqwest) can implement this trait
/// to provide their specific proxy behavior while maintaining a consistent API.
#[async_trait]
pub trait HttpProxyCore: Send + Sync {
    /// Initialize the proxy with configuration
    async fn initialize(&mut self, config: &ProxyConfig, impl_config: &ProxyImplConfig) -> ProxyResult<()>;
    
    /// Start the proxy server on the specified address
    async fn start(&self, listen_addr: SocketAddr) -> ProxyResult<()>;
    
    /// Process a single HTTP request
    async fn process_request(&self, request: ProxyRequest) -> ProxyResult<ProxyResponse>;
    
    /// Handle health check requests
    async fn handle_health_check(&self) -> ProxyResult<ProxyResponse>;
    
    /// Get proxy statistics and metrics
    async fn get_metrics(&self) -> ProxyResult<ProxyMetrics>;
    
    /// Gracefully shutdown the proxy
    async fn shutdown(&self) -> ProxyResult<()>;
    
    /// Get the implementation name
    fn implementation_name(&self) -> &'static str;
}

/// Metrics and statistics for proxy implementations
#[derive(Debug, Clone, Serialize)]
pub struct ProxyMetrics {
    pub implementation: String,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time_ms: f64,
    pub active_connections: u64,
    pub uptime_seconds: u64,
}

/// Factory for creating proxy implementations
pub struct ProxyFactory;

impl ProxyFactory {
    /// Create a proxy implementation based on configuration
    pub fn create_proxy(implementation: ProxyImplementation) -> Box<dyn HttpProxyCore> {
        match implementation {
            ProxyImplementation::Pingora => {
                Box::new(crate::proxy::pingora_impl::PingoraProxy::new())
            }
            ProxyImplementation::Hyper => {
                Box::new(crate::proxy::hyper_impl::HyperProxy::new())
            }
            ProxyImplementation::Reqwest => {
                Box::new(crate::proxy::reqwest_impl::ReqwestProxy::new())
            }
        }
    }
    
    /// Create proxy from string name
    pub fn create_proxy_from_name(name: &str) -> ProxyResult<Box<dyn HttpProxyCore>> {
        let implementation = match name.to_lowercase().as_str() {
            "pingora" => ProxyImplementation::Pingora,
            "hyper" => ProxyImplementation::Hyper,
            "reqwest" => ProxyImplementation::Reqwest,
            _ => return Err(ProxyError::ConfigError(format!("Unknown proxy implementation: {}", name))),
        };
        
        Ok(Self::create_proxy(implementation))
    }
}

/// Middleware trait for request/response processing
#[async_trait]
pub trait ProxyMiddleware: Send + Sync {
    /// Process request before forwarding
    async fn process_request(&self, request: &mut ProxyRequest) -> ProxyResult<()>;
    
    /// Process response before returning to client
    async fn process_response(&self, response: &mut ProxyResponse, request: &ProxyRequest) -> ProxyResult<()>;
    
    /// Handle request/response logging
    async fn log_transaction(&self, request: &ProxyRequest, response: Option<&ProxyResponse>, error: Option<&ProxyError>);
}

/// Default middleware implementation for logging
pub struct DefaultLoggingMiddleware;

#[async_trait]
impl ProxyMiddleware for DefaultLoggingMiddleware {
    async fn process_request(&self, _request: &mut ProxyRequest) -> ProxyResult<()> {
        // Default: no request modification
        Ok(())
    }
    
    async fn process_response(&self, _response: &mut ProxyResponse, _request: &ProxyRequest) -> ProxyResult<()> {
        // Default: no response modification
        Ok(())
    }
    
    async fn log_transaction(&self, request: &ProxyRequest, response: Option<&ProxyResponse>, error: Option<&ProxyError>) {
        // Convert to our logging format
        let mut request_data = RequestData::new(
            request.method.clone(),
            request.uri.clone(),
            request.client_addr.ip(),
            request.client_addr.port(),
        );
        
        // Extract headers and other data
        for (key, value) in &request.headers {
            if key.to_lowercase() == "content-type" {
                request_data.content_type = Some(value.clone());
            }
        }
        
        let response_data = response.map(|resp| {
            ResponseData::new(
                resp.status_code,
                resp.status_code.to_string(),
                resp.headers.get("content-type").cloned().unwrap_or_default(),
                resp.body.to_vec(),
                0, // Duration not available at this level
            )
        });
        
        let log_entry = ProxyLog {
            request: request_data,
            response: response_data,
            error: error.map(|e| e.to_string()),
        };
        
        // Log the transaction
        crate::log_proxy_transaction!(&log_entry);
    }
}

/// Manager for proxy implementations with middleware support
pub struct ProxyManager {
    proxy: Box<dyn HttpProxyCore>,
    middleware: Vec<Box<dyn ProxyMiddleware>>,
    config: ProxyImplConfig,
}

impl ProxyManager {
    /// Create a new proxy manager with specified implementation
    pub fn new(implementation: ProxyImplementation) -> Self {
        Self {
            proxy: ProxyFactory::create_proxy(implementation.clone()),
            middleware: vec![Box::new(DefaultLoggingMiddleware)],
            config: ProxyImplConfig {
                implementation,
                ..Default::default()
            },
        }
    }
    
    /// Add middleware to the proxy
    pub fn add_middleware(&mut self, middleware: Box<dyn ProxyMiddleware>) {
        self.middleware.push(middleware);
    }
    
    /// Initialize the proxy with configuration
    pub async fn initialize(&mut self, config: &ProxyConfig) -> ProxyResult<()> {
        self.proxy.initialize(config, &self.config).await
    }
    
    /// Start the proxy server
    pub async fn start(&self, listen_addr: SocketAddr) -> ProxyResult<()> {
        self.proxy.start(listen_addr).await
    }
    
    /// Process a request through all middleware and the proxy
    pub async fn process_request(&self, mut request: ProxyRequest) -> ProxyResult<ProxyResponse> {
        // Process request through middleware
        for middleware in &self.middleware {
            middleware.process_request(&mut request).await?;
        }
        
        // Process through proxy
        let result = self.proxy.process_request(request.clone()).await;
        
        match result {
            Ok(mut response) => {
                // Process response through middleware
                for middleware in &self.middleware {
                    middleware.process_response(&mut response, &request).await?;
                }
                
                // Log successful transaction
                for middleware in &self.middleware {
                    middleware.log_transaction(&request, Some(&response), None).await;
                }
                
                Ok(response)
            }
            Err(error) => {
                // Log failed transaction
                for middleware in &self.middleware {
                    middleware.log_transaction(&request, None, Some(&error)).await;
                }
                
                Err(error)
            }
        }
    }
    
    /// Get proxy metrics
    pub async fn get_metrics(&self) -> ProxyResult<ProxyMetrics> {
        self.proxy.get_metrics().await
    }
    
    /// Shutdown the proxy
    pub async fn shutdown(&self) -> ProxyResult<()> {
        self.proxy.shutdown().await
    }
    
    /// Get implementation name
    pub fn implementation_name(&self) -> &'static str {
        self.proxy.implementation_name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_proxy_factory() {
        let proxy = ProxyFactory::create_proxy(ProxyImplementation::Pingora);
        assert_eq!(proxy.implementation_name(), "pingora");
        
        let proxy = ProxyFactory::create_proxy_from_name("hyper").unwrap();
        assert_eq!(proxy.implementation_name(), "hyper");
        
        assert!(ProxyFactory::create_proxy_from_name("unknown").is_err());
    }
    
    #[test]
    fn test_proxy_manager() {
        let manager = ProxyManager::new(ProxyImplementation::Pingora);
        assert_eq!(manager.implementation_name(), "pingora");
    }
}
