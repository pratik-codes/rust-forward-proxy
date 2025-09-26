//! Rust Forward Proxy - A high-performance HTTP/HTTPS forward proxy server
//! 
//! This library provides a production-grade proxy server with middleware support,
//! comprehensive logging, and configuration management.

pub mod cli;
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
    
    /// Create a Tokio runtime based on the configuration
    pub fn create_runtime(config: &RuntimeConfig) -> Result<Runtime> {
        match config.mode.as_str() {
            "single_threaded" => {
                tracing::info!("🧵 Initializing single-threaded runtime");
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
        let runtime = create_runtime(config)?;
        runtime.block_on(future)
    }
}

