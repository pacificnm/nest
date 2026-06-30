//! CLI command trait definitions.

use clap::{ArgMatches, Command};
use nest_core::AppContext;
use nest_error::NestResult;

/// A synchronous CLI subcommand.
pub trait CliCommand: Send + Sync + 'static {
    /// Subcommand name (clap command id).
    fn name(&self) -> &'static str;

    /// Short command description.
    fn about(&self) -> &'static str;

    /// Configures clap arguments for this command.
    fn configure(&self, cmd: Command) -> Command;

    /// Runs the command against the built application context.
    fn run(&self, ctx: &AppContext, matches: &ArgMatches) -> NestResult<()>;
}

#[cfg(feature = "async")]
mod async_command {
    use super::*;

    /// An asynchronous CLI subcommand.
    #[async_trait::async_trait]
    pub trait AsyncCliCommand: Send + Sync + 'static {
        /// Subcommand name (clap command id).
        fn name(&self) -> &'static str;

        /// Short command description.
        fn about(&self) -> &'static str;

        /// Configures clap arguments for this command.
        fn configure(&self, cmd: Command) -> Command;

        /// Runs the command asynchronously.
        async fn run_async(&self, ctx: &AppContext, matches: &ArgMatches) -> NestResult<()>;
    }
}

#[cfg(feature = "async")]
pub use async_command::AsyncCliCommand;
