use std::{fs::File, path::PathBuf};

use clap::{ArgMatches, Command};

use crate::Error;

mod extract;
mod inspect;
mod unpack;

pub use inspect::inspect;

#[must_use]
pub fn cli() -> Command {
    Command::new("pbo")
        .about("Commands for PBO files")
        .arg_required_else_help(true)
        .subcommand(extract::cli())
        .subcommand(unpack::cli())
        .subcommand(
            Command::new("inspect")
                .about("Inspect a PBO")
                .arg(clap::Arg::new("pbo").help("PBO to inspect").required(true)),
        )
}

/// Execute the pbo command
///
/// # Errors
/// [`Error`] depending on the modules
///
/// # Panics
/// If the args are not present from clap
pub fn execute(matches: &ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        Some(("extract", matches)) => extract::execute(matches),
        Some(("unpack", matches)) => unpack::execute(matches),

        Some(("inspect", matches)) => inspect::inspect(File::open(PathBuf::from(
            matches.get_one::<String>("pbo").expect("required"),
        ))?),

        _ => unreachable!(),
    }
}
