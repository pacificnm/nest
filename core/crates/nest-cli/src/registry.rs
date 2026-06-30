//! Command registry and clap tree construction.

use clap::Command;

use crate::command::CliCommand;

#[cfg(feature = "async")]
use crate::command::AsyncCliCommand;

/// Registry of CLI commands.
pub struct CommandRegistry {
    sync_commands: Vec<Box<dyn CliCommand>>,
    #[cfg(feature = "async")]
    async_commands: Vec<Box<dyn AsyncCliCommand>>,
}

impl CommandRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self {
            sync_commands: Vec::new(),
            #[cfg(feature = "async")]
            async_commands: Vec::new(),
        }
    }

    /// Registers a synchronous command.
    pub fn register_sync(&mut self, command: Box<dyn CliCommand>) {
        self.sync_commands.push(command);
    }

    /// Registers an asynchronous command.
    #[cfg(feature = "async")]
    pub fn register_async(&mut self, command: Box<dyn AsyncCliCommand>) {
        self.async_commands.push(command);
    }

    /// Returns whether any async commands are registered.
    #[cfg(feature = "async")]
    pub fn has_async_commands(&self) -> bool {
        !self.async_commands.is_empty()
    }

    /// Builds the root clap command.
    pub fn build_clap_command(
        &self,
        app_name: &'static str,
        about: Option<&'static str>,
        long_about: Option<&'static str>,
    ) -> Command {
        let mut root = Command::new(app_name)
            .subcommand_required(true)
            .arg_required_else_help(true);
        if let Some(about) = about {
            root = root.about(about);
        }
        if let Some(long_about) = long_about {
            root = root.long_about(long_about);
        }
        let mut root = crate::globals::attach_global_args(root);

        for command in &self.sync_commands {
            let sub = Command::new(command.name()).about(command.about());
            let sub = command.configure(sub);
            root = root.subcommand(sub);
        }

        #[cfg(feature = "async")]
        for command in &self.async_commands {
            let sub = Command::new(command.name()).about(command.about());
            let sub = command.configure(sub);
            root = root.subcommand(sub);
        }

        root
    }

    /// Finds a sync command by name.
    pub fn find_sync(&self, name: &str) -> Option<&dyn CliCommand> {
        self.sync_commands
            .iter()
            .find(|command| command.name() == name)
            .map(|command| command.as_ref())
    }

    /// Finds an async command by name.
    #[cfg(feature = "async")]
    pub fn find_async(&self, name: &str) -> Option<&dyn AsyncCliCommand> {
        self.async_commands
            .iter()
            .find(|command| command.name() == name)
            .map(|command| command.as_ref())
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}
