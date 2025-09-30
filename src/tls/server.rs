//! TLS server implementation for HTTPS interception

use crate::config::settings::ProxyConfig;
use crate::tls::{get_or_generate_certificate, create_server_config, validate_tls_config};
use crate::proxy::server::handle_request;
use anyhow::{anyhow, Result};
use hyper::service::service_fn;
use hyper::{Body, Request, Response};
use socket2::{Domain, Socket, Type};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::TlsAcceptor;
use tracing::{info, debug, error, warn};

/// Create a socket with SO_REUSEPORT support for multi-process TLS binding
fn create_reusable_tls_socket(addr: SocketAddr) -> Result<Socket> {
    use std::env;
    
    let domain = if addr.is_ipv6() { Domain::IPV6 } else { Domain::IPV4 };
    let socket = Socket::new(domain, Type::STREAM, None)?;
    
    // Enable SO_REUSEADDR (always good to have)
    socket.set_reuse_address(true)?;
    
    // Enable SO_REUSEPORT if requested (Linux/macOS)
    if env::var("PROXY_USE_REUSEPORT").unwrap_or_default() == "true" {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            socket.set_reuse_port(true)?;
            info!("✅ SO_REUSEPORT enabled for TLS multi-process support");
        }
        
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            warn!("⚠️  SO_REUSEPORT requested for TLS but not supported on this platform");
        }
    }
    
    // Set to non-blocking
    socket.set_nonblocking(true)?;
    
    // Bind to address
    socket.bind(&addr.into())?;
    
    // Listen with backlog
    socket.listen(1024)?;
    
    Ok(socket)
}

/// TLS-enabled proxy server for HTTPS interception
pub struct TlsProxyServer {
    config: ProxyConfig,
}

impl TlsProxyServer {
    /// Create a new TLS proxy server
    pub fn new(config: ProxyConfig) -> Self {
        Self { config }
    }

