use clap::{ArgAction, ArgMatches, Command};
use hemtt_bin_error::Error;

use crate::{
    addons::Location,
    context::Context,
    executor::Executor,
    modules::{pbo::Collapse, Binarize, FilePatching, Files, Preprocessor},
};

#[must_use]
pub fn cli() -> Command {
    Command::new("dev")
        .about("Mod Development")
        .long_about("Build and test your mod.")
        .arg(
            clap::Arg::new("binarize")
                .long("binarize")
                .short('b')
                .help("Use BI's binarize on supported files")
                .action(ArgAction::SetTrue),
        )
}

pub fn execute(matches: &ArgMatches) -> Result<(), Error> {
    let ctx = Context::new(&[Location::Addons], "dev")?.filter(|a, config| {
        !config
            .hemtt()
            .dev()
            .exclude()
            .iter()
            .any(|e| (a.folder() + "/").starts_with(&format!("{e}/")))
    });
    let mut executor = Executor::new(&ctx);

    executor.collapse(Collapse::Yes);

    executor.add_module(Box::new(Preprocessor::new()));
    executor.add_module(Box::new(Files::new()));
    executor.add_module(Box::new(FilePatching::new()));
    if matches.get_one::<bool>("binarize") == Some(&true) {
        executor.add_module(Box::new(Binarize::new()));
    }

    executor.init()?;
    executor.check()?;
    executor.build()?;

    Ok(())
}
