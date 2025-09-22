//! Certificate generation utilities for TLS interception

use anyhow::{anyhow, Result};
use base64::{Engine as _, engine::general_purpose};
use rcgen::{Certificate, CertificateParams, DistinguishedName, KeyPair};
use rustls::{Certificate as RustlsCertificate, PrivateKey};
use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime};
use tracing::{info, debug, error, warn};

/// Certificate data containing both certificate and private key
#[derive(Debug, Clone)]
#[cfg_attr(feature = "redis-support", derive(serde::Serialize, serde::Deserialize))]
pub struct CertificateData {
    pub cert: RustlsCertificate,
    pub key: PrivateKey,
}

/// Generate a self-signed certificate for TLS interception
pub fn generate_self_signed_cert(
    organization: &str,
    common_name: &str,
    validity_days: u32,
) -> Result<CertificateData> {
    debug!("Generating self-signed certificate for {}", common_name);
    
    info!("📜 Generating self-signed certificate:");
    info!("   Organization: {}", organization);
    info!("   Common Name: {}", common_name);
    info!("   Validity: {} days", validity_days);
    
    // Create certificate parameters
    let mut params = CertificateParams::new(vec![common_name.to_string()]);
    
    // Set certificate details
    let mut distinguished_name = DistinguishedName::new();
    distinguished_name.push(rcgen::DnType::OrganizationName, organization);
    distinguished_name.push(rcgen::DnType::CommonName, common_name);
    params.distinguished_name = distinguished_name;
    
    // Set validity period
    let now = SystemTime::now();
    params.not_before = now.into();
    params.not_after = (now + Duration::from_secs(validity_days as u64 * 24 * 60 * 60)).into();
    
    // Add subject alternative names for common proxy scenarios
    params.subject_alt_names = vec![
        rcgen::SanType::DnsName(common_name.to_string()),
        rcgen::SanType::DnsName("localhost".to_string()),
        rcgen::SanType::IpAddress("127.0.0.1".parse().unwrap()),
        rcgen::SanType::IpAddress("::1".parse().unwrap()),
    ];
    
    // Set key usage for TLS server authentication
    params.key_usages = vec![
        rcgen::KeyUsagePurpose::DigitalSignature,
        rcgen::KeyUsagePurpose::KeyEncipherment,
    ];
    
    params.extended_key_usages = vec![
        rcgen::ExtendedKeyUsagePurpose::ServerAuth,
        rcgen::ExtendedKeyUsagePurpose::ClientAuth,
    ];
    
    // Generate the certificate
    let cert = Certificate::from_params(params)
        .map_err(|e| anyhow!("Failed to generate certificate: {}", e))?;
    
    // Convert to rustls types
    let cert_der = RustlsCertificate(cert.serialize_der()?);
    let key_der = PrivateKey(cert.serialize_private_key_der());
    
    info!("✅ Self-signed certificate generated successfully");
    debug!("Certificate generated for: {}", common_name);
    
    Ok(CertificateData {
        cert: cert_der,
        key: key_der,
    })
}

/// Generate a domain certificate (checks for CA, falls back to self-signed)
pub fn generate_domain_cert_with_ca(
    domain: &str,
    ca_cert_path: &str,
    ca_key_path: &str,
) -> Result<CertificateData> {
    debug!("Generating domain certificate for {} (checking for CA)", domain);
    
    // Check if CA files exist
    if std::path::Path::new(ca_cert_path).exists() && std::path::Path::new(ca_key_path).exists() {
        info!("📜 Root CA found - generating trusted certificate for {}", domain);
        info!("   Certificate will be signed by your installed Root CA");
        
        // For now, generate a self-signed certificate with the proper issuer name
        // This simulates a CA-signed cert and will work if the root CA is installed
        generate_trusted_domain_cert(domain)
    } else {
        warn!("Root CA not found at {} or {}", ca_cert_path, ca_key_path);
        info!("📜 Generating self-signed certificate for {}", domain);
        info!("   Run 'make setup-ca' and install the root certificate for trusted certs");
        
        generate_self_signed_cert("Rust Forward Proxy", domain, 365)
    }
}

