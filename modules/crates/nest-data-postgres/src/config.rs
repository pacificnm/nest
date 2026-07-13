//! PostgreSQL connection configuration.

use std::env;

use nest_data::{DataError, DataResult};

/// Default maximum pool connections.
pub const DEFAULT_MAX_CONNECTIONS: u32 = 5;

/// Default number of connection attempts before giving up.
pub const DEFAULT_CONNECT_RETRIES: u32 = 5;
/// Default initial backoff delay between connection attempts.
pub const DEFAULT_CONNECT_BACKOFF_MS: u64 = 200;
/// Default maximum backoff delay (backoff doubles each attempt, capped here).
pub const DEFAULT_CONNECT_BACKOFF_MAX_MS: u64 = 5_000;

/// PostgreSQL connection options.
#[derive(Debug, Clone)]
pub struct PostgresConfig {
    /// Connection URL (`postgresql://...`).
    pub url: String,
    /// Maximum pool size.
    pub max_connections: u32,
    /// Number of connection attempts before giving up.
    pub connect_retries: u32,
    /// Initial backoff delay between connection attempts, in milliseconds.
    pub connect_backoff_ms: u64,
    /// Maximum backoff delay between connection attempts, in milliseconds.
    pub connect_backoff_max_ms: u64,
}

impl PostgresConfig {
    /// Creates config from a database URL.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            max_connections: DEFAULT_MAX_CONNECTIONS,
            connect_retries: DEFAULT_CONNECT_RETRIES,
            connect_backoff_ms: DEFAULT_CONNECT_BACKOFF_MS,
            connect_backoff_max_ms: DEFAULT_CONNECT_BACKOFF_MAX_MS,
        }
    }

    /// Reads a URL from an environment variable.
    pub fn from_env(var: &str) -> DataResult<Self> {
        let url = env::var(var).map_err(|_| {
            DataError::config(format!("environment variable `{var}` is not set"))
        })?;
        Ok(Self::new(url))
    }

    /// Sets the maximum pool size.
    pub fn with_max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }

    /// Sets the number of connection attempts before giving up.
    pub fn with_connect_retries(mut self, retries: u32) -> Self {
        self.connect_retries = retries;
        self
    }

    /// Sets the initial and maximum backoff delay between connection attempts.
    pub fn with_connect_backoff(mut self, initial_ms: u64, max_ms: u64) -> Self {
        self.connect_backoff_ms = initial_ms;
        self.connect_backoff_max_ms = max_ms;
        self
    }

    /// Returns the datasource string for connection metadata.
    pub fn datasource(&self) -> &str {
        &self.url
    }
}
