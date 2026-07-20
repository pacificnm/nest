#![allow(clippy::result_large_err)]

use clap::{ArgMatches, Command};
use nest_cli::{CliApp, CliCommand as CliHostCommand};
use nest_error::NestResult;
use nest_logging::LoggingConfig;
use {{app_id_snake}}_core::{run_command, CliCommand};

fn main() -> NestResult<()> {
    CliApp::new("{{app_id}}")
        .with_logging(LoggingConfig::new("{{app_id}}").with_file("./logs"))
        .command(GreetCommand)
        .command(AboutVersionCommand)
        .command(RecipesCommand)
        .try_run()
}

struct GreetCommand;

impl CliHostCommand for GreetCommand {
    fn name(&self) -> &'static str {
        "greet"
    }

    fn about(&self) -> &'static str {
        "Greet someone"
    }

    fn configure(&self, cmd: Command) -> Command {
        use clap::Arg;
        cmd.arg(Arg::new("name").default_value("World"))
    }

    fn run(&self, _ctx: &nest_cli::AppContext, matches: &ArgMatches) -> NestResult<()> {
        let name = matches.get_one::<String>("name").unwrap();
        let output = run_command(CliCommand::Greet { name: name.clone() })
            .map_err(nest_error::NestError::unknown)?;
        println!("{output}");
        Ok(())
    }
}

struct AboutVersionCommand;

impl CliHostCommand for AboutVersionCommand {
    fn name(&self) -> &'static str {
        "about-version"
    }

    fn about(&self) -> &'static str {
        "Print the application version"
    }

    fn configure(&self, cmd: Command) -> Command {
        cmd
    }

    fn run(&self, _ctx: &nest_cli::AppContext, _matches: &ArgMatches) -> NestResult<()> {
        let output =
            run_command(CliCommand::AboutVersion).map_err(nest_error::NestError::unknown)?;
        println!("{output}");
        Ok(())
    }
}

struct RecipesCommand;

impl CliHostCommand for RecipesCommand {
    fn name(&self) -> &'static str {
        "recipes"
    }

    fn about(&self) -> &'static str {
        "List recipes applied to this app"
    }

    fn configure(&self, cmd: Command) -> Command {
        cmd
    }

    fn run(&self, _ctx: &nest_cli::AppContext, _matches: &ArgMatches) -> NestResult<()> {
        let output = run_command(CliCommand::ListRecipes).map_err(nest_error::NestError::unknown)?;
        println!("{output}");
        Ok(())
    }
}
