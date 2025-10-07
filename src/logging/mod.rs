use crate::models::ProxyLog;
use anyhow::Result;
use chrono::Utc;
use log::{debug, error, info, trace, warn, LevelFilter};
use serde_json;
use std::fs;
use std::path::Path;
use std::sync::Once;
use tracing::Level;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt, Registry};
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::{FormatEvent, FormatFields};
use tracing_appender::{rolling, non_blocking};
use tracing_log::LogTracer;
use std::fmt;
use tracing::{Event, Subscriber};
use tracing_subscriber::registry::LookupSpan;

static INIT: Once = Once::new();

/// Custom formatter for detailed logging with PID
pub struct DetailedFormatter;

impl<S, N> FormatEvent<S, N> for DetailedFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &tracing_subscriber::fmt::FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        // Get the current timestamp
        let now = chrono::Utc::now();
        let timestamp = now.format("%Y-%m-%d %H:%M:%S%.3f UTC");
        
        // Get the log level
        let level = event.metadata().level();
        
        // Get the process ID
        let process_id = std::process::id();
        
        // Get the thread ID  
        let thread_id = format!("{:?}", std::thread::current().id())
            .replace("ThreadId(", "").replace(")", "");
        
        // Get the file name (without path)
        let file = event.metadata().file().unwrap_or("unknown");
        let file_name = std::path::Path::new(file)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");
        
        // Get the line number
        let line = event.metadata().line().unwrap_or(0);
        
        // Get the target (module path - we'll use this as function context)
        let target = event.metadata().target();
        let function_name = target.split("::").last().unwrap_or("unknown");
        
        // Format: LEVEL TIMESTAMP PID:X TID:Y FILENAME:LINE FUNCTION content
        write!(
            writer,
            "{} {} PID:{} TID:{} {}:{} {} ",
            level,
            timestamp,
            process_id,
            thread_id,
            file_name,
            line,
            function_name
        )?;
        
        // Format the event fields (the actual log message)
        ctx.field_format().format_fields(writer.by_ref(), event)?;
        
        writeln!(writer)
    }
}

/// Custom file formatter (without colors for file output)
pub struct FileFormatter;

impl<S, N> FormatEvent<S, N> for FileFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &tracing_subscriber::fmt::FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        // Get the current timestamp
        let now = chrono::Utc::now();
        let timestamp = now.format("%Y-%m-%d %H:%M:%S%.3f UTC");
        
        // Get the log level
        let level = event.metadata().level();
        
        // Get the process ID
        let process_id = std::process::id();
        
        // Get the thread ID  
        let thread_id = format!("{:?}", std::thread::current().id())
            .replace("ThreadId(", "").replace(")", "");
        
        // Get the file name (without path)
        let file = event.metadata().file().unwrap_or("unknown");
        let file_name = std::path::Path::new(file)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");
        
        // Get the line number
        let line = event.metadata().line().unwrap_or(0);
        
        // Get the target (module path - we'll use this as function context)
        let target = event.metadata().target();
        let function_name = target.split("::").last().unwrap_or("unknown");
        
        // Format: LEVEL TIMESTAMP PID:X TID:Y FILENAME:LINE FUNCTION content
        write!(
            writer,
            "{} {} PID:{} TID:{} {}:{} {} ",
            level,
            timestamp,
            process_id,
            thread_id,
            file_name,
            line,
            function_name
        )?;
        
        // Format the event fields (the actual log message)
        ctx.field_format().format_fields(writer.by_ref(), event)?;
        
        writeln!(writer)
    }
}

/// Create a process information prefix for enhanced logging (backwards compatibility)
pub fn process_info() -> String {
    let process_id = std::process::id();
    let thread_id = format!("{:?}", std::thread::current().id())
        .replace("ThreadId(", "").replace(")", "");
    format!("PID:{} TID:{}", process_id, thread_id)
}

/// Ensure the logs directory exists
fn ensure_logs_directory() -> Result<()> {
    let logs_dir = Path::new("logs");
    if !logs_dir.exists() {
        fs::create_dir_all(logs_dir)?;
        println!("📁 Created logs directory: {}", logs_dir.display());
    }
    Ok(())
}

