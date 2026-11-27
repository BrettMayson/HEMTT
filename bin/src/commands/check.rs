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
    #[arg(long, short = 'e', action = clap::ArgAction::SetTrue)]
    /// Treat all help and warning messages as errors
    error_on_all: bool,
    #[arg(long, short = 'L', action = clap::ArgAction::Append)]
    /// Explicit Lints
    ///
    /// Enable specific lints, even ones that are disabled by default or in the project config.
    /// A lint can be given by its code or its name, the same as [`hemtt explain`](./explain.md).
    /// Can be used multiple times.
    /// Example: `hemtt check -L S14 -L unused_external`
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
        let mut idents = Vec::with_capacity(cmd.check.lints.len());
        let mut report = Report::new();
        for query in &cmd.check.lints {
            match crate::commands::explain::find(query) {
                Some(lint) => idents.push(lint.ident.to_string()),
                None => report.push(crate::commands::explain::unknown_lint(query)),
            }
        }
        if report.failed() {
            return Ok(report);
        }
        let runtime = ctx.config().runtime().clone().with_explicit_lints(idents);
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

    executor
        .run()
        .map(|report| report.with_error_on_all(cmd.check.error_on_all))
}
