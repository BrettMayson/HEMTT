use std::path::PathBuf;

use clap::{ArgMatches, Command};

use crate::Error;

mod inspect;

pub use inspect::inspect;

#[must_use]
pub fn cli() -> Command {
    Command::new("paa")
        .about("Commands for PAA files")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("inspect").about("Inspect a config file").arg(
                clap::Arg::new("config")
                    .help("Config to inspect")
                    .required(true),
            ),
        )
}

/// Execute the config command
///
/// # Errors
/// [`Error`] depending on the modules
///
/// # Panics
/// If the args are not present from clap
pub fn execute(matches: &ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        Some(("inspect", matches)) => inspect::inspect(&PathBuf::from(
            matches.get_one::<String>("config").expect("required"),
        )),

        _ => unreachable!(),
    }
}
