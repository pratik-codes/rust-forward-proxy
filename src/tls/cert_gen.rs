//! Certificate generation utilities for TLS interception

use anyhow::{anyhow, Result};
use base64::{Engine as _, engine::general_purpose};
use rcgen::{Certificate, CertificateParams, DistinguishedName};
use rustls::{Certificate as RustlsCertificate, PrivateKey};
use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime};
use tracing::{info, debug, error, warn};

/// Certificate data containing both certificate and private key
#[derive(Debug, Clone)]
#[cfg_attr(feature = "redis-support", derive(serde::Serialize, serde::Deserialize))]
pub struct CertificateData {
    cert_bytes: Vec<u8>,
    key_bytes: Vec<u8>,
}

impl CertificateData {
    pub fn new(cert: RustlsCertificate, key: PrivateKey) -> Self {
        Self {
            cert_bytes: cert.0,
            key_bytes: key.0,
        }
    }
    
    pub fn cert(&self) -> RustlsCertificate {
        RustlsCertificate(self.cert_bytes.clone())
    }
    
    pub fn key(&self) -> PrivateKey {
        PrivateKey(self.key_bytes.clone())
    }
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
    
    Ok(CertificateData::new(cert_der, key_der))
}

/// Generate a domain certificate signed by CA
pub fn generate_domain_cert_with_ca(
    domain: &str,
    ca_cert_path: &str,
    ca_key_path: &str,
) -> Result<CertificateData> {
    debug!("Generating domain certificate for {} using CA", domain);
    
    // Check if CA files exist
    if std::path::Path::new(ca_cert_path).exists() && std::path::Path::new(ca_key_path).exists() {
        info!("📜 Root CA found - generating CA-signed certificate for {}", domain);
        info!("   Certificate will be signed by Root CA: {}", ca_cert_path);
        
        // Load the CA certificate and key
        let ca_cert_data = load_cert_from_files(ca_cert_path, ca_key_path)?;
        
        // Generate domain certificate signed by the CA
        generate_ca_signed_domain_cert(domain, &ca_cert_data)
    } else {
        warn!("Root CA not found at {} or {}", ca_cert_path, ca_key_path);
        info!("📜 Generating self-signed certificate for {}", domain);
        info!("   Run 'make setup-ca' and install the root certificate for trusted certs");
        
        generate_self_signed_cert("Rust Forward Proxy", domain, 365)
    }
}

/// Generate a domain certificate signed by a CA using OpenSSL
fn generate_ca_signed_domain_cert(domain: &str, ca_data: &CertificateData) -> Result<CertificateData> {
    debug!("Generating CA-signed certificate for {}", domain);
    
    // Load the CA certificate to get issuer information
    let ca_cert_der = &ca_data.cert().0;
    let ca_cert = x509_parser::parse_x509_certificate(ca_cert_der)
        .map_err(|e| anyhow!("Failed to parse CA certificate: {}", e))?
        .1;
    
    // Extract CA subject to use as issuer
    let ca_subject = ca_cert.subject();
    let ca_cn = ca_subject.iter_common_name().next()
        .and_then(|attr| attr.as_str().ok())
        .unwrap_or("Rust Proxy Root CA");
    
    info!("📝 Signing certificate for {} with CA: {}", domain, ca_cn);
    
    // Get CA file paths
    let ca_cert_path = std::env::var("TLS_CA_CERT_PATH").unwrap_or_else(|_| "ca-certs/rootCA.crt".to_string());
    let ca_key_path = std::env::var("TLS_CA_KEY_PATH").unwrap_or_else(|_| "ca-certs/rootCA.key".to_string());
    
    // Generate certificate using OpenSSL command line (more reliable for CA signing)
    match generate_cert_with_openssl(domain, &ca_cert_path, &ca_key_path) {
        Ok(cert_data) => {
            info!("✅ CA-signed certificate generated for {} using OpenSSL (signed by: {})", domain, ca_cn);
            Ok(cert_data)
        }
        Err(e) => {
            warn!("OpenSSL certificate generation failed: {}, falling back to rcgen self-signed", e);
            // Fall back to self-signed certificate if OpenSSL fails
            generate_self_signed_cert("Rust Forward Proxy", domain, 30)
        }
    }
}

