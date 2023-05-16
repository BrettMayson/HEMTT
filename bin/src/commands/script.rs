use clap::{ArgAction, ArgMatches, Command};

use crate::{addons::Location, context::Context, error::Error, modules::Hooks};

#[must_use]
pub fn cli() -> Command {
    add_args(
        Command::new("script")
            .about("Run a Rhai script on the project")
            .long_about("Run a Rhai script on the project, this is useful for automating tasks in a platform agnostic way, or requiring external dependencies.").arg(
                clap::Arg::new("name")
                    .help("Name of the new mod")
                    .required(true),
            ),
    )
}

pub fn add_args(cmd: Command) -> Command {
    cmd.arg(
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

pub fn execute(matches: &ArgMatches) -> Result<(), Error> {
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

    let name = matches.get_one::<String>("name").unwrap();
    Hooks::run_file(&ctx, name)?;

    Ok(())
}
