//! Upstream HTTP client

use crate::error::Result;
use hyper::{Client, Request, Response, Body};
use std::time::Duration;

/// HTTP client for upstream servers
pub struct UpstreamClient {
    client: Client<hyper::client::HttpConnector>,
    timeout: Duration,
}

impl UpstreamClient {
    pub fn new(timeout: Duration) -> Self {
        let client = Client::builder()
            .pool_idle_timeout(Duration::from_secs(30))
            .build(hyper::client::HttpConnector::new());
            
        Self { client, timeout }
    }
    
    pub async fn request(&self, req: Request<Body>) -> Result<Response<Body>> {
        let response = tokio::time::timeout(self.timeout, self.client.request(req)).await??;
        Ok(response)
    }
}