/// Generate a certificate using OpenSSL command line (more reliable for CA signing)
fn generate_cert_with_openssl(domain: &str, ca_cert_path: &str, ca_key_path: &str) -> Result<CertificateData> {
    use std::process::Command;
    use tempfile::TempDir;
    
    // Create temporary directory for certificate generation
    let temp_dir = TempDir::new().map_err(|e| anyhow!("Failed to create temp dir: {}", e))?;
    let temp_path = temp_dir.path();
    
    let domain_key_path = temp_path.join("domain.key");
    let domain_csr_path = temp_path.join("domain.csr");
    let domain_cert_path = temp_path.join("domain.crt");
    let config_path = temp_path.join("domain.conf");
    
    // Create OpenSSL config file for the domain certificate
    let config_content = format!(
        r#"[req]
distinguished_name = req_distinguished_name
req_extensions = v3_req
prompt = no

[req_distinguished_name]
O = Rust Forward Proxy
CN = {}

[v3_req]
basicConstraints = CA:FALSE
keyUsage = keyEncipherment, dataEncipherment, digitalSignature
extendedKeyUsage = serverAuth
subjectAltName = @alt_names

[alt_names]
DNS.1 = {}
"#,
        domain, domain
    );
    
    std::fs::write(&config_path, config_content)
        .map_err(|e| anyhow!("Failed to write OpenSSL config: {}", e))?;
    
    // Generate private key for the domain
    let output = Command::new("openssl")
        .args(&[
            "genrsa",
            "-out",
            domain_key_path.to_str().unwrap(),
            "2048",
        ])
        .output()
        .map_err(|e| anyhow!("Failed to run openssl genrsa: {}", e))?;
    
    if !output.status.success() {
        return Err(anyhow!("openssl genrsa failed: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    // Generate certificate signing request
    let output = Command::new("openssl")
        .args(&[
            "req",
            "-new",
            "-key",
            domain_key_path.to_str().unwrap(),
            "-out",
            domain_csr_path.to_str().unwrap(),
            "-config",
            config_path.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| anyhow!("Failed to run openssl req: {}", e))?;
    
    if !output.status.success() {
        return Err(anyhow!("openssl req failed: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    // Sign the certificate with the CA
    let output = Command::new("openssl")
        .args(&[
            "x509",
            "-req",
            "-in",
            domain_csr_path.to_str().unwrap(),
            "-CA",
            ca_cert_path,
            "-CAkey",
            ca_key_path,
            "-CAcreateserial",
            "-out",
            domain_cert_path.to_str().unwrap(),
            "-days",
            "30",
            "-extensions",
            "v3_req",
            "-extfile",
            config_path.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| anyhow!("Failed to run openssl x509: {}", e))?;
    
    if !output.status.success() {
        return Err(anyhow!("openssl x509 failed: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    // Read the generated certificate and private key
    let cert_pem = std::fs::read_to_string(&domain_cert_path)
        .map_err(|e| anyhow!("Failed to read generated certificate: {}", e))?;
    
    let key_pem = std::fs::read_to_string(&domain_key_path)
        .map_err(|e| anyhow!("Failed to read generated private key: {}", e))?;
    
    // Parse the certificate and key using rustls-pemfile
    let cert_der = rustls_pemfile::certs(&mut cert_pem.as_bytes())
        .map_err(|e| anyhow!("Failed to parse certificate PEM: {}", e))?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("No certificate found in PEM"))?;
    
    // Try different private key formats (PKCS#8 first as that's what OpenSSL generates by default)
    let key_der = if let Ok(keys) = rustls_pemfile::pkcs8_private_keys(&mut key_pem.as_bytes()) {
        if !keys.is_empty() {
            keys.into_iter().next().unwrap()
        } else {
            return Err(anyhow!("No PKCS8 private key found in PEM"));
        }
    } else if let Ok(keys) = rustls_pemfile::rsa_private_keys(&mut key_pem.as_bytes()) {
        if !keys.is_empty() {
            keys.into_iter().next().unwrap()
        } else {
            return Err(anyhow!("No RSA private key found in PEM"));
        }
    } else if let Ok(keys) = rustls_pemfile::ec_private_keys(&mut key_pem.as_bytes()) {
        if !keys.is_empty() {
            keys.into_iter().next().unwrap()
        } else {
            return Err(anyhow!("No EC private key found in PEM"));
        }
    } else {
        return Err(anyhow!("No supported private key format found in PEM"));
    };
    
    // Convert to rustls types
    let cert = RustlsCertificate(cert_der);
    let key = PrivateKey(key_der);
    
    info!("✅ OpenSSL CA-signed certificate generated for {}", domain);
    
    Ok(CertificateData::new(cert, key))
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
    
    Ok(CertificateData::new(cert, key))
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
    let cert_b64 = general_purpose::STANDARD.encode(&cert_data.cert().0);
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
    let key_b64 = general_purpose::STANDARD.encode(&cert_data.key().0);
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

/// Load root CA certificate for trust store integration
pub fn load_root_ca_cert(cert_path: &str) -> Result<RustlsCertificate> {
    debug!("Loading root CA certificate from {}", cert_path);
    
    let cert_path = Path::new(cert_path);
    
    if !cert_path.exists() {
        return Err(anyhow!("Root CA certificate file not found: {}", cert_path.display()));
    }
    
    let cert_data = fs::read(cert_path)
        .map_err(|e| anyhow!("Failed to read root CA certificate file: {}", e))?;
    
    // Parse certificate
    let cert = if cert_path.extension().map_or(false, |ext| ext == "der") {
        RustlsCertificate(cert_data)
    } else {
        // Assume PEM format
        parse_pem_certificate(&cert_data)?
    };
    
    info!("📜 Loaded root CA certificate from {}", cert_path.display());
    
    Ok(cert)
}

/// Validate that a certificate is a proper CA certificate
pub fn validate_ca_certificate(cert: &RustlsCertificate) -> Result<()> {
    debug!("Validating CA certificate");
    
    // TODO: Add proper X.509 certificate parsing to validate:
    // 1. Basic Constraints: CA=true
    // 2. Key Usage: Certificate Sign, CRL Sign
    // 3. Certificate validity dates
    // 4. Certificate chain validation
    
    // For now, basic validation - ensure certificate data is present
    if cert.0.is_empty() {
        return Err(anyhow!("Empty certificate data"));
    }
    
    info!("✅ CA certificate validation passed (basic check)");
    
    Ok(())
}
