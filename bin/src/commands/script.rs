use clap::{ArgMatches, Command};

use crate::{context::Context, error::Error, modules::Hooks, report::Report};

#[must_use]
pub fn cli() -> Command {
    Command::new("script")
        .about("Run a Rhai script on the project")
        .long_about("Run a Rhai script on the project, this is useful for automating tasks in a platform agnostic way, or requiring external dependencies.")
        .arg(
            clap::Arg::new("name")
                .help("Name of the new mod")
                .required(true),
        )
}

/// Execute the script command
///
/// # Errors
/// [`Error`] depending on the modules
///
/// # Panics
/// If a name is not provided, but this is usually handled by clap
pub fn execute(matches: &ArgMatches) -> Result<Report, Error> {
    let ctx = Context::new(
        Some("script"),
        crate::context::PreservePrevious::Remove,
        true,
    )?;
    let name = matches
        .get_one::<String>("name")
        .expect("name to be set as required");
    Hooks::run_file(&ctx, name).map(|(report, _)| report)
}
