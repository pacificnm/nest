use clap::{ArgMatches, Command};
use nest_cli::{AppContext, CliApp, CliCommand as CliHostCommand};
use nest_cli_command::CliCommand;
use nest_error::NestResult;

fn main() -> NestResult<()> {
    CliApp::new("nest-desktop-template")
        .command(AboutVersionCommand)
        .run()
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

    fn run(&self, _ctx: &AppContext, _matches: &ArgMatches) -> NestResult<()> {
        let output =
            run_cli_command(CliCommand::AboutVersion).map_err(nest_error::NestError::unknown)?;
        println!("{output}");
        Ok(())
    }
}

// Same dispatch as src-tauri/src/main.rs's `run_cli` Tauri command — same
// CliCommand variants, same results, whether this runs as a standalone
// binary or is invoked through the desktop app's IPC bridge. The desktop
// app is a thin client on top of this; it does not own the command logic.
fn run_cli_command(command: CliCommand) -> Result<String, String> {
    match command {
        CliCommand::AboutVersion => {
            nest_version::app_version(std::path::Path::new(".")).map_err(|e| e.to_string())
        }
        _ => Err("Unsupported command".into()),
    }
}
