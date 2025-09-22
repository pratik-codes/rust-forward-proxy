#!/bin/bash

# Setup Root CA for HTTPS Interception
# This creates a root certificate that can be installed in browsers

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CA_DIR="$SCRIPT_DIR/../ca-certs"

echo "🔒 Setting up Root CA for HTTPS Interception..."

# Create CA directory
mkdir -p "$CA_DIR"
cd "$CA_DIR"

# Generate Root CA private key
if [ ! -f "rootCA.key" ]; then
    echo "📜 Generating Root CA private key..."
    openssl genrsa -out rootCA.key 4096
fi

# Generate Root CA certificate  
if [ ! -f "rootCA.crt" ]; then
    echo "📜 Generating Root CA certificate..."
    openssl req -x509 -new -nodes -key rootCA.key -sha256 -days 365 -out rootCA.crt -subj "/O=Rust Forward Proxy CA/CN=Rust Proxy Root CA"
fi

echo "✅ Root CA setup complete!"
echo ""
echo "📋 Next steps:"
echo "1. Install rootCA.crt in your browser's certificate store:"
echo "   - Chrome/Edge: Settings → Privacy → Manage certificates → Trusted Root → Import"
echo "   - Firefox: Settings → Privacy → Certificates → Import"
echo "   - macOS: Double-click rootCA.crt → Add to Keychain → Always Trust"
echo ""
echo "2. Root CA certificate location: $CA_DIR/rootCA.crt"
echo "3. After installing, restart your browser"
echo "4. Test with: curl -x http://127.0.0.1:8080 https://httpbin.org/get"
echo ""
echo "⚠️  Note: Only install this certificate on systems you control for testing!"

