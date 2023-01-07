use clap::{ArgAction, ArgMatches, Command};
use hemtt_bin_error::Error;

use crate::{
    executor::Executor,
    modules::{pbo::Collapse, Binarize, Files, Preprocessor},
};

#[must_use]
pub fn cli() -> Command {
    add_args(
        Command::new("build")
            .about("Build the project")
            .long_about("Build your project"),
    )
}

pub fn add_args(cmd: Command) -> Command {
    cmd.arg(
        clap::Arg::new("no-binarize")
            .long("no-binarize")
            .help("Do not binarize the project")
            .action(ArgAction::SetTrue),
    )
}

pub fn execute(matches: &ArgMatches, executor: &mut Executor) -> Result<(), Error> {
    executor.collapse(Collapse::No);

    executor.add_module(Box::new(Preprocessor::new()));
    if matches.get_one::<bool>("no-binarize") != Some(&true) {
        executor.add_module(Box::new(Binarize::new()));
    }
    executor.add_module(Box::new(Files::new()));

    executor.init()?;
    executor.check()?;
    executor.build()?;

    Ok(())
}
