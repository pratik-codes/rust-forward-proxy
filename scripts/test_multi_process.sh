#!/bin/bash

# Test script for multi-process mode
# This demonstrates running 4 single-threaded processes

echo "🚀 Rust Forward Proxy - Multi-Process Mode Demo"
echo "================================================"
echo ""

# Check if the binary exists
if [ ! -f "target/release/rust-forward-proxy" ]; then
    echo "❌ Binary not found. Building first..."
    cargo build --release
fi

echo "📋 Configuration:"
echo "  - Mode: multi_process"
echo "  - Process Count: 4"
echo "  - Each process: single-threaded"
echo "  - Port Sharing: SO_REUSEPORT (Linux/macOS)"
echo ""

# Show configuration
echo "🔧 Multi-process configuration:"
cat config.yml | grep -A 5 "runtime:"
echo ""

echo "🎯 Starting multi-process proxy server..."
echo "   Each process will be single-threaded"
echo "   All 4 processes will share ports 80/443 using SO_REUSEPORT"
echo "   Running with sudo for privileged ports"
echo ""

# The main config.yml now has multi-process configuration

echo "💡 You should see 4 separate processes starting..."
echo "   Use 'ps aux | grep rust-forward-proxy' in another terminal to verify"
echo ""
echo "🌐 Test endpoints (privileged ports):"
echo "   HTTP:  curl -v -x http://127.0.0.1:80 http://httpbin.org/get"
echo "   HTTPS: curl -v -x http://127.0.0.1:80 https://httpbin.org/get"
echo ""
echo "Press Ctrl+C to stop all processes"
echo ""

# Start the proxy with multi-process configuration using sudo
echo "🔐 Starting with sudo for privileged ports..."
exec sudo ./target/release/rust-forward-proxy
