use clap::{ArgAction, ArgMatches, Command};

use crate::{
    context::{self, Context},
    error::Error,
    executor::Executor,
    modules::{pbo::Collapse, Binarize, Files, Hooks, Rapifier, SQFCompiler},
    report::Report,
};

#[cfg(not(target_os = "macos"))]
use crate::modules::asc::ArmaScriptCompiler;

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
    .arg(
        clap::Arg::new("asc")
            .long("asc")
            .help("Use ArmaScriptCompiler instead of HEMTT's SQF compiler")
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
pub fn execute(matches: &ArgMatches) -> Result<Report, Error> {
    let just = matches
        .get_many::<String>("just")
        .unwrap_or_default()
        .map(|s| s.to_lowercase())
        .collect::<Vec<_>>();
    let mut ctx = Context::new(
        Some("build"),
        if just.is_empty() {
            context::PreservePrevious::Remove
        } else {
            warn!("keeping previous build artifacts");
            context::PreservePrevious::Keep
        },
        true,
    )?;
    if !just.is_empty() {
        ctx = ctx.filter(|a, _| just.contains(&a.name().to_lowercase()));
    }
    let mut executor = executor(ctx, matches);

    if !just.is_empty() {
        warn!("Use of `--just` is not recommended, only use it if you know what you're doing");
    }

    info!("Creating `build` version");

    executor.run()
}

#[must_use]
pub fn executor(ctx: Context, matches: &ArgMatches) -> Executor {
    let mut executor = Executor::new(ctx);

    let use_asc = matches.get_one::<bool>("asc") == Some(&true);

    executor.collapse(Collapse::No);

    executor.add_module(Box::<Hooks>::default());
    if matches.get_one::<bool>("no-rap") != Some(&true) {
        executor.add_module(Box::<Rapifier>::default());
    }
    executor.add_module(Box::new(SQFCompiler::new(!use_asc)));
    #[cfg(not(target_os = "macos"))]
    if use_asc {
        executor.add_module(Box::<ArmaScriptCompiler>::default());
    }
    if matches.get_one::<bool>("no-bin") != Some(&true) {
        executor.add_module(Box::<Binarize>::default());
    }
    executor.add_module(Box::<Files>::default());

    executor.init();
    executor.check();
    executor.build(true);

    executor
}
