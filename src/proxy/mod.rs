//! Proxy server module

// Legacy implementations
pub mod server;
pub mod http_client;
pub mod streaming;
pub mod pingora_server;

// New pluggable architecture
pub mod core;
pub mod pingora_impl;
pub mod hyper_impl;
pub mod reqwest_impl;

// Re-exports
pub use server::ProxyServer;
pub use pingora_server::ForwardProxy;
pub use core::{HttpProxyCore, ProxyManager, ProxyFactory, ProxyImplementation, ProxyImplConfig};
