use clap::{ArgMatches, Command};
use hemtt_sqf::parser::database::Database;

use crate::{error::Error, report::Report};

#[must_use]
pub fn cli() -> Command {
    Command::new("wiki")
        .about("Manage the Arma 3 wiki")
        .subcommand(
            Command::new("force-pull")
                .about("Force pull the wiki, if updates you need have been pushed very recently"),
        )
}

/// Execute the wiki command
///
/// # Errors
/// [`Error`] depending on the modules
///
/// # Panics
/// If a name is not provided, but this is usually handled by clap
pub fn execute(_matches: &ArgMatches) -> Result<Report, Error> {
    // TODO right now just assumes force-pull since that's the only subcommand
    let _ = Database::empty(true);
    Ok(Report::new())
}
