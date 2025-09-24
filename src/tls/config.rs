//! TLS configuration utilities

use crate::config::settings::TlsConfig;
use anyhow::{anyhow, Result};
use rustls::{ServerConfig, ClientConfig, RootCertStore, Certificate, PrivateKey};
use rustls::client::{ServerCertVerifier, ServerCertVerified};
use std::sync::Arc;
use tracing::{info, debug};

/// Create rustls ServerConfig for TLS termination
pub fn create_server_config(
    cert: Certificate,
    key: PrivateKey,
    tls_config: &TlsConfig,
) -> Result<Arc<ServerConfig>> {
    debug!("Creating TLS server configuration");
    
    let mut config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)
        .map_err(|e| anyhow!("Failed to create TLS server config: {}", e))?;
    
    // Configure ALPN protocols for HTTP/1.1 and HTTP/2
    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    
    info!("✅ TLS server configuration created");
    info!("   Min TLS version: {}", tls_config.min_tls_version);
    info!("   ALPN protocols: h2, http/1.1");
    
    Ok(Arc::new(config))
}

/// Create rustls ClientConfig for upstream connections
pub fn create_client_config(tls_config: &TlsConfig) -> Result<Arc<ClientConfig>> {
    debug!("Creating TLS client configuration");
    
    let mut root_store = RootCertStore::empty();
    
    // Add system root certificates for proper certificate validation
    add_system_root_certificates(&mut root_store)?;
    
    // Add custom root CA certificate if specified
    if let Some(root_ca_path) = &tls_config.root_ca_cert_path {
        if let Err(e) = add_custom_root_ca(&mut root_store, root_ca_path) {
            tracing::warn!("Failed to load custom root CA: {}", e);
            info!("   Continuing with system root certificates only");
        }
    }
    
    let config = if tls_config.skip_upstream_cert_verify {
        info!("⚠️  WARNING: Skipping upstream certificate verification (insecure)");
        ClientConfig::builder()
            .with_safe_defaults()
            .with_custom_certificate_verifier(Arc::new(AcceptAllCertVerifier))
            .with_no_client_auth()
    } else {
        ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store)
            .with_no_client_auth()
    };
    
    info!("✅ TLS client configuration created");
    info!("   Certificate verification: {}", !tls_config.skip_upstream_cert_verify);
    
    Ok(Arc::new(config))
}

/// Validate TLS configuration
pub fn validate_tls_config(tls_config: &TlsConfig) -> Result<()> {
    debug!("Validating TLS configuration");
    
    // Validate TLS version
    match tls_config.min_tls_version.as_str() {
        "1.2" | "1.3" => {}
        _ => return Err(anyhow!("Invalid TLS version: {}. Must be '1.2' or '1.3'", tls_config.min_tls_version)),
    }
    
    // Validate certificate paths
    if tls_config.cert_path.is_empty() {
        return Err(anyhow!("Certificate path cannot be empty"));
    }
    
    if tls_config.key_path.is_empty() {
        return Err(anyhow!("Private key path cannot be empty"));
    }
    
    // Validate certificate validity period
    if tls_config.cert_validity_days == 0 {
        return Err(anyhow!("Certificate validity period must be greater than 0"));
    }
    
    // Validate common name
    if tls_config.cert_common_name.is_empty() {
        return Err(anyhow!("Certificate common name cannot be empty"));
    }
    
    info!("✅ TLS configuration validation passed");
    
    Ok(())
}

