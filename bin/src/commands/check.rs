use crate::{
    commands::global_modules,
    context::Context,
    error::Error,
    executor::Executor,
    modules::{pbo::Collapse, Binarize, Rapifier},
    report::Report,
};

#[derive(clap::Parser)]
/// Checks the project for errors
///
/// `hemtt check` is the quickest way to check your project for errors.
/// All the same checks are run as [`hemtt dev`](./dev.md), but it will not
/// write files to disk, saving time and resources.
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
    pedantic: bool,
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