/// Initialize the global logger with production-grade configuration
/// This should be called once at the start of the application
pub fn init_logger() {
    INIT.call_once(|| {
        // Ensure logs directory exists
        if let Err(e) = ensure_logs_directory() {
            eprintln!("Warning: Failed to create logs directory: {:?}", e);
        }

        // Create file appender for daily rolling logs
        let file_appender = rolling::never("logs", "proxy.log");
        let (non_blocking_file, _guard) = non_blocking(file_appender);

        // Create console layer with custom detailed formatter (includes PID)
        let console_layer = tracing_subscriber::fmt::layer()
            .event_format(DetailedFormatter);

        // Create file layer with custom file formatter (includes PID, no colors)
        let file_layer = tracing_subscriber::fmt::layer()
            .event_format(FileFormatter)
            .with_writer(non_blocking_file);

        // Initialize the subscriber with both console and file output
        tracing_subscriber::registry()
            .with(EnvFilter::from_default_env())
            .with(console_layer)
            .with(file_layer)
            .init();

        // Initialize LogTracer to bridge log events to tracing (after subscriber is set up)
        if let Err(e) = LogTracer::init() {
            eprintln!("Warning: Failed to initialize LogTracer: {:?}", e);
        }

        // Set the log crate's max level to match tracing
        log::set_max_level(LevelFilter::Debug);

        // Log that file logging is enabled
        info!("📁 Logging initialized - Console + File (logs/proxy.log)");
        std::mem::forget(_guard); // Keep the guard alive for the lifetime of the program
    });
}

/// Initialize logger with custom log level
pub fn init_logger_with_level(level: Level) {
    INIT.call_once(|| {
        // Ensure logs directory exists
        if let Err(e) = ensure_logs_directory() {
            eprintln!("Warning: Failed to create logs directory: {:?}", e);
        }

        // Create file appender for daily rolling logs
        let file_appender = rolling::never("logs", "proxy.log");
        let (non_blocking_file, _guard) = non_blocking(file_appender);

        // Create console layer with custom detailed formatter (includes PID)
        let console_layer = tracing_subscriber::fmt::layer()
            .event_format(DetailedFormatter);

        // Create file layer with custom file formatter (includes PID, no colors)
        let file_layer = tracing_subscriber::fmt::layer()
            .event_format(FileFormatter)
            .with_writer(non_blocking_file);

        // Create an EnvFilter from the level
        let filter = match level {
            Level::ERROR => EnvFilter::new("error"),
            Level::WARN => EnvFilter::new("warn"),
            Level::INFO => EnvFilter::new("info"),
            Level::DEBUG => EnvFilter::new("debug"),
            Level::TRACE => EnvFilter::new("trace"),
        };

        // Initialize the subscriber with both console and file output and custom level
        tracing_subscriber::registry()
            .with(filter)
            .with(console_layer)
            .with(file_layer)
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

        // Log that file logging is enabled
        info!("📁 Logging initialized with level {:?} - Console + File (logs/proxy.log)", level);
        std::mem::forget(_guard); // Keep the guard alive for the lifetime of the program
    });
}

/// Initialize logger with configuration parameters
/// This is the recommended way to initialize logging with explicit configuration
pub fn init_logger_with_config(log_level: &str, enable_file_logging: bool) {
    INIT.call_once(|| {
        // Parse log level from string
        let level = log_level
            .parse::<LevelFilter>()
            .unwrap_or(LevelFilter::Info);
        
        log::set_max_level(level);

        // Create console layer with custom detailed formatter (includes PID)
        let console_layer = tracing_subscriber::fmt::layer()
            .event_format(DetailedFormatter);

        if enable_file_logging {
            // Ensure logs directory exists only if file logging is enabled
            if let Err(e) = ensure_logs_directory() {
                eprintln!("Warning: Failed to create logs directory: {:?}", e);
            }

            // Create file appender for daily rolling logs
            let file_appender = rolling::never("logs", "proxy.log");
            let (non_blocking_file, _guard) = non_blocking(file_appender);

            // Create file layer with custom file formatter (includes PID, no colors)
            let file_layer = tracing_subscriber::fmt::layer()
                .event_format(FileFormatter)
                .with_writer(non_blocking_file);

            // Initialize subscriber with both console and file layers
            let subscriber = Registry::default()
                .with(EnvFilter::new(log_level))
                .with(console_layer)
                .with(file_layer);

            if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
                eprintln!("Failed to set global subscriber: {}", e);
            }

            // Initialize log-to-tracing bridge
            LogTracer::init().expect("Failed to set logger");

            // Store the guard to prevent early cleanup
            Box::leak(Box::new(_guard));
        } else {
            // Initialize subscriber with console layer only
            let subscriber = Registry::default()
                .with(EnvFilter::new(log_level))
                .with(console_layer);

            if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
                eprintln!("Failed to set global subscriber: {}", e);
            }

            // Initialize log-to-tracing bridge
            LogTracer::init().expect("Failed to set logger");
        }
    });
}