    /// Start the TLS proxy server with actual TLS termination
    pub async fn start(self) -> Result<()> {
        if !self.config.tls.enabled {
            return Err(anyhow!("TLS is not enabled in configuration"));
        }

        info!("🔒 Starting TLS proxy server on {}", self.config.tls.https_listen_addr);

        // Validate TLS configuration
        validate_tls_config(&self.config.tls)?;

        // Get or generate certificate
        let cert_data = get_or_generate_certificate(
            &self.config.tls.cert_path,
            &self.config.tls.key_path,
            self.config.tls.auto_generate_cert,
            &self.config.tls.cert_organization,
            &self.config.tls.cert_common_name,
            self.config.tls.cert_validity_days,
        )?;

        // Create TLS server configuration
        let server_config = create_server_config(
            cert_data.cert(),
            cert_data.key(),
            &self.config.tls,
        )?;

        info!("✅ TLS proxy server configuration completed");
        info!("📜 Certificate: {}", self.config.tls.cert_path);
        info!("🔐 Private key: {}", self.config.tls.key_path);
        info!("🔒 Interception mode: enabled");

        // Create TLS acceptor
        let tls_acceptor = TlsAcceptor::from(server_config);

        // Bind TCP listener with SO_REUSEPORT support for multi-process mode
        let listener = if std::env::var("PROXY_USE_REUSEPORT").unwrap_or_default() == "true" {
            // Use reusable socket for multi-process support
            let socket = create_reusable_tls_socket(self.config.tls.https_listen_addr)?;
            let std_listener = std::net::TcpListener::from(socket);
            TcpListener::from_std(std_listener)
                .map_err(|e| anyhow!("Failed to create TLS listener from reusable socket: {}", e))?
        } else {
            // Regular binding for single-process mode
            TcpListener::bind(&self.config.tls.https_listen_addr).await
                .map_err(|e| anyhow!("Failed to bind HTTPS listener: {}", e))?
        };

        info!("🔒 TLS proxy server listening on https://{}", self.config.tls.https_listen_addr);
        info!("🌐 Ready to intercept HTTPS traffic!");

        // Accept connections loop
        loop {
            match listener.accept().await {
                Ok((stream, remote_addr)) => {
                    let acceptor = tls_acceptor.clone();
                    let config = self.config.clone();
                    
                    // Spawn a task to handle each connection
                    tokio::spawn(async move {
                        if let Err(e) = handle_tls_connection(stream, remote_addr, acceptor, config).await {
                            error!("TLS connection error from {}: {}", remote_addr, e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept TLS connection: {}", e);
                }
            }
        }
    }
}

/// Handle a TLS connection - perform handshake and process HTTP requests
async fn handle_tls_connection(
    stream: TcpStream,
    remote_addr: SocketAddr,
    acceptor: TlsAcceptor,
    config: ProxyConfig,
) -> Result<()> {
    debug!("🔒 New TLS connection from {}", remote_addr);

    // Perform TLS handshake
    let tls_stream = match acceptor.accept(stream).await {
        Ok(stream) => {
            debug!("✅ TLS handshake completed for {}", remote_addr);
            stream
        }
        Err(e) => {
            warn!("❌ TLS handshake failed for {}: {}", remote_addr, e);
            return Err(anyhow!("TLS handshake failed: {}", e));
        }
    };

    // Create HTTP service for this TLS connection
    let http_service = service_fn(move |req| {
        handle_tls_request(req, remote_addr, config.clone())
    });
    
    // Serve HTTP over the TLS connection
    if let Err(e) = hyper::server::conn::Http::new()
        .serve_connection(tls_stream, http_service)
        .await
    {
        debug!("HTTP over TLS connection ended for {}: {}", remote_addr, e);
    }

    Ok(())
}

/// Handle incoming HTTPS request (after TLS termination)
async fn handle_tls_request(
    req: Request<Body>,
    remote_addr: SocketAddr,
    config: ProxyConfig,
) -> Result<Response<Body>, Infallible> {
    debug!("🔒 Processing decrypted HTTPS request from {}", remote_addr);

    // Full interception mode - treat decrypted HTTPS as regular HTTP
    // This gives us full visibility into the request/response
    debug!("🔍 HTTPS interception mode: full request/response logging enabled");
    
    // Use the same handler as regular HTTP requests
    // This provides complete transparency into HTTPS traffic
    // Create a dummy certificate manager for TLS-terminated requests (no interception needed)
    let cert_manager = Arc::new(crate::tls::CertificateManager::new());
    // Create optimized HTTP client for performance with configuration
    let client_manager = Arc::new(crate::proxy::http_client::HttpClient::from_config(&config.http_client));
    // Create smart body handler for streaming optimization with configuration
    let body_handler = Arc::new(crate::proxy::streaming::SmartBodyHandler::from_config(&config.streaming));
    handle_request(req, remote_addr, false, cert_manager, client_manager, body_handler, &config.tls).await // Don't enable HTTPS interception here since we're already handling TLS
}

/// Start both HTTP and HTTPS servers concurrently
pub async fn start_dual_servers(config: ProxyConfig) -> Result<()> {
    info!("🚀 Starting dual HTTP/HTTPS proxy servers");

    if config.tls.enabled {
        // Start both HTTP and HTTPS servers
        let http_config = config.clone();
        let https_config = config.clone();

        let http_server = tokio::spawn(async move {
            let server = crate::proxy::server::ProxyServer::with_https_interception_and_config(http_config.listen_addr, true, &http_config);
            if let Err(e) = server.start().await {
                error!("HTTP server failed: {}", e);
            }
        });

        let https_server = tokio::spawn(async move {
            let tls_server = TlsProxyServer::new(https_config);
            if let Err(e) = tls_server.start().await {
                error!("HTTPS server failed: {}", e);
            }
        });

        info!("🌐 HTTP proxy: http://{}", config.listen_addr);
        info!("🔒 HTTPS proxy: https://{}", config.tls.https_listen_addr);
        info!("🔍 HTTPS interception: ENABLED");

        // Wait for both servers
        tokio::try_join!(http_server, https_server)?;
    } else {
        // Start only HTTP server
        info!("🌐 Starting HTTP-only proxy server (TLS disabled)");
        let server = crate::proxy::server::ProxyServer::new(config.listen_addr);
        server.start().await?;
    }

    Ok(())
}
