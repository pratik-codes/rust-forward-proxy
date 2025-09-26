# 🌐 Browser Proxy Configuration Guide

## ❌ ISSUE IDENTIFIED: Incorrect Browser Configuration

Based on the logs, your browser is **incorrectly configured**. All errors show TLS handshake failures on port 443, which means the browser is trying to connect to the HTTPS proxy port instead of the HTTP proxy port.

## ✅ CORRECT Browser Configuration

### For Production Mode (80/443):
```
HTTP Proxy:  127.0.0.1:80
HTTPS Proxy: 127.0.0.1:80  ← IMPORTANT: Use port 80, NOT 443!
SOCKS Proxy: [Leave empty]
```

### For Development Mode (8080/8443):
```
HTTP Proxy:  127.0.0.1:8080
HTTPS Proxy: 127.0.0.1:8080  ← IMPORTANT: Use port 8080, NOT 8443!
SOCKS Proxy: [Leave empty]
```

## 🔧 Browser-Specific Configuration

### Chrome/Edge:
1. Settings → Advanced → System → Open proxy settings
2. Manual proxy configuration:
   - HTTP proxy: `127.0.0.1:80`
   - Secure proxy (HTTPS): `127.0.0.1:80`
   - Use this proxy server for all protocols: ✅

### Firefox:
1. Settings → Network Settings → Settings button
2. Manual proxy configuration:
   - HTTP Proxy: `127.0.0.1` Port: `80`
   - Use this proxy server for all protocols: ✅
   - HTTPS Proxy: `127.0.0.1` Port: `80`

### Safari (macOS):
1. System Preferences → Network → Advanced → Proxies
2. Check "Web Proxy (HTTP)": `127.0.0.1:80`
3. Check "Secure Web Proxy (HTTPS)": `127.0.0.1:80`

## 🚫 Common Mistakes

❌ **DON'T** configure HTTPS proxy as `127.0.0.1:443`  
❌ **DON'T** configure HTTPS proxy as `127.0.0.1:8443`  
❌ **DON'T** use different ports for HTTP and HTTPS  

## ✅ How It Works

```
Browser Request Flow:
┌─────────┐    HTTP/HTTPS     ┌─────────────┐    CONNECT     ┌─────────────┐
│ Browser │ ───────────────→  │ HTTP Proxy  │ ─────────────→ │ Target Site │
│         │    Port 80/8080   │ (Port 80)   │                │             │
└─────────┘                   └─────────────┘                └─────────────┘

WRONG:
┌─────────┐    TLS Handshake  ┌─────────────┐
│ Browser │ ─────────X─────→  │ HTTPS Proxy │ ← This causes the errors!
│         │    Port 443       │ (Port 443)  │
└─────────┘                   └─────────────┘
```

## 🧪 Test Your Configuration

After configuring your browser correctly:

1. **Start the proxy:**
   ```bash
   sudo make dev
   ```

2. **Test with curl (to verify proxy works):**
   ```bash
   curl -x http://127.0.0.1:80 http://httpbin.org/get
   curl -x http://127.0.0.1:80 https://httpbin.org/get -k
   ```

3. **Test in browser:**
   - Navigate to `http://httpbin.org/get`
   - Navigate to `https://httpbin.org/get`
   - Check logs: `tail -f logs/proxy.log.2025-09-26`

## 🔍 Expected Log Output (Correct Configuration)

With correct configuration, you should see:
```
INFO: Processing HTTP request: GET http://httpbin.org/get
INFO: Processing CONNECT request for httpbin.org:443
```

**NOT** constant TLS handshake errors!

## 🎯 Summary

The key insight: **Both HTTP and HTTPS traffic should go through the HTTP proxy port (80/8080)**. The HTTPS proxy port (443/8443) is for direct TLS termination, not for browser proxy traffic.
