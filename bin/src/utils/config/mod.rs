mod fmt;

use clap::{ArgMatches, Command};

use crate::Error;

#[must_use]
pub fn cli() -> Command {
    Command::new("config")
        .about("Commands for Config files")
        .arg_required_else_help(true)
        .subcommand(fmt::cli())
}

/// Execute the paa command
///
/// # Errors
/// [`Error`] depending on the modules
///
/// # Panics
/// If the args are not present from clap
pub fn execute(matches: &ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        Some(("fmt", matches)) => fmt::execute(matches),

        _ => unreachable!(),
    }
}
