//! Rust Forward Proxy - A high-performance HTTP/HTTPS forward proxy server
//! 
//! This library provides a production-grade proxy server with middleware support,
//! comprehensive logging, and configuration management.

pub mod config;
pub mod error;
pub mod logging;
pub mod models;
pub mod proxy;
pub mod tls;
pub mod utils;

// Re-export commonly used items
pub use error::{Error, Result};
pub use logging::{init_logger, init_logger_with_env, init_logger_with_config, log_info, log_error, log_debug, log_warning, log_trace};
pub use models::{ProxyLog, RequestData, ResponseData};
pub use proxy::server::ProxyServer;
pub use config::settings::ProxyConfig;

/// Runtime utilities for creating single-threaded vs multi-threaded Tokio runtimes
pub mod runtime {
    use crate::config::settings::RuntimeConfig;
    use tokio::runtime::{Builder, Runtime};
    use anyhow::{Result, Context};
    use std::process::Command;
    use std::env;
    use tracing::{info, error};

    /// Multi-process launcher for running multiple single-threaded processes
    pub fn launch_multi_process(config: &RuntimeConfig) -> Result<()> {
        let process_count = config.process_count.unwrap_or(4);
        
        info!("🚀 Starting multi-process mode with {} processes", process_count);
        
        let current_exe = env::current_exe()
            .context("Failed to get current executable path")?;
        
        let mut children = Vec::new();
        let mut child_pids = Vec::new();
        
        // Set environment variables for all child processes
        env::set_var("PROXY_RUNTIME_MODE", "single_threaded");
        if config.use_reuseport {
            env::set_var("PROXY_USE_REUSEPORT", "true");
        }
        
        // Spawn child processes
        for i in 0..process_count {
            // Set process index for this child
            env::set_var("PROXY_PROCESS_INDEX", i.to_string());
            
            info!("🔄 Starting process {} of {}", i + 1, process_count);
            
            let child = Command::new(&current_exe)
                .env("PROXY_PROCESS_INDEX", i.to_string())
                .env("PROXY_RUNTIME_MODE", "single_threaded")
                .env("PROXY_USE_REUSEPORT", if config.use_reuseport { "true" } else { "false" })
                .spawn()
                .context("Failed to spawn child process")?;
            
            let child_pid = child.id();
            info!("🔧 Child process {} spawned with PID: {}", i + 1, child_pid);
            child_pids.push(child_pid);
            
            children.push(child);
        }
        
        // Log all subprocess PIDs in a summary
        let pids_string = child_pids.iter().map(|pid| pid.to_string()).collect::<Vec<_>>().join(", ");
        info!("✅ All {} processes started successfully", process_count);
        info!("🏷️  SUBPROCESS PIDs: [{}]", pids_string);
        
        // Wait for all child processes to complete
        for (i, mut child) in children.into_iter().enumerate() {
            match child.wait() {
                Ok(status) => {
                    if status.success() {
                        info!("✅ Process {} completed successfully", i);
                    } else {
                        error!("❌ Process {} exited with error: {:?}", i, status);
                    }
                }
                Err(e) => {
                    error!("❌ Failed to wait for process {}: {}", i, e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Check if running in multi-process child mode
    pub fn is_child_process() -> bool {
        env::var("PROXY_PROCESS_INDEX").is_ok()
    }
    
    /// Get the current process index (0-based)
    pub fn get_process_index() -> Option<usize> {
        env::var("PROXY_PROCESS_INDEX")
            .ok()
            .and_then(|s| s.parse().ok())
    }
    
    /// Create a Tokio runtime based on the configuration
    pub fn create_runtime(config: &RuntimeConfig) -> Result<Runtime> {
        match config.mode.as_str() {
            "single_threaded" => {
                let process_info = if let Some(index) = get_process_index() {
                    format!(" (process {})", index)
                } else {
                    String::new()
                };
                tracing::info!("🧵 Initializing single-threaded runtime{}", process_info);
                Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .context("Failed to create single-threaded runtime")
            }
            "multi_threaded" => {
                let mut builder = Builder::new_multi_thread();
                builder.enable_all();
                
                if let Some(threads) = config.worker_threads {
                    if threads > 0 {
                        tracing::info!("🧵 Initializing multi-threaded runtime with {} worker threads", threads);
                        builder.worker_threads(threads);
                    } else {
                        tracing::info!("🧵 Initializing multi-threaded runtime with auto-detected CPU cores");
                    }
                } else {
                    tracing::info!("🧵 Initializing multi-threaded runtime with auto-detected CPU cores");
                }
                
                builder.build()
                    .context("Failed to create multi-threaded runtime")
            }
            "multi_process" => {
                // In multi-process mode, child processes should use single-threaded runtime
                if is_child_process() {
                    let process_index = get_process_index().unwrap_or(0);
                    tracing::info!("🧵 Child process {} initializing single-threaded runtime", process_index);
                    Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .context("Failed to create single-threaded runtime for child process")
                } else {
                    // Parent process - this shouldn't happen in normal flow
                    tracing::warn!("⚠️  Multi-process mode requested but not in child process");
                    Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .context("Failed to create runtime")
                }
            }
            _ => {
                tracing::warn!("⚠️  Unknown runtime mode '{}', defaulting to multi-threaded", config.mode);
                Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .context("Failed to create default multi-threaded runtime")
            }
        }
    }
    
    /// Execute an async function with the configured runtime
    pub fn run_with_runtime<F, T>(config: &RuntimeConfig, future: F) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>>,
    {
        // Handle multi-process mode
        if config.mode == "multi_process" && !is_child_process() {
            // We're the parent process - launch child processes instead
            launch_multi_process(config)?;
            // Parent process completes after all children finish
            return Ok(unsafe { std::mem::zeroed() });
        }
        
        let runtime = create_runtime(config)?;
        runtime.block_on(future)
    }
}

