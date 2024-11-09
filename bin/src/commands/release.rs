use crate::{context::Context, error::Error, modules::Sign, report::Report};

use super::build;

#[derive(clap::Parser)]
#[command(long_about = "Build your project for full release, with signing and archiving.")]
/// Build the project for release
pub struct Command {
    #[clap(flatten)]
    build: build::Args,

    #[clap(flatten)]
    release: Args,
}

#[derive(clap::Args)]
pub struct Args {
    #[arg(long, action = clap::ArgAction::SetTrue)]
    /// Do not sign the PBOs
    no_sign: bool,
    #[arg(long, action = clap::ArgAction::SetTrue)]
    /// Do not create an archive of the release
    no_archive: bool,
}

/// Execute the release command
///
/// # Errors
/// [`Error`] depending on the modules
pub fn execute(cmd: &Command) -> Result<Report, Error> {
    let ctx = Context::new(
        Some("release"),
        crate::context::PreservePrevious::Remove,
        true,
    )?;
    let mut executor = build::executor(ctx, &cmd.build);

    if !cmd.release.no_sign && executor.ctx().config().hemtt().release().sign() {
        executor.add_module(Box::new(Sign::new()));
    }

    let archive = if cmd.release.no_archive {
        false
    } else {
        executor.ctx().config().hemtt().release().archive()
    };

    executor.release(archive);

    executor.run()
}
