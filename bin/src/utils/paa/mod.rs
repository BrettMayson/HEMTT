use std::{fs::File, path::PathBuf};

use clap::{ArgMatches, Command};

use crate::Error;

mod convert;
mod inspect;

pub use inspect::inspect;

#[must_use]
pub fn cli() -> Command {
    Command::new("paa")
        .about("Commands for PAA files")
        .arg_required_else_help(true)
        .subcommand(convert::cli())
        .subcommand(
            Command::new("inspect")
                .about("Inspect a PAA")
                .arg(clap::Arg::new("paa").help("PAA to inspect").required(true)),
        )
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
        Some(("convert", matches)) => convert::execute(matches),

        Some(("inspect", matches)) => inspect::inspect(File::open(PathBuf::from(
            matches.get_one::<String>("paa").expect("required"),
        ))?),

        _ => unreachable!(),
    }
}
