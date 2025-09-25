#!/bin/bash

# Generate a Trusted Certificate for Rust Forward Proxy
# This script creates a self-signed certificate with proper configuration for proxy use

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROXY_DIR="$SCRIPT_DIR/.."
CERTS_DIR="$PROXY_DIR/certs"

echo "🔒 Generating Trusted Certificate for Rust Forward Proxy"

# Default values
COMMON_NAME="proxy.local"
ORGANIZATION="Rust Forward Proxy"
VALIDITY_DAYS=365
OUTPUT_CERT="$CERTS_DIR/proxy.crt"
OUTPUT_KEY="$CERTS_DIR/proxy.key"
FORCE_OVERWRITE=false

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --common-name NAME   Common name for certificate (default: proxy.local)"
    echo "  --organization ORG   Organization name (default: Rust Forward Proxy)"
    echo "  --validity-days NUM  Certificate validity in days (default: 365)"
    echo "  --output-cert PATH   Output certificate file (default: certs/proxy.crt)"
    echo "  --output-key PATH    Output private key file (default: certs/proxy.key)"
    echo "  --force              Overwrite existing files"
    echo "  --help               Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                                          # Generate with defaults"
    echo "  $0 --common-name my-proxy.example.com      # Custom hostname"
    echo "  $0 --validity-days 30 --force              # Short-lived cert, overwrite"
    echo ""
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --common-name)
            COMMON_NAME="$2"
            shift 2
            ;;
        --organization)
            ORGANIZATION="$2"
            shift 2
            ;;
        --validity-days)
            VALIDITY_DAYS="$2"
            shift 2
            ;;
        --output-cert)
            OUTPUT_CERT="$2"
            shift 2
            ;;
        --output-key)
            OUTPUT_KEY="$2"
            shift 2
            ;;
        --force)
            FORCE_OVERWRITE=true
            shift
            ;;
        --help)
            show_usage
            exit 0
            ;;
        *)
            echo "❌ Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Check if OpenSSL is available
if ! command -v openssl >/dev/null 2>&1; then
    echo "❌ OpenSSL is required but not installed"
    exit 1
fi

# Create certs directory if it doesn't exist
mkdir -p "$CERTS_DIR"

# Check if files already exist
if [[ -f "$OUTPUT_CERT" || -f "$OUTPUT_KEY" ]] && [[ "$FORCE_OVERWRITE" != true ]]; then
    echo "❌ Certificate files already exist:"
    [[ -f "$OUTPUT_CERT" ]] && echo "   Certificate: $OUTPUT_CERT"
    [[ -f "$OUTPUT_KEY" ]] && echo "   Private key: $OUTPUT_KEY"
    echo ""
    echo "Use --force to overwrite existing files"
    exit 1
fi

echo "📋 Certificate Configuration:"
echo "   Common Name: $COMMON_NAME"
echo "   Organization: $ORGANIZATION"
echo "   Validity: $VALIDITY_DAYS days"
echo "   Certificate: $OUTPUT_CERT"
echo "   Private Key: $OUTPUT_KEY"
echo ""

# Create OpenSSL configuration file for SAN
TEMP_CONFIG=$(mktemp)
cat > "$TEMP_CONFIG" << EOF
[req]
default_bits = 4096
prompt = no
default_md = sha256
distinguished_name = dn
req_extensions = v3_req

[dn]
O=$ORGANIZATION
CN=$COMMON_NAME

[v3_req]
basicConstraints = CA:FALSE
keyUsage = nonRepudiation, digitalSignature, keyEncipherment
extendedKeyUsage = serverAuth, clientAuth
subjectAltName = @alt_names

[alt_names]
DNS.1 = $COMMON_NAME
DNS.2 = localhost
DNS.3 = *.local
IP.1 = 127.0.0.1
IP.2 = ::1
EOF

echo "🔑 Generating private key..."
openssl genrsa -out "$OUTPUT_KEY" 4096

echo "📜 Generating certificate..."
openssl req -new -x509 -key "$OUTPUT_KEY" -out "$OUTPUT_CERT" \
    -days "$VALIDITY_DAYS" -config "$TEMP_CONFIG" -extensions v3_req

# Clean up temp config
rm "$TEMP_CONFIG"

# Set appropriate permissions
chmod 644 "$OUTPUT_CERT"
chmod 600 "$OUTPUT_KEY"

echo "✅ Certificate generated successfully!"
echo ""

# Show certificate details
echo "📋 Certificate Details:"
openssl x509 -in "$OUTPUT_CERT" -text -noout | grep -E "(Subject|Issuer|Not Before|Not After|DNS:|IP Address:)" | sed 's/^/   /'

echo ""
echo "🔧 Next Steps:"
echo ""
echo "1. Start the proxy with your new certificate:"
echo "   ./rust-forward-proxy server \\"
echo "     --enable-tls \\"
echo "     --cert-path \"$OUTPUT_CERT\" \\"
echo "     --key-path \"$OUTPUT_KEY\" \\"
echo "     --auto-generate-cert false"
echo ""

echo "2. For browser trust, install the certificate:"
echo "   - macOS: open \"$OUTPUT_CERT\" and add to Keychain, mark as trusted"
echo "   - Linux: Copy to /usr/local/share/ca-certificates/ and run update-ca-certificates"
echo "   - Windows: Import to 'Trusted Root Certification Authorities'"
echo ""

echo "3. Test your setup:"
echo "   curl -k https://127.0.0.1:8443/"
echo "   curl -x http://127.0.0.1:8080 https://httpbin.org/get"
echo ""

echo "💡 Tips:"
echo "   - Use --common-name with your actual hostname for production"
echo "   - Consider using a proper CA-signed certificate for production use"
echo "   - Set up certificate renewal for long-running deployments"
echo ""

echo "✅ Trusted certificate setup completed!"


