//! Server management CLI commands

use crate::config::settings::ProxyConfig;
use crate::tls::start_dual_servers;
use crate::proxy::server::ProxyServer;
use anyhow::Result;
use clap::Args;
use std::net::SocketAddr;
use tracing::{info, debug};

#[derive(Debug, Args)]
pub struct ServerArgs {
    /// HTTP proxy listening address
    #[arg(long, default_value = "127.0.0.1:8080")]
    pub listen_addr: String,
    
    /// HTTPS proxy listening address
    #[arg(long, default_value = "127.0.0.1:8443")]
    pub https_listen_addr: String,
    
    /// Enable TLS/HTTPS support
    #[arg(long, default_value = "false")]
    pub enable_tls: bool,
    
    /// Enable HTTPS interception mode
    #[arg(long, default_value = "true")]
    pub enable_interception: bool,
    
    /// Auto-generate certificates if missing
    #[arg(long, default_value = "true")]
    pub auto_generate_cert: bool,
    
    /// Certificate file path
    #[arg(long, default_value = "certs/proxy.crt")]
    pub cert_path: String,
    
    /// Private key file path
    #[arg(long, default_value = "certs/proxy.key")]
    pub key_path: String,
    
    /// Skip upstream certificate verification (insecure)
    #[arg(long, default_value = "false")]
    pub skip_cert_verify: bool,
    
    /// Request timeout in seconds
    #[arg(long, default_value = "30")]
    pub request_timeout: u64,
    
    /// Maximum request body size in bytes
    #[arg(long, default_value = "1048576")]
    pub max_body_size: usize,
    
    /// Log level
    #[arg(long, default_value = "info")]
    pub log_level: String,
}

impl ServerArgs {
    /// Convert CLI arguments to ProxyConfig
    pub fn to_config(&self) -> Result<ProxyConfig> {
        debug!("Converting CLI arguments to ProxyConfig");
        
        let listen_addr: SocketAddr = self.listen_addr.parse()
            .map_err(|e| anyhow::anyhow!("Invalid listen address '{}': {}", self.listen_addr, e))?;
            
        let https_listen_addr: SocketAddr = self.https_listen_addr.parse()
            .map_err(|e| anyhow::anyhow!("Invalid HTTPS listen address '{}': {}", self.https_listen_addr, e))?;
        
        let mut config = ProxyConfig::default();
        
        // Basic server config
        config.listen_addr = listen_addr;
        config.log_level = self.log_level.clone();
        config.request_timeout = self.request_timeout;
        config.max_body_size = self.max_body_size;
        
        // TLS configuration
        config.tls.enabled = self.enable_tls;
        config.tls.https_listen_addr = https_listen_addr;
        config.tls.interception_enabled = true; // Always enable interception
        config.tls.auto_generate_cert = self.auto_generate_cert;
        config.tls.cert_path = self.cert_path.clone();
        config.tls.key_path = self.key_path.clone();
        config.tls.skip_upstream_cert_verify = self.skip_cert_verify;
        
        debug!("ProxyConfig created from CLI arguments");
        debug!("  HTTP: {}", config.listen_addr);
        debug!("  HTTPS: {} (enabled: {})", config.tls.https_listen_addr, config.tls.enabled);
        debug!("  Interception: enabled");
        
        Ok(config)
    }
    
    /// Start the proxy server with CLI configuration
    pub async fn start_server(&self) -> Result<()> {
        info!("🚀 Starting proxy server with CLI configuration");
        
        let config: ProxyConfig = self.to_config()?;
        
        // Show startup information
        info!("📋 Server Configuration:");
        info!("   HTTP proxy: {}", config.listen_addr);
        if config.tls.enabled {
            info!("   HTTPS proxy: {} (TLS enabled)", config.tls.https_listen_addr);
            info!("   Certificate: {}", config.tls.cert_path);
            info!("   Private key: {}", config.tls.key_path);
            info!("   Interception: enabled");
            info!("   Auto-cert: {}", if config.tls.auto_generate_cert { "enabled" } else { "disabled" });
        } else {
            info!("   HTTPS proxy: disabled");
        }
        info!("   Request timeout: {}s", config.request_timeout);
        info!("   Max body size: {} bytes", config.max_body_size);
        info!("   Log level: {}", config.log_level);
        
        // Start server(s)
        if config.tls.enabled {
            info!("🔒 Starting dual HTTP/HTTPS proxy servers");
            start_dual_servers(config).await
        } else {
            info!("🌐 Starting HTTP-only proxy server");
            let server = ProxyServer::new(config.listen_addr);
            server.start().await
        }
    }
}

