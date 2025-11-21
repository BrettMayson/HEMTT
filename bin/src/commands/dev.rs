use hemtt_workspace::addons::Location;

use crate::{
    commands::global_modules,
    context::Context,
    error::Error,
    executor::Executor,
    modules::{
        Binarize, FilePatching, Files, Rapifier, pbo::Collapse, summary::Summary,
        tex_headers::TexHeaders,
    },
    report::Report,
};

use super::JustArgs;

#[derive(clap::Parser)]
#[command(verbatim_doc_comment)]
/// Build the project for development
///
/// `hemtt dev` is designed to help your development workflows.
/// It will build your mod into `.hemttout/dev`, with links back
/// to the original addon folders. This allows you to use
/// file-patching with optional mods for easy development.
///
/// ## Configuration
///
/// ```toml,fp=.hemtt/project.toml
/// [hemtt.dev]
/// exclude = ["addons/unused"]
/// ```
///
/// ### exclude
///
/// A list of addons to exclude from the development build.
/// Includes from excluded addons can be used, but they will not be built or linked.
pub struct Command {
    #[clap(flatten)]
    pub(crate) dev: DevArgs,

    #[clap(flatten)]
    pub(crate) binarize: BinarizeArgs,

    #[clap(flatten)]
    pub(crate) just: JustArgs,

    #[clap(flatten)]
    pub(crate) global: crate::GlobalArgs,
}

#[derive(clap::Args)]
#[allow(clippy::module_name_repetitions)]
pub struct DevArgs {
    #[arg(long, short, action = clap::ArgAction::Append, verbatim_doc_comment)]
    /// Include an optional addon folder
    ///
    /// This can be used multiple times to include multiple optional addons.
    /// Optional addons are stored in the `optionals/` directory and can be used
    /// to separate compatibility patches or features that not all users need.
    ///
    /// ```bash
    /// hemtt dev -o caramel -o chocolate
    /// ```
    pub(crate) optional: Vec<String>,
    #[arg(long, short = 'O', action = clap::ArgAction::SetTrue, conflicts_with = "optional", verbatim_doc_comment)]
    /// Include all optional addon folders
    ///
    /// Builds all addons from the `optionals/` directory in addition to main addons.
    pub(crate) all_optionals: bool,
    #[arg(long, action = clap::ArgAction::SetTrue, verbatim_doc_comment)]
    /// Do not rapify (cpp, rvmat, ext, sqm, bikb, bisurf)
    ///
    /// They will be copied directly into the PBO, not .bin version is created.
    pub(crate) no_rap: bool,
}

#[derive(Clone, clap::Args)]
pub struct BinarizeArgs {
    #[arg(long, short, action = clap::ArgAction::SetTrue, verbatim_doc_comment)]
    /// Use BI's binarize on supported files
    ///
    /// By default, `hemtt dev` will not binarize any files, but rather pack them as-is.
    /// Binarization is often not needed for development.
    pub(crate) binarize: bool,
}

/// Execute the dev command
///
/// # Errors
/// [`Error`] depending on the modules
pub fn execute(
    cmd: &Command,
    launch_optionals: &[String],
    force_binarize: bool,
) -> Result<(Report, Context), Error> {
    let mut executor = context(
        &cmd.dev,
        &cmd.binarize,
        &cmd.just,
        launch_optionals,
        force_binarize,
        true,
    )?;
    executor.run().map(|r| (r, executor.into_ctx()))
}

/// Create a new executor for the dev command
///
/// # Errors
/// [`Error`] depending on the modules
pub fn context(
    dev: &DevArgs,
    binarize: &BinarizeArgs,
    just: &JustArgs,
    launch_optionals: &[String],
    force_binarize: bool,
    rapify: bool,
) -> Result<Executor, Error> {
    let all_optionals = dev.all_optionals;
    let optionals = dev
        .optional
        .iter()
        .map(std::string::String::as_str)
        .collect::<Vec<_>>();

    let just = just
        .just
        .iter()
        .map(|s| s.to_lowercase())
        .collect::<Vec<_>>();

    let mut ctx = Context::new(
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

    if !just.is_empty() {
        let runtime = ctx.config().runtime().clone().with_just(true);
        let config = ctx.config().clone().with_runtime(runtime);
        ctx = ctx.with_config(config);
    }

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
    executor.add_module(Box::<TexHeaders>::default());
    executor.add_module(Box::<Files>::default());
    executor.add_module(Box::<FilePatching>::default());
    if force_binarize || binarize.binarize {
        executor.add_module(Box::<Binarize>::default());
    }
    executor.add_module(Box::<Summary>::default());

    info!("Creating `dev` version");

    executor.init();
    executor.check();
    executor.build(true);

    if !just.is_empty() {
        warn!("Use of `--just` is not recommended, only use it if you know what you're doing");
    }

    Ok(executor)
}
