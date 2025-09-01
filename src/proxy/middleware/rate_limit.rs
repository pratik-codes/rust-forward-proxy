//! Rate limiting middleware for the proxy server

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use crate::error::Result;

/// Rate limiting middleware
pub struct RateLimitMiddleware {
    requests: Mutex<HashMap<String, Vec<Instant>>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimitMiddleware {
    pub fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            requests: Mutex::new(HashMap::new()),
            max_requests,
            window,
        }
    }
    
    pub fn check_rate_limit(&self, client_id: &str) -> Result<bool> {
        let mut requests = self.requests.lock().unwrap();
        let now = Instant::now();
        
        // Clean old requests
        let window_start = now - self.window;
        if let Some(client_requests) = requests.get_mut(client_id) {
            client_requests.retain(|&time| time > window_start);
        }
        
        // Check if limit exceeded
        let current_requests = requests
            .get(client_id)
            .map(|reqs| reqs.len())
            .unwrap_or(0);
            
        if current_requests >= self.max_requests {
            return Ok(false);
        }
        
        // Add current request
        requests
            .entry(client_id.to_string())
            .or_insert_with(Vec::new)
            .push(now);
            
        Ok(true)
    }
}
