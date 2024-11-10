use crate::{context::Context, error::Error, modules::Hooks, report::Report};

#[derive(clap::Parser)]
#[command(arg_required_else_help = true, verbatim_doc_comment)]
/// Run a Rhai script on the project
///
/// `hemtt script` is used to run a Rhai script on the project
/// This is useful for automating tasks in a platform agnostic way,
/// or requiring external dependencies.
///
/// Learn more about [Scripts](../rhai/scripts).
pub struct Command {
    #[clap(name = "name")]
    /// The name of the script to run, without .rhai
    ///
    /// Scripts are kept in `.hemtt/scripts/`
    name: String,

    #[clap(flatten)]
    global: crate::GlobalArgs,
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
