use crate::{report::Report, Error};

mod coverage;
mod sort;

#[derive(clap::Parser)]
#[command(arg_required_else_help = true)]
/// Manage localization stringtables
pub struct Command {
    #[command(subcommand)]
    commands: Subcommands,
}

#[derive(clap::Subcommand)]
enum Subcommands {
    /// Generate a coverage report
    Coverage(coverage::Args),
    /// Sort the stringtables
    Sort(sort::Args),
}

/// Execute the localization command
///
/// # Errors
/// [`Error`] depending on the modules
pub fn execute(cmd: &Command) -> Result<Report, Error> {
    match &cmd.commands {
        Subcommands::Coverage(cmd) => coverage::coverage(cmd),
        Subcommands::Sort(cmd) => sort::sort(cmd),
    }
}
