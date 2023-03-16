use clap::{ArgAction, ArgMatches, Command};

use crate::{
    addons::Location,
    context::Context,
    error::Error,
    executor::Executor,
    modules::{pbo::Collapse, Binarize, FilePatching, Files, Hooks, Lint, Rapifier},
};

#[must_use]
pub fn cli() -> Command {
    add_args(
        Command::new("dev")
            .about("Mod Development")
            .long_about("Build your mod for local development and testing."),
    )
}

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
}

pub fn execute(matches: &ArgMatches) -> Result<Context, Error> {
    let all_optionals = matches.get_one::<bool>("optionals") == Some(&true);
    let optionals = matches
        .get_many::<String>("optional")
        .unwrap_or_default()
        .map(std::string::String::as_str)
        .collect::<Vec<_>>();

    let ctx = Context::new("dev")?.filter(|a, config| {
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
            return Err(Error::AddonOptionalNotFound(optional.to_owned()));
        }
    }

    let mut executor = Executor::new(&ctx);

    executor.collapse(Collapse::Yes);

    executor.add_module(Box::<Lint>::default());
    executor.add_module(Box::<Hooks>::default());
    executor.add_module(Box::<Rapifier>::default());
    executor.add_module(Box::<Files>::default());
    executor.add_module(Box::<FilePatching>::default());
    if matches.get_one::<bool>("binarize") == Some(&true) {
        executor.add_module(Box::<Binarize>::default());
    }

    info!("Creating `dev` version");

    executor.init()?;
    executor.check()?;
    executor.build()?;

    Ok(ctx)
}
