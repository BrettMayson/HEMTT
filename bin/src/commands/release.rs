use clap::{ArgAction, ArgMatches, Command};

use crate::{context::Context, error::Error, modules::Sign, report::Report};

use super::build;

#[must_use]
pub fn cli() -> Command {
    build::add_args(
        Command::new("release")
            .about("Build the project for release")
            .long_about("Build your project for full release, with signing and archiving."),
    )
    .arg(
        clap::Arg::new("no-sign")
            .long("no-sign")
            .help("Do not sign the PBOs")
            .action(ArgAction::SetTrue),
    )
    .arg(
        clap::Arg::new("no-archive")
            .long("no-archive")
            .help("Do not create an archive of the release")
            .action(ArgAction::SetTrue),
    )
}

/// Execute the release command
///
/// # Errors
/// [`Error`] depending on the modules
pub fn execute(matches: &ArgMatches) -> Result<Report, Error> {
    let ctx = Context::new(
        Some("release"),
        crate::context::PreservePrevious::Remove,
        true,
    )?;
    let mut executor = build::executor(ctx, matches);

    if matches.get_one::<bool>("no-sign") != Some(&true)
        && executor.ctx().config().hemtt().release().sign()
    {
        executor.add_module(Box::new(Sign::new()));
    }

    let archive = if matches.get_one::<bool>("no-archive") == Some(&true) {
        false
    } else {
        executor.ctx().config().hemtt().release().archive()
    };

    executor.release(archive);

    executor.run()
}
