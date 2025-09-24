# Securly Certificate Integration

This document explains how to integrate and use the Securly certificate with the Rust Forward Proxy for HTTPS interception.

## Overview

The Securly certificate (`securly_ca_2034.crt`) has been successfully integrated into the proxy's TLS configuration. This allows the proxy to:

1. **Trust Securly-signed certificates** - The Securly CA is added to the proxy's root certificate store
2. **Generate trusted domain certificates** - Using the existing Root CA for signing domain certificates
3. **Provide seamless HTTPS interception** - When browsers trust the appropriate certificates

## Certificate Details

**Securly CA Certificate:**
- **Path:** `ca-certs/securly_ca.crt`
- **Subject:** `C=US, ST=California, O=Securly, Inc, OU=Production, CN=*.securly.com`
- **Validity:** June 1, 2019 - January 9, 2035
- **Type:** Root CA Certificate (CA:TRUE)
- **Key Usage:** Certificate Sign, CRL Sign, Digital Signature

## Configuration

The proxy automatically loads the Securly certificate when configured with these environment variables:

```bash
# Enable TLS with Securly certificate integration
TLS_ENABLED=true
TLS_ROOT_CA_CERT_PATH=ca-certs/securly_ca.crt
TLS_CA_CERT_PATH=ca-certs/rootCA.crt
TLS_CA_KEY_PATH=ca-certs/rootCA.key
TLS_INTERCEPTION_ENABLED=true
```

## Usage Instructions

### 1. Start the Proxy

```bash
# Copy the test environment configuration
cp .env.test .env

# Start the proxy
cargo run
```

The proxy will:
- Load the Securly certificate into its trust store
- Generate domain certificates signed by the Root CA
- Listen for HTTPS connections on port 8443
- Listen for HTTP proxy connections on port 8080

### 2. Configure Browser Trust

For full HTTPS interception, install the certificates in your browser:

#### Install Securly Certificate (for Securly-signed sites):
```bash
# Export the certificate in browser-compatible format
openssl x509 -in ca-certs/securly_ca.crt -out securly_ca.pem

# Import into browser's "Trusted Root Certification Authorities"
```

#### Install Root CA (for proxy-generated domain certificates):
```bash
# Install the root CA certificate in browser
# Browser: Settings > Security > Manage Certificates > Trusted Root Certification Authorities
# Import: ca-certs/rootCA.crt
```

### 3. Test HTTPS Interception

```bash
# Test through the proxy
curl -x http://127.0.0.1:8080 https://httpbin.org/ip

# Test direct HTTPS
curl -k https://127.0.0.1:8443
```

### 4. Monitor Proxy Logs

Look for these log messages indicating successful integration:

```
✅ Custom root CA certificate added to trust store
   Certificate: ca-certs/securly_ca.crt
📜 Root CA found - generating CA-signed certificate for example.com
✅ CA-signed certificate generated for example.com
```

## Architecture

The integration follows this architecture:

```
Browser Request (HTTPS)
         ↓
    Proxy Intercepts
         ↓
   ┌─────────────────────┐
   │  Certificate Store  │
   │  ┌───────────────┐  │
   │  │ System CAs    │  │
   │  │ Securly CA    │  │ ← Added for verification
   │  │ Root CA       │  │ ← Used for signing
   │  └───────────────┘  │
   └─────────────────────┘
         ↓
   Domain Certificate Generation
   (Signed by Root CA)
         ↓
   TLS Connection to Upstream
   (Verified against all CAs)
```

## Certificate Types

1. **Securly CA Certificate (`securly_ca.crt`)**:
   - Purpose: Trust and verify Securly-signed certificates
   - Contains: Public key only (no private key)
   - Usage: Added to root certificate store for verification

2. **Root CA Certificate (`rootCA.crt` + `rootCA.key`)**:
   - Purpose: Sign domain certificates for interception
   - Contains: Certificate and private key pair
   - Usage: Signs new certificates for intercepted domains

3. **Domain Certificates (Generated)**:
   - Purpose: Present to browsers for specific domains
   - Generated: On-demand for each intercepted domain
   - Signed by: Root CA (trusted if Root CA is installed in browser)

## Security Considerations

1. **Certificate Trust Chain**:
   - Browsers must trust both the Securly CA and the Root CA
   - Securly CA handles verification of Securly-managed sites
   - Root CA handles newly generated domain certificates

2. **Private Key Security**:
   - Securly certificate contains only public key (secure)
   - Root CA private key must be protected (`rootCA.key`)
   - Domain certificate keys are generated per-session

3. **Certificate Validation**:
   - Upstream certificates are validated against full trust store
   - Domain certificates are generated with proper SAN entries
   - Certificate validity periods are enforced

## Troubleshooting

### Common Issues

1. **"Certificate not trusted" errors**:
   ```
   Solution: Install both Securly CA and Root CA in browser trust store
   ```

2. **"Failed to load custom root CA"**:
   ```
   Check: Verify ca-certs/securly_ca.crt exists and is readable
   Check: Ensure certificate is in valid PEM format
   ```

3. **"Root CA not found" warnings**:
   ```
   Solution: Run 'make setup-ca' or ensure ca-certs/rootCA.* files exist
   ```

### Verification Commands

```bash
# Verify Securly certificate
openssl x509 -in ca-certs/securly_ca.crt -text -noout

# Test certificate loading
cargo run --bin proxy -- cert info --cert-path ca-certs/securly_ca.crt

# Check proxy configuration
cargo run -- --help
```

## Files Modified

The integration required changes to these files:

- `src/config/settings.rs` - Added root CA configuration options
- `src/tls/config.rs` - Added custom CA loading to trust store
- `src/tls/cert_gen.rs` - Enhanced domain certificate generation
- `env.example` - Added new configuration variables
- `scripts/test_securly_integration.sh` - Integration test script

## Next Steps

1. **Enhanced CA Integration**: Implement proper certificate chain validation
2. **Certificate Caching**: Cache generated domain certificates for performance
3. **Certificate Rotation**: Implement automatic certificate renewal
4. **Advanced Verification**: Add OCSP and CRL checking

## Support

For issues related to Securly certificate integration:

1. Run the integration test: `./scripts/test_securly_integration.sh`
2. Check proxy logs for detailed error messages
3. Verify certificate validity and format
4. Ensure proper browser certificate installation

---

*Last updated: September 24, 2025*
*Securly Certificate Expiry: January 9, 2035*
