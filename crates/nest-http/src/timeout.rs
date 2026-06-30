//! Timeout configuration.

use std::time::Duration;

/// Connect and request timeout settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeoutConfig {
    /// Maximum time to establish a connection.
    pub connect: Duration,
    /// Maximum time for the full request (including body).
    pub request: Duration,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            connect: Duration::from_secs(10),
            request: Duration::from_secs(30),
        }
    }
}

impl TimeoutConfig {
    /// Creates a timeout config with explicit durations.
    pub fn new(connect: Duration, request: Duration) -> Self {
        Self { connect, request }
    }
}
