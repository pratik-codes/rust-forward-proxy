//! Pingora-based HTTP proxy implementation

use async_trait::async_trait;
use pingora::prelude::*;
use pingora::proxy::{ProxyHttp, Session};
use pingora::upstreams::peer::HttpPeer;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tracing::{info, debug, error};
use url::Url;

use crate::proxy::core::{
    HttpProxyCore, ProxyRequest, ProxyResponse, ProxyError, ProxyResult, 
    ProxyMetrics, ProxyImplConfig
};
use crate::config::settings::ProxyConfig;
use crate::models::RequestData;
use crate::tls::CertificateManager;

/// Pingora-based proxy implementation
pub struct PingoraProxy {
    config: Option<ProxyConfig>,
    impl_config: Option<ProxyImplConfig>,
    cert_manager: Arc<CertificateManager>,
    start_time: Instant,
    metrics: Arc<tokio::sync::Mutex<PingoraMetrics>>,
}

/// Internal metrics for pingora proxy
#[derive(Debug, Default)]
struct PingoraMetrics {
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    total_response_time_ms: u64,
    active_connections: u64,
}

/// Context for pingora request processing
pub struct PingoraContext {
    pub request_data: RequestData,
    pub start_time: Instant,
    pub is_connect: bool,
    pub connect_host: Option<String>,
    pub connect_port: Option<u16>,
}

/// The actual pingora service implementation
pub struct PingoraService {
    config: Arc<ProxyConfig>,
    cert_manager: Arc<CertificateManager>,
    metrics: Arc<tokio::sync::Mutex<PingoraMetrics>>,
}

impl PingoraProxy {
    pub fn new() -> Self {
        Self {
            config: None,
            impl_config: None,
            cert_manager: Arc::new(CertificateManager::new()),
            start_time: Instant::now(),
            metrics: Arc::new(tokio::sync::Mutex::new(PingoraMetrics::default())),
        }
    }

    /// Parse target from URI
    fn parse_target(&self, uri: &str, is_connect: bool) -> ProxyResult<(String, u16, bool)> {
        if is_connect {
            // CONNECT format: "host:port"
            let parts: Vec<&str> = uri.split(':').collect();
            if parts.len() != 2 {
                return Err(ProxyError::RequestFailed("Invalid CONNECT target format".into()));
            }
            let host = parts[0].to_string();
            let port: u16 = parts[1].parse()
                .map_err(|_| ProxyError::RequestFailed("Invalid port in CONNECT target".into()))?;
            let is_https = port == 443;
            Ok((host, port, is_https))
        } else {
            // Regular HTTP URL
            let parsed_url = Url::parse(uri)
                .map_err(|e| ProxyError::RequestFailed(format!("Invalid URL: {}", e)))?;
            let host = parsed_url.host_str()
                .ok_or_else(|| ProxyError::RequestFailed("No host in URL".into()))?
                .to_string();
            let port = parsed_url.port()
                .unwrap_or(if parsed_url.scheme() == "https" { 443 } else { 80 });
            let is_https = parsed_url.scheme() == "https";
            Ok((host, port, is_https))
        }
    }
}

#[async_trait]
impl HttpProxyCore for PingoraProxy {
    async fn initialize(&mut self, config: &ProxyConfig, impl_config: &ProxyImplConfig) -> ProxyResult<()> {
        self.config = Some(config.clone());
        self.impl_config = Some(impl_config.clone());
        
        info!("🚀 Initializing Pingora proxy implementation");
        info!("   Max connections: {}", impl_config.max_connections);
        info!("   Connection timeout: {}ms", impl_config.connection_timeout_ms);
        info!("   HTTP/2 enabled: {}", impl_config.enable_http2);
        
        Ok(())
    }

