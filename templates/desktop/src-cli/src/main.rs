use clap::{ArgMatches, Command};
use nest_cli::{AppContext, CliApp, CliCommand as CliHostCommand, LoggingConfig};
use nest_cli_command::CliCommand;
use nest_error::NestResult;

fn main() -> NestResult<()> {
    CliApp::new("nest-desktop-template")
        .with_logging(LoggingConfig::new("nest-desktop-template").with_file("../logs"))
        .command(AboutVersionCommand)
        .command(RecipesCommand)
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

    fn run(&self, _ctx: &AppContext, _matches: &ArgMatches) -> NestResult<()> {
        let output = run_cli_command(CliCommand::ListRecipes).map_err(nest_error::NestError::unknown)?;
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
            nest_version::app_version(std::path::Path::new(env!("CARGO_MANIFEST_DIR")))
                .map_err(|e| e.to_string())
        }
        CliCommand::ListRecipes => list_recipes(),
        _ => Err("Unsupported command".into()),
    }
}

/// Reads `.nest-recipes` from the app root and returns a human-readable list
/// of applied recipes and their versions.
fn list_recipes() -> Result<String, String> {
    let path = find_app_root()
        .map(|root| root.join(".nest-recipes"))
        .unwrap_or_else(|| std::path::PathBuf::from(".nest-recipes"));

    if !path.exists() {
        return Ok("No recipes applied.".to_string());
    }

    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut lines: Vec<String> = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((id, version)) = line.split_once('@') {
            lines.push(format!("{} (v{})", id, version));
        }
    }

    if lines.is_empty() {
        return Ok("No recipes applied.".to_string());
    }

    Ok(lines.join("\n"))
}

/// Walks up from the current directory looking for a directory that contains
/// `.nest-recipes` or a `Cargo.toml` workspace.
fn find_app_root() -> Option<std::path::PathBuf> {
    let mut dir = std::env::current_dir().ok()?;

    loop {
        if dir.join(".nest-recipes").exists() {
            return Some(dir);
        }

        if let Ok(contents) = std::fs::read_to_string(dir.join("Cargo.toml")) {
            if contents.contains("[workspace]") {
                return Some(dir);
            }
        }

        if !dir.pop() {
            break;
        }
    }

    None
}
