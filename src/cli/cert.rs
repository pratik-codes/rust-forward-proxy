//! Certificate management CLI commands

use crate::tls::{
    generate_self_signed_cert, 
    load_cert_from_files, 
    save_cert_to_files,
};
use anyhow::{anyhow, Result};
use clap::{Args, Subcommand};
use std::fs;
use std::path::Path;
use tracing::{info, error, debug};

#[derive(Debug, Subcommand)]
pub enum CertCommand {
    /// Generate a new self-signed certificate
    Generate(GenerateCertArgs),
    
    /// Validate an existing certificate
    Validate(ValidateCertArgs),
    
    /// Inspect certificate details
    Inspect(InspectCertArgs),
    
    /// Convert certificate between formats
    Convert(ConvertCertArgs),
}

#[derive(Debug, Args)]
pub struct GenerateCertArgs {
    /// Organization name for the certificate
    #[arg(long, default_value = "Rust Forward Proxy")]
    pub organization: String,
    
    /// Common name (hostname) for the certificate
    #[arg(long, default_value = "proxy.local")]
    pub common_name: String,
    
    /// Certificate validity period in days
    #[arg(long, default_value = "365")]
    pub validity_days: u32,
    
    /// Output path for certificate file
    #[arg(long, default_value = "certs/proxy.crt")]
    pub cert_path: String,
    
    /// Output path for private key file
    #[arg(long, default_value = "certs/proxy.key")]
    pub key_path: String,
    
    /// Force overwrite existing certificates
    #[arg(long, default_value = "false")]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct ValidateCertArgs {
    /// Path to certificate file
    #[arg(long)]
    pub cert_path: String,
    
    /// Path to private key file  
    #[arg(long)]
    pub key_path: String,
    
    /// Check certificate expiration
    #[arg(long, default_value = "true")]
    pub check_expiry: bool,
}

#[derive(Debug, Args)]
pub struct InspectCertArgs {
    /// Path to certificate file
    #[arg(long)]
    pub cert_path: String,
    
    /// Show detailed certificate information
    #[arg(long, default_value = "false")]
    pub verbose: bool,
    
    /// Output format (text, json)
    #[arg(long, default_value = "text")]
    pub format: String,
}

#[derive(Debug, Args)]
pub struct ConvertCertArgs {
    /// Input certificate file path
    #[arg(long)]
    pub input: String,
    
    /// Output certificate file path
    #[arg(long)]
    pub output: String,
    
    /// Input format (pem, der)
    #[arg(long, default_value = "pem")]
    pub input_format: String,
    
    /// Output format (pem, der)
    #[arg(long, default_value = "pem")]
    pub output_format: String,
}

impl CertCommand {
    pub async fn execute(&self) -> Result<()> {
        match self {
            CertCommand::Generate(args) => generate_certificate(args).await,
            CertCommand::Validate(args) => validate_certificate(args).await,
            CertCommand::Inspect(args) => inspect_certificate(args).await,
            CertCommand::Convert(args) => convert_certificate(args).await,
        }
    }
}

/// Generate a new self-signed certificate
async fn generate_certificate(args: &GenerateCertArgs) -> Result<()> {
    info!("🔧 Generating self-signed certificate");
    info!("   Organization: {}", args.organization);
    info!("   Common Name: {}", args.common_name);
    info!("   Validity: {} days", args.validity_days);
    
    // Check if files already exist
    if !args.force && (Path::new(&args.cert_path).exists() || Path::new(&args.key_path).exists()) {
        return Err(anyhow!(
            "Certificate files already exist. Use --force to overwrite.\n  Certificate: {}\n  Key: {}", 
            args.cert_path, args.key_path
        ));
    }
    
    // Generate certificate
    let cert_data = generate_self_signed_cert(
        &args.organization,
        &args.common_name,
        args.validity_days,
    )?;
    
    // Save to files
    save_cert_to_files(&cert_data, &args.cert_path, &args.key_path)?;
    
    info!("✅ Certificate generated successfully!");
    info!("📜 Certificate: {}", args.cert_path);
    info!("🔐 Private key: {}", args.key_path);
    
    // Show certificate details
    show_certificate_summary(&args.cert_path)?;
    
    Ok(())
}