    async fn start(&self, listen_addr: SocketAddr) -> ProxyResult<()> {
        let config = self.config.as_ref()
            .ok_or_else(|| ProxyError::ConfigError("Proxy not initialized".into()))?;

        info!("🌐 Starting Pingora proxy server on {}", listen_addr);

        // Create pingora server
        let mut my_server = Server::new(Some(Opt::parse_args()))
            .map_err(|e| ProxyError::Internal(format!("Failed to create server: {}", e)))?;
        my_server.bootstrap();

        // Create the pingora service
        let service = PingoraService {
            config: Arc::new(config.clone()),
            cert_manager: Arc::clone(&self.cert_manager),
            metrics: Arc::clone(&self.metrics),
        };

        // Create HTTP proxy service
        let mut proxy_service = pingora::proxy::http_proxy_service(&my_server.configuration, service);
        proxy_service.add_tcp(&listen_addr.to_string());

        info!("✅ Pingora proxy service configured");
        info!("Test with: curl -x http://{} http://httpbin.org/get", listen_addr);

        // Add the proxy service to the server
        my_server.add_service(proxy_service);

        // This will block and run the server
        my_server.run_forever();

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

        // Parse the target
        let (host, port, _is_https) = self.parse_target(&request.uri, request.is_connect)?;

        info!("🔗 Processing request: {} {} -> {}:{}", 
             request.method, request.uri, host, port);

        // For now, return a simple response indicating that the request would be processed
        // In a real implementation, this would use pingora's internal routing
        let response = ProxyResponse {
            status_code: 200,
            headers: {
                let mut headers = HashMap::new();
                headers.insert("content-type".to_string(), "application/json".to_string());
                headers.insert("x-proxy-impl".to_string(), "pingora".to_string());
                headers
            },
            body: format!(r#"{{"status":"processed","target":"{}:{}","method":"{}"}}"#, 
                         host, port, request.method).into(),
        };

        // Update metrics
        let response_time = start_time.elapsed().as_millis() as u64;
        {
            let mut metrics = self.metrics.lock().await;
            metrics.successful_requests += 1;
            metrics.active_connections -= 1;
            metrics.total_response_time_ms += response_time;
        }

        Ok(response)
    }

    async fn handle_health_check(&self) -> ProxyResult<ProxyResponse> {
        let uptime = self.start_time.elapsed().as_secs();
        let metrics = self.metrics.lock().await;
        
        let health_data = serde_json::json!({
            "status": "healthy",
            "implementation": "pingora",
            "service": "rust-forward-proxy",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "uptime_seconds": uptime,
            "total_requests": metrics.total_requests,
            "successful_requests": metrics.successful_requests,
            "failed_requests": metrics.failed_requests,
            "active_connections": metrics.active_connections,
            "version": env!("CARGO_PKG_VERSION")
        });

        Ok(ProxyResponse {
            status_code: 200,
            headers: {
                let mut headers = HashMap::new();
                headers.insert("content-type".to_string(), "application/json".to_string());
                headers.insert("cache-control".to_string(), "no-cache".to_string());
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
            implementation: "pingora".to_string(),
            total_requests: metrics.total_requests,
            successful_requests: metrics.successful_requests,
            failed_requests: metrics.failed_requests,
            average_response_time_ms: avg_response_time,
            active_connections: metrics.active_connections,
            uptime_seconds: uptime,
        })
    }

    async fn shutdown(&self) -> ProxyResult<()> {
        info!("🛑 Shutting down Pingora proxy");
        // In a real implementation, this would gracefully shutdown the pingora server
        Ok(())
    }

    fn implementation_name(&self) -> &'static str {
        "pingora"
    }
}

#[async_trait]
impl ProxyHttp for PingoraService {
    type CTX = PingoraContext;

    fn new_ctx(&self) -> Self::CTX {
        PingoraContext {
            request_data: RequestData::new(
                "GET".to_string(),
                "".to_string(),
                std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
                0,
            ),
            start_time: Instant::now(),
            is_connect: false,
            connect_host: None,
            connect_port: None,
        }
    }

