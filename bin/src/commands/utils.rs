use crate::{Error, report::Report, utils};

#[derive(clap::Parser)]
#[command(arg_required_else_help = true)]
/// Use HEMTT standalone utils
pub struct Command {
    #[command(subcommand)]
    commands: Subcommands,

    #[clap(flatten)]
    global: crate::GlobalArgs,
}

#[derive(clap::Subcommand)]
enum Subcommands {
    Audio(utils::audio::Command),
    Bom(utils::bom::Command),
    Config(utils::config::Command),
    Fnl(utils::fnl::Command),
    Inspect(utils::inspect::Command),
    P3d(utils::p3d::Command),
    Paa(utils::paa::Command),
    Pbo(utils::pbo::Command),
    Sqf(utils::sqf::Command),
    Verify(utils::verify::Command),
}

/// Execute the utils command
///
/// # Errors
/// [`Error`] depending on the modules
pub fn execute(cmd: &Command) -> Result<Report, Error> {
    match &cmd.commands {
        Subcommands::Audio(cmd) => {
            utils::audio::execute(cmd)?;
        }
        Subcommands::Bom(cmd) => {
            utils::bom::execute(cmd)?;
        }
        Subcommands::Config(cmd) => {
            utils::config::execute(cmd)?;
        }
        Subcommands::Fnl(cmd) => {
            utils::fnl::execute(cmd)?;
        }
        Subcommands::Inspect(cmd) => {
            utils::inspect::execute(cmd)?;
        }
        Subcommands::P3d(cmd) => {
            utils::p3d::execute(cmd)?;
        }
        Subcommands::Paa(cmd) => {
            utils::paa::execute(cmd)?;
        }
        Subcommands::Pbo(cmd) => {
            utils::pbo::execute(cmd)?;
        }
        Subcommands::Sqf(cmd) => {
            utils::sqf::execute(cmd)?;
        }
        Subcommands::Verify(cmd) => {
            utils::verify::execute(cmd)?;
        }
    }
    Ok(Report::new())
}
