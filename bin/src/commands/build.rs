use crate::{
    context::{self, Context},
    error::Error,
    executor::Executor,
    modules::{pbo::Collapse, Binarize, Files, Rapifier},
    report::Report,
};

use super::global_modules;

#[derive(clap::Parser)]
#[command(
    long_about = "Build your project in release mode for testing, without signing for full release."
)]
/// Build the project for final testing
pub struct Command {
    #[clap(flatten)]
    build: Args,

    #[clap(flatten)]
    just: super::JustArgs,
}

#[derive(clap::Args)]
pub struct Args {
    #[arg(long, action = clap::ArgAction::SetTrue)]
    /// Do not binarize the project
    no_bin: bool,
    #[arg(long, action = clap::ArgAction::SetTrue)]
    /// Do not rapify (cpp, rvmat)
    no_rap: bool,
}

/// Execute the build command, build a new executor
///
/// # Errors
/// [`Error`] depending on the modules
pub fn execute(cmd: &Command) -> Result<Report, Error> {
    let just = cmd
        .just
        .just
        .iter()
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
    let mut executor = executor(ctx, &cmd.build);

    if !just.is_empty() {
        warn!("Use of `--just` is not recommended, only use it if you know what you're doing");
    }

    info!("Creating `build` version");

    executor.run()
}

#[must_use]
pub fn executor(ctx: Context, args: &Args) -> Executor {
    let mut executor = Executor::new(ctx);
    global_modules(&mut executor);

    executor.collapse(Collapse::No);

    if !args.no_rap {
        executor.add_module(Box::<Rapifier>::default());
    }
    if !args.no_bin {
        executor.add_module(Box::<Binarize>::default());
    }
    executor.add_module(Box::<Files>::default());

    executor.init();
    executor.check();
    executor.build(true);

    executor
}
