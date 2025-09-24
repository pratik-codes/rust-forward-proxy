# 🔐 Certificate Modes for Development

The proxy supports multiple certificate authorities for HTTPS interception:

## 🚀 **Quick Usage**

### **Default Mode (rootCA)**
```bash
make dev
```
Uses your own generated root CA certificates from `ca-certs/rootCA.crt`

### **Securly Mode**
```bash
# Option 1: Environment variable
CERT=securly make dev

# Option 2: Dedicated target
make dev-securly
```
Uses Securly's CA certificate from `ca-certs/securly_ca.crt`

## 📋 **Certificate Files Required**

### **rootCA Mode**
- ✅ `ca-certs/rootCA.crt` (Root CA certificate)
- ✅ `ca-certs/rootCA.key` (Root CA private key)

### **Securly Mode**
- ✅ `ca-certs/securly_ca.crt` (Securly CA certificate)
- ⚠️ `ca-certs/securly_ca.key` (Securly CA private key - usually not available)

## 🔧 **How It Works**

1. **Certificate Selection**: The `CERT` environment variable determines which CA to use
2. **Environment Variables**: Sets `TLS_CA_CERT_PATH` and `TLS_CA_KEY_PATH` accordingly
3. **Fallback Behavior**: If Securly private key is missing, falls back to self-signed certificates

## 🌐 **Browser Setup**

### **For rootCA Mode**
Install `ca-certs/rootCA.crt` in your browser's certificate store

### **For Securly Mode**
Install `ca-certs/securly_ca.crt` in your browser's certificate store

## ⚠️ **Important Notes**

- **Securly Private Key**: Typically not available for security reasons
- **Fallback**: Without the private key, certificates will be self-signed
- **Browser Trust**: You must install the appropriate CA certificate in your browser
- **Security**: Only use on systems you control for testing

## 🎯 **Example Output**

### **rootCA Mode**
```
🔒 Starting proxy with rootCA certificates...
📜 CA Certificate: ca-certs/rootCA.crt
⚠️  Make sure to install rootCA.crt in your browser
```

### **Securly Mode**
```
🔒 Starting proxy with Securly CA certificates...
📜 CA Certificate: ca-certs/securly_ca.crt
⚠️  Warning: securly_ca.key not found - will fallback to self-signed certificates
⚠️  Make sure to install securly_ca.crt in your browser
```
