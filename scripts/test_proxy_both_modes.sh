#!/bin/bash

# Test script for both development and production proxy modes
# Tests both 8080/8443 (dev) and 80/443 (production) configurations

set -e

echo "🧪 Testing Rust Forward Proxy - Both Modes"
echo "========================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Test development mode (8080/8443)
test_development_mode() {
    print_status "Testing Development Mode (8080/8443)..."
    echo "--------------------------------------------"
    
    # Update config for development ports
    sed -i.bak 's/listen_addr: "127.0.0.1:80"/listen_addr: "127.0.0.1:8080"/' config.yml
    sed -i.bak 's/https_listen_addr: "127.0.0.1:443"/https_listen_addr: "127.0.0.1:8443"/' config.yml
    
    # Run development tests
    HTTP_PROXY_PORT=8080 HTTPS_PROXY_PORT=8443 ./scripts/test_https_proxy.sh
    local dev_result=$?
    
    return $dev_result
}

# Test production mode (80/443)
test_production_mode() {
    print_status "Testing Production Mode (80/443)..."
    echo "-------------------------------------------"
    
    # Update config for production ports
    sed -i.bak 's/listen_addr: "127.0.0.1:8080"/listen_addr: "127.0.0.1:80"/' config.yml
    sed -i.bak 's/https_listen_addr: "127.0.0.1:8443"/https_listen_addr: "127.0.0.1:443"/' config.yml
    
    # Run production tests
    HTTP_PROXY_PORT=80 HTTPS_PROXY_PORT=443 ./scripts/test_https_proxy.sh
    local prod_result=$?
    
    return $prod_result
}

# Simple manual tests
test_basic_functionality() {
    local http_port=$1
    local https_port=$2
    
    print_status "Running basic tests on ports $http_port/$https_port..."
    
    # Test HTTP proxy
    local http_response=$(curl -s --connect-timeout 5 -x "http://127.0.0.1:$http_port" "http://httpbin.org/get" 2>/dev/null || echo "FAILED")
    if echo "$http_response" | grep -q '"url": "http://httpbin.org/get"'; then
        print_success "✅ HTTP proxy working on port $http_port"
    else
        print_error "❌ HTTP proxy failed on port $http_port"
        return 1
    fi
    
    # Test HTTPS tunneling
    local https_response=$(curl -s --connect-timeout 5 -k -x "http://127.0.0.1:$http_port" "https://httpbin.org/get" 2>/dev/null || echo "FAILED")
    if echo "$https_response" | grep -q '"url": "https://httpbin.org/get"'; then
        print_success "✅ HTTPS tunneling working on port $http_port"
    else
        print_error "❌ HTTPS tunneling failed on port $http_port"
        return 1
    fi
    
    return 0
}

cleanup() {
    print_status "Cleaning up..."
    sudo pkill rust-forward-proxy 2>/dev/null || true
    pkill cargo 2>/dev/null || true
    
    # Restore original config if backup exists
    if [ -f "config.yml.bak" ]; then
        mv config.yml.bak config.yml
        print_status "Restored original config.yml"
    fi
}

# Set up cleanup trap
trap cleanup EXIT

# Main execution
main() {
    print_status "Starting comprehensive proxy testing..."
    
    # Build project first
    print_status "Building project..."
    if ! cargo build --release >/dev/null 2>&1; then
        print_error "Build failed"
        exit 1
    fi
    
    print_status "Testing different port configurations..."
    echo ""
    
    # Test 1: Development mode
    print_status "=== TEST 1: Development Mode (8080/8443) ==="
    test_development_mode
    dev_result=$?
    
    sleep 2
    cleanup
    sleep 2
    
    # Test 2: Production mode  
    print_status "=== TEST 2: Production Mode (80/443) ==="
    test_production_mode
    prod_result=$?
    
    echo ""
    echo "========================================"
    print_status "Test Results Summary:"
    
    if [ $dev_result -eq 0 ]; then
        print_success "Development Mode (8080/8443): PASSED"
    else
        print_error "Development Mode (8080/8443): FAILED"
    fi
    
    if [ $prod_result -eq 0 ]; then
        print_success "Production Mode (80/443): PASSED"
    else
        print_error "Production Mode (80/443): FAILED"
    fi
    
    echo ""
    print_status "Usage examples:"
    print_status "Development: HTTP_PROXY_PORT=8080 HTTPS_PROXY_PORT=8443 ./scripts/test_https_proxy.sh"
    print_status "Production:  HTTP_PROXY_PORT=80 HTTPS_PROXY_PORT=443 ./scripts/test_https_proxy.sh"
    
    if [ $dev_result -eq 0 ] && [ $prod_result -eq 0 ]; then
        print_success "🎉 All tests passed!"
        exit 0
    else
        print_warning "Some tests failed - check logs for details"
        exit 1
    fi
}

main "$@"
