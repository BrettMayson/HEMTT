use clap::{ArgAction, ArgMatches, Command};

use crate::{report::Report, Error};

mod coverage;
mod sort;

#[must_use]
pub fn cli() -> Command {
    Command::new("localization")
        .visible_alias("ln")
        .about("Manage localization stringtables")
        .subcommand(Command::new("coverage").about("Check the coverage of localization"))
        .subcommand(
            Command::new("sort").about("Sort the stringtables").arg(
                clap::Arg::new("only-lang")
                    .long("only-lang")
                    .help("Sort only the languages")
                    .action(ArgAction::SetTrue),
            ),
        )
}

/// Execute the localization command
///
/// # Errors
/// [`Error`] depending on the modules
///
/// # Panics
/// If a name is not provided, but this is usually handled by clap
pub fn execute(matches: &ArgMatches) -> Result<Report, Error> {
    match matches.subcommand() {
        Some(("coverage", _)) => coverage::coverage(),
        Some(("sort", matches)) => sort::sort(matches),
        _ => {
            cli().print_help().expect("Failed to print help");
            Ok(Report::new())
        }
    }
}
