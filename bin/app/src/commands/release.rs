use clap::{ArgMatches, Command};
use hemtt_bin_error::Error;

use crate::{
    addons::Location,
    context::Context,
    executor::Executor,
    modules::{pbo::Collapse, Binarize, Files, Preprocessor, Sign},
};

#[must_use]
pub fn cli() -> Command {
    Command::new("release")
        .about("Release the project")
        .long_about("Release your project")
}

pub fn execute(_matches: &ArgMatches) -> Result<(), Error> {
    let ctx = Context::new(&[Location::Addons, Location::Optionals], "release")?;
    let mut executor = Executor::new(&ctx);

    executor.collapse(Collapse::No);

    executor.add_module(Box::new(Preprocessor::new()));
    executor.add_module(Box::new(Binarize::new()));
    executor.add_module(Box::new(Files::new()));
    executor.add_module(Box::new(Sign::new()));

    executor.init()?;
    executor.check()?;
    executor.build()?;
    executor.release()?;

    Ok(())
}
