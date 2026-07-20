#![allow(clippy::result_large_err)]

use clap::{ArgMatches, Command};
use nest_cli::{CliApp, CliCommand as CliHostCommand, LoggingConfig};
use nest_error::NestResult;
use {{app_id_snake}}_core::greet;

fn main() -> NestResult<()> {
    CliApp::new("{{app_id}}")
        .with_logging(LoggingConfig::new("{{app_id}}").with_file("./logs"))
        .command(GreetCommand)
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
        println!("{}", greet(name));
        Ok(())
    }
}
