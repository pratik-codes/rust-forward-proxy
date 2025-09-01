# Deployment Guide

This guide provides detailed instructions for deploying the Rust Forward Proxy in various environments, updated for the current modular architecture with full HTTP/HTTPS support.

## Current Implementation Features

The proxy now includes:
- ✅ **Full HTTP Interception** - Complete request/response logging and processing
- ✅ **HTTPS Tunneling** - Proper CONNECT method support with hyper upgrade mechanism  
- ✅ **Production Logging** - Two-tier logging system (clean INFO, verbose DEBUG)
- ✅ **Modular Architecture** - Well-organized codebase following DRY principles
- ✅ **High Performance** - Async Tokio/Hyper implementation with bidirectional tunneling

## Prerequisites

Before deploying the proxy, ensure you have the following:

- Rust toolchain (stable version 1.70.0 or newer)
- Cargo package manager
- Git
- System dependencies (OpenSSL, etc.)
- A target server or cloud environment

## Deployment Options

### 1. Local Development Deployment

#### Building the Proxy

```bash
# Clone the repository
git clone <repository-url>
cd rust-forward-proxy

# Build the proxy in debug mode
cargo build

# Run the proxy with default settings
cargo run
```

#### Testing the Deployment

```bash
# Test HTTP request interception
curl -x http://localhost:8080 http://httpbin.org/get

# Test HTTPS tunneling (CONNECT method)
curl -x http://localhost:8080 https://httpbin.org/get

# Test with verbose output to see CONNECT tunnel
curl -v -x http://localhost:8080 https://www.google.com

# Test POST request with data
curl -x http://localhost:8080 -X POST http://httpbin.org/post \
  -H "Content-Type: application/json" \
  -d '{"test":"data"}'
```

#### Verifying Logging Output

**INFO Level (Production):**
```bash
RUST_LOG=info cargo run
# Should show clean, single-line logs:
# 📥 GET http://httpbin.org/get from 127.0.0.1
# 🔄 Forwarding GET to upstream
# 📤 Upstream response: 200 (156ms)
# ✅ GET /get → 200 OK (158ms)
```

**DEBUG Level (Development):**
```bash
RUST_LOG=debug cargo run
# Should show verbose logs with full request/response data
```

### 2. Production Deployment

#### Building for Production

```bash
# Build an optimized release binary
cargo build --release

# The binary will be available at target/release/rust-forward-proxy
```

#### Running as a Service

##### Systemd Service (Linux)

Create a systemd service file at `/etc/systemd/system/rust-forward-proxy.service`:

```ini
[Unit]
Description=Rust Forward Proxy
After=network.target

[Service]
Type=simple
User=proxy
ExecStart=/path/to/rust-forward-proxy
Environment="RUST_LOG=info"
Restart=always
RestartSec=5
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
```

Enable and start the service:

```bash
sudo systemctl daemon-reload
sudo systemctl enable rust-forward-proxy
sudo systemctl start rust-forward-proxy
sudo systemctl status rust-forward-proxy
```

##### Using Supervisor (Unix-like systems)

Install supervisor:

```bash
# Ubuntu/Debian
sudo apt-get install supervisor

# CentOS/RHEL
sudo yum install supervisor
```

Create a configuration file at `/etc/supervisor/conf.d/rust-forward-proxy.conf`:

```ini
[program:rust-forward-proxy]
command=/path/to/rust-forward-proxy
autostart=true
autorestart=true
startsecs=5
startretries=3
user=proxy
redirect_stderr=true
stdout_logfile=/var/log/rust-forward-proxy.log
environment=RUST_LOG=info
```

Update supervisor:

```bash
sudo supervisorctl reread
sudo supervisorctl update
sudo supervisorctl status rust-forward-proxy
```

### 3. Docker Deployment

#### Creating a Dockerfile

Create a `Dockerfile` in the project root:

