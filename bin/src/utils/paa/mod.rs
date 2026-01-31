use std::{fs::File, path::PathBuf};

use crate::Error;

mod convert;
mod inspect;
mod cxam_fix;

pub use inspect::inspect;

#[derive(clap::Parser)]
#[command(arg_required_else_help = true)]
/// Tools for working with PAA (PAX Archive) - Arma's texture format.
/// Convert between common image formats and PAA, or inspect PAA files.
pub struct Command {
    #[command(subcommand)]
    commands: Subcommands,
}

#[derive(clap::Subcommand)]
enum Subcommands {
    /// Convert an image to/from PAA format
    ///
    /// For PAAs, extracts the first mipmap and saves it as an image.
    /// Useful for viewing or editing Arma textures in standard image editors.
    ///
    /// Supports most common image formats (PNG, JPEG, BMP, etc.) based on file extension.
    Convert(convert::PaaConvertArgs),
    /// Inspect a PAA file
    Inspect(inspect::PaaInspectArgs),
    /// Fix PAAs with incorrect CXAM (color ambient max) tagg values
    CxamFix(cxam_fix::Command),
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
        Subcommands::CxamFix(args) => cxam_fix::execute(args),
    }
}
