use clap::{ArgMatches, Command};

use crate::{
    addons::Location,
    context::Context,
    error::Error,
    executor::Executor,
    modules::{pbo::Collapse, Binarize, Files, Preprocessor},
};

#[must_use]
pub fn cli() -> Command {
    Command::new("build")
        .about("Build the project")
        .long_about("Build your project")
}

pub fn execute(_matches: &ArgMatches) -> Result<(), Error> {
    let ctx = Context::new(
        &[Location::Addons, Location::Optionals, Location::Compats],
        "build",
    )?;
    let mut executor = Executor::new(&ctx);

    executor.collapse(Collapse::No);

    executor.add_module(Box::new(Preprocessor::new()));
    executor.add_module(Box::new(Binarize::new()));
    executor.add_module(Box::new(Files::new()));

    executor.init()?;
    executor.check()?;
    executor.build()?;

    Ok(())
}