```dockerfile
FROM rust:1.70 as builder
WORKDIR /usr/src/rust-forward-proxy
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/rust-forward-proxy/target/release/rust-forward-proxy /usr/local/bin/
ENV RUST_LOG=info
EXPOSE 8080
CMD ["rust-forward-proxy"]
```

#### Building and Running the Docker Image

```bash
# Build the Docker image
docker build -t rust-forward-proxy .

# Run the container
docker run -p 8080:8080 rust-forward-proxy

# Run with custom environment variables
docker run -p 8080:8080 -e RUST_LOG=debug rust-forward-proxy
```

### 4. Cloud Deployment

#### AWS EC2

1. Launch an EC2 instance (Amazon Linux 2 recommended)
2. Install Rust and build tools:

```bash
sudo yum update -y
sudo yum install -y gcc openssl-devel git
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

3. Clone and build the project:

```bash
git clone <repository-url>
cd rust-forward-proxy
cargo build --release
```

4. Create a systemd service as described above
5. Configure security groups to allow inbound traffic on port 8080

#### Google Cloud Run

1. Create a `Dockerfile` as described above
2. Build and push the Docker image:

```bash
# Build the image
docker build -t gcr.io/your-project/rust-forward-proxy .

# Push the image to Google Container Registry
docker push gcr.io/your-project/rust-forward-proxy
```

3. Deploy to Cloud Run:

```bash
gcloud run deploy rust-forward-proxy \
  --image gcr.io/your-project/rust-forward-proxy \
  --platform managed \
  --port 8080 \
  --memory 512Mi \
  --set-env-vars="RUST_LOG=info"
```

## Configuration

### Environment Variables

The proxy can be configured using the following environment variables:

- `RUST_LOG`: Sets the logging level (trace, debug, info, warn, error)
- `PROXY_HOST`: Host address to bind to (default: 127.0.0.1)
- `PROXY_PORT`: Port to listen on (default: 8080)
- `REQUEST_TIMEOUT`: Request timeout in seconds (default: 30)
- `MAX_BODY_SIZE`: Maximum request body size in bytes (default: 1MB)

Example:

```bash
RUST_LOG=debug PROXY_PORT=9090 ./rust-forward-proxy
```

### Configuration File (Optional)

For more advanced configurations, create a `config.toml` file:

```toml
# Server configuration
[server]
listen_addr = "127.0.0.1:8080"
log_level = "info"
request_timeout = 30
max_body_size = 1048576  # 1MB

# Upstream configuration
[upstream]
url = "http://localhost:3000"
connect_timeout = 5
keep_alive_timeout = 60

# Rate limiting
[rate_limit]
enabled = true
requests_per_minute = 100
```

## Security Considerations

### TLS Configuration

To enable TLS:

1. Generate or obtain TLS certificates:

```bash
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes
```

2. Update your configuration to use TLS:

```toml
[server]
listen_addr = "127.0.0.1:8443"
tls_cert_path = "cert.pem"
tls_key_path = "key.pem"
```

### Authentication

To enable API key authentication:

```toml
[auth]
enabled = true
api_key = "your-secret-api-key"
```

### Firewall Configuration

Configure your firewall to allow only necessary connections:

```bash
# Allow incoming connections to the proxy port
sudo ufw allow 8080/tcp

# Restrict access to specific IP ranges
sudo ufw allow from 192.168.1.0/24 to any port 8080
```

## Monitoring and Logging

### Log Collection

Configure log collection using a tool like Filebeat:

```yaml
# filebeat.yml
filebeat.inputs:
- type: log
  enabled: true
  paths:
    - /var/log/rust-forward-proxy.log
```

### Metrics Collection

Expose metrics for Prometheus:

1. Add a metrics endpoint to your proxy
2. Configure Prometheus to scrape metrics:

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'rust-forward-proxy'
    scrape_interval: 15s
    static_configs:
      - targets: ['localhost:8080']
```

### Health Checking

Configure health checks for your deployment environment:

