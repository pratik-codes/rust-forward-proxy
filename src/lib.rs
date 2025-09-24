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
pub use logging::{init_logger, init_logger_with_env, log_info, log_error, log_debug, log_warning, log_trace};
pub use models::{ProxyLog, RequestData, ResponseData};
pub use proxy::server::ProxyServer;
pub use config::settings::ProxyConfig;

