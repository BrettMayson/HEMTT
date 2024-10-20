use clap::{ArgMatches, Command};

use crate::{report::Report, utils, Error};

#[must_use]
pub fn cli() -> Command {
    Command::new("utils")
        .about("Use HEMTT standalone utils")
        .subcommand_required(false)
        .arg_required_else_help(true)
        .subcommand(utils::config::cli())
        .subcommand(utils::inspect::cli())
        .subcommand(utils::paa::cli())
        .subcommand(utils::pbo::cli())
        .subcommand(utils::sqf::cli())
        .subcommand(utils::verify::cli())
}

/// Execute the utils command
///
/// # Errors
/// [`Error`] depending on the modules
pub fn execute(matches: &ArgMatches) -> Result<Report, Error> {
    match matches.subcommand() {
        Some(("config", matches)) => utils::config::execute(matches),
        Some(("inspect", matches)) => utils::inspect::execute(matches),
        Some(("paa", matches)) => utils::paa::execute(matches),
        Some(("pbo", matches)) => utils::pbo::execute(matches),
        Some(("sqf", matches)) => utils::sqf::execute(matches),
        Some(("verify", matches)) => utils::verify::execute(matches),
        _ => unreachable!(),
    }?;
    Ok(Report::new())
}
