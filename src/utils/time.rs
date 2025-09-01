//! Time utility functions

use chrono::{DateTime, Utc};
use std::time::{Duration, Instant};

/// Get current timestamp
pub fn now() -> DateTime<Utc> {
    Utc::now()
}

/// Format timestamp as RFC3339
pub fn format_timestamp(timestamp: DateTime<Utc>) -> String {
    timestamp.to_rfc3339()
}

/// Calculate duration between two instants
pub fn duration_between(start: Instant, end: Instant) -> Duration {
    end.duration_since(start)
}

/// Convert duration to milliseconds
pub fn duration_to_ms(duration: Duration) -> u64 {
    duration.as_millis() as u64
}
