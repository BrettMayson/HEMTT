#![deny(clippy::all, clippy::nursery)]
#![warn(clippy::pedantic)]

use clap::{ArgMatches, Command};
use hemtt_error::AppError;

mod preprocessor;
mod rapify;

#[must_use]
pub fn cli() -> Command {
    Command::new("hemtt-app-internal")
        .about("HEMTT Internal Commands")
        .long_about("While these commands are exposed, they are not intended to be used directly by users or scripts. They may change at any time.")
        .arg_required_else_help(true)
        .subcommand(preprocessor::cli())
        .subcommand(rapify::cli())
}

pub fn execute(matches: &ArgMatches) -> Result<(), AppError> {
    match matches.subcommand() {
        Some(("preprocessor", matches)) => preprocessor::execute(matches),
        Some(("rapify", matches)) => rapify::execute(matches),
        _ => unreachable!(),
    }
}
