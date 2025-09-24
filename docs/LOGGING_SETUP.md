# 📁 Logging Configuration

The proxy now supports both console and file logging simultaneously. All logs are automatically saved to files in the `logs/` directory.

## 🔧 **Logging Features**

### **Automatic File Logging**
- ✅ **Console Output**: Real-time logs displayed in terminal
- ✅ **File Output**: All logs saved to `logs/proxy.log.*`
- ✅ **Daily Rotation**: Log files rotate daily (e.g., `proxy.log.2025-09-24`)
- ✅ **Thread-Safe**: Non-blocking async file writing
- ✅ **Structured Format**: Includes timestamps, thread IDs, file:line, level

### **Log Rotation**
- **Pattern**: `logs/proxy.log.YYYY-MM-DD`
- **Retention**: Manual cleanup required
- **Format**: Each day gets a new file
- **Non-blocking**: Uses background writer for performance

### **Log Details Include**
- **Timestamp**: ISO 8601 format with microsecond precision
- **Level**: INFO, WARN, ERROR, DEBUG, TRACE
- **Thread ID**: Identifies which thread generated the log
- **Source Location**: File name and line number
- **Message**: Formatted log content with emojis for easy scanning

## 📂 **Log File Structure**

```
logs/
├── proxy.log.2025-09-24    # Today's logs
├── proxy.log.2025-09-23    # Yesterday's logs  
└── proxy.log.2025-09-22    # Previous day's logs
```

## 🎯 **Sample Log Format**

```
2025-09-24T09:06:10.983197Z  INFO main ThreadId(01) src/logging/mod.rs:207: 📁 Logging initialized with env config - Console + File (logs/proxy.log)
2025-09-24T09:06:12.005732Z  INFO tokio-runtime-worker ThreadId(03) src/proxy/server.rs:177: 🔍 Starting HTTPS interception for example.com:443
2025-09-24T09:06:12.345234Z  INFO tokio-runtime-worker ThreadId(05) src/proxy/server.rs:339: 🔍 INTERCEPTED HTTPS: GET https://example.com/api/data
```

## ⚙️ **Configuration**

### **Environment Variables**
- `RUST_LOG=debug` - Enable debug logging
- `RUST_LOG=info` - Default info logging (recommended)
- `RUST_LOG=warn` - Only warnings and errors
- `RUST_LOG=error` - Only errors

### **Log Levels**
- **ERROR**: Critical errors that may cause the proxy to fail
- **WARN**: Warnings about certificate issues, connection failures
- **INFO**: Normal operational messages (default)
- **DEBUG**: Detailed debugging information
- **TRACE**: Very verbose tracing (not recommended for production)

## 🚀 **Usage Examples**

### **Start with Default Logging**
```bash
make dev
# Logs to console + logs/proxy.log.YYYY-MM-DD
```

### **Start with Debug Logging**
```bash
RUST_LOG=debug make dev
# Enhanced logging with debug details
```

### **View Live Logs**
```bash
# Console output
make dev

# Follow log file
tail -f logs/proxy.log.$(date +%Y-%m-%d)
```

### **Search Logs**
```bash
# Find certificate generation events
grep "📜.*CA-signed" logs/proxy.log.*

# Find TLS handshake failures
grep "❌.*TLS handshake failed" logs/proxy.log.*

# Find intercepted HTTPS requests
grep "🔍 INTERCEPTED HTTPS" logs/proxy.log.*
```

## 🔍 **Log Analysis**

### **Common Log Patterns**

**Certificate Generation:**
```
📜 Root CA found - generating CA-signed certificate for example.com
✅ OpenSSL CA-signed certificate generated for example.com
```

**HTTPS Interception:**
```
🔍 Starting HTTPS interception for example.com:443
✅ TLS handshake successful for example.com:443
🔍 INTERCEPTED HTTPS: GET https://example.com/api/data
```

**Connection Issues:**
```
❌ TLS handshake failed for example.com:443: received fatal alert: CertificateUnknown
```

## 📊 **Log File Management**

### **Automatic Features**
- ✅ Daily rotation prevents huge single files
- ✅ Non-blocking writes don't impact performance
- ✅ Automatic directory creation

### **Manual Cleanup**
```bash
# Remove logs older than 7 days
find logs/ -name "proxy.log.*" -mtime +7 -delete

# Compress old logs
gzip logs/proxy.log.$(date -d "yesterday" +%Y-%m-%d)

# Archive logs by month
mkdir -p archive/$(date +%Y-%m)
mv logs/proxy.log.$(date +%Y-%m)-* archive/$(date +%Y-%m)/
```

## 🛠️ **Troubleshooting**

### **No Log Files Created**
1. Check if `logs/` directory exists
2. Verify write permissions
3. Check disk space

### **Missing Log Entries**
1. Check log level (`RUST_LOG` environment variable)
2. Verify the proxy is running
3. Some debug messages only appear with `RUST_LOG=debug`

### **Large Log Files**
1. Logs rotate daily automatically
2. Consider cleaning up old logs periodically
3. Use log level filtering to reduce verbosity

## 🔄 **Integration with Development**

The logging system integrates seamlessly with all development commands:

```bash
# Standard development with logging
make dev

# Development with debug logging
RUST_LOG=debug make dev

# Securly certificate mode with logging
CERT=securly make dev

# All commands automatically log to files
```

All proxy operations including certificate generation, HTTPS interception, request/response handling, and error conditions are logged to both console and files for complete visibility and debugging capabilities.
