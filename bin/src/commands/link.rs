use crate::{context::Context, error::Error, modules::FilePatching, report::Report};

#[derive(clap::Parser)]
#[command(verbatim_doc_comment)]
/// Manage the project's symbolic link for file patching
///
/// Creates or removes a symbolic link in the Arma 3 directory for file patching.
/// By default, creates a link.
pub struct Command {
    #[command(subcommand)]
    subcommand: Option<Subcommand>,
}

#[derive(clap::Subcommand)]
enum Subcommand {
    /// Create a symbolic link in the Arma 3 directory for file patching
    Create,
    /// Remove the symbolic link from the Arma 3 directory
    Remove,
}

/// Execute the link command
///
/// # Errors
/// [`Error`] depending on the modules
///
/// # Panics
/// If a name is not provided, but this is usually handled by clap
pub fn execute(cmd: &Command) -> Result<Report, Error> {
    let subcommand = match &cmd.subcommand {
        Some(Subcommand::Create) | None => Subcommand::Create,
        Some(Subcommand::Remove) => Subcommand::Remove,
    };

    let file_patching = FilePatching::default();
    let ctx = Context::new(None, crate::context::PreservePrevious::Keep, true)?;

    match subcommand {
        Subcommand::Create => file_patching.create(&ctx),
        Subcommand::Remove => file_patching.remove(&ctx),
    }
}
