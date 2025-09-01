use crate::models::ProxyLog;
use anyhow::Result;
use chrono::Utc;
use log::{debug, error, info, trace, warn, LevelFilter};
use serde_json;
use std::sync::Once;
use tracing::Level;
use tracing_subscriber::{EnvFilter, FmtSubscriber};
use tracing_log::LogTracer;

static INIT: Once = Once::new();

/// Initialize the global logger with production-grade configuration
/// This should be called once at the start of the application
pub fn init_logger() {
    INIT.call_once(|| {
        // Set up tracing subscriber with console output
        FmtSubscriber::builder()
            .with_env_filter(EnvFilter::from_default_env())
            .with_target(false)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true)
            .with_level(true)
            .with_ansi(true)
            .pretty()
            .init();

        // Initialize LogTracer to bridge log events to tracing (after subscriber is set up)
        if let Err(e) = LogTracer::init() {
            eprintln!("Warning: Failed to initialize LogTracer: {:?}", e);
        }

        // Set the log crate's max level to match tracing
        log::set_max_level(LevelFilter::Debug);
    });
}

/// Initialize logger with custom log level
pub fn init_logger_with_level(level: Level) {
    INIT.call_once(|| {
        FmtSubscriber::builder()
            .with_max_level(level)
            .with_target(false)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true)
            .with_level(true)
            .with_ansi(true)
            .pretty()
            .init();

        // Initialize LogTracer to bridge log events to tracing (after subscriber is set up)
        if let Err(e) = LogTracer::init() {
            eprintln!("Warning: Failed to initialize LogTracer: {:?}", e);
        }

        log::set_max_level(match level {
            Level::ERROR => LevelFilter::Error,
            Level::WARN => LevelFilter::Warn,
            Level::INFO => LevelFilter::Info,
            Level::DEBUG => LevelFilter::Debug,
            Level::TRACE => LevelFilter::Trace,
        });
    });
}

/// Initialize logger with environment variable support
/// Uses RUST_LOG environment variable for configuration
pub fn init_logger_with_env() {
    INIT.call_once(|| {
        // Set log level based on environment first
        let level = std::env::var("RUST_LOG")
            .unwrap_or_else(|_| "info".to_string())
            .parse::<LevelFilter>()
            .unwrap_or(LevelFilter::Info);
        
        log::set_max_level(level);

        // Set up tracing subscriber with console output
        FmtSubscriber::builder()
            .with_env_filter(EnvFilter::from_default_env())
            .with_target(false)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true)
            .with_level(true)
            .with_ansi(true)
            .pretty()
            .init();

        // Initialize LogTracer to bridge log events to tracing (after subscriber is set up)
        if let Err(e) = LogTracer::init() {
            eprintln!("Warning: Failed to initialize LogTracer: {:?}", e);
        }
    });
}

/// Log a proxy transaction using log (bridged to tracing via tracing-log)
pub fn log_transaction(log_entry: &ProxyLog) -> Result<()> {
    let timestamp = Utc::now().to_rfc3339();
    let log_message = serde_json::to_string_pretty(log_entry)?;
    let formatted_message = format!("[{}] TRANSACTION:\n{}", timestamp, log_message);
    
    // Log using debug level so it only appears in debug mode
    debug!("{}", formatted_message);
    
    Ok(())
}

/// Log an error message
pub fn log_error(message: &str) {
    error!("{}", message);
}

/// Log an info message
pub fn log_info(message: &str) {
    info!("{}", message);
}

/// Log a warning message
pub fn log_warning(message: &str) {
    warn!("{}", message);
}

/// Log a debug message
pub fn log_debug(message: &str) {
    debug!("{}", message);
}

/// Log a trace message
pub fn log_trace(message: &str) {
    trace!("{}", message);
}

/// Log a formatted message with custom level
pub fn log_with_level(level: log::Level, message: &str) {
    match level {
        log::Level::Error => log_error(message),
        log::Level::Warn => log_warning(message),
        log::Level::Info => log_info(message),
        log::Level::Debug => log_debug(message),
        log::Level::Trace => log_trace(message),
    }
}

/// Log a formatted message with tracing level
pub fn log_with_tracing_level(level: Level, message: &str) {
    match level {
        Level::ERROR => log_error(message),
        Level::WARN => log_warning(message),
        Level::INFO => log_info(message),
        Level::DEBUG => log_debug(message),
        Level::TRACE => log_trace(message),
    }
}

/// Convenience macro for logging proxy transactions
#[macro_export]
macro_rules! log_proxy_transaction {
    ($log_entry:expr) => {
        if let Err(e) = $crate::logging::log_transaction($log_entry) {
            eprintln!("Failed to log transaction: {}", e);
        }
    };
}

/// Convenience macro for logging errors
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::logging::log_error(&format!($($arg)*));
    };
}

/// Convenience macro for logging info messages
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::logging::log_info(&format!($($arg)*));
    };
}

/// Convenience macro for logging warning messages
#[macro_export]
macro_rules! log_warning {
    ($($arg:tt)*) => {
        $crate::logging::log_warning(&format!($($arg)*));
    };
}

/// Convenience macro for logging debug messages
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::logging::log_debug(&format!($($arg)*));
    };
}

/// Convenience macro for logging trace messages
#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)*) => {
        $crate::logging::log_trace(&format!($($arg)*));
    };
}

/// Legacy compatibility struct for existing code
/// This provides a drop-in replacement for the old logger interface
#[derive(Clone)]
pub struct SharedLogger;

impl SharedLogger {
    /// Create a new shared logger (no-op for compatibility)
    pub fn new(_logger: ProxyLogger) -> Self {
        Self
    }

    /// Log a transaction (thread-safe)
    pub async fn log_transaction(&self, log_entry: &ProxyLog) -> Result<()> {
        log_transaction(log_entry)
    }

    /// Log an error message (thread-safe)
    pub async fn log_error(&self, error: &str) -> Result<()> {
        log_error(error);
        Ok(())
    }

    /// Log an info message (thread-safe)
    pub async fn log_info(&self, message: &str) -> Result<()> {
        log_info(message);
        Ok(())
    }

    /// Log a warning message (thread-safe)
    pub async fn log_warning(&self, message: &str) -> Result<()> {
        log_warning(message);
        Ok(())
    }

    /// Log a debug message (thread-safe)
    pub async fn log_debug(&self, message: &str) -> Result<()> {
        log_debug(message);
        Ok(())
    }

    /// Get the log file path (returns empty string for console-only logging)
    pub fn get_log_file(&self) -> &str {
        ""
    }
}

/// Legacy compatibility struct for existing code
pub struct ProxyLogger;

impl ProxyLogger {
    /// Create a new logger instance (no-op for compatibility)
    pub fn new(_log_file: String, _enable_console: bool, _enable_file: bool) -> Result<Self> {
        Ok(Self)
    }

    /// Create a logger with default settings (console-only)
    pub fn default() -> Result<Self> {
        Ok(Self)
    }

    /// Create a console-only logger
    pub fn console_only() -> Self {
        Self
    }

    /// Create a file-only logger (now console-only for compatibility)
    pub fn file_only(_log_file: String) -> Result<Self> {
        Ok(Self)
    }

    /// Get the log file path (returns empty string for console-only logging)
    pub fn get_log_file(&self) -> &str {
        ""
    }

    /// Check if console logging is enabled (always true)
    pub fn is_console_enabled(&self) -> bool {
        true
    }

    /// Check if file logging is enabled (always false)
    pub fn is_file_enabled(&self) -> bool {
        false
    }
}
