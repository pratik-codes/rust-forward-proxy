# HTTPS Interception Demo

## What This Shows You

This demo shows you how to capture and log **all HTTP and HTTPS request/response content** that passes through your proxy.

## 🔒 How HTTPS Interception Works

1. **Normal HTTPS**: `Browser ←→ [encrypted tunnel] ←→ Website`
   - Proxy can't see content, only connection metadata

2. **With Interception**: `Browser ←→ Proxy [decrypt & log] ←→ Website`
   - Proxy decrypts HTTPS, logs all content, re-encrypts to destination

## 🚀 Quick Start

### Terminal 1: Start Intercepting Proxy
```bash
make dev-local-intercept
```

### Terminal 2: Test HTTP (should show detailed logs)
```bash
curl -x http://127.0.0.1:8080 http://httpbin.org/get
```

### Terminal 3: Test HTTPS (should now show detailed content!)
```bash
curl -x http://127.0.0.1:8080 https://httpbin.org/get --proxy-insecure
```

## 📊 What You'll See in the Logs

### HTTP Request (as before):
```
📥 GET http://httpbin.org/get from 127.0.0.1
🔄 Forwarding GET to upstream  
📤 Upstream response: 200 (800ms)
✅ GET /get → 200 OK (801ms)
```

### HTTPS Request (with interception):
```
📥 CONNECT httpbin.org:443 from 127.0.0.1
🔍 CONNECT httpbin.org:443 - INTERCEPTING (will decrypt and log HTTPS)
📜 Generating self-signed certificate for httpbin.org
✅ TLS handshake successful for httpbin.org:443
🌐 Processing decrypted HTTPS traffic for httpbin.org:443
🔍 INTERCEPTED HTTPS: GET https://httpbin.org/get (decrypted from httpbin.org:443)
📋 Request Headers:
  user-agent: curl/8.7.1
  accept: */*
🔄 Forwarding intercepted GET request to httpbin.org:443
📤 Upstream HTTPS response: 200 OK
📋 Response Headers:
  content-type: application/json
  content-length: 255
📄 Response Body (255 bytes):
{
  "args": {}, 
  "headers": {
    "Accept": "*/*", 
    "Host": "httpbin.org", 
    "User-Agent": "curl/8.7.1"
  }, 
  "origin": "223.233.83.142", 
  "url": "https://httpbin.org/get"
}
✅ INTERCEPTED GET /get → 200 OK (4056 ms)
```

## ⚠️  Certificate Warnings

Your browser/curl will show certificate warnings because the proxy uses a self-signed certificate. This is normal for development.

## 🌐 Browser Setup

To use with your browser:
1. Configure proxy: `http://127.0.0.1:8080`
2. Accept certificate warnings
3. Browse normally - all HTTPS content will be logged!

## 🛡️ Security Note

HTTPS interception should only be used in development/testing environments where you control both ends. Never intercept traffic you don't own.
