//! Pingora-based proxy server implementation

use async_trait::async_trait;
use pingora::prelude::*;
use pingora::upstreams::peer::HttpPeer;
use pingora::proxy::{ProxyHttp, Session};
use std::sync::Arc;
use url::Url;
// use tracing::{info, debug};

use crate::models::{ProxyLog, RequestData, ResponseData};
use crate::{log_info, log_error, log_debug};
use crate::config::settings::ProxyConfig;
use crate::tls::CertificateManager;
// Note: extract_headers not compatible with pingora's http version

/// Pingora-based forward proxy with HTTPS interception capabilities
pub struct ForwardProxy {
    /// Configuration for the proxy
    config: Arc<ProxyConfig>,
    /// Certificate manager for HTTPS interception
    cert_manager: Arc<CertificateManager>,
}

/// Context for sharing state across request phases
pub struct ProxyContext {
    /// Request data for logging
    pub request_data: RequestData,
    /// Start time for performance measurement  
    pub start_time: std::time::Instant,
    /// Whether this is an HTTPS CONNECT request
    pub is_connect: bool,
    /// Original host for CONNECT requests
    pub connect_host: Option<String>,
    /// Original port for CONNECT requests
    pub connect_port: Option<u16>,
}

impl ForwardProxy {
    /// Create a new forward proxy with the given configuration
    pub fn new(config: ProxyConfig) -> Self {
        Self {
            config: Arc::new(config),
            cert_manager: Arc::new(CertificateManager::new()),
        }
    }

    /// Get the pingora server configuration
    pub fn get_pingora_config(&self) -> Arc<pingora::server::configuration::ServerConf> {
        let mut conf = pingora::server::configuration::ServerConf::new().unwrap();
        
        // Configure based on our proxy config
        conf.threads = self.config.http_client.max_idle_per_host.min(4) as usize; // Reasonable thread count
        
        Arc::new(conf)
    }

    /// Parse target from CONNECT request or regular URL
    fn parse_target(&self, uri: &str, is_connect: bool) -> Result<(String, u16, bool), Box<dyn std::error::Error + Send + Sync>> {
        if is_connect {
            // CONNECT format: "host:port"
            let parts: Vec<&str> = uri.split(':').collect();
            if parts.len() != 2 {
                return Err("Invalid CONNECT target format".into());
            }
            let host = parts[0].to_string();
            let port: u16 = parts[1].parse()?;
            let is_https = port == 443;
            Ok((host, port, is_https))
        } else {
            // Regular HTTP URL
            let parsed_url = Url::parse(uri)?;
            let host = parsed_url.host_str().ok_or("No host in URL")?.to_string();
            let port = parsed_url.port().unwrap_or(if parsed_url.scheme() == "https" { 443 } else { 80 });
            let is_https = parsed_url.scheme() == "https";
            Ok((host, port, is_https))
        }
    }
}

#[async_trait]
impl ProxyHttp for ForwardProxy {
    type CTX = ProxyContext;

    fn new_ctx(&self) -> Self::CTX {
        ProxyContext {
            request_data: RequestData::new(
                "GET".to_string(),
                "".to_string(),
                std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
                0,
            ),
            start_time: std::time::Instant::now(),
            is_connect: false,
            connect_host: None,
            connect_port: None,
        }
    }

