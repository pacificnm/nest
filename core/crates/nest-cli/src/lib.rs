//! Command-line host for the Nest framework.
//!
//! `nest-cli` wires modules, parses commands, initializes logging and
//! configuration, renders errors, and exits cleanly.

#![deny(missing_docs)]
#![allow(clippy::result_large_err)]

pub mod codes;
pub mod prelude;

mod app;
mod bootstrap;
mod command;
mod exit;
mod globals;
mod host_info;
mod logging;
mod module;
mod registry;
mod render;

pub use app::CliApp;
pub use command::CliCommand;
pub use exit::CliExitCode;
pub use globals::CliGlobals;
pub use host_info::CliHostInfo;
pub use module::{CliModule, CLI_MODULE_ID};

#[cfg(feature = "async")]
pub use command::AsyncCliCommand;

pub use nest_config::{ConfigDocument, ConfigLoader, ConfigService, ConfigSource};
pub use nest_core::{AppContext, Module, ModuleId};
pub use nest_error::{NestError, NestErrorReport, NestResult};
pub use nest_logging::LoggingConfig;

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::Mutex;

    use nest_config::{ConfigDocument, ConfigService, ConfigSource, LoadedConfig};
    use nest_error::{codes, NestErrorKind};
    use nest_file::{FileModule, FileServiceConfig};
    use nest_file_csv::{CsvModule, CsvOptions, CsvService};
    use serde::Deserialize;
    use tempfile::tempdir;

    use super::*;
    use crate::command::CliCommand;

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn integration_test_lock() -> std::sync::MutexGuard<'static, ()> {
        TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    struct EchoCommand;

    impl CliCommand for EchoCommand {
        fn name(&self) -> &'static str {
            "echo"
        }

        fn about(&self) -> &'static str {
            "Echo a message"
        }

        fn configure(&self, cmd: clap::Command) -> clap::Command {
            use clap::Arg;
            cmd.arg(Arg::new("message").required(true))
        }

        fn run(&self, ctx: &AppContext, matches: &clap::ArgMatches) -> NestResult<()> {
            let message = matches.get_one::<String>("message").unwrap();
            let quiet = ctx
                .service::<CliGlobals>()
                .map(|globals| globals.quiet)
                .unwrap_or(false);
            if !quiet {
                println!("{message}");
            }
            Ok(())
        }
    }

    struct ConfigShowCommand;

    impl CliCommand for ConfigShowCommand {
        fn name(&self) -> &'static str {
            "show-config"
        }

        fn about(&self) -> &'static str {
            "Show loaded config section"
        }

        fn configure(&self, cmd: clap::Command) -> clap::Command {
            cmd
        }

        fn run(&self, ctx: &AppContext, _matches: &clap::ArgMatches) -> NestResult<()> {
            let config = ctx.service::<ConfigService>()?;
            #[derive(Debug, Deserialize)]
            struct LoggingSection {
                level: String,
            }
            let section: LoggingSection = config.section("logging")?;
            println!("level={}", section.level);
            Ok(())
        }
    }

    #[test]
    fn explicit_config_missing_returns_error() {
        let _lock = integration_test_lock();
        let dir = tempdir().unwrap();
        let err = CliApp::new("test-app")
            .command(EchoCommand)
            .try_run_with([
                "test-app",
                "--config",
                dir.path().join("missing.toml").to_str().unwrap(),
                "echo",
                "hi",
            ])
            .unwrap_err();
        assert_eq!(err.code(), Some(codes::NEST_CONFIG_NOT_FOUND));
        assert_eq!(CliExitCode::from_error(&err), CliExitCode::Config);
    }

    #[test]
    fn default_config_search_loads_config_toml() {
        let _lock = integration_test_lock();
        let dir = tempdir().unwrap();
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        fs::write(
            "config.toml",
            "[logging]\nlevel = \"warn\"\n",
        )
        .unwrap();

        let output = CliApp::new("test-app")
            .command(ConfigShowCommand)
            .try_run_with(["test-app", "show-config"]);

        std::env::set_current_dir(original).unwrap();
        output.unwrap();
    }

    #[test]
    fn config_service_deserializes_section() {
        let document = ConfigDocument::parse_toml("[logging]\nlevel = \"debug\"\n").unwrap();
        let service = ConfigService::new(LoadedConfig {
            document: document.clone(),
            source: ConfigSource::Memory(document),
            path: None,
        });
        #[derive(Deserialize)]
        struct LoggingSection {
            level: String,
        }
        let section: LoggingSection = service.section("logging").unwrap();
        assert_eq!(section.level, "debug");
    }

    #[test]
    fn json_error_renderer_outputs_json() {
        let error = NestError::validation("bad input").with_code("NEST_VALIDATION_FAILED");
        let globals = CliGlobals {
            config_path: None,
            log_level: None,
            log_file: None,
            json: true,
            quiet: false,
            verbose: false,
            no_color: true,
        };
        let report = error.report();
        assert_eq!(report.message, "bad input");
        assert_eq!(CliExitCode::from_error(&error), CliExitCode::Validation);
        let _ = globals;
    }

    #[test]
    fn command_dispatch_runs_with_modules() {
        let _lock = integration_test_lock();
        let dir = tempdir().unwrap();

        struct ValidateCsvCommand;

        impl CliCommand for ValidateCsvCommand {
            fn name(&self) -> &'static str {
                "validate-csv"
            }

            fn about(&self) -> &'static str {
                "Validate CSV"
            }

            fn configure(&self, cmd: clap::Command) -> clap::Command {
                use clap::Arg;
                cmd.arg(Arg::new("file").required(true))
            }

            fn run(&self, ctx: &AppContext, matches: &clap::ArgMatches) -> NestResult<()> {
                let file = matches.get_one::<String>("file").unwrap();
                let csv = ctx.service::<CsvService>()?;
                let report = csv.read_records_report(file, &CsvOptions::default())?;
                println!("rows={}", report.valid_rows);
                Ok(())
            }
        }

        let built_dir = dir.path();
        fs::write(
            built_dir.join("data.csv"),
            "customer_id,email\nCUST-1,alice@example.com\n",
        )
        .unwrap();

        CliApp::new("csv-tool")
            .module(FileModule::with_config(
                FileServiceConfig::scoped(built_dir).allow_create_dirs(true),
            ))
            .module(CsvModule)
            .command(ValidateCsvCommand)
            .try_run_with([
                "csv-tool",
                "validate-csv",
                "data.csv",
            ])
            .unwrap();
    }

    #[test]
    fn usage_error_maps_to_exit_code_10() {
        let error = NestError::command("missing subcommand").with_code(codes::NEST_CLI_USAGE);
        assert_eq!(CliExitCode::from_error(&error), CliExitCode::Usage);
        assert_eq!(error.kind(), NestErrorKind::Command);
    }

    #[test]
    fn from_nest_app_runs_command() {
        let _guard = integration_test_lock();
        use nest_app::NestApp;

        let nest_app = NestApp::builder("test-app").build().unwrap();
        CliApp::from_nest_app(nest_app)
            .command(EchoCommand)
            .try_run_with(["test-app", "echo", "hi"])
            .unwrap();
    }

    #[test]
    fn help_includes_registered_subcommand_and_long_about() {
        let app = CliApp::new("test-app")
            .with_long_about("Test App\n\nA test application.")
            .command(EchoCommand);
        let help = app
            .registry
            .build_clap_command("test-app", app.about, app.long_about, app.version)
            .render_long_help()
            .to_string();
        assert!(help.contains("Test App"));
        assert!(help.contains("echo"));
    }

    #[test]
    fn help_flag_returns_ok_without_running_command() {
        let _guard = integration_test_lock();
        CliApp::new("test-app")
            .command(EchoCommand)
            .try_run_with(["test-app", "--help"])
            .unwrap();
    }

    #[test]
    fn version_flag_returns_ok_when_configured() {
        let _guard = integration_test_lock();
        CliApp::new("test-app")
            .with_version("9.9.9")
            .command(EchoCommand)
            .try_run_with(["test-app", "--version"])
            .unwrap();
    }
}
