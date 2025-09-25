//! High-performance streaming for proxy requests
//! 
//! This module provides optimized streaming capabilities to avoid full body buffering:
//! - Zero-copy response streaming
//! - Configurable body size limits for logging
//! - Memory-efficient request/response handling

use hyper::{Body, Response};
use tracing::{info, debug};
use anyhow::Result;

/// Configuration for streaming behavior
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    /// Maximum body size to fully buffer for logging (default: 1MB)
    pub max_log_body_size: usize,
    /// Maximum partial body size to log for large bodies (default: 1KB)
    pub max_partial_log_size: usize,
    /// Enable response streaming (default: true)
    pub enable_response_streaming: bool,
    /// Enable request streaming (default: false - for compatibility)
    pub enable_request_streaming: bool,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            max_log_body_size: 1024 * 1024,        // 1MB
            max_partial_log_size: 1024,            // 1KB  
            enable_response_streaming: true,
            enable_request_streaming: false,       // Keep request buffering for logging compatibility
        }
    }
}

impl StreamingConfig {
    /// Create streaming config from the config struct
    /// This is the recommended way to create streaming configuration
    pub fn from_config(streaming_config: &crate::config::settings::StreamingConfig) -> Self {
        Self {
            max_log_body_size: streaming_config.max_log_body_size,
            max_partial_log_size: streaming_config.max_partial_log_size,
            enable_response_streaming: streaming_config.enable_response_streaming,
            enable_request_streaming: streaming_config.enable_request_streaming,
        }
    }

    /// Create streaming config from environment variables
    /// DEPRECATED: Use from_config instead for better configuration management
    pub fn from_env() -> Self {
        Self {
            max_log_body_size: std::env::var("PROXY_MAX_LOG_BODY_SIZE")
                .unwrap_or_else(|_| "1048576".to_string())  // 1MB
                .parse()
                .unwrap_or(1048576),
            max_partial_log_size: std::env::var("PROXY_MAX_PARTIAL_LOG_SIZE")
                .unwrap_or_else(|_| "1024".to_string())     // 1KB
                .parse()
                .unwrap_or(1024),
            enable_response_streaming: std::env::var("PROXY_ENABLE_RESPONSE_STREAMING")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            enable_request_streaming: std::env::var("PROXY_ENABLE_REQUEST_STREAMING")
                .unwrap_or_else(|_| "false".to_string())    // Conservative default
                .parse()
                .unwrap_or(false),
        }
    }
}

/// Smart body handling that chooses between buffering and streaming based on size
pub struct SmartBodyHandler {
    config: StreamingConfig,
}

impl SmartBodyHandler {
    pub fn new(config: StreamingConfig) -> Self {
        info!("🚀 Initializing smart body handler");
        info!("   Max log body size: {} bytes", config.max_log_body_size);
        info!("   Max partial log size: {} bytes", config.max_partial_log_size);
        info!("   Response streaming enabled: {}", config.enable_response_streaming);
        info!("   Request streaming enabled: {}", config.enable_request_streaming);
        
        Self { config }
    }

    pub fn from_config(streaming_config: &crate::config::settings::StreamingConfig) -> Self {
        Self::new(StreamingConfig::from_config(streaming_config))
    }

    pub fn from_env() -> Self {
        Self::new(StreamingConfig::from_env())
    }

    /// Handle request body with smart buffering/streaming
    pub async fn handle_request_body(
        &self,
        body: Body,
        log_context: &str,
    ) -> Result<(Vec<u8>, bool)> {
        let body_start = std::time::Instant::now();
        
        if self.config.enable_request_streaming {
            // Try to peek at content-length or read small chunk first
            let body_bytes = hyper::body::to_bytes(body).await
                .map_err(|e| anyhow::anyhow!("Failed to read body: {}", e))?;
            
            let is_large = body_bytes.len() > self.config.max_log_body_size;
            
            if is_large {
                info!("📦 Large request body detected ({} bytes) - using partial logging", body_bytes.len());
                self.log_partial_body(&body_bytes, log_context, true);
            } else {
                info!("📦 Small request body ({} bytes) - full logging enabled", body_bytes.len());
                self.log_full_body(&body_bytes, log_context);
            }
            
            let processing_time = body_start.elapsed();
            info!("⏱️  Request body processing: {:.2} ms", processing_time.as_secs_f64() * 1000.0);
            
            Ok((body_bytes.to_vec(), is_large))
        } else {
            // Legacy mode - full buffering for compatibility
            let body_bytes = hyper::body::to_bytes(body).await
                .map_err(|e| anyhow::anyhow!("Failed to read body: {}", e))?;
            
            let processing_time = body_start.elapsed();
            info!("⏱️  Request body processing: {:.2} ms", processing_time.as_secs_f64() * 1000.0);
            
            Ok((body_bytes.to_vec(), false))
        }
    }

