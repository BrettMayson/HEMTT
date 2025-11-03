use crate::{
    commands::global_modules,
    context::Context,
    error::Error,
    executor::Executor,
    modules::{Binarize, Rapifier, pbo::Collapse},
    report::Report,
};

#[derive(clap::Parser)]
/// Checks the project for errors
///
/// `hemtt check` is the quickest way to check your project for errors.
/// All the same checks are run as [`hemtt dev`](./dev.md), but it will not
/// write files to disk, saving time and resources.
///
/// This is ideal for CI/CD pipelines and quick validation during development.
pub struct Command {
    #[clap(flatten)]
    pub(crate) check: CheckArgs,

    #[clap(flatten)]
    global: crate::GlobalArgs,
}

#[derive(clap::Args)]
#[allow(clippy::module_name_repetitions)]
pub struct CheckArgs {
    #[arg(long, short = 'p', action = clap::ArgAction::SetTrue)]
    /// Run all lints that are disabled by default (but not explicitly disabled via project config)
    ///
    /// Enables stricter checking for code quality and best practices.
    pedantic: bool,
    #[arg(long, short = 'L', action = clap::ArgAction::Append)]
    /// Explicit Lints
    ///
    /// Enable specific lints by name. Can be used multiple times.
    /// Example: `hemtt check -L s01-invalid-command -L s02-unknown-command`
    lints: Vec<String>,
}

/// Execute the check command
///
/// # Errors
/// [`Error`] depending on the modules
pub fn execute(cmd: &Command) -> Result<Report, Error> {
    let mut ctx = Context::new(
        Some("check"),
        crate::context::PreservePrevious::Remove,
        true,
    )?;

    if cmd.check.pedantic {
        let runtime = ctx.config().runtime().clone().with_pedantic(true);
        let config = ctx.config().clone().with_runtime(runtime);
        ctx = ctx.with_config(config);
    }

    if !cmd.check.lints.is_empty() {
        let runtime = ctx
            .config()
            .runtime()
            .clone()
            .with_explicit_lints(cmd.check.lints.clone());
        let config = ctx.config().clone().with_runtime(runtime);
        ctx = ctx.with_config(config);
    }

    let mut executor = Executor::new(ctx);
    global_modules(&mut executor);

    executor.collapse(Collapse::Yes);

    executor.add_module(Box::<Rapifier>::default());
    executor.add_module(Box::<Binarize>::new(Binarize::new(true)));

    info!("Running checks");

    executor.init();
    executor.check();
    executor.build(false);

    executor.run()
}
