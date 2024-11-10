use crate::{
    commands::global_modules,
    context::Context,
    error::Error,
    executor::Executor,
    modules::{pbo::Collapse, Binarize, Rapifier},
    report::Report,
};

#[derive(clap::Parser)]
/// Check the project for errors
pub struct Command {
    #[clap(flatten)]
    global: crate::GlobalArgs,
}

/// Execute the dev command
///
/// # Errors
/// [`Error`] depending on the modules
pub fn execute(_: &Command) -> Result<Report, Error> {
    let ctx = Context::new(
        Some("check"),
        crate::context::PreservePrevious::Remove,
        true,
    )?;

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
