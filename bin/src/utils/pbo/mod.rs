use std::{fs::File, path::PathBuf};

use crate::Error;

mod extract;
mod inspect;
mod unpack;

pub use inspect::inspect;

#[derive(clap::Parser)]
#[command(arg_required_else_help = true)]
/// Tools for working with PBO (Packed Bank Of files) - Arma's archive format.
pub struct Command {
    #[command(subcommand)]
    commands: Subcommands,
}

#[derive(clap::Subcommand)]
enum Subcommands {
    /// Extract a file from a PBO
    ///
    /// Useful for quickly retrieving a specific file without unpacking the entire PBO.
    Extract(extract::PboExtractArgs),
    #[command(verbatim_doc_comment)]
    /// Inspect a PBO file
    ///
    /// ## Example
    /// Check `abe_main.pbo` located in the build folder
    ///
    /// ```bash
    /// hemtt.exe utils pbo inspect .hemttout\build\addons\abe_main.pbo
    /// ```
    Inspect(inspect::PboInspectArgs),
    /// Unpack a PBO file
    ///
    /// A `$PBOPREFIX$` file will be created in the output directory containing the prefix of the PBO.
    /// All other properties from the PBO will be saved into `properties.txt`
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