/// Validate an existing certificate
async fn validate_certificate(args: &ValidateCertArgs) -> Result<()> {
    info!("🔍 Validating certificate");
    info!("   Certificate: {}", args.cert_path);
    info!("   Private key: {}", args.key_path);
    
    // Try to load the certificate
    match load_cert_from_files(&args.cert_path, &args.key_path) {
        Ok(_) => {
            info!("✅ Certificate validation successful!");
            info!("   Certificate and private key match");
            
            if args.check_expiry {
                // TODO: Add expiration check
                info!("⏰ Certificate expiration check: Not yet implemented");
            }
            
            show_certificate_summary(&args.cert_path)?;
            Ok(())
        }
        Err(e) => {
            error!("❌ Certificate validation failed: {}", e);
            Err(e)
        }
    }
}

/// Inspect certificate details
async fn inspect_certificate(args: &InspectCertArgs) -> Result<()> {
    info!("🔍 Inspecting certificate: {}", args.cert_path);
    
    if !Path::new(&args.cert_path).exists() {
        return Err(anyhow!("Certificate file not found: {}", args.cert_path));
    }
    
    show_certificate_details(&args.cert_path, args.verbose, &args.format)?;
    
    Ok(())
}

/// Convert certificate between formats
async fn convert_certificate(args: &ConvertCertArgs) -> Result<()> {
    info!("🔄 Converting certificate");
    info!("   Input: {} ({})", args.input, args.input_format);
    info!("   Output: {} ({})", args.output, args.output_format);
    
    // TODO: Implement certificate format conversion
    error!("⚠️  Certificate format conversion not yet implemented");
    Err(anyhow!("Certificate conversion feature coming soon"))
}

/// Show a summary of certificate details
fn show_certificate_summary(cert_path: &str) -> Result<()> {
    debug!("Showing certificate summary for {}", cert_path);
    
    if !Path::new(cert_path).exists() {
        return Err(anyhow!("Certificate file not found: {}", cert_path));
    }
    
    // Read certificate file
    let cert_data = fs::read(cert_path)?;
    let cert_content = String::from_utf8_lossy(&cert_data);
    
    info!("📋 Certificate Summary:");
    info!("   File: {}", cert_path);
    info!("   Size: {} bytes", cert_data.len());
    info!("   Format: PEM");
    
    // Basic certificate info extraction
    if cert_content.contains("-----BEGIN CERTIFICATE-----") {
        info!("   Type: X.509 Certificate");
    }
    
    // TODO: Parse certificate and extract more details
    info!("   Details: Use 'inspect' command for full details");
    
    Ok(())
}

/// Show detailed certificate information
fn show_certificate_details(cert_path: &str, verbose: bool, format: &str) -> Result<()> {
    let cert_data = fs::read(cert_path)?;
    
    match format.to_lowercase().as_str() {
        "json" => {
            println!("{{");
            println!("  \"file\": \"{}\",", cert_path);
            println!("  \"size\": {},", cert_data.len());
            println!("  \"format\": \"PEM\",");
            println!("  \"type\": \"X.509\"");
            println!("}}");
        }
        "text" | _ => {
            println!("Certificate Details:");
            println!("  File: {}", cert_path);
            println!("  Size: {} bytes", cert_data.len());
            println!("  Format: PEM");
            println!("  Type: X.509 Certificate");
            
            if verbose {
                println!("\nRaw Certificate Data:");
                println!("{}", String::from_utf8_lossy(&cert_data));
            }
        }
    }
    
    if verbose {
        // TODO: Add detailed certificate parsing and display
        println!("\n⚠️  Detailed certificate parsing not yet implemented");
        println!("    This would show: Subject, Issuer, Serial, Validity, Extensions, etc.");
    }
    
    Ok(())
}

/// Test certificate generation and validation
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[tokio::test]
    async fn test_generate_certificate() {
        let temp_dir = tempdir().unwrap();
        let cert_path = temp_dir.path().join("test.crt").to_string_lossy().to_string();
        let key_path = temp_dir.path().join("test.key").to_string_lossy().to_string();
        
        let args = GenerateCertArgs {
            organization: "Test Org".to_string(),
            common_name: "test.local".to_string(),
            validity_days: 30,
            cert_path,
            key_path,
            force: false,
        };
        
        let result = generate_certificate(&args).await;
        assert!(result.is_ok());
        
        // Verify files were created
        assert!(Path::new(&args.cert_path).exists());
        assert!(Path::new(&args.key_path).exists());
    }
    
    #[tokio::test]
    async fn test_validate_nonexistent_certificate() {
        let args = ValidateCertArgs {
            cert_path: "/nonexistent/cert.pem".to_string(),
            key_path: "/nonexistent/key.pem".to_string(),
            check_expiry: false,
        };
        
        let result = validate_certificate(&args).await;
        assert!(result.is_err());
    }
}