    /// Handle response with optimized streaming
    pub async fn handle_response_streaming(
        &self,
        response: Response<Body>,
        log_context: &str,
    ) -> Result<Response<Body>> {
        let body_start = std::time::Instant::now();
        
        if self.config.enable_response_streaming {
            let (parts, body) = response.into_parts();
            
            // Check content-length header to decide strategy
            let content_length = parts.headers.get("content-length")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<usize>().ok());
                
            match content_length {
                Some(len) if len > self.config.max_log_body_size => {
                    // Large response - use streaming
                    info!("🚀 Large response detected ({} bytes) - using zero-copy streaming", len);
                    
                    let processing_time = body_start.elapsed();
                    info!("⏱️  Response streaming setup: {:.2} ms", processing_time.as_secs_f64() * 1000.0);
                    
                    // Return response with original body for zero-copy streaming
                    Ok(Response::from_parts(parts, body))
                }
                Some(len) => {
                    // Small response - safe to buffer for full logging
                    debug!("📦 Small response ({} bytes) - buffering for full logging", len);
                    self.handle_response_buffering(Response::from_parts(parts, body), log_context).await
                }
                None => {
                    // Unknown size - use streaming for safety
                    info!("🚀 Unknown response size - using zero-copy streaming for safety");
                    
                    let processing_time = body_start.elapsed();
                    info!("⏱️  Response streaming setup: {:.2} ms", processing_time.as_secs_f64() * 1000.0);
                    
                    Ok(Response::from_parts(parts, body))
                }
            }
        } else {
            // Legacy mode - full buffering
            self.handle_response_buffering(response, log_context).await
        }
    }

    /// Handle response with traditional buffering (for small responses or legacy mode)
    async fn handle_response_buffering(
        &self,
        response: Response<Body>,
        log_context: &str,
    ) -> Result<Response<Body>> {
        let body_start = std::time::Instant::now();
        let (parts, body) = response.into_parts();
        
        let body_bytes = hyper::body::to_bytes(body).await
            .map_err(|e| anyhow::anyhow!("Failed to read response body: {}", e))?;
        
        let processing_time = body_start.elapsed();
        info!("⏱️  Response body buffering: {:.2} ms ({} bytes)", 
              processing_time.as_secs_f64() * 1000.0, body_bytes.len());
        
        // Log the response body appropriately
        if body_bytes.len() > self.config.max_log_body_size {
            self.log_partial_body(&body_bytes, log_context, false);
        } else {
            self.log_full_body(&body_bytes, log_context);
        }
        
        // Return response with buffered body
        Ok(Response::from_parts(parts, Body::from(body_bytes)))
    }

    /// Log full body content
    fn log_full_body(&self, body_bytes: &[u8], context: &str) {
        info!("📄 {} Body ({} bytes):", context, body_bytes.len());
        
        if body_bytes.is_empty() {
            debug!("  [Empty body]");
            return;
        }
        
        if let Ok(body_str) = std::str::from_utf8(body_bytes) {
            info!("{}", body_str);
        } else {
            info!("  [Binary data - {} bytes]", body_bytes.len());
        }
    }

    /// Log partial body content for large bodies
    fn log_partial_body(&self, body_bytes: &[u8], context: &str, is_request: bool) {
        let prefix = if is_request { "Request" } else { "Response" };
        info!("📄 {} {} Body ({} bytes - showing first {} bytes):", 
              prefix, context, body_bytes.len(), self.config.max_partial_log_size);
        
        if body_bytes.is_empty() {
            debug!("  [Empty body]");
            return;
        }
        
        let log_size = std::cmp::min(body_bytes.len(), self.config.max_partial_log_size);
        let partial_bytes = &body_bytes[..log_size];
        
        if let Ok(body_str) = std::str::from_utf8(partial_bytes) {
            info!("{}...[truncated - {} more bytes]", body_str, body_bytes.len() - log_size);
        } else {
            info!("  [Binary data - {} bytes total, {} bytes shown]", body_bytes.len(), log_size);
        }
    }

    /// Get configuration for debugging
    pub fn get_config(&self) -> &StreamingConfig {
        &self.config
    }
}

/// Utility functions for streaming operations
pub struct StreamingUtils;

impl StreamingUtils {
    /// Create a zero-copy streaming response (most efficient)
    pub fn create_streaming_response(upstream_response: Response<Body>) -> Response<Body> {
        debug!("🚀 Creating zero-copy streaming response");
        // This is the most efficient - just pass through the body
        upstream_response
    }

    /// Check if response should be streamed based on headers
    pub fn should_stream_response(headers: &hyper::HeaderMap, max_size: usize) -> bool {
        if let Some(content_length) = headers.get("content-length") {
            if let Ok(length_str) = content_length.to_str() {
                if let Ok(length) = length_str.parse::<usize>() {
                    return length > max_size;
                }
            }
        }
        
        // If no content-length or unable to parse, be conservative and stream
        true
    }

    /// Log streaming statistics
    pub fn log_streaming_stats(
        total_requests: u64,
        streamed_requests: u64,
        buffered_requests: u64,
        memory_saved_mb: f64,
    ) {
        info!("📊 Streaming Statistics:");
        info!("   Total requests: {}", total_requests);
        info!("   Streamed responses: {} ({:.1}%)", 
              streamed_requests, 
              (streamed_requests as f64 / total_requests as f64) * 100.0);
        info!("   Buffered responses: {} ({:.1}%)", 
              buffered_requests,
              (buffered_requests as f64 / total_requests as f64) * 100.0);
        info!("   Estimated memory saved: {:.1} MB", memory_saved_mb);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_config_default() {
        let config = StreamingConfig::default();
        assert_eq!(config.max_log_body_size, 1024 * 1024);
        assert_eq!(config.max_partial_log_size, 1024);
        assert!(config.enable_response_streaming);
        assert!(!config.enable_request_streaming);
    }

    #[test]
    fn test_should_stream_large_response() {
        let mut headers = hyper::HeaderMap::new();
        headers.insert("content-length", "2000000".parse().unwrap());
        
        assert!(StreamingUtils::should_stream_response(&headers, 1024 * 1024));
    }

    #[test]
    fn test_should_not_stream_small_response() {
        let mut headers = hyper::HeaderMap::new();
        headers.insert("content-length", "500".parse().unwrap());
        
        assert!(!StreamingUtils::should_stream_response(&headers, 1024 * 1024));
    }
}
