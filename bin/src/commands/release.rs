use clap::{ArgMatches, Command};

use crate::{context::Context, error::Error, executor::Executor, modules::Sign};

use super::build;

#[must_use]
pub fn cli() -> Command {
    build::add_args(
        Command::new("release")
            .about("Release the project")
            .long_about("Release your project"),
    )
    .arg(
        clap::Arg::new("no-sign")
            .long("no-sign")
            .help("Do not sign the PBOs"),
    )
    .arg(
        clap::Arg::new("no-archive")
            .long("no-archive")
            .help("Do not create an archive of the release"),
    )
}

pub fn execute(matches: &ArgMatches) -> Result<(), Error> {
    let ctx = Context::new("release")?;
    let mut executor = Executor::new(&ctx);

    if matches.get_one::<bool>("no-sign") != Some(&true) && ctx.config().hemtt().release().sign() {
        executor.add_module(Box::new(Sign::new()));
    }

    let archive = if matches.get_one::<bool>("no-archive") == Some(&true) {
        false
    } else {
        ctx.config().hemtt().release().archive()
    };

    build::execute(matches, &mut executor)?;

    executor.release(archive)?;

    Ok(())
}