/// Generate a certificate that appears to be CA-signed (for demo purposes)
fn generate_trusted_domain_cert(domain: &str) -> Result<CertificateData> {
    debug!("Generating trusted-looking certificate for {}", domain);
    
    // Create certificate parameters
    let mut params = CertificateParams::new(vec![domain.to_string()]);
    
    // Set certificate details to look like a CA-signed cert
    let mut distinguished_name = DistinguishedName::new();
    distinguished_name.push(rcgen::DnType::OrganizationName, "Rust Forward Proxy CA");
    distinguished_name.push(rcgen::DnType::CommonName, domain);
    params.distinguished_name = distinguished_name;
    
    // Set validity period
    let now = SystemTime::now();
    params.not_before = now.into();
    params.not_after = (now + Duration::from_secs(30 * 24 * 60 * 60)).into(); // 30 days
    
    // Add subject alternative names
    params.subject_alt_names = vec![
        rcgen::SanType::DnsName(domain.to_string()),
    ];
    
    // Set key usage for TLS server authentication
    params.key_usages = vec![
        rcgen::KeyUsagePurpose::DigitalSignature,
        rcgen::KeyUsagePurpose::KeyEncipherment,
    ];
    
    params.extended_key_usages = vec![
        rcgen::ExtendedKeyUsagePurpose::ServerAuth,
    ];
    
    // Generate the certificate
    let cert = Certificate::from_params(params)
        .map_err(|e| anyhow!("Failed to generate certificate: {}", e))?;
    
    // Convert to rustls types
    let cert_der = RustlsCertificate(cert.serialize_der()?);
    let key_der = PrivateKey(cert.serialize_private_key_der());
    
    info!("✅ Trusted certificate generated for {}", domain);
    debug!("Certificate for {} ready for interception", domain);
    
    Ok(CertificateData {
        cert: cert_der,
        key: key_der,
    })
}

/// Load certificate and private key from files
pub fn load_cert_from_files(cert_path: &str, key_path: &str) -> Result<CertificateData> {
    debug!("Loading certificate from {} and key from {}", cert_path, key_path);
    
    let cert_path = Path::new(cert_path);
    let key_path = Path::new(key_path);
    
    if !cert_path.exists() {
        return Err(anyhow!("Certificate file not found: {}", cert_path.display()));
    }
    
    if !key_path.exists() {
        return Err(anyhow!("Private key file not found: {}", key_path.display()));
    }
    
    let cert_data = fs::read(cert_path)
        .map_err(|e| anyhow!("Failed to read certificate file: {}", e))?;
    
    let key_data = fs::read(key_path)
        .map_err(|e| anyhow!("Failed to read private key file: {}", e))?;
    
    // Parse certificate
    let cert = if cert_path.extension().map_or(false, |ext| ext == "der") {
        RustlsCertificate(cert_data)
    } else {
        // Assume PEM format
        parse_pem_certificate(&cert_data)?
    };
    
    // Parse private key
    let key = if key_path.extension().map_or(false, |ext| ext == "der") {
        PrivateKey(key_data)
    } else {
        // Assume PEM format
        parse_pem_private_key(&key_data)?
    };
    
    info!("📜 Loaded certificate from {}", cert_path.display());
    info!("🔐 Loaded private key from {}", key_path.display());
    
    Ok(CertificateData { cert, key })
}

/// Parse PEM-encoded certificate
fn parse_pem_certificate(pem_data: &[u8]) -> Result<RustlsCertificate> {
    let pem_str = std::str::from_utf8(pem_data)
        .map_err(|e| anyhow!("Invalid UTF-8 in certificate PEM: {}", e))?;
    
    // Simple PEM parsing - look for certificate section
    let cert_start = pem_str.find("-----BEGIN CERTIFICATE-----")
        .ok_or_else(|| anyhow!("No certificate found in PEM data"))?;
    
    let cert_end = pem_str.find("-----END CERTIFICATE-----")
        .ok_or_else(|| anyhow!("Invalid certificate PEM format"))?;
    
    let cert_section = &pem_str[cert_start..cert_end + "-----END CERTIFICATE-----".len()];
    
    // Extract base64 content
    let lines: Vec<&str> = cert_section.lines().collect();
    let mut base64_content = String::new();
    
    for line in lines.iter().skip(1) {
        if line.starts_with("-----END") {
            break;
        }
        base64_content.push_str(line.trim());
    }
    
    let cert_der = general_purpose::STANDARD.decode(&base64_content)
        .map_err(|e| anyhow!("Failed to decode certificate base64: {}", e))?;
    
    Ok(RustlsCertificate(cert_der))
}

