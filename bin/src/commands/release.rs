use crate::{context::Context, error::Error, modules::Sign, report::Report};

use super::build;

#[derive(clap::Parser)]
#[command(verbatim_doc_comment)]
/// Build the project for release
///
/// `hemtt release` will build your mod into `.hemttout/release`.
/// It will create `bisign` files for all addons, and a `bikey` for validation.
///
/// It is intended to be used for releasing your mod.
///
/// It will create two zip archives in the `releases` folder: - `{name}-latest.zip` - `{name}-{version}.zip`
///
/// ## Configuration
///
/// `hemtt release` is built the same way as [`hemtt build`](build.md), and will use its configuration.
///
/// ```toml
/// [hemtt.release]
/// sign = false # Default: true
/// archive = false # Default: true
/// ```
///
/// ### sign
///
/// If `sign` is set to `false`, a `bikey` will not be created, and the PBOs will not be signed.
///
/// ```admonish danger
/// All public releases of your mods should be signed. This will be a requirement of
/// many communities, and is an important security feature. Do not use this
/// unless you know what you are doing.
/// ```
///
/// ### archive
///
/// If `archive` is set to `false`, a zip archive will not be created. The output will be in `.hemttout/release`.
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
    #[arg(long, action = clap::ArgAction::SetTrue, verbatim_doc_comment)]
    /// Do not sign the PBOs or create a `bikey`.
    ///
    /// ```admonish danger
    /// All public releases of your mods should be signed. This will be a requirement of
    /// many communities, and is an important security feature. Do not use this
    /// unless you know what you are doing.
    /// ```
    no_sign: bool,
    #[arg(long, action = clap::ArgAction::SetTrue)]
    /// Do not create a zip archive of the release.
    ///
    /// The output will be in `.hemttout/release`.
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
