//! Reqwest-based HTTP proxy implementation

use async_trait::async_trait;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tracing::{info, debug, error};

use crate::proxy::core::{
    HttpProxyCore, ProxyRequest, ProxyResponse, ProxyError, ProxyResult, 
    ProxyMetrics, ProxyImplConfig
};
use crate::config::settings::ProxyConfig;

/// Reqwest-based proxy implementation
pub struct ReqwestProxy {
    config: Option<ProxyConfig>,
    impl_config: Option<ProxyImplConfig>,
    start_time: Instant,
    metrics: Arc<tokio::sync::Mutex<ReqwestMetrics>>,
    client: Option<reqwest::Client>,
}

/// Internal metrics for reqwest proxy
#[derive(Debug, Default)]
struct ReqwestMetrics {
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    total_response_time_ms: u64,
    active_connections: u64,
}

impl ReqwestProxy {
    pub fn new() -> Self {
        Self {
            config: None,
            impl_config: None,
            start_time: Instant::now(),
            metrics: Arc::new(tokio::sync::Mutex::new(ReqwestMetrics::default())),
            client: None,
        }
    }
}

#[async_trait]
impl HttpProxyCore for ReqwestProxy {
    async fn initialize(&mut self, config: &ProxyConfig, impl_config: &ProxyImplConfig) -> ProxyResult<()> {
        self.config = Some(config.clone());
        self.impl_config = Some(impl_config.clone());
        
        info!("🚀 Initializing Reqwest proxy implementation");
        info!("   Max connections: {}", impl_config.max_connections);
        info!("   Connection timeout: {}ms", impl_config.connection_timeout_ms);
        info!("   HTTP/2 enabled: {}", impl_config.enable_http2);

        // Create reqwest client with configuration
        let mut client_builder = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(impl_config.request_timeout_ms))
            .connect_timeout(std::time::Duration::from_millis(impl_config.connection_timeout_ms))
            .pool_max_idle_per_host(impl_config.max_connections)
            .user_agent(format!("rust-forward-proxy/{}", env!("CARGO_PKG_VERSION")));

        if !impl_config.enable_http2 {
            client_builder = client_builder.http1_only();
        }

        self.client = Some(client_builder.build()
            .map_err(|e| ProxyError::ConfigError(format!("Failed to create reqwest client: {}", e)))?);
        
        info!("✅ Reqwest client configured successfully");
        
