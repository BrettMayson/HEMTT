mod case;

use crate::Error;

#[derive(clap::Parser)]
#[command(arg_required_else_help = true)]
/// Commands for SQF files
pub struct Command {
    #[command(subcommand)]
    commands: Subcommands,
}

#[derive(clap::Subcommand)]
enum Subcommands {
    /// Convert case
    Case(case::Args),
}

/// Execute the paa command
///
/// # Errors
/// [`Error`] depending on the modules
///
/// # Panics
/// If the args are not present from clap
pub fn execute(cmd: &Command) -> Result<(), Error> {
    match &cmd.commands {
        Subcommands::Case(args) => case::execute(args),
    }
}
