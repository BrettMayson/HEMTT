use crate::Error;

mod json;

#[derive(clap::Parser)]
#[command(arg_required_else_help = true)]
/// Commands for p3d files
pub struct Command {
    #[command(subcommand)]
    commands: Subcommands,
}

#[derive(clap::Subcommand)]
enum Subcommands {
    Json(json::JsonArgs),
}

/// Execute the p3d command
///
/// # Errors
/// [`Error`] depending on the modules
///
/// # Panics
/// If the args are not present from clap
pub fn execute(cmd: &Command) -> Result<(), Error> {
    match &cmd.commands {
        Subcommands::Json(args) => json::execute(args),
    }
}
