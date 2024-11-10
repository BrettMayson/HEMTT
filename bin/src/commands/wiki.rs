use hemtt_sqf::parser::database::Database;

use crate::{error::Error, report::Report};

#[derive(clap::Parser)]
#[command(arg_required_else_help = true)]
/// Manage the Arma 3 wiki
pub struct Command {
    #[command(subcommand)]
    commands: Subcommands,

    #[clap(flatten)]
    global: crate::GlobalArgs,
}

#[derive(clap::Subcommand)]
enum Subcommands {
    /// Force pull the wiki, regardless of the last pull time
    ForcePull,
}

/// Execute the wiki command
///
/// # Errors
/// [`Error`] depending on the modules
///
/// # Panics
/// If a name is not provided, but this is usually handled by clap
pub fn execute(_cmd: &Command) -> Result<Report, Error> {
    // TODO right now just assumes force-pull since that's the only subcommand
    let _ = Database::empty(true);
    Ok(Report::new())
}