/// Initialize logger with environment variable support and optional file logging
/// Uses RUST_LOG environment variable for configuration
/// Uses PROXY_ENABLE_FILE_LOGGING environment variable to control file logging (default: true)
/// DEPRECATED: Use init_logger_with_config instead for better configuration management
pub fn init_logger_with_env() {
    INIT.call_once(|| {
        // Check if file logging is enabled via environment variable
        let enable_file_logging = std::env::var("PROXY_ENABLE_FILE_LOGGING")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);

        // Set log level based on environment first
        let level = std::env::var("RUST_LOG")
            .unwrap_or_else(|_| "info".to_string())
            .parse::<LevelFilter>()
            .unwrap_or(LevelFilter::Info);
        
        log::set_max_level(level);

        // Create console layer with custom detailed formatter (includes PID)
        let console_layer = tracing_subscriber::fmt::layer()
            .event_format(DetailedFormatter);

        if enable_file_logging {
            // Ensure logs directory exists only if file logging is enabled
            if let Err(e) = ensure_logs_directory() {
                eprintln!("Warning: Failed to create logs directory: {:?}", e);
            }

            // Create file appender for daily rolling logs
            let file_appender = rolling::never("logs", "proxy.log");
            let (non_blocking_file, _guard) = non_blocking(file_appender);

            // Create file layer with custom file formatter (includes PID, no colors)
            let file_layer = tracing_subscriber::fmt::layer()
                .event_format(FileFormatter)
                .with_writer(non_blocking_file);

            // Initialize the subscriber with both console and file output
            tracing_subscriber::registry()
                .with(EnvFilter::from_default_env())
                .with(console_layer)
                .with(file_layer)
                .init();

            // Initialize LogTracer to bridge log events to tracing (after subscriber is set up)
            if let Err(e) = LogTracer::init() {
                eprintln!("Warning: Failed to initialize LogTracer: {:?}", e);
            }

            // Log that both console and file logging are enabled
            info!("📁 Logging initialized with env config - Console + File (logs/proxy.log)");
            std::mem::forget(_guard); // Keep the guard alive for the lifetime of the program
        } else {
            // Initialize the subscriber with console output only
            tracing_subscriber::registry()
                .with(EnvFilter::from_default_env())
                .with(console_layer)
                .init();

            // Initialize LogTracer to bridge log events to tracing (after subscriber is set up)
            if let Err(e) = LogTracer::init() {
                eprintln!("Warning: Failed to initialize LogTracer: {:?}", e);
            }

            // Log that only console logging is enabled
            info!("📺 Logging initialized with env config - Console Only (file logging disabled)");
        }
    });
}

/// Initialize logger with console output only (no file logging)
/// Perfect for production environments where you want to avoid file I/O overhead
pub fn init_console_only_logger() {
    INIT.call_once(|| {
        // Set log level based on environment
        let level = std::env::var("RUST_LOG")
            .unwrap_or_else(|_| "info".to_string())
            .parse::<LevelFilter>()
            .unwrap_or(LevelFilter::Info);
        
        log::set_max_level(level);

        // Create console layer with custom detailed formatter (includes PID)
        let console_layer = tracing_subscriber::fmt::layer()
            .event_format(DetailedFormatter);

        // Initialize the subscriber with console output only
        tracing_subscriber::registry()
            .with(EnvFilter::from_default_env())
            .with(console_layer)
            .init();

        // Initialize LogTracer to bridge log events to tracing (after subscriber is set up)
        if let Err(e) = LogTracer::init() {
            eprintln!("Warning: Failed to initialize LogTracer: {:?}", e);
        }

        // Log that only console logging is enabled
        info!("🚀 High-performance console-only logging initialized (no file I/O overhead)");
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