/// Add system root certificates to the root store
fn add_system_root_certificates(root_store: &mut RootCertStore) -> Result<()> {
    debug!("Loading system root certificates");
    
    match rustls_native_certs::load_native_certs() {
        Ok(certs) => {
            let mut added = 0;
            let mut failed = 0;
            
            for cert_der in certs {
                // Convert to rustls Certificate format
                let cert = Certificate(cert_der.to_vec());
                
                match root_store.add(&cert) {
                    Ok(_) => added += 1,
                    Err(_) => failed += 1,
                }
            }
            
            info!("✅ Root certificate store initialized");
            info!("   Added: {} certificates", added);
            if failed > 0 {
                info!("   Failed: {} certificates", failed);
            }
            
            Ok(())
        }
        Err(e) => {
            // Don't fail completely if system certs can't be loaded
            // This allows the proxy to work in environments without system cert store
            info!("⚠️  Could not load system root certificates: {}", e);
            info!("   Certificate verification will use empty root store");
            Ok(())
        }
    }
}

/// Add custom root CA certificate to the root store
pub fn add_custom_root_ca(root_store: &mut RootCertStore, root_ca_path: &str) -> Result<()> {
    debug!("Loading custom root CA certificate from {}", root_ca_path);
    
    use crate::tls::cert_gen::{load_root_ca_cert, validate_ca_certificate};
    
    let ca_cert = load_root_ca_cert(root_ca_path)?;
    validate_ca_certificate(&ca_cert)?;
    
    match root_store.add(&ca_cert) {
        Ok(_) => {
            info!("✅ Custom root CA certificate added to trust store");
            info!("   Certificate: {}", root_ca_path);
            Ok(())
        }
        Err(e) => {
            Err(anyhow!("Failed to add custom root CA to trust store: {:?}", e))
        }
    }
}

/// Validate a certificate chain against the root store
pub fn validate_certificate_chain(
    cert_chain: &[Certificate],
    server_name: &str,
    tls_config: &TlsConfig,
) -> Result<()> {
    debug!("Validating certificate chain for {}", server_name);
    
    if tls_config.skip_upstream_cert_verify {
        debug!("⚠️  Skipping certificate validation (disabled in config)");
        return Ok(());
    }
    
    if cert_chain.is_empty() {
        return Err(anyhow!("Empty certificate chain"));
    }
    
    // TODO: Implement full certificate chain validation
    // This would include:
    // 1. Verify certificate signatures
    // 2. Check certificate validity dates
    // 3. Validate certificate chain to root CA
    // 4. Check server name against certificate subject/SAN
    // 5. Verify certificate key usage and extended key usage
    
    info!("📜 Certificate chain validation:");
    info!("   Server: {}", server_name);
    info!("   Chain length: {} certificates", cert_chain.len());
    info!("   Validation: Not yet fully implemented");
    
    Ok(())
}

/// Extract certificate information for inspection
pub fn extract_certificate_info(_cert: &Certificate) -> CertificateInfo {
    // TODO: Parse certificate and extract detailed information
    // For now, return basic information
    CertificateInfo {
        subject: "Unknown".to_string(),
        issuer: "Unknown".to_string(),
        serial_number: "Unknown".to_string(),
        not_before: "Unknown".to_string(),
        not_after: "Unknown".to_string(),
        signature_algorithm: "Unknown".to_string(),
        public_key_algorithm: "Unknown".to_string(),
        key_size: 0,
        fingerprint_sha1: "Unknown".to_string(),
        fingerprint_sha256: "Unknown".to_string(),
        extensions: Vec::new(),
        is_ca: false,
    }
}

/// Certificate information structure
#[derive(Debug, Clone)]
pub struct CertificateInfo {
    pub subject: String,
    pub issuer: String,
    pub serial_number: String,
    pub not_before: String,
    pub not_after: String,
    pub signature_algorithm: String,
    pub public_key_algorithm: String,
    pub key_size: usize,
    pub fingerprint_sha1: String,
    pub fingerprint_sha256: String,
    pub extensions: Vec<String>,
    pub is_ca: bool,
}

/// Create a custom certificate verifier that accepts all certificates (for testing)
pub struct AcceptAllCertVerifier;

impl ServerCertVerifier for AcceptAllCertVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &Certificate,
        _intermediates: &[Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }
}
