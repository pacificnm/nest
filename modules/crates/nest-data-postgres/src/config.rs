//! PostgreSQL connection configuration.

use std::env;

use nest_data::{DataError, DataResult};

/// Default maximum pool connections.
pub const DEFAULT_MAX_CONNECTIONS: u32 = 5;

/// PostgreSQL connection options.
#[derive(Debug, Clone)]
pub struct PostgresConfig {
    /// Connection URL (`postgresql://...`).
    pub url: String,
    /// Maximum pool size.
    pub max_connections: u32,
}

impl PostgresConfig {
    /// Creates config from a database URL.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            max_connections: DEFAULT_MAX_CONNECTIONS,
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

    /// Returns the datasource string for connection metadata.
    pub fn datasource(&self) -> &str {
        &self.url
    }
}