    /// Early request filter - first phase of every request
    async fn early_request_filter(
        &self,
        session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<()> {
        ctx.start_time = std::time::Instant::now();
        
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

        log_info!("🌐 {} {}", 
                 ctx.request_data.method, 
                 ctx.request_data.url);

        Ok(())
    }

    /// Request filter - validate inputs and initialize context
    async fn request_filter(
        &self,
        session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<bool> {
        // Handle health check endpoint locally
        if session.req_header().uri.path() == "/health" {
            let health_response = serde_json::json!({
                "status": "healthy",
                "service": "rust-forward-proxy",
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "uptime_ms": ctx.start_time.elapsed().as_millis(),
                "version": env!("CARGO_PKG_VERSION")
            });

            session.respond_error(200).await?;
            session.write_response_header(Box::new(
                pingora::http::ResponseHeader::build(200, Some(16))
                    .map_err(|_e| pingora::Error::new_str("Failed to build response header"))?
            ), false).await?;
            session.write_response_body(Some(health_response.to_string().into()), true).await?;
            session.finish_body().await?;

            return Ok(true); // Request handled
        }

        // Note: Header extraction disabled due to HTTP version incompatibility
        // self.extract_request_data(&mut ctx.request_data, session).await;

        Ok(false) // Continue processing
    }

    /// Determine upstream peer to connect to
    async fn upstream_peer(
        &self,
        session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        let uri = session.req_header().uri.to_string();
        
        // Parse the target from the request
        let (host, port, is_https) =         self.parse_target(&uri, ctx.is_connect)
            .map_err(|_e| pingora::Error::new_str("Failed to parse target"))?;

        // Store connect info for CONNECT requests
        if ctx.is_connect {
            ctx.connect_host = Some(host.clone());
            ctx.connect_port = Some(port);
        }

        log_info!("🔗 Connecting to {}:{} (HTTPS: {})", host, port, is_https);

        // Create the peer
        let peer = Box::new(HttpPeer::new(
            format!("{}:{}", host, port),
            is_https,
            host.clone(),
        ));

        Ok(peer)
    }

    /// Modify the request before sending to upstream
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

        // Add proxy headers for regular HTTP requests
        // Note: client_addr API different in pingora 0.6
        upstream_request.insert_header("X-Forwarded-For", "127.0.0.1")?;

        upstream_request.insert_header("X-Forwarded-Proto", 
            if upstream_request.uri.scheme_str() == Some("https") { "https" } else { "http" })?;

        log_debug!("📤 Modified upstream request headers");

        Ok(())
    }

    // Note: upstream_response_filter temporarily removed due to lifetime issues

    /// Final logging phase
    async fn logging(
        &self,
        session: &mut Session,
        e: Option<&pingora::Error>,
        ctx: &mut Self::CTX,
    ) {
        let total_time = ctx.start_time.elapsed().as_millis();
        let method = &ctx.request_data.method;
        let url = &ctx.request_data.url;

        if let Some(error) = e {
            log_error!("❌ {} {} → ERROR: {} ({}ms)", method, url, error, total_time);
            
            // Log error transaction
            let log_entry = ProxyLog {
                request: ctx.request_data.clone(),
                response: None,
                error: Some(error.to_string()),
            };
            crate::log_proxy_transaction!(&log_entry);
        } else {
            let status = session.response_written()
                .map(|resp| resp.status.as_u16())
                .unwrap_or(0);

            log_info!("✅ {} {} → {} ({}ms)", method, url, status, total_time);

            // Create response data for logging
            let response_data = ResponseData::new(
                status,
                status.to_string(),
                "".to_string(), // Content-Type not easily accessible here
                vec![], // Body not accessible in logging phase
                total_time as u64,
            );

            // Log successful transaction
            let log_entry = ProxyLog {
                request: ctx.request_data.clone(),
                response: Some(response_data),
                error: None,
            };
            crate::log_proxy_transaction!(&log_entry);
        }
    }
}

impl ForwardProxy {
    /// Extract request data for logging purposes
    async fn extract_request_data(
        &self,
        request_data: &mut RequestData,
        session: &Session,
    ) {
        let req_header = session.req_header();
        
        // Update URL with full path and query
        request_data.url = req_header.uri.to_string();
        request_data.path = req_header.uri.path().to_string();
        
        if let Some(query) = req_header.uri.query() {
            request_data.query_string = Some(query.to_string());
        }

        // Note: Header extraction disabled due to HTTP version incompatibility
        // extract_headers(&req_header.headers, request_data);
        // extract_cookies_to_request_data(&req_header.headers, request_data);

        // Set HTTPS flag
        request_data.is_https = req_header.uri.scheme_str() == Some("https") || 
                                request_data.method == "CONNECT";
    }
}
