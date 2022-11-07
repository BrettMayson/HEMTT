use clap::{ArgAction, ArgMatches, Command};
use hemtt_error::AppError;

use crate::{
    addons::Location,
    context::Context,
    executor::{Executor, PboTarget},
    modules::{Binarize, Preprocessor},
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

pub fn execute(matches: &ArgMatches) -> Result<(), AppError> {
    let mut context = Context::new(&[Location::Addons])?;
    let mut executor = Executor::new();

    executor.build_pbo(PboTarget::BySource);

    executor.add_module(Box::new(Preprocessor::new()));
    if matches.get_one::<bool>("binarize") == Some(&true) {
        executor.add_module(Box::new(Binarize::new()));
    }

    executor.init(&mut context)?;
    executor.check(&context)?;
    executor.build(&context)?;

    Ok(())
}
