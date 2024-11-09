use crate::{context::Context, error::Error, modules::Hooks, report::Report};

#[derive(clap::Parser)]
#[command(arg_required_else_help = true)]
/// Run a Rhai script on the project
///
/// Run a Rhai script on the project, this is useful for automating tasks
/// in a platform agnostic way, or requiring external dependencies.
pub struct Command {
    #[clap(name = "name")]
    name: String,
}

/// Execute the script command
///
/// # Errors
/// [`Error`] depending on the modules
///
/// # Panics
/// If a name is not provided, but this is usually handled by clap
pub fn execute(cmd: &Command) -> Result<Report, Error> {
    let ctx = Context::new(
        Some("script"),
        crate::context::PreservePrevious::Remove,
        true,
    )?;
    Hooks::run_file(&ctx, &cmd.name).map(|(report, _)| report)
}
