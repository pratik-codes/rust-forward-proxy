# Using Your Own Trusted Certificate with Rust Forward Proxy

This guide explains how to configure the Rust Forward Proxy to use your own trusted certificate instead of auto-generated self-signed certificates.

## 🎯 Quick Start

### Option 1: Use the Setup Script (Recommended)

```bash
# Copy your certificate files to the proxy directory
./scripts/setup_custom_cert.sh \
  --cert-file /path/to/your/certificate.crt \
  --key-file /path/to/your/private.key \
  --copy \
  --validate

# Start the proxy with TLS enabled
./rust-forward-proxy server --enable-tls --auto-generate-cert false
```

### Option 2: Command Line Configuration

```bash
./rust-forward-proxy server \
  --enable-tls \
  --cert-path /path/to/your/certificate.crt \
  --key-path /path/to/your/private.key \
  --auto-generate-cert false \
  --listen-addr 127.0.0.1:8080 \
  --https-listen-addr 127.0.0.1:8443
```

### Option 3: Environment Variables

Create a `.env` file or set environment variables:

```bash
TLS_ENABLED=true
TLS_CERT_PATH=/path/to/your/certificate.crt
TLS_KEY_PATH=/path/to/your/private.key
TLS_AUTO_GENERATE_CERT=false
HTTPS_LISTEN_ADDR=127.0.0.1:8443
```

Then start the proxy:

```bash
./rust-forward-proxy server
```

## 📜 Certificate Requirements

Your certificate should meet these requirements:

### 1. **Format Support**
- **PEM format** (`.pem`, `.crt`): Text format with `-----BEGIN CERTIFICATE-----`
- **DER format** (`.der`): Binary format
- Both certificate and private key must be in the same format

### 2. **Certificate Properties**
- Valid X.509 certificate
- Private key must match the certificate
- Certificate should include appropriate Subject Alternative Names (SANs)
- Key usage should include "Digital Signature" and "Key Encipherment"
- Extended key usage should include "Server Authentication"

### 3. **Hostname Configuration**
For the proxy to work properly, your certificate should include:

```
Subject Alternative Names:
- DNS: proxy.local (or your chosen hostname)
- DNS: localhost
- IP: 127.0.0.1
- IP: ::1
```

## 🔧 Certificate Management

### Validate Your Certificate

Use the built-in certificate CLI:

```bash
# Inspect certificate details
./rust-forward-proxy cert inspect --cert-path /path/to/your/certificate.crt

# Validate certificate and key match
./rust-forward-proxy cert validate \
  --cert-path /path/to/your/certificate.crt \
  --key-path /path/to/your/private.key
```

### Using OpenSSL to Validate

```bash
# Check if certificate and private key match
openssl x509 -noout -modulus -in certificate.crt | openssl md5
openssl rsa -noout -modulus -in private.key | openssl md5
# The output should be identical

# View certificate details
openssl x509 -in certificate.crt -text -noout

# Check certificate expiration
openssl x509 -in certificate.crt -noout -dates
```

## 🏭 Certificate Types and Use Cases

### 1. **Self-Signed Certificates**

**Use Case**: Development and testing

```bash
# Generate your own self-signed certificate
openssl req -x509 -newkey rsa:4096 -keyout private.key -out certificate.crt -days 365 -nodes \
  -subj "/O=My Organization/CN=proxy.local"

# Configure the proxy
./rust-forward-proxy server \
  --enable-tls \
  --cert-path certificate.crt \
  --key-path private.key \
  --auto-generate-cert false
```

### 2. **CA-Signed Certificates**

**Use Case**: Production environments

```bash
# Use your existing CA-signed certificate
./scripts/setup_custom_cert.sh \
  --cert-file /path/to/ca-signed-cert.crt \
  --key-file /path/to/ca-signed-key.key \
  --copy \
  --validate
```

### 3. **Let's Encrypt Certificates**

**Use Case**: Public-facing proxies

```bash
# Use Let's Encrypt certificate (typically in /etc/letsencrypt/live/domain.com/)
./rust-forward-proxy server \
  --enable-tls \
  --cert-path /etc/letsencrypt/live/proxy.example.com/fullchain.pem \
  --key-path /etc/letsencrypt/live/proxy.example.com/privkey.pem \
  --auto-generate-cert false
```

### 4. **Corporate/Enterprise Certificates**

**Use Case**: Enterprise environments with internal CA

```bash
# Use your corporate certificate
./scripts/setup_custom_cert.sh \
  --cert-file /path/to/corporate-cert.pem \
  --key-file /path/to/corporate-key.pem \
  --link \
  --validate
```

## 🛡️ Security Considerations

### Certificate Trust

- **For Development**: Install your root CA certificate in your browser's trust store
- **For Production**: Use certificates from trusted public CAs
- **For Corporate**: Ensure corporate CA is in client trust stores

### File Permissions

Secure your certificate files:

