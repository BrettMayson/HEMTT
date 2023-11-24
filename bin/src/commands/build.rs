use clap::{ArgAction, ArgMatches, Command};

use crate::{
    context::{self, Context},
    error::Error,
    executor::Executor,
    modules::{pbo::Collapse, ArmaScriptCompiler, Binarize, Files, Hooks, Lint, Rapifier},
};

#[must_use]
pub fn cli() -> Command {
    add_just(add_args(
        Command::new("build")
            .about("Build the project for final testing")
            .long_about(
                "Build your project in release mode for testing, without signing for full release.",
            ),
    ))
}

#[must_use]
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

#[must_use]
pub fn add_just(cmd: Command) -> Command {
    cmd.arg(
        clap::Arg::new("just")
            .long("just")
            .help("Only build the given addon")
            .action(ArgAction::Append),
    )
}

/// Execute the build command, build a new executor
///
/// # Errors
/// [`Error`] depending on the modules
pub fn pre_execute(matches: &ArgMatches) -> Result<(), Error> {
    let just = matches
        .get_many::<String>("just")
        .unwrap_or_default()
        .map(|s| s.to_lowercase())
        .collect::<Vec<_>>();
    let mut ctx = Context::new(
        std::env::current_dir()?,
        "build",
        if just.is_empty() {
            context::PreservePrevious::Remove
        } else {
            warn!("keeping previous build artifacts");
            context::PreservePrevious::Keep
        },
    )?;
    if !just.is_empty() {
        ctx = ctx.filter(|a, _| just.contains(&a.name().to_lowercase()));
    }
    let mut executor = Executor::new(&ctx);

    execute(matches, &mut executor)?;

    if !just.is_empty() {
        warn!("Use of `--just` is not recommended, only use it if you know what you're doing");
    }

    Ok(())
}

/// Execute the build command, with a given executor
///
/// # Errors
/// [`Error`] depending on the modules
pub fn execute(matches: &ArgMatches, executor: &mut Executor) -> Result<(), Error> {
    executor.collapse(Collapse::No);

    executor.add_module(Box::<Hooks>::default());
    executor.add_module(Box::<Lint>::default());
    if matches.get_one::<bool>("no-rap") != Some(&true) {
        executor.add_module(Box::<Rapifier>::default());
    }
    executor.add_module(Box::<ArmaScriptCompiler>::default());
    if matches.get_one::<bool>("no-bin") != Some(&true) {
        executor.add_module(Box::<Binarize>::default());
    }
    executor.add_module(Box::<Files>::default());

    info!("Creating `build` version");

    executor.init()?;
    executor.check()?;
    executor.build()?;

    Ok(())
}
