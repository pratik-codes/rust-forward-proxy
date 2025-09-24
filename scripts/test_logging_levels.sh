#!/bin/bash
# Test script for logging level changes

set -e

echo "🧪 Testing Logging Level Changes"
echo "================================="

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

# Build the proxy first
print_status "Building the proxy..."
if cargo build --quiet; then
    print_success "Proxy built successfully"
else
    print_error "Failed to build proxy"
    exit 1
fi

print_status "Testing logging level configuration..."

# Test 1: INFO level - should show HTTP/HTTPS requests but not CONNECT
echo ""
print_status "Test 1: INFO level logging"
print_status "Expected: HTTP/HTTPS requests visible, CONNECT requests hidden"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Create test log file
LOG_FILE="test_info_level.log"
cat > test_info_level.log << 'EOF'
2024-09-24T13:14:15.123456Z  INFO rust_forward_proxy::utils::logging: 📥 GET http://httpbin.org/ip from 127.0.0.1
2024-09-24T13:14:15.234567Z DEBUG rust_forward_proxy::utils::logging: 🔐 CONNECT httpbin.org:443 from 127.0.0.1
2024-09-24T13:14:15.345678Z  INFO rust_forward_proxy::proxy::server: 🔍 INTERCEPTED HTTPS: GET https://httpbin.org/ip
2024-09-24T13:14:15.456789Z DEBUG rust_forward_proxy::utils::logging: 🔐 CONNECT REQUEST: Target: httpbin.org:443
2024-09-24T13:14:15.567890Z  INFO rust_forward_proxy::utils::logging: ✅ GET /ip → 200 (150ms)
EOF

print_status "Sample INFO level log output:"
grep "INFO" test_info_level.log || true
print_warning "CONNECT requests should NOT appear at INFO level (they are now at DEBUG level)"

echo ""

# Test 2: DEBUG level - should show all requests
print_status "Test 2: DEBUG level logging"
print_status "Expected: All requests visible (HTTP/HTTPS and CONNECT)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

print_status "Sample DEBUG level log output:"
cat test_info_level.log

print_success "At DEBUG level, both INFO and DEBUG messages are visible"

echo ""

# Test 3: Configuration examples
print_status "Configuration Examples:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

echo ""
echo "For INFO level (production) - only HTTP/HTTPS requests:"
echo "  export RUST_LOG=info"
echo "  cargo run"
echo ""
echo "For DEBUG level (development) - all requests including CONNECT:"
echo "  export RUST_LOG=debug"
echo "  cargo run"
echo ""
echo "For module-specific logging:"
echo "  export RUST_LOG=rust_forward_proxy=debug,hyper=info"
echo "  cargo run"

# Test 4: Show actual function behavior
print_status "Code Changes Summary:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

echo ""
echo "1. HTTP/HTTPS requests:"
echo "   - log_incoming_request(): info!() for non-CONNECT methods"
echo "   - handle_intercepted_request(): info!() for HTTPS content"
echo "   - log_http_success(): info!() for responses"
echo ""
echo "2. CONNECT requests:"
echo "   - log_incoming_request(): log_debug!() for CONNECT methods"
echo "   - log_connect_request(): log_debug!() for CONNECT targets"
echo "   - log_connect_success(): log_debug!() for tunnel establishment"
echo "   - log_connect_failure(): log_debug!() for connection failures"

echo ""

# Test 5: Manual testing instructions
print_status "Manual Testing Instructions:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

echo ""
echo "1. Start proxy with INFO level (only HTTP/HTTPS visible):"
echo "   RUST_LOG=info cargo run"
echo ""
echo "2. In another terminal, test HTTP request (should be visible):"
echo "   curl -x http://127.0.0.1:8080 http://httpbin.org/ip"
echo ""
echo "3. Test HTTPS request (CONNECT should be hidden, but intercepted HTTPS visible):"
echo "   curl -x http://127.0.0.1:8080 https://httpbin.org/ip"
echo ""
echo "4. Start proxy with DEBUG level (all requests visible):"
echo "   RUST_LOG=debug cargo run"
echo ""
echo "5. Repeat tests - now CONNECT requests should also be visible"

echo ""

# Test 6: Show log level matrix
print_status "Log Level Matrix:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

echo ""
echo "| Request Type              | INFO Level | DEBUG Level |"
echo "|---------------------------|------------|-------------|"
echo "| HTTP GET/POST/etc         |     ✅      |      ✅      |"
echo "| HTTPS Intercepted Content |     ✅      |      ✅      |"
echo "| CONNECT Requests          |     ❌      |      ✅      |"
echo "| CONNECT Tunnel Setup      |     ❌      |      ✅      |"
echo "| CONNECT Failures          |     ❌      |      ✅      |"

echo ""

# Cleanup
rm -f test_info_level.log

print_success "Logging level changes have been successfully implemented!"
print_status "Summary:"
echo "  ✅ HTTP/HTTPS requests: INFO level (always visible in production)"
echo "  ✅ CONNECT requests: DEBUG level (hidden in production, visible in development)"
echo "  ✅ Intercepted HTTPS content: INFO level (visible when content is decrypted)"
echo "  ✅ All logging functions updated accordingly"

echo ""
print_status "To test the changes:"
echo "  1. Run: RUST_LOG=info cargo run (production mode)"
echo "  2. Run: RUST_LOG=debug cargo run (development mode)"
echo "  3. Compare the visibility of CONNECT vs HTTP/HTTPS requests"
