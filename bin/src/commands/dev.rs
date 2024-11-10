use hemtt_workspace::addons::Location;

use crate::{
    commands::global_modules,
    context::Context,
    error::Error,
    executor::Executor,
    modules::{pbo::Collapse, Binarize, FilePatching, Files, Rapifier},
    report::Report,
};

use super::JustArgs;

#[derive(clap::Parser)]
/// Build the project for development
///
/// Build your project for local development and testing.
/// It is built without binarization of .p3d and .rtm files.
pub struct Command {
    #[clap(flatten)]
    dev: DevArgs,

    #[clap(flatten)]
    just: JustArgs,

    #[clap(flatten)]
    global: crate::GlobalArgs,
}

#[derive(clap::Args)]
#[allow(clippy::module_name_repetitions)]
pub struct DevArgs {
    #[arg(long, short, action = clap::ArgAction::SetTrue)]
    /// Use BI's binarize on supported files
    binarize: bool,
    #[arg(long = "optional", short, action = clap::ArgAction::Append)]
    /// Include an optional addon folder
    optionals: Vec<String>,
    #[arg(long, short = 'O', action = clap::ArgAction::SetTrue)]
    /// Include all optional addon folders
    all_optionals: bool,
    #[arg(long, action = clap::ArgAction::SetTrue)]
    /// Do not rapify (cpp, rvmat)
    no_rap: bool,
}

/// Execute the dev command
///
/// # Errors
/// [`Error`] depending on the modules
pub fn execute(cmd: &Command, launch_optionals: &[String]) -> Result<Report, Error> {
    let mut executor = context(&cmd.dev, &cmd.just, launch_optionals, false, true)?;
    executor.run()
}

/// Create a new executor for the dev command
///
/// # Errors
/// [`Error`] depending on the modules
pub fn context(
    dev: &DevArgs,
    just: &JustArgs,
    launch_optionals: &[String],
    force_binarize: bool,
    rapify: bool,
) -> Result<Executor, Error> {
    let all_optionals = dev.all_optionals;
    let optionals = dev
        .optionals
        .iter()
        .map(std::string::String::as_str)
        .collect::<Vec<_>>();

    let just = just
        .just
        .iter()
        .map(|s| s.to_lowercase())
        .collect::<Vec<_>>();

    let ctx = Context::new(
        Some("dev"),
        if just.is_empty() {
            crate::context::PreservePrevious::Remove
        } else {
            warn!("keeping previous build artifacts");
            crate::context::PreservePrevious::Keep
        },
        true,
    )?
    .filter(|a, config| {
        if !just.is_empty() && !just.contains(&a.name().to_lowercase()) {
            return false;
        }
        if launch_optionals.iter().any(|o| o == a.name()) {
            return true;
        }
        if a.location() == &Location::Optionals && !all_optionals && !optionals.contains(&a.name())
        {
            debug!("ignoring optional {}", a.name());
            return false;
        }
        !config
            .hemtt()
            .dev()
            .exclude()
            .iter()
            .any(|e| (a.folder() + "/").starts_with(&format!("{e}/")))
    });

    for optional in optionals {
        if !ctx.addons().iter().any(|a| a.name() == optional) {
            return Err(Error::Addon(
                hemtt_workspace::addons::Error::OptionalNotFound(optional.to_owned()),
            ));
        }
    }

    let mut executor = Executor::new(ctx);
    global_modules(&mut executor);

    executor.collapse(Collapse::Yes);

    if rapify && !dev.no_rap {
        executor.add_module(Box::<Rapifier>::default());
    }
    executor.add_module(Box::<Files>::default());
    executor.add_module(Box::<FilePatching>::default());
    if force_binarize || dev.binarize {
        executor.add_module(Box::<Binarize>::default());
    }

    info!("Creating `dev` version");

    executor.init();
    executor.check();
    executor.build(true);

    if !just.is_empty() {
        warn!("Use of `--just` is not recommended, only use it if you know what you're doing");
    }

    Ok(executor)
}
