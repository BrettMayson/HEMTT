use std::path::PathBuf;

use crate::Error;

mod inspect;

pub use inspect::inspect;

#[derive(clap::Parser)]
#[command(arg_required_else_help = true)]
/// Commands for config files
pub struct Command {
    #[command(subcommand)]
    commands: Subcommands,
}

#[derive(clap::Subcommand)]
enum Subcommands {
    /// Inspect a config file
    Inspect(inspect::InspectArgs),
}

/// Execute the config command
///
/// # Errors
/// [`Error`] depending on the modules
///
/// # Panics
/// If the args are not present from clap
pub fn execute(cmd: &Command) -> Result<(), Error> {
    match &cmd.commands {
        Subcommands::Inspect(args) => inspect::inspect(&PathBuf::from(&args.config)),
    }
}
