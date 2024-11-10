use crate::{context::Context, error::Error, modules::Sign, report::Report};

use super::build;

#[derive(clap::Parser)]
/// Build the project for release
///
/// Build your project for full release, with signing and archiving.
pub struct Command {
    #[clap(flatten)]
    build: build::BuildArgs,

    #[clap(flatten)]
    release: ReleaseArgs,

    #[clap(flatten)]
    global: crate::GlobalArgs,
}

#[derive(clap::Args)]
#[allow(clippy::module_name_repetitions)]
pub struct ReleaseArgs {
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
