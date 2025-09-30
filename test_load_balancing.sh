#!/bin/bash

# Test script to verify load balancing across multiple processes
# This script sends multiple HTTP requests through the proxy and checks 
# if they are distributed across different PIDs

echo "🧪 Testing load balancing across proxy processes..."

# Start the proxy in background
echo "🚀 Starting proxy in multi-process mode..."
./target/release/rust-forward-proxy &
PROXY_PID=$!

# Wait for proxy to start
sleep 3

echo "📊 Sending test requests through proxy..."

# Send 100 requests through the proxy and capture the log output
for i in {1..100}; do
    curl -s -x http://127.0.0.1:80 http://httpbin.org/get > /dev/null 2>&1 &
    if [ $((i % 20)) -eq 0 ]; then
        echo "   Sent $i requests..."
    fi
done

# Wait for all requests to complete
wait

echo "⏱️  Waiting 5 seconds for requests to process..."
sleep 5

echo "📈 Analyzing PID distribution from logs..."

# Analyze the PID distribution in the logs
if [ -f "logs/proxy.log" ]; then
    echo "🔍 PID distribution analysis:"
    echo "   Process distribution (requests per PID):"
    
    # Extract unique PIDs and count their requests
    grep "✅.*INTERCEPTED\|✅.*completed" logs/proxy.log | \
    grep -o "PID:[0-9]*" | \
    sort | uniq -c | sort -nr | \
    head -10 | \
    while read count pid; do
        echo "      $pid: $count requests"
    done
    
    echo ""
    
    # Check if requests are distributed across multiple processes
    unique_pids=$(grep "✅.*INTERCEPTED\|✅.*completed" logs/proxy.log | \
                  grep -o "PID:[0-9]*" | sort -u | wc -l | tr -d ' ')
    
    total_requests=$(grep "✅.*INTERCEPTED\|✅.*completed" logs/proxy.log | wc -l | tr -d ' ')
    
    echo "📊 Load balancing summary:"
    echo "   Total unique processes handling requests: $unique_pids"
    echo "   Total requests processed: $total_requests"
    
    if [ "$unique_pids" -gt 1 ]; then
        echo "   ✅ SUCCESS: Load balancing is working - requests distributed across $unique_pids processes!"
    else
        echo "   ❌ ISSUE: All requests handled by single process - load balancing not working"
    fi
else
    echo "❌ No log file found - cannot analyze load distribution"
fi

# Clean up
echo "🧹 Stopping proxy..."
kill $PROXY_PID
wait $PROXY_PID 2>/dev/null

echo "✅ Test completed!"

