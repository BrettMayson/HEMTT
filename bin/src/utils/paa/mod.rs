use std::{fs::File, path::PathBuf};

use crate::Error;

mod convert;
mod inspect;

pub use inspect::inspect;

#[derive(clap::Parser)]
#[command(arg_required_else_help = true)]
/// Commands for PAA files
pub struct Command {
    #[command(subcommand)]
    commands: Subcommands,
}

#[derive(clap::Subcommand)]
enum Subcommands {
    Convert(convert::Args),
    Inspect(inspect::Args),
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
        Subcommands::Convert(args) => convert::execute(args),
        Subcommands::Inspect(args) => {
            inspect::inspect(File::open(PathBuf::from(&args.paa))?, &args.format)
        }
    }
}
