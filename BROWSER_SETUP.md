# 🌐 Browser HTTPS Interception Setup

This guide will help you configure your browser to work with HTTPS interception so you can see all encrypted traffic content.

## 🔧 **Quick Setup Steps**

### **Step 1: Generate Root CA**
```bash
make setup-ca
```

### **Step 2: Install Root Certificate in Browser**

#### **Chrome/Edge/Safari (macOS)**
1. Open the generated certificate:
   ```bash
   open ca-certs/rootCA.crt
   ```
2. **macOS Keychain** will open
3. **Add** the certificate to your keychain
4. Find "Rust Proxy Root CA" in Keychain Access
5. **Double-click** the certificate
6. Expand **"Trust"** section
7. Set **"When using this certificate"** to **"Always Trust"**
8. **Save changes** (enter password if prompted)

#### **Chrome/Edge (Manual)**
1. Open Chrome → **Settings**
2. Go to **Privacy and Security** → **Security** → **Manage certificates**
3. Go to **Trusted Root Certification Authorities** tab
4. Click **Import**
5. Select `ca-certs/rootCA.crt`
6. Place in **"Trusted Root Certification Authorities"** store
7. **Restart browser**

#### **Firefox**
1. Open Firefox → **Settings**
2. Go to **Privacy & Security**
3. Scroll to **Certificates** → Click **View Certificates**
4. Go to **Authorities** tab
5. Click **Import**
6. Select `ca-certs/rootCA.crt` 
7. Check **"Trust this CA to identify websites"**
8. **OK** and restart Firefox

### **Step 3: Configure Browser Proxy**

#### **Chrome/Edge**
1. Go to **Settings** → **Advanced** → **System**
2. Click **"Open your computer's proxy settings"**
3. **Manual proxy setup:**
   - **HTTP Proxy:** `127.0.0.1:8080`
   - **HTTPS Proxy:** `127.0.0.1:8080`
   - **Use proxy for all protocols:** ✅

#### **Firefox**
1. Go to **Settings** → **Network Settings**
2. Click **"Settings"** button
3. Select **"Manual proxy configuration"**
4. **HTTP Proxy:** `127.0.0.1` **Port:** `8080`
5. **SSL Proxy:** `127.0.0.1` **Port:** `8080`
6. Check **"Use this proxy server for all protocols"**

### **Step 4: Start Intercepting Proxy**
```bash
make dev-local-intercept
```

### **Step 5: Test**
1. Browse to any HTTPS website (e.g., https://httpbin.org/get)
2. Check the proxy logs - you should now see **complete HTTPS content**!

## 🎯 **What You'll See**

**Before Certificate Installation:**
```
❌ TLS handshake failed: received fatal alert: CertificateUnknown
```

**After Certificate Installation:**
```
✅ TLS handshake successful for httpbin.org:443
🌐 Processing decrypted HTTPS traffic for httpbin.org:443
🔍 INTERCEPTED HTTPS: GET https://httpbin.org/get
📋 Request Headers:
  user-agent: Mozilla/5.0...
  cookie: session=abc123...
📤 Upstream HTTPS response: 200 OK
📄 Response Body: {"success": true}
✅ INTERCEPTED GET /get → 200 OK
```

## ⚠️ **Security Notes**

- **Development Only**: Only install the root certificate on systems you control
- **Remove After Testing**: Delete the root certificate when done testing
- **Certificate Warnings**: You may still see warnings for some sites - this is normal

## 🔧 **Troubleshooting**

### **Still Getting Certificate Errors?**
1. **Verify certificate installation**: Look for "Rust Proxy Root CA" in certificate store
2. **Restart browser** completely after certificate installation
3. **Clear browser cache/data**
4. **Try incognito/private mode**

### **Some Sites Don't Work?**
Some sites use **certificate pinning** and will reject intercepted certificates. This is intentional security behavior.

### **Remove Certificate**
**macOS:** Keychain Access → Find "Rust Proxy Root CA" → Delete
**Windows:** certmgr.msc → Trusted Root → Delete certificate
**Firefox:** Settings → Certificates → Authorities → Delete certificate

## 🚀 **Advanced Usage**

### **Command Line Testing**
```bash
# Test without certificate issues
curl -x http://127.0.0.1:8080 https://httpbin.org/get -k

# Test with proper certificate validation (after browser setup)
curl -x http://127.0.0.1:8080 https://httpbin.org/get
```

### **Different Browsers**
You can configure different browsers with different settings to test various scenarios.

## 🎉 **Success!**

Once configured, you'll have **complete visibility** into all HTTPS traffic passing through your browser - headers, body content, cookies, everything!

Perfect for:
- **API Development**: See exactly what your web apps send
- **Security Testing**: Understand data transmission
- **Debugging**: Track down mysterious network issues
- **Learning**: Understand how HTTPS really works