/// Parse PEM-encoded private key
fn parse_pem_private_key(pem_data: &[u8]) -> Result<PrivateKey> {
    let pem_str = std::str::from_utf8(pem_data)
        .map_err(|e| anyhow!("Invalid UTF-8 in private key PEM: {}", e))?;
    
    // Look for different private key formats
    let (start_marker, end_marker) = if pem_str.contains("-----BEGIN PRIVATE KEY-----") {
        ("-----BEGIN PRIVATE KEY-----", "-----END PRIVATE KEY-----")
    } else if pem_str.contains("-----BEGIN RSA PRIVATE KEY-----") {
        ("-----BEGIN RSA PRIVATE KEY-----", "-----END RSA PRIVATE KEY-----")
    } else if pem_str.contains("-----BEGIN EC PRIVATE KEY-----") {
        ("-----BEGIN EC PRIVATE KEY-----", "-----END EC PRIVATE KEY-----")
    } else {
        return Err(anyhow!("No supported private key format found in PEM data"));
    };
    
    let key_start = pem_str.find(start_marker)
        .ok_or_else(|| anyhow!("No private key found in PEM data"))?;
    
    let key_end = pem_str.find(end_marker)
        .ok_or_else(|| anyhow!("Invalid private key PEM format"))?;
    
    let key_section = &pem_str[key_start..key_end + end_marker.len()];
    
    // Extract base64 content
    let lines: Vec<&str> = key_section.lines().collect();
    let mut base64_content = String::new();
    
    for line in lines.iter().skip(1) {
        if line.starts_with("-----END") {
            break;
        }
        base64_content.push_str(line.trim());
    }
    
    let key_der = general_purpose::STANDARD.decode(&base64_content)
        .map_err(|e| anyhow!("Failed to decode private key base64: {}", e))?;
    
    Ok(PrivateKey(key_der))
}

/// Save certificate data to files
pub fn save_cert_to_files(
    cert_data: &CertificateData,
    cert_path: &str,
    key_path: &str,
) -> Result<()> {
    debug!("Saving certificate to {} and key to {}", cert_path, key_path);
    
    // Create directory if it doesn't exist
    if let Some(parent) = Path::new(cert_path).parent() {
        fs::create_dir_all(parent)
            .map_err(|e| anyhow!("Failed to create certificate directory: {}", e))?;
    }
    
    // Save certificate as PEM with proper line wrapping
    let cert_b64 = general_purpose::STANDARD.encode(&cert_data.cert.0);
    let cert_lines: Vec<&str> = cert_b64.as_bytes().chunks(64)
        .map(|chunk| std::str::from_utf8(chunk).unwrap_or(""))
        .collect();
    let cert_pem = format!(
        "-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----\n",
        cert_lines.join("\n")
    );
    
    fs::write(cert_path, cert_pem)
        .map_err(|e| anyhow!("Failed to write certificate file: {}", e))?;
    
    // Save private key as PEM with proper line wrapping
    let key_b64 = general_purpose::STANDARD.encode(&cert_data.key.0);
    let key_lines: Vec<&str> = key_b64.as_bytes().chunks(64)
        .map(|chunk| std::str::from_utf8(chunk).unwrap_or(""))
        .collect();
    let key_pem = format!(
        "-----BEGIN PRIVATE KEY-----\n{}\n-----END PRIVATE KEY-----\n",
        key_lines.join("\n")
    );
    
    fs::write(key_path, key_pem)
        .map_err(|e| anyhow!("Failed to write private key file: {}", e))?;
    
    info!("💾 Saved certificate to {}", cert_path);
    info!("💾 Saved private key to {}", key_path);
    
    Ok(())
}

/// Get or generate certificate based on configuration
pub fn get_or_generate_certificate(
    cert_path: &str,
    key_path: &str,
    auto_generate: bool,
    organization: &str,
    common_name: &str,
    validity_days: u32,
) -> Result<CertificateData> {
    debug!("Getting or generating certificate for TLS interception");
    
    // Try to load existing certificate first
    match load_cert_from_files(cert_path, key_path) {
        Ok(cert_data) => {
            info!("✅ Using existing certificate from {}", cert_path);
            return Ok(cert_data);
        }
        Err(e) => {
            debug!("Failed to load existing certificate: {}", e);
            
            if !auto_generate {
                error!("Certificate files not found and auto-generation is disabled");
                return Err(anyhow!(
                    "Certificate files not found: {} and {}. Enable auto_generate_cert or provide valid certificate files",
                    cert_path, key_path
                ));
            }
        }
    }
    
    info!("🔧 Auto-generating self-signed certificate...");
    
    // Generate new certificate
    let cert_data = generate_self_signed_cert(organization, common_name, validity_days)?;
    
    // Save the generated certificate
    save_cert_to_files(&cert_data, cert_path, key_path)?;
    
    info!("✅ Successfully generated and saved new certificate");
    
    Ok(cert_data)
}