        Ok(())
    }

    async fn start(&self, listen_addr: SocketAddr) -> ProxyResult<()> {
        let _config = self.config.as_ref()
            .ok_or_else(|| ProxyError::ConfigError("Proxy not initialized".into()))?;

        info!("🌐 Starting Reqwest proxy server on {}", listen_addr);
        info!("✅ Reqwest proxy service would start here");
        info!("Test with: curl -x http://{} http://httpbin.org/get", listen_addr);

        // Note: Reqwest is typically used as a client library, not a server
        // In a real implementation, you would combine reqwest with a server framework like axum or warp
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        info!("🔄 Reqwest proxy server simulation running...");
        
        Ok(())
    }

    async fn process_request(&self, request: ProxyRequest) -> ProxyResult<ProxyResponse> {
        let start_time = Instant::now();
        
        // Update metrics
        {
            let mut metrics = self.metrics.lock().await;
            metrics.total_requests += 1;
            metrics.active_connections += 1;
        }

        let client = self.client.as_ref()
            .ok_or_else(|| ProxyError::Internal("Client not initialized".into()))?;

        info!("🔗 Processing request with Reqwest: {} {}", 
             request.method, request.uri);

        // For demonstration, we'll make a real HTTP request using reqwest
        let result = match request.method.as_str() {
            "GET" => {
                client.get(&request.uri)
                    .headers(convert_headers(&request.headers)?)
                    .send()
                    .await
            }
            "POST" => {
                client.post(&request.uri)
                    .headers(convert_headers(&request.headers)?)
                    .body(request.body.to_vec())
                    .send()
                    .await
            }
            "PUT" => {
                client.put(&request.uri)
                    .headers(convert_headers(&request.headers)?)
                    .body(request.body.to_vec())
                    .send()
                    .await
            }
            "DELETE" => {
                client.delete(&request.uri)
                    .headers(convert_headers(&request.headers)?)
                    .send()
                    .await
            }
            _ => {
                return Err(ProxyError::RequestFailed(format!("Unsupported method: {}", request.method)));
            }
        };

        let response_time = start_time.elapsed().as_millis() as u64;

        match result {
            Ok(resp) => {
                let status_code = resp.status().as_u16();
                let headers = convert_headers_back(resp.headers());
                let body = resp.bytes().await
                    .map_err(|e| ProxyError::ResponseFailed(format!("Failed to read response body: {}", e)))?;

                // Update metrics
                {
                    let mut metrics = self.metrics.lock().await;
                    metrics.successful_requests += 1;
                    metrics.active_connections -= 1;
                    metrics.total_response_time_ms += response_time;
                }

                debug!("✅ Reqwest request processed in {}ms, status: {}", response_time, status_code);

                Ok(ProxyResponse {
                    status_code,
                    headers,
                    body: body.into(),
                })
            }
            Err(e) => {
                // Update metrics
                {
                    let mut metrics = self.metrics.lock().await;
                    metrics.failed_requests += 1;
                    metrics.active_connections -= 1;
                }

                error!("❌ Reqwest request failed after {}ms: {}", response_time, e);
                Err(ProxyError::RequestFailed(format!("Reqwest error: {}", e)))
            }
        }
    }

    async fn handle_health_check(&self) -> ProxyResult<ProxyResponse> {
        let uptime = self.start_time.elapsed().as_secs();
        let metrics = self.metrics.lock().await;
        
        let health_data = serde_json::json!({
            "status": "healthy",
            "implementation": "reqwest",
            "service": "rust-forward-proxy",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "uptime_seconds": uptime,
            "total_requests": metrics.total_requests,
            "successful_requests": metrics.successful_requests,
            "failed_requests": metrics.failed_requests,
            "active_connections": metrics.active_connections,
            "features": {
                "http2": self.impl_config.as_ref().map(|c| c.enable_http2).unwrap_or(false),
                "connection_pooling": self.impl_config.as_ref().map(|c| c.enable_connection_pooling).unwrap_or(false),
                "real_http_client": true
            },
            "version": env!("CARGO_PKG_VERSION")
        });

        Ok(ProxyResponse {
            status_code: 200,
            headers: {
                let mut headers = HashMap::new();
                headers.insert("content-type".to_string(), "application/json".to_string());
                headers.insert("cache-control".to_string(), "no-cache".to_string());
                headers.insert("x-proxy-impl".to_string(), "reqwest".to_string());
                headers
            },
            body: health_data.to_string().into(),
        })
    }

    async fn get_metrics(&self) -> ProxyResult<ProxyMetrics> {
        let metrics = self.metrics.lock().await;
        let uptime = self.start_time.elapsed().as_secs();
        
        let avg_response_time = if metrics.successful_requests > 0 {
            metrics.total_response_time_ms as f64 / metrics.successful_requests as f64
        } else {
            0.0
        };

        Ok(ProxyMetrics {
            implementation: "reqwest".to_string(),
            total_requests: metrics.total_requests,
            successful_requests: metrics.successful_requests,
            failed_requests: metrics.failed_requests,
            average_response_time_ms: avg_response_time,
            active_connections: metrics.active_connections,
            uptime_seconds: uptime,
        })
    }

    async fn shutdown(&self) -> ProxyResult<()> {
        info!("🛑 Shutting down Reqwest proxy");
        // Reqwest clients cleanup automatically when dropped
        Ok(())
    }

    fn implementation_name(&self) -> &'static str {
        "reqwest"
    }
}

/// Convert HashMap headers to reqwest::header::HeaderMap
fn convert_headers(headers: &HashMap<String, String>) -> ProxyResult<reqwest::header::HeaderMap> {
    let mut header_map = reqwest::header::HeaderMap::new();
    
    for (key, value) in headers {
        let header_name = reqwest::header::HeaderName::from_bytes(key.as_bytes())
            .map_err(|e| ProxyError::RequestFailed(format!("Invalid header name '{}': {}", key, e)))?;
        let header_value = reqwest::header::HeaderValue::from_str(value)
            .map_err(|e| ProxyError::RequestFailed(format!("Invalid header value for '{}': {}", key, e)))?;
        header_map.insert(header_name, header_value);
    }
    
    Ok(header_map)
}

/// Convert reqwest::header::HeaderMap back to HashMap
fn convert_headers_back(headers: &reqwest::header::HeaderMap) -> HashMap<String, String> {
    let mut header_map = HashMap::new();
    
    for (key, value) in headers {
        if let Ok(value_str) = value.to_str() {
            header_map.insert(key.to_string(), value_str.to_string());
        }
    }
    
    header_map
}
