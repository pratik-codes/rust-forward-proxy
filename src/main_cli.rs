//! CLI version of the Rust Forward Proxy with certificate management

use clap::{Parser, Subcommand};
use rust_forward_proxy::{
    cli::{CertCommand, ServerArgs},
    init_logger_with_env,
    log_info, log_error,
    ProxyConfig,
    runtime::run_with_runtime,
};
use tracing::error;

#[derive(Parser)]
#[command(name = "rust-forward-proxy")]
#[command(about = "A high-performance HTTP/HTTPS forward proxy with TLS interception")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(author = "Rust Forward Proxy Team")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
    
    /// Log level (trace, debug, info, warn, error)
    #[arg(long, global = true, default_value = "info")]
    log_level: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the proxy server
    #[command(name = "server")]
    Server(ServerArgs),
    
    /// Certificate management commands
    #[command(name = "cert")]
    #[command(subcommand)]
    Cert(CertCommand),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    // Initialize logging based on CLI arguments
    unsafe {
        if cli.verbose {
            std::env::set_var("RUST_LOG", "debug");
        } else {
            std::env::set_var("RUST_LOG", &cli.log_level);
        }
    }
    
    init_logger_with_env();
    
    // Try to load config from YAML file first, or use CLI defaults
    let config = ProxyConfig::load_config().unwrap_or_else(|_| {
        log_info!("⚠️  No config.yml found, using CLI defaults");
        ProxyConfig::default()
    });
    
    // Clone the runtime config and run the async main function
    let runtime_config = config.runtime.clone();
    run_with_runtime(&runtime_config, async_main_cli(cli, config))
}

async fn async_main_cli(cli: Cli, _config: ProxyConfig) -> anyhow::Result<()> {
    // Handle commands
    match cli.command {
        Some(Commands::Server(args)) => {
            log_info!("🚀 Starting Rust Forward Proxy Server");
            log_info!("📋 Version: {}", env!("CARGO_PKG_VERSION"));
            
            if let Err(e) = args.start_server().await {
                log_error!("Server error: {}", e);
                error!("Failed to start server: {}", e);
                std::process::exit(1);
            }
        }
        Some(Commands::Cert(cert_cmd)) => {
            log_info!("🔧 Certificate Management");
            
            if let Err(e) = cert_cmd.execute().await {
                log_error!("Certificate command error: {}", e);
                error!("Certificate operation failed: {}", e);
                std::process::exit(1);
            }
        }
        None => {
            // Default action: start server with default configuration
            log_info!("🚀 Starting Rust Forward Proxy Server (default configuration)");
            log_info!("📋 Version: {}", env!("CARGO_PKG_VERSION"));
            log_info!("💡 Use --help to see available commands");
            
            let default_args = ServerArgs {
                listen_addr: "127.0.0.1:8080".to_string(),
                https_listen_addr: "127.0.0.1:8443".to_string(),
                enable_tls: true, // Disabled by default
                enable_interception: true,
                auto_generate_cert: true,
                cert_path: "certs/proxy.crt".to_string(),
                key_path: "certs/proxy.key".to_string(),
                skip_cert_verify: false,
                request_timeout: 30,
                max_body_size: 1048576,
                log_level: cli.log_level,
            };
            
            if let Err(e) = default_args.start_server().await {
                log_error!("Server error: {}", e);
                error!("Failed to start server: {}", e);
                std::process::exit(1);
            }
        }
    }
    
    Ok(())
}
