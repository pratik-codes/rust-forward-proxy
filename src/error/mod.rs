//! Error handling module for the proxy server

use thiserror::Error;
use tokio::time::error::Elapsed;

/// Custom error type for the proxy server
#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] hyper::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("URL parsing error: {0}")]
    UrlParse(#[from] url::ParseError),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Upstream connection error: {0}")]
    UpstreamConnection(String),
    
    #[error("Request processing error: {0}")]
    RequestProcessing(String),
    
    #[error("Response processing error: {0}")]
    ResponseProcessing(String),
    
    #[error("Logging error: {0}")]
    Logging(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
    
    #[error("Timeout error: {0}")]
    Timeout(#[from] Elapsed),
}

/// Result type for the proxy server
pub type Result<T> = std::result::Result<T, Error>;

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Unknown(err.to_string())
    }
}
