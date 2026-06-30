//! Application identity and environment metadata.

/// Deployment environment for a Nest application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppEnvironment {
    /// Local development.
    #[default]
    Development,
    /// Production deployment.
    Production,
    /// Automated test runs.
    Test,
}

impl AppEnvironment {
    /// Returns a stable lowercase label for logging.
    pub fn label(self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Production => "production",
            Self::Test => "test",
        }
    }
}

/// Metadata describing a Nest application container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppMetadata {
    /// Application name (non-empty).
    pub name: String,
    /// Optional semantic version string.
    pub version: Option<String>,
    /// Deployment environment.
    pub environment: AppEnvironment,
}

impl AppMetadata {
    /// Creates metadata with the given application name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: None,
            environment: AppEnvironment::default(),
        }
    }

    /// Sets the application version.
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Sets the deployment environment.
    pub fn with_environment(mut self, environment: AppEnvironment) -> Self {
        self.environment = environment;
        self
    }
}
