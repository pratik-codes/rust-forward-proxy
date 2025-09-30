# Multi-Process Mode Setup

This document explains how to run the Rust Forward Proxy in multi-process mode with 4 single-threaded processes.

## Overview

The proxy now supports three runtime modes:
- **single_threaded**: One process with one thread (current default)
- **multi_threaded**: One process with multiple threads
- **multi_process**: Multiple processes, each single-threaded

## Multi-Process Architecture

When running in `multi_process` mode:
- The parent process spawns 4 child processes
- Each child process runs a single-threaded Tokio runtime
- All processes share the same port using SO_REUSEPORT (Linux/macOS)
- Load balancing is handled by the kernel

## Configuration

### In config.yml:

```yaml
runtime:
  mode: "multi_process"       # Enable multi-process mode
  worker_threads: null        # Not used in multi_process mode
  process_count: 4            # Number of processes to spawn
  use_reuseport: true         # Enable SO_REUSEPORT for port sharing
```

### Environment Variables:

```bash
# Override config file settings
export PROXY_RUNTIME_MODE=multi_process
export PROXY_PROCESS_COUNT=4
export PROXY_USE_REUSEPORT=true
```

## How It Works

1. **Parent Process**: 
   - Loads configuration
   - Spawns N child processes (default: 4)
   - Waits for all children to complete
   - Handles graceful shutdown

2. **Child Processes**:
   - Each gets a unique process index (0, 1, 2, 3)
   - Runs single-threaded Tokio runtime
   - Binds to the same port using SO_REUSEPORT
   - Handles requests independently

3. **Load Distribution**:
   - Kernel distributes incoming connections across processes
   - Natural load balancing without additional complexity

## Running Multi-Process Mode

### Using the test script:
```bash
./test_multi_process.sh
```

### Manual execution:
```bash
# Build the proxy
cargo build --release

# Run with the unified config.yml (now includes multi-process mode)
./target/release/rust-forward-proxy
```

## Verification

Check that 4 processes are running:
```bash
# In another terminal
ps aux | grep rust-forward-proxy

# You should see:
# - 1 parent process (the launcher)
# - 4 child processes (the actual proxy workers)
```

Monitor process activity:
```bash
# Send test requests
curl -x http://127.0.0.1:8080 http://httpbin.org/get
curl -x http://127.0.0.1:8080 https://httpbin.org/get

# Check logs to see which process handled each request
tail -f logs/proxy.log
```

## Benefits

1. **True Parallelism**: Each process runs independently
2. **Fault Isolation**: One process crash doesn't affect others
3. **Memory Isolation**: Each process has its own memory space
4. **CPU Utilization**: Better distribution across CPU cores
5. **Simplicity**: Single-threaded processes are easier to reason about

## Platform Support

- **Linux**: Full support with SO_REUSEPORT
- **macOS**: Full support with SO_REUSEPORT  
- **Windows**: Falls back to single process (SO_REUSEPORT not available)

## Performance Comparison

| Mode | Processes | Threads per Process | Memory Usage | CPU Usage | Complexity |
|------|-----------|-------------------|--------------|-----------|------------|
| single_threaded | 1 | 1 | Low | Single core | Low |
| multi_threaded | 1 | N (auto/config) | Medium | Multi-core | Medium |
| multi_process | 4 | 1 each | Higher | Multi-core | Low per process |

## Troubleshooting

### Port binding issues:
```bash
# Check if port is already in use
lsof -i :8080

# Verify SO_REUSEPORT support
# If you see binding errors, set use_reuseport: false
```

### Process count not matching:
```bash
# Check if processes are actually starting
ps aux | grep rust-forward-proxy | wc -l

# Should show 5 total (1 parent + 4 children)
```

### Performance not improving:
- Verify all processes are handling requests
- Check CPU core count vs process count
- Monitor individual process metrics
- Consider tuning process_count based on workload

## Monitoring

Each process logs with its index:
```
🧵 Child process 0 initializing single-threaded runtime
🧵 Child process 1 initializing single-threaded runtime
🧵 Child process 2 initializing single-threaded runtime
🧵 Child process 3 initializing single-threaded runtime
```

Use process-specific monitoring:
```bash
# Monitor specific child process
top -p $(pgrep -f "rust-forward-proxy" | head -n 1)
```
