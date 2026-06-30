//! EnvFilter construction from logging configuration.

use nest_error::{NestError, NestResult};
use tracing_subscriber::EnvFilter;

use crate::codes::NEST_LOGGING_FILTER_INVALID;
use crate::config::LoggingConfig;

/// Builds the EnvFilter directive string from configuration.
pub fn directive_string(config: &LoggingConfig) -> String {
    let mut parts = vec![config.level.directive().to_string()];

    let mut modules: Vec<_> = config.module_levels.iter().collect();
    modules.sort_by_key(|(target, _)| target.as_str());

    for (target, level) in modules {
        parts.push(format!("{target}={}", level.directive()));
    }

    parts.join(",")
}

/// Builds an [`EnvFilter`] from configuration, honoring `RUST_LOG` when enabled.
pub fn build_env_filter(config: &LoggingConfig) -> NestResult<EnvFilter> {
    if config.env_override && std::env::var_os("RUST_LOG").is_some() {
        return EnvFilter::try_from_default_env().map_err(|err| {
            NestError::config(format!("invalid RUST_LOG environment variable: {err}"))
                .with_code(NEST_LOGGING_FILTER_INVALID)
        });
    }

    let directive = directive_string(config);
    EnvFilter::try_new(&directive).map_err(|err| {
        NestError::config(format!(
            "invalid logging filter directive `{directive}`: {err}"
        ))
        .with_code(NEST_LOGGING_FILTER_INVALID)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level::LogLevel;

    #[test]
    fn builds_directive_string() {
        let config = LoggingConfig::new("kiwi")
            .with_default_level(LogLevel::Info)
            .with_module_level("nest_core", LogLevel::Warn)
            .with_module_level("nest_data", LogLevel::Debug);

        assert_eq!(
            directive_string(&config),
            "info,nest_core=warn,nest_data=debug"
        );
    }

    #[test]
    fn builds_env_filter_from_config() {
        let config = LoggingConfig::new("kiwi").with_default_level(LogLevel::Debug);
        let filter = build_env_filter(&config).unwrap();
        let _ = filter;
    }
}
