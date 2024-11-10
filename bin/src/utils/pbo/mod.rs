use std::{fs::File, path::PathBuf};

use crate::Error;

mod extract;
mod inspect;
mod unpack;

pub use inspect::inspect;

#[derive(clap::Parser)]
#[command(arg_required_else_help = true)]
/// Commands for PBO files
pub struct Command {
    #[command(subcommand)]
    commands: Subcommands,
}

#[derive(clap::Subcommand)]
enum Subcommands {
    /// Extract a file from a PBO
    Extract(extract::PboExtractArgs),
    /// Inspect a PBO file
    Inspect(inspect::PboInspectArgs),
    /// Unpack a PBO file
    Unpack(unpack::PboUnpackArgs),
}

/// Execute the pbo command
///
/// # Errors
/// [`Error`] depending on the modules
///
/// # Panics
/// If the args are not present from clap
pub fn execute(cmd: &Command) -> Result<(), Error> {
    match &cmd.commands {
        Subcommands::Extract(args) => extract::execute(args),
        Subcommands::Inspect(args) => {
            inspect::inspect(File::open(PathBuf::from(&args.pbo))?, &args.format)
        }
        Subcommands::Unpack(args) => unpack::execute(args),
    }
}