    async fn early_request_filter(
        &self,
        session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<()> {
        ctx.start_time = Instant::now();
        
        // Get client info - pingora uses different SocketAddr type
        let default_addr = std::net::SocketAddr::new(
            std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
            0,
        );

        // Initialize request data with default values since pingora API is different
        ctx.request_data = RequestData::new(
            session.req_header().method.to_string(),
            session.req_header().uri.to_string(),
            default_addr.ip(),
            default_addr.port(),
        );

        // Check if this is a CONNECT request
        ctx.is_connect = session.req_header().method.as_str() == "CONNECT";

        debug!("🌐 {} {}", 
               ctx.request_data.method, 
               ctx.request_data.url);

        Ok(())
    }

    async fn request_filter(
        &self,
        session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<bool> {
        // Handle health check endpoint locally
        if session.req_header().uri.path() == "/health" {
            let health_response = serde_json::json!({
                "status": "healthy",
                "service": "rust-forward-proxy-pingora",
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "uptime_ms": ctx.start_time.elapsed().as_millis(),
                "version": env!("CARGO_PKG_VERSION")
            });

            let response_header = pingora::http::ResponseHeader::build(200, Some(16))
                .map_err(|_e| pingora::Error::new_str("Failed to build response header"))?;
            
            session.write_response_header(Box::new(response_header), false).await?;
            session.write_response_body(Some(health_response.to_string().into()), true).await?;
            session.finish_body().await?;

            return Ok(true); // Request handled
        }

        Ok(false) // Continue processing
    }

    async fn upstream_peer(
        &self,
        session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        let uri = session.req_header().uri.to_string();
        
        // Parse the target from the request
        let (host, port, is_https) = if ctx.is_connect {
            // CONNECT format: "host:port"
            let parts: Vec<&str> = uri.split(':').collect();
            if parts.len() != 2 {
                return Err(pingora::Error::new_str("Invalid CONNECT target format"));
            }
            let host = parts[0].to_string();
            let port: u16 = parts[1].parse()
                .map_err(|_| pingora::Error::new_str("Invalid port in CONNECT target"))?;
            let is_https = port == 443;
            (host, port, is_https)
        } else {
            // Regular HTTP URL
            let parsed_url = Url::parse(&uri)
                .map_err(|_| pingora::Error::new_str("Invalid URL"))?;
            let host = parsed_url.host_str()
                .ok_or_else(|| pingora::Error::new_str("No host in URL"))?
                .to_string();
            let port = parsed_url.port()
                .unwrap_or(if parsed_url.scheme() == "https" { 443 } else { 80 });
            let is_https = parsed_url.scheme() == "https";
            (host, port, is_https)
        };

        // Store connect info for CONNECT requests
        if ctx.is_connect {
            ctx.connect_host = Some(host.clone());
            ctx.connect_port = Some(port);
        }

        info!("🔗 Connecting to {}:{} (HTTPS: {})", host, port, is_https);

        // Create the peer
        let peer = Box::new(HttpPeer::new(
            format!("{}:{}", host, port),
            is_https,
            host.clone(),
        ));

        Ok(peer)
    }

    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        upstream_request: &mut pingora::http::RequestHeader,
        ctx: &mut Self::CTX,
    ) -> Result<()> {
        // For CONNECT requests, we don't modify the request
        if ctx.is_connect {
            return Ok(());
        }

        // Ensure proper headers are set
        if let Some(host) = upstream_request.uri.host() {
            let host_str = host.to_string();
            upstream_request.insert_header("Host", &host_str)?;
        }

        debug!("📤 Modified upstream request headers");

        Ok(())
    }

    async fn logging(
        &self,
        session: &mut Session,
        e: Option<&pingora::Error>,
        ctx: &mut Self::CTX,
    ) {
        let total_time = ctx.start_time.elapsed().as_millis();
        let method = &ctx.request_data.method;
        let url = &ctx.request_data.url;

        // Update metrics
        {
            let mut metrics = self.metrics.lock().await;
            if e.is_some() {
                metrics.failed_requests += 1;
            }
        }

        if let Some(error) = e {
            error!("❌ {} {} → ERROR: {} ({}ms)", method, url, error, total_time);
        } else {
            let status = session.response_written()
                .map(|resp| resp.status.as_u16())
                .unwrap_or(0);

            info!("✅ {} {} → {} ({}ms)", method, url, status, total_time);
        }
    }
}