```bash
# Set appropriate permissions
chmod 644 /path/to/certificate.crt    # Certificate can be world-readable
chmod 600 /path/to/private.key        # Private key should be owner-only
chown proxy:proxy /path/to/certificate.crt /path/to/private.key
```

### Certificate Rotation

Set up automatic certificate renewal:

```bash
# Example systemd service for certificate renewal
[Unit]
Description=Proxy Certificate Renewal
After=certbot.service

[Service]
Type=oneshot
ExecStart=/path/to/scripts/setup_custom_cert.sh --cert-file /new/cert.crt --key-file /new/key.key --copy
ExecStartPost=/bin/systemctl reload rust-forward-proxy

[Install]
WantedBy=multi-user.target
```

## 🔍 Troubleshooting

### Common Issues

1. **"Certificate and private key do not match"**
   ```bash
   # Verify they match
   ./rust-forward-proxy cert validate --cert-path cert.crt --key-path key.key
   ```

2. **"Certificate file not found"**
   ```bash
   # Check file paths and permissions
   ls -la /path/to/certificate.crt
   ```

3. **"TLS handshake failed"**
   ```bash
   # Check certificate validity and hostname
   openssl s_client -connect 127.0.0.1:8443 -servername proxy.local
   ```

4. **"Invalid certificate format"**
   ```bash
   # Convert certificate format if needed
   openssl x509 -in certificate.der -inform DER -out certificate.pem -outform PEM
   ```

### Debug Mode

Enable debug logging to troubleshoot certificate issues:

```bash
RUST_LOG=debug ./rust-forward-proxy server \
  --enable-tls \
  --cert-path certificate.crt \
  --key-path private.key \
  --log-level debug
```

## 📊 Testing Your Setup

### 1. **Test HTTP Proxy**

```bash
curl -x http://127.0.0.1:8080 http://httpbin.org/get
```

### 2. **Test HTTPS Proxy**

```bash
curl -x http://127.0.0.1:8080 https://httpbin.org/get
```

### 3. **Test Direct HTTPS Connection**

```bash
curl -k https://127.0.0.1:8443/
```

### 4. **Test with Browser**

1. Configure your browser to use `127.0.0.1:8080` as HTTP/HTTPS proxy
2. Visit any HTTPS website
3. Check that the connection is secure and uses your certificate

## 📋 Configuration Reference

### All TLS Configuration Options

```bash
# Command line options
--enable-tls                    # Enable TLS/HTTPS support
--https-listen-addr ADDR        # HTTPS listening address (default: 127.0.0.1:8443)
--cert-path PATH               # Certificate file path
--key-path PATH                # Private key file path
--auto-generate-cert BOOL      # Auto-generate cert if missing (default: true)
--skip-cert-verify             # Skip upstream certificate verification
--enable-interception          # Enable HTTPS interception (default: true)

# Environment variables
TLS_ENABLED=true
HTTPS_LISTEN_ADDR=127.0.0.1:8443
TLS_CERT_PATH=certs/proxy.crt
TLS_KEY_PATH=certs/proxy.key
TLS_AUTO_GENERATE_CERT=false
TLS_INTERCEPTION_ENABLED=true
TLS_CERT_ORGANIZATION="Your Organization"
TLS_CERT_COMMON_NAME=proxy.local
TLS_CERT_VALIDITY_DAYS=365
TLS_MIN_TLS_VERSION=1.2
TLS_SKIP_UPSTREAM_CERT_VERIFY=false
```

## 🎓 Advanced Topics

### Custom Certificate Authority

If you want to create your own CA for testing:

```bash
# Generate CA private key
openssl genrsa -out ca.key 4096

# Generate CA certificate
openssl req -new -x509 -days 365 -key ca.key -out ca.crt \
  -subj "/O=My Test CA/CN=My Test CA"

# Generate server private key
openssl genrsa -out server.key 4096

# Generate server certificate signing request
openssl req -new -key server.key -out server.csr \
  -subj "/O=My Organization/CN=proxy.local"

# Sign server certificate with CA
openssl x509 -req -in server.csr -CA ca.crt -CAkey ca.key -CAcreateserial \
  -out server.crt -days 365 -extensions v3_req

# Use the server certificate with your proxy
./rust-forward-proxy server \
  --enable-tls \
  --cert-path server.crt \
  --key-path server.key \
  --auto-generate-cert false
```

### Docker Configuration

When using Docker, mount your certificate directory:

```yaml
# docker-compose.yml
version: '3.8'
services:
  proxy:
    build: .
    ports:
      - "8080:8080"
      - "8443:8443"
    volumes:
      - ./certs:/app/certs:ro
    environment:
      - TLS_ENABLED=true
      - TLS_CERT_PATH=certs/proxy.crt
      - TLS_KEY_PATH=certs/proxy.key
      - TLS_AUTO_GENERATE_CERT=false
```

---

💡 **Need help?** Check the main [README.md](./README.md) or create an issue on GitHub!

