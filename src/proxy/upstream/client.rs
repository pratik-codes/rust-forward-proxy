//! Upstream HTTP/HTTPS client with TLS support

use crate::error::Result;
use crate::config::settings::TlsConfig;
use crate::tls::{create_client_config, AcceptAllCertVerifier};
use hyper::{Client, Request, Response, Body};
use hyper_rustls::HttpsConnectorBuilder;
use rustls::ClientConfig;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, debug, warn};

/// HTTP/HTTPS client for upstream servers with TLS support
pub struct UpstreamClient {
    http_client: Client<hyper::client::HttpConnector>,
    https_client: Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
    timeout: Duration,
    tls_config: TlsConfig,
}

impl UpstreamClient {
    /// Create a new upstream client with TLS support
    pub fn new(timeout: Duration, tls_config: TlsConfig) -> Result<Self> {
        debug!("Creating upstream client with TLS support");

        // Create HTTP-only client for regular requests
        let http_client = Client::builder()
            .pool_idle_timeout(Duration::from_secs(30))
            .build(hyper::client::HttpConnector::new());

        // Create TLS client configuration
        let client_config = create_client_config(&tls_config)?;

        // Create HTTPS connector
        let https_connector = if tls_config.skip_upstream_cert_verify {
            warn!("⚠️  Creating HTTPS client with disabled certificate verification (insecure)");
            
            // Use custom verifier that accepts all certificates
            let dangerous_config = ClientConfig::builder()
                .with_safe_defaults()
                .with_custom_certificate_verifier(Arc::new(AcceptAllCertVerifier))
                .with_no_client_auth();

            HttpsConnectorBuilder::new()
                .with_tls_config(dangerous_config)
                .https_or_http()
                .enable_http1()
                .build()
        } else {
            debug!("Creating HTTPS client with certificate verification enabled");
            
            HttpsConnectorBuilder::new()
                .with_tls_config((*client_config).clone())
                .https_or_http()
                .enable_http1()
                .build()
        };

        let https_client = Client::builder()
            .pool_idle_timeout(Duration::from_secs(30))
            .build(https_connector);

        info!("✅ Upstream client created with TLS support");
        info!("   HTTP client: enabled");
        info!("   HTTPS client: enabled");
        info!("   Certificate verification: {}", !tls_config.skip_upstream_cert_verify);
        
        Ok(Self {
            http_client,
            https_client,
            timeout,
            tls_config,
        })
    }

    /// Create a simple upstream client without TLS support (for backward compatibility)
    pub fn new_simple(timeout: Duration) -> Self {
        let http_client = Client::builder()
            .pool_idle_timeout(Duration::from_secs(30))
            .build(hyper::client::HttpConnector::new());

        // Create a basic HTTPS client for backward compatibility
        let https_connector = HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_or_http()
            .enable_http1()
            .build();

        let https_client = Client::builder()
            .pool_idle_timeout(Duration::from_secs(30))
            .build(https_connector);

        Self {
            http_client,
            https_client,
            timeout,
            tls_config: TlsConfig::default(),
        }
    }
    
    /// Make an HTTP or HTTPS request to upstream server
    pub async fn request(&self, req: Request<Body>) -> Result<Response<Body>> {
        let uri = req.uri();
        let is_https = uri.scheme_str() == Some("https");
        
        debug!("Making {} request to {}", if is_https { "HTTPS" } else { "HTTP" }, uri);

        let response = if is_https {
            // Use HTTPS client for secure connections
            debug!("🔒 Using HTTPS client for upstream connection");
            tokio::time::timeout(self.timeout, self.https_client.request(req)).await??
        } else {
            // Use HTTP client for regular connections  
            debug!("🌐 Using HTTP client for upstream connection");
            tokio::time::timeout(self.timeout, self.http_client.request(req)).await??
        };

        debug!("✅ Upstream response received: {}", response.status());
        Ok(response)
    }

    /// Get TLS configuration
    pub fn tls_config(&self) -> &TlsConfig {
        &self.tls_config
    }

    /// Check if certificate verification is enabled
    pub fn cert_verification_enabled(&self) -> bool {
        !self.tls_config.skip_upstream_cert_verify
    }
}
