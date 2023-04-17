use clap::{ArgAction, ArgMatches, Command};

use crate::{
    error::Error,
    executor::Executor,
    modules::{pbo::Collapse, ArmaScriptCompiler, Binarize, Files, Hooks, Lint, Rapifier},
};

#[must_use]
pub fn cli() -> Command {
    add_args(
        Command::new("build")
            .about("Build the project for final testing")
            .long_about(
                "Build your project in release mode for testing, without signing for full release.",
            ),
    )
}

pub fn add_args(cmd: Command) -> Command {
    cmd.arg(
        clap::Arg::new("no-bin")
            .long("no-bin")
            .help("Do not binarize the project")
            .action(ArgAction::SetTrue),
    )
    .arg(
        clap::Arg::new("no-rap")
            .long("no-rap")
            .help("Do not rapify (cpp, rvmat)")
            .action(ArgAction::SetTrue),
    )
}

pub fn execute(matches: &ArgMatches, executor: &mut Executor) -> Result<(), Error> {
    executor.collapse(Collapse::No);

    executor.add_module(Box::<Lint>::default());
    if matches.get_one::<bool>("no-rap") != Some(&true) {
        executor.add_module(Box::<Rapifier>::default());
    }
    if matches.get_one::<bool>("no-bin") != Some(&true) {
        executor.add_module(Box::<Binarize>::default());
    }
    executor.add_module(Box::<Hooks>::default());
    executor.add_module(Box::<ArmaScriptCompiler>::default());
    executor.add_module(Box::<Files>::default());

    info!("Creating `build` version");

    executor.init()?;
    executor.check()?;
    executor.build()?;

    Ok(())
}
