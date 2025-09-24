#!/bin/bash

# Setup Custom Trusted Certificate for Rust Forward Proxy
# This script helps you configure your proxy to use your own trusted certificate

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROXY_DIR="$SCRIPT_DIR/.."
CERTS_DIR="$PROXY_DIR/certs"

echo "🔒 Setting up Custom Trusted Certificate for Rust Forward Proxy"
echo ""

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --cert-file PATH     Path to your certificate file (.crt, .pem, .der)"
    echo "  --key-file PATH      Path to your private key file (.key, .pem, .der)"
    echo "  --copy              Copy certificate files to proxy certs directory"
    echo "  --link              Create symbolic links to certificate files"
    echo "  --validate          Validate certificate and key match"
    echo "  --help              Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 --cert-file /path/to/my-cert.crt --key-file /path/to/my-key.key --copy"
    echo "  $0 --cert-file ./mycert.pem --key-file ./mykey.pem --link --validate"
    echo ""
}

# Parse command line arguments
CERT_FILE=""
KEY_FILE=""
COPY_MODE=false
LINK_MODE=false
VALIDATE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --cert-file)
            CERT_FILE="$2"
            shift 2
            ;;
        --key-file)
            KEY_FILE="$2"
            shift 2
            ;;
        --copy)
            COPY_MODE=true
            shift
            ;;
        --link)
            LINK_MODE=true
            shift
            ;;
        --validate)
            VALIDATE=true
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

# Check if certificate and key files are provided
if [[ -z "$CERT_FILE" || -z "$KEY_FILE" ]]; then
    echo "❌ Both certificate and key files must be specified"
    echo ""
    show_usage
    exit 1
fi

# Check if files exist
if [[ ! -f "$CERT_FILE" ]]; then
    echo "❌ Certificate file not found: $CERT_FILE"
    exit 1
fi

if [[ ! -f "$KEY_FILE" ]]; then
    echo "❌ Private key file not found: $KEY_FILE"
    exit 1
fi

# Create certs directory if it doesn't exist
mkdir -p "$CERTS_DIR"

echo "📋 Configuration Summary:"
echo "   Certificate file: $CERT_FILE"
echo "   Private key file: $KEY_FILE"
echo "   Proxy certs directory: $CERTS_DIR"
echo ""

# Validate certificate and key match if requested
if [[ "$VALIDATE" == true ]]; then
    echo "🔍 Validating certificate and private key..."
    
    # Check if certificate and key match using OpenSSL
    if command -v openssl >/dev/null 2>&1; then
        CERT_HASH=$(openssl x509 -noout -modulus -in "$CERT_FILE" 2>/dev/null | openssl md5)
        KEY_HASH=$(openssl rsa -noout -modulus -in "$KEY_FILE" 2>/dev/null | openssl md5)
        
        if [[ "$CERT_HASH" == "$KEY_HASH" ]]; then
            echo "✅ Certificate and private key match!"
        else
            echo "❌ Certificate and private key do not match!"
            exit 1
        fi
        
        # Show certificate details
        echo ""
        echo "📜 Certificate Details:"
        openssl x509 -in "$CERT_FILE" -text -noout | grep -E "(Subject|Issuer|Not Before|Not After|Subject Alternative Name)" || true
    else
        echo "⚠️  OpenSSL not available - skipping validation"
    fi
    echo ""
fi

# Setup certificate files
TARGET_CERT="$CERTS_DIR/proxy.crt"
TARGET_KEY="$CERTS_DIR/proxy.key"

if [[ "$COPY_MODE" == true ]]; then
    echo "📁 Copying certificate files to proxy directory..."
    cp "$CERT_FILE" "$TARGET_CERT"
    cp "$KEY_FILE" "$TARGET_KEY"
    echo "✅ Certificate files copied successfully"
    
elif [[ "$LINK_MODE" == true ]]; then
    echo "🔗 Creating symbolic links to certificate files..."
    
    # Remove existing files/links
    [[ -e "$TARGET_CERT" ]] && rm "$TARGET_CERT"
    [[ -e "$TARGET_KEY" ]] && rm "$TARGET_KEY"
    
    # Create symbolic links
    ln -s "$(realpath "$CERT_FILE")" "$TARGET_CERT"
    ln -s "$(realpath "$KEY_FILE")" "$TARGET_KEY"
    echo "✅ Symbolic links created successfully"
    
else
    echo "📝 Certificate files found - use --copy or --link to configure proxy"
fi

echo ""
echo "🔧 Next Steps:"
echo ""

# Create environment configuration
echo "1. Update your environment configuration:"
echo "   TLS_ENABLED=true"
echo "   TLS_CERT_PATH=certs/proxy.crt"
echo "   TLS_KEY_PATH=certs/proxy.key"
echo "   TLS_AUTO_GENERATE_CERT=false"
echo ""

# Show how to start the proxy
echo "2. Start the proxy with TLS enabled:"
echo "   ./rust-forward-proxy server \\"
echo "     --enable-tls \\"
echo "     --cert-path certs/proxy.crt \\"
echo "     --key-path certs/proxy.key \\"
echo "     --auto-generate-cert false"
echo ""

# Alternative with custom paths
if [[ "$COPY_MODE" == false && "$LINK_MODE" == false ]]; then
    echo "   Or use your certificate files directly:"
    echo "   ./rust-forward-proxy server \\"
    echo "     --enable-tls \\"
    echo "     --cert-path \"$CERT_FILE\" \\"
    echo "     --key-path \"$KEY_FILE\" \\"
    echo "     --auto-generate-cert false"
    echo ""
fi

echo "3. Test your proxy:"
echo "   HTTP:  curl -x http://127.0.0.1:8080 http://httpbin.org/get"
echo "   HTTPS: curl -x http://127.0.0.1:8080 https://httpbin.org/get"
echo ""

echo "✅ Custom certificate setup completed!"

# Show certificate manager CLI options
echo ""
echo "💡 Additional Tools:"
echo "   Use the certificate CLI for management:"
echo "   ./rust-forward-proxy cert inspect --cert-path \"$CERT_FILE\""
echo "   ./rust-forward-proxy cert validate --cert-path \"$CERT_FILE\" --key-path \"$KEY_FILE\""
echo ""

echo "⚠️  Important Notes:"
echo "   - Ensure your certificate includes the hostname/IP you'll use for the proxy"
echo "   - For HTTPS interception, clients need to trust your certificate"
echo "   - Use valid certificates from a trusted CA for production use"
echo ""

