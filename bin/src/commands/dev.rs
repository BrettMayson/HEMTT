use clap::{ArgAction, ArgMatches, Command};
use hemtt_workspace::addons::Location;

use crate::{
    commands::global_modules,
    context::Context,
    error::Error,
    executor::Executor,
    modules::{pbo::Collapse, Binarize, FilePatching, Files, Rapifier},
    report::Report,
};

use super::build::add_just;

#[must_use]
pub fn cli() -> Command {
    add_just(add_args(
        Command::new("dev")
            .about("Build the project for development")
            .long_about("Build your project for local development and testing. It is built without binarization of .p3d and .rtm files."),
    ))
}

#[must_use]
pub fn add_args(cmd: Command) -> Command {
    cmd.arg(
        clap::Arg::new("binarize")
            .long("binarize")
            .short('b')
            .help("Use BI's binarize on supported files")
            .action(ArgAction::SetTrue),
    )
    .arg(
        clap::Arg::new("optional")
            .long("optional")
            .short('o')
            .help("Include an optional addon folder")
            .action(ArgAction::Append),
    )
    .arg(
        clap::Arg::new("optionals")
            .long("all-optionals")
            .short('O')
            .help("Include all optional addon folders")
            .action(ArgAction::SetTrue),
    )
    .arg(
        clap::Arg::new("no-rap")
            .long("no-rap")
            .help("Do not rapify (cpp, rvmat)")
            .action(ArgAction::SetTrue),
    )
}

/// Execute the dev command
///
/// # Errors
/// [`Error`] depending on the modules
pub fn execute(matches: &ArgMatches, launch_optionals: &[String]) -> Result<Report, Error> {
    let mut executor = context(matches, launch_optionals, false, true)?;
    executor.run()
}

/// Create a new executor for the dev command
///
/// # Errors
/// [`Error`] depending on the modules
pub fn context(
    matches: &ArgMatches,
    launch_optionals: &[String],
    force_binarize: bool,
    rapify: bool,
) -> Result<Executor, Error> {
    let all_optionals = matches.get_one::<bool>("optionals") == Some(&true);
    let optionals = matches
        .get_many::<String>("optional")
        .unwrap_or_default()
        .map(std::string::String::as_str)
        .collect::<Vec<_>>();

    let just = matches
        .get_many::<String>("just")
        .unwrap_or_default()
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

    if rapify && matches.get_one::<bool>("no-rap") != Some(&true) {
        executor.add_module(Box::<Rapifier>::default());
    }
    executor.add_module(Box::<Files>::default());
    executor.add_module(Box::<FilePatching>::default());
    if force_binarize || matches.get_one::<bool>("binarize") == Some(&true) {
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
