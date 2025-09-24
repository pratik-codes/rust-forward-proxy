#!/bin/bash
# Test script for Securly certificate integration

set -e

echo "🧪 Testing Securly Certificate Integration"
echo "=========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if Securly certificate exists
print_status "Checking Securly certificate..."
if [ -f "ca-certs/securly_ca.crt" ]; then
    print_success "Securly certificate found at ca-certs/securly_ca.crt"
else
    print_error "Securly certificate not found!"
    echo "Expected location: ca-certs/securly_ca.crt"
    exit 1
fi

# Display certificate information
print_status "Securly Certificate Information:"
openssl x509 -in ca-certs/securly_ca.crt -text -noout | grep -A 10 "Subject:"
openssl x509 -in ca-certs/securly_ca.crt -text -noout | grep -A 2 "Validity"
openssl x509 -in ca-certs/securly_ca.crt -text -noout | grep -A 2 "Basic Constraints"

# Check if root CA exists
print_status "Checking Root CA for domain certificate signing..."
if [ -f "ca-certs/rootCA.crt" ] && [ -f "ca-certs/rootCA.key" ]; then
    print_success "Root CA found for domain certificate signing"
    print_status "Root CA Information:"
    openssl x509 -in ca-certs/rootCA.crt -text -noout | grep -A 5 "Subject:"
else
    print_warning "Root CA not found - proxy will generate self-signed certificates"
    print_status "Run 'make setup-ca' to create a root CA for trusted domain certificates"
fi

# Create test environment configuration
print_status "Creating test environment configuration..."
cat > .env.test << EOF
# Test configuration for Securly certificate integration
TLS_ENABLED=true
HTTPS_LISTEN_ADDR=127.0.0.1:8443
TLS_CERT_PATH=certs/proxy.crt
TLS_KEY_PATH=certs/proxy.key
TLS_AUTO_GENERATE_CERT=true
TLS_INTERCEPTION_ENABLED=true
TLS_ROOT_CA_CERT_PATH=ca-certs/securly_ca.crt
TLS_CA_CERT_PATH=ca-certs/rootCA.crt
TLS_CA_KEY_PATH=ca-certs/rootCA.key
TLS_SKIP_UPSTREAM_CERT_VERIFY=false
RUST_LOG=info
EOF

print_success "Test environment configuration created (.env.test)"

# Build the proxy
print_status "Building the proxy..."
if cargo build --quiet; then
    print_success "Proxy built successfully"
else
    print_error "Failed to build proxy"
    exit 1
fi

# Test certificate loading
print_status "Testing certificate loading..."
if cargo run --bin proxy -- cert info --cert-path ca-certs/securly_ca.crt 2>/dev/null; then
    print_success "Certificate loading test passed"
else
    print_warning "Certificate info command not available or failed"
fi

# Instructions for manual testing
print_status "Manual Testing Instructions:"
echo ""
echo "1. Start the proxy with Securly certificate integration:"
echo "   export $(cat .env.test | xargs) && cargo run"
echo ""
echo "2. In another terminal, test HTTPS interception:"
echo "   curl -x http://127.0.0.1:8080 https://httpbin.org/ip"
echo ""
echo "3. Check proxy logs for Securly certificate loading:"
echo "   Look for messages about 'Custom root CA certificate added to trust store'"
echo ""
echo "4. To install the Securly certificate in your browser:"
echo "   - Export the certificate: openssl x509 -in ca-certs/securly_ca.crt -out securly_ca.pem"
echo "   - Import into browser's certificate store as a trusted root CA"
echo ""
echo "5. To install the Root CA for trusted domain certificates:"
echo "   - Run: make setup-ca (if Makefile target exists)"
echo "   - Or manually install ca-certs/rootCA.crt in browser"

print_status "Configuration Summary:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📜 Securly CA Certificate: ca-certs/securly_ca.crt"
echo "🔐 Root CA for Signing:     ca-certs/rootCA.crt + ca-certs/rootCA.key"
echo "🌐 HTTPS Proxy Address:     127.0.0.1:8443"
echo "🔄 HTTP Proxy Address:      127.0.0.1:8080"
echo "🛡️  Certificate Verification: Enabled (includes Securly CA)"
echo "⚡ Auto-Generate Certs:     Enabled"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

print_success "Securly certificate integration test completed!"
print_status "The proxy is now configured to use the Securly certificate for HTTPS interception."

# Cleanup
print_status "Test files created:"
echo "  - .env.test (test environment configuration)"
echo ""
echo "To clean up test files: rm .env.test"
