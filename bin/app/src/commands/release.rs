use clap::{ArgMatches, Command};
use hemtt_bin_error::Error;

use crate::{context::Context, executor::Executor, modules::Sign};

use super::build;

#[must_use]
pub fn cli() -> Command {
    build::add_args(
        Command::new("release")
            .about("Release the project")
            .long_about("Release your project"),
    )
}

pub fn execute(matches: &ArgMatches) -> Result<(), Error> {
    let ctx = Context::new("release")?;
    let mut executor = Executor::new(&ctx);

    executor.add_module(Box::new(Sign::new()));

    build::execute(matches, &mut executor)?;

    executor.release()?;

    Ok(())
}
