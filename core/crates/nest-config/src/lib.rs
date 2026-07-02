//! Configuration loading for the Nest framework.
//!
//! `nest-config` loads, validates, and exposes application configuration for
//! CLI, desktop, server, and test hosts. It does not depend on `nest-core`;
//! hosts register [`ConfigService`] into the Nest service registry.

#![deny(missing_docs)]
#![allow(clippy::result_large_err)]

pub mod codes;
pub mod prelude;

mod document;
mod format;
mod loader;
mod paths;
mod service;
mod source;

pub use document::ConfigDocument;
pub use format::ConfigFormat;
pub use loader::{ConfigLoader, LoadedConfig};
pub use paths::{default_search_paths, resolve_search};
pub use service::ConfigService;
pub use source::ConfigSource;

pub use nest_error::{NestError, NestResult};

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::Mutex;

    use nest_error::codes;
    use serde::Deserialize;
    use tempfile::tempdir;

    use super::*;

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn test_lock() -> std::sync::MutexGuard<'static, ()> {
        TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    #[test]
    fn parse_toml_and_section() {
        let document = ConfigDocument::parse_toml("[logging]\nlevel = \"info\"\n").unwrap();
        #[derive(Deserialize, PartialEq, Debug)]
        struct LoggingSection {
            level: String,
        }
        let section: LoggingSection = document.section("logging").unwrap();
        assert_eq!(section.level, "info");
    }

    #[test]
    fn optional_section_returns_none_when_missing() {
        let document = ConfigDocument::empty();
        let section: Option<String> = document.optional_section("missing").unwrap();
        assert!(section.is_none());
    }

    #[test]
    fn section_missing_uses_code() {
        let document = ConfigDocument::empty();
        let err = document.section::<String>("logging").unwrap_err();
        assert_eq!(err.code(), Some(codes::NEST_CONFIG_SECTION_MISSING));
    }

    #[test]
    fn section_invalid_uses_code() {
        let document = ConfigDocument::parse_toml("[logging]\nlevel = 1\n").unwrap();
        #[derive(Deserialize, Debug)]
        struct LoggingSection {
            #[allow(dead_code)]
            level: String,
        }
        let err = document.section::<LoggingSection>("logging").unwrap_err();
        assert_eq!(err.code(), Some(codes::NEST_CONFIG_SECTION_INVALID));
    }

    #[test]
    fn dotted_section_path() {
        let document =
            ConfigDocument::parse_toml("[data.sqlite]\ndatabase = \"./data/app.db\"\n").unwrap();
        #[derive(Deserialize)]
        struct SqliteSection {
            database: String,
        }
        let section: SqliteSection = document.section("data.sqlite").unwrap();
        assert_eq!(section.database, "./data/app.db");
    }

    #[test]
    fn file_missing_returns_not_found() {
        let err = ConfigLoader::new("test-app")
            .source(ConfigSource::File("missing.toml".into()))
            .load()
            .unwrap_err();
        assert_eq!(err.code(), Some(codes::NEST_CONFIG_NOT_FOUND));
    }

    #[test]
    fn search_defaults_finds_config_toml() {
        let _lock = test_lock();
        let dir = tempdir().unwrap();
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();
        fs::write("config.toml", "[app]\nname = \"demo\"\n").unwrap();

        let loaded = ConfigLoader::new("demo-app").load().unwrap();
        std::env::set_current_dir(original).unwrap();

        assert!(loaded.path.is_some());
        assert_eq!(loaded.source.label(), ConfigSource::SearchDefaults.label());
    }

    #[test]
    fn search_defaults_empty_when_none_found() {
        let _lock = test_lock();
        let dir = tempdir().unwrap();
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let loaded = ConfigLoader::new("demo-app").load().unwrap();
        std::env::set_current_dir(original).unwrap();

        assert!(loaded.path.is_none());
        assert!(!loaded.document.has_section("logging"));
    }

    #[test]
    fn file_or_search_explicit_missing_errors() {
        let err = ConfigLoader::file_or_search("demo", Some("nope.toml".into()))
            .load()
            .unwrap_err();
        assert_eq!(err.code(), Some(codes::NEST_CONFIG_NOT_FOUND));
    }

    #[test]
    fn auto_format_rejects_unknown_extension() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.yaml");
        fs::write(&path, "key: value\n").unwrap();
        let err = ConfigLoader::new("demo")
            .source(ConfigSource::File(path))
            .format(ConfigFormat::Auto)
            .load()
            .unwrap_err();
        assert_eq!(err.code(), Some(codes::NEST_CONFIG_UNSUPPORTED_FORMAT));
    }

    #[test]
    fn read_failed_when_not_readable() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        fs::write(&path, "[app]\nname = \"x\"\n").unwrap();
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o000);
        fs::set_permissions(&path, permissions).unwrap();

        let result = ConfigLoader::new("demo")
            .source(ConfigSource::File(path.clone()))
            .load();

        let mut restore = fs::metadata(&path).unwrap().permissions();
        restore.set_mode(0o644);
        fs::set_permissions(&path, restore).unwrap();

        let err = result.unwrap_err();
        assert_eq!(err.code(), Some(codes::NEST_CONFIG_READ_FAILED));
    }

    #[cfg(feature = "json")]
    #[test]
    fn parse_json_root_object() {
        let document =
            ConfigDocument::parse_json(r#"{ "logging": { "level": "debug" } }"#).unwrap();
        #[derive(Deserialize)]
        struct LoggingSection {
            level: String,
        }
        let section: LoggingSection = document.section("logging").unwrap();
        assert_eq!(section.level, "debug");
    }
}