- **HTTP Health Check**: `GET /health`
- **Load Balancer**: Configure to monitor the health endpoint
- **Container Orchestration**: Set up liveness and readiness probes

## Scaling

### Horizontal Scaling

To scale horizontally:

1. Deploy multiple instances of the proxy
2. Set up a load balancer in front of the instances
3. Configure session affinity if needed

Example with Nginx as a load balancer:

```nginx
upstream proxy_backend {
    server 127.0.0.1:8081;
    server 127.0.0.1:8082;
    server 127.0.0.1:8083;
}

server {
    listen 80;
    
    location / {
        proxy_pass http://proxy_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

### Vertical Scaling

To scale vertically:

1. Increase server resources (CPU, memory)
2. Adjust the proxy configuration to utilize available resources:

```toml
[server]
worker_threads = 16  # Set based on CPU cores
connection_limit = 10000
```

## Troubleshooting

### Common Issues

1. **Connection Refused**: Ensure the proxy is running and the port is correctly configured
2. **High Latency**: Check system resources and connection pool settings
3. **Memory Usage**: Adjust maximum body size and connection limits
4. **Certificate Errors**: Verify TLS certificate paths and validity

### Debugging

Enable debug logging:

```bash
RUST_LOG=debug ./rust-forward-proxy
```

Trace network traffic:

```bash
sudo tcpdump -i any port 8080 -w proxy.pcap
```

## Maintenance

### Updates

To update the proxy:

```bash
# Pull latest changes
git pull

# Rebuild and restart
cargo build --release
sudo systemctl restart rust-forward-proxy
```

### Backup Configuration

Regularly backup your configuration:

```bash
# Create a backup directory
mkdir -p /backup/rust-forward-proxy

# Backup configuration
cp config.toml /backup/rust-forward-proxy/config.toml.$(date +%Y%m%d)
```

## Production Checklist

Before deploying to production, ensure:

- [ ] Logging is properly configured
- [ ] Rate limiting is enabled
- [ ] Authentication is configured (if needed)
- [ ] TLS is configured (if needed)
- [ ] Health checks are implemented
- [ ] Monitoring is set up
- [ ] System resources are adequate
- [ ] Firewall rules are configured
- [ ] Backup procedures are in place

## Advanced Deployment

### Kubernetes Deployment

Create a `kubernetes.yaml` file:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: rust-forward-proxy
spec:
  replicas: 3
  selector:
    matchLabels:
      app: rust-forward-proxy
  template:
    metadata:
      labels:
        app: rust-forward-proxy
    spec:
      containers:
      - name: proxy
        image: your-registry/rust-forward-proxy:latest
        ports:
        - containerPort: 8080
        env:
        - name: RUST_LOG
          value: "info"
        resources:
          limits:
            cpu: "1"
            memory: "512Mi"
          requests:
            cpu: "500m"
            memory: "256Mi"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 3
          periodSeconds: 3
---
apiVersion: v1
kind: Service
metadata:
  name: rust-forward-proxy
spec:
  selector:
    app: rust-forward-proxy
  ports:
  - port: 80
    targetPort: 8080
  type: LoadBalancer
```

Deploy to Kubernetes:

```bash
kubectl apply -f kubernetes.yaml
```

### CI/CD Integration

Example GitHub Actions workflow:

```yaml
name: Build and Deploy

on:
  push:
    branches: [ main ]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v2
    
    - name: Build and Test
      run: |
        cargo build --release
        cargo test
        
    - name: Build Docker image
      run: docker build -t rust-forward-proxy .
      
    - name: Push to registry
      run: |
        echo ${{ secrets.DOCKER_PASSWORD }} | docker login -u ${{ secrets.DOCKER_USERNAME }} --password-stdin
        docker tag rust-forward-proxy ${{ secrets.DOCKER_USERNAME }}/rust-forward-proxy:latest
        docker push ${{ secrets.DOCKER_USERNAME }}/rust-forward-proxy:latest
```