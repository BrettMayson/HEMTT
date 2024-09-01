mod case;

use clap::{ArgMatches, Command};

use crate::Error;

#[must_use]
pub fn cli() -> Command {
    Command::new("sqf")
        .about("Commands for SQF files")
        .arg_required_else_help(true)
        .subcommand(case::cli())
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
        Some(("case", matches)) => case::execute(matches),

        _ => unreachable!(),
    }
}
