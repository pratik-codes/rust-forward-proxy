#!/bin/bash

# Interactive script for testing multi-process proxy on privileged ports
# This script requires user interaction for sudo password

echo "🔐 Multi-Process Proxy on Privileged Ports"
echo "=========================================="
echo ""
echo "This script will:"
echo "  ✅ Start 4 single-threaded processes"
echo "  ✅ Use SO_REUSEPORT for load balancing"
echo "  ✅ Run on privileged ports 80 and 443"
echo "  ✅ Require sudo permissions"
echo ""

# Check if running as root
if [ "$EUID" -eq 0 ]; then
    echo "✅ Running as root"
else
    echo "⚠️  This script needs sudo permissions for privileged ports"
    echo "   You will be prompted for your password"
    echo ""
fi

# Create temporary config for privileged ports
echo "🔧 Creating privileged port configuration..."
cp config.yml config-privileged.yml
sed -i.bak 's/127.0.0.1:8080/127.0.0.1:80/g' config-privileged.yml
sed -i.bak 's/127.0.0.1:8443/127.0.0.1:443/g' config-privileged.yml

echo "📋 Privileged port configuration:"
grep -E "(listen_addr|https_listen_addr)" config-privileged.yml
echo ""

echo "🚀 Starting proxy with sudo..."
echo "Press Ctrl+C to stop all processes"
echo ""

# Set config file environment variable and run with sudo
export CONFIG_FILE=config-privileged.yml

# Run the proxy (this will prompt for password)
sudo -E ./target/release/rust-forward-proxy

# Cleanup
echo ""
echo "🧹 Cleaning up..."
rm -f config-privileged.yml config-privileged.yml.bak
