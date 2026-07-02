//! FFprobe and transcode configuration.

use nest_error::NestResult;

/// Default FFprobe binary name.
pub const DEFAULT_FFPROBE_PATH: &str = "ffprobe";

/// Default environment variable for FFprobe path override.
pub const DEFAULT_FFPROBE_PATH_ENV: &str = "FFPROBE_PATH";

/// Default probe timeout in seconds.
pub const DEFAULT_TIMEOUT_SECONDS: u32 = 60;

/// Resolved transcode / probe configuration.
#[derive(Debug, Clone)]
pub struct TranscodeConfig {
    /// Path to the ffprobe binary.
    pub ffprobe_path: String,
    /// Optional ffmpeg binary path (reserved for v0.2 transcode jobs).
    pub ffmpeg_path: Option<String>,
    /// Maximum seconds to wait for ffprobe.
    pub timeout_seconds: u32,
    /// Extra arguments appended before the media file path.
    pub extra_ffprobe_args: Vec<String>,
}

impl TranscodeConfig {
    /// Creates configuration from environment (`FFPROBE_PATH` optional).
    pub fn from_env() -> NestResult<Self> {
        Self::builder().build()
    }

    /// Returns a configuration builder.
    pub fn builder() -> TranscodeConfigBuilder {
        TranscodeConfigBuilder::new()
    }
}

/// Programmatic configuration builder.
#[derive(Debug, Clone)]
pub struct TranscodeConfigBuilder {
    ffprobe_path: Option<String>,
    ffmpeg_path: Option<String>,
    timeout_seconds: u32,
    extra_ffprobe_args: Vec<String>,
}

impl TranscodeConfigBuilder {
    /// Creates a builder with defaults.
    pub fn new() -> Self {
        Self {
            ffprobe_path: None,
            ffmpeg_path: None,
            timeout_seconds: DEFAULT_TIMEOUT_SECONDS,
            extra_ffprobe_args: Vec::new(),
        }
    }

    /// Sets the ffprobe binary path.
    pub fn ffprobe_path(mut self, path: impl Into<String>) -> Self {
        self.ffprobe_path = Some(path.into());
        self
    }

    /// Sets the ffmpeg binary path (unused in v0.1).
    pub fn ffmpeg_path(mut self, path: impl Into<String>) -> Self {
        self.ffmpeg_path = Some(path.into());
        self
    }

    /// Sets the probe timeout in seconds.
    pub fn timeout_seconds(mut self, seconds: u32) -> Self {
        self.timeout_seconds = seconds;
        self
    }

    /// Adds extra ffprobe arguments.
    pub fn extra_ffprobe_arg(mut self, arg: impl Into<String>) -> Self {
        self.extra_ffprobe_args.push(arg.into());
        self
    }

    /// Builds the resolved configuration.
    pub fn build(self) -> NestResult<TranscodeConfig> {
        let ffprobe_path = resolve_ffprobe_path(self.ffprobe_path.as_deref())?;
        Ok(TranscodeConfig {
            ffprobe_path,
            ffmpeg_path: self.ffmpeg_path,
            timeout_seconds: self.timeout_seconds,
            extra_ffprobe_args: self.extra_ffprobe_args,
        })
    }
}

impl Default for TranscodeConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Resolves the ffprobe binary path from explicit config or environment.
pub fn resolve_ffprobe_path(path: Option<&str>) -> NestResult<String> {
    if let Some(path) = path.map(str::trim).filter(|value| !value.is_empty()) {
        return Ok(path.to_string());
    }

    if let Ok(path) = std::env::var(DEFAULT_FFPROBE_PATH_ENV) {
        let path = path.trim();
        if !path.is_empty() {
            return Ok(path.to_string());
        }
    }

    Ok(DEFAULT_FFPROBE_PATH.to_string())
}

#[cfg(feature = "config")]
mod config_service {
    use nest_config::{ConfigDocument, ConfigService};
    use nest_error::NestResult;
    use serde::Deserialize;

    use super::{resolve_ffprobe_path, TranscodeConfig, DEFAULT_TIMEOUT_SECONDS};

    #[derive(Debug, Clone, Deserialize)]
    pub(crate) struct TranscodeSection {
        #[serde(default)]
        ffprobe_path: Option<String>,
        #[serde(default)]
        ffmpeg_path: Option<String>,
        #[serde(default)]
        timeout_seconds: Option<u32>,
        #[serde(default)]
        extra_ffprobe_args: Vec<String>,
    }

    impl TranscodeConfig {
        /// Loads configuration from a [`ConfigDocument`].
        pub fn from_document(document: &ConfigDocument) -> NestResult<Self> {
            let section: TranscodeSection = document.section("transcode")?;
            Ok(Self {
                ffprobe_path: resolve_ffprobe_path(section.ffprobe_path.as_deref())?,
                ffmpeg_path: section.ffmpeg_path,
                timeout_seconds: section.timeout_seconds.unwrap_or(DEFAULT_TIMEOUT_SECONDS),
                extra_ffprobe_args: section.extra_ffprobe_args,
            })
        }

        /// Loads configuration from a [`ConfigService`].
        pub fn from_config_service(config: &ConfigService) -> NestResult<Self> {
            Self::from_document(config.document())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_uses_explicit_ffprobe_path() {
        let config = TranscodeConfig::builder()
            .ffprobe_path("/usr/bin/ffprobe")
            .timeout_seconds(120)
            .build()
            .unwrap();
        assert_eq!(config.ffprobe_path, "/usr/bin/ffprobe");
        assert_eq!(config.timeout_seconds, 120);
    }
}
