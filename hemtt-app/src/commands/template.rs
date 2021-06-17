use crate::Command;
use hemtt::{templates::Templates, Addon, AddonLocation, HEMTTError, Project};

use clap::{App, Arg, ArgMatches, SubCommand};

pub struct Template;
impl Command for Template {
    fn register(&self) -> App {
        SubCommand::with_name("template")
            .version(*crate::VERSION)
            .about("Manage the project template")
            .subcommand(
                SubCommand::with_name("init")
                    .about("Initialize a template")
                    .arg(
                        Arg::with_name("template")
                            .required(true)
                            .validator(Templates::validate),
                    ),
            )
            .subcommand(
                SubCommand::with_name("addon")
                    .about("Create a new addon")
                    .arg(Arg::with_name("name").required(true))
                    .arg(
                        Arg::with_name("location")
                            .required(false)
                            .validator(AddonLocation::validate)
                            .default_value("addons"),
                    )
                    .arg(
                        Arg::with_name("no-handlebars")
                            .long("no-handlebars")
                            .required(false)
                            .takes_value(false),
                    ),
            )
            .subcommand(
                SubCommand::with_name("function")
                    .about("Create a new function")
                    .arg(Arg::with_name("addon").required(true))
                    .arg(Arg::with_name("name").required(true)),
            )
    }

    fn require_project(&self) -> bool {
        false
    }

    fn run_no_project(&self, a: &ArgMatches) -> Result<(), HEMTTError> {
        if let ("init", Some(b)) = a.subcommand() {
            return match b.value_of("template").unwrap() {
                "cba" => hemtt::templates::init(
                    hemtt::templates::Templates::CBA,
                    std::env::current_dir()?,
                ),
                unknown => Err(HEMTTError::TemplateUnknown(unknown.to_string())),
            };
        }
        let p = Project::read()?;
        let template: Box<dyn hemtt::Template> = match p.template().to_lowercase().as_str() {
            "cba" => Box::new(hemtt::templates::cba::CBA::new(hemtt::Project::find_root()?)),
            _ => return Err(HEMTTError::TemplateUnknown(p.template().to_string())),
        };
        match a.subcommand() {
            ("addon", Some(b)) => {
                let location = AddonLocation::from(b.value_of("location").unwrap());
                let name = b.value_of("name").unwrap().to_string();
                debug!("Creating addon `{}` in location `{:?}`", name, location);
                if let Some(existing) = Addon::locate(&name)? {
                    return Err(HEMTTError::AddonConflict(
                        name,
                        location,
                        existing.location(),
                    ));
                }
                template.new_addon(&Addon::new(name.clone(), location)?)?;
                info!("Addon `{}` created in {}", name, location.to_string());
                Ok(())
            }
            ("function", Some(b)) => {
                let name = b.value_of("name").unwrap().to_string();
                let addon = b.value_of("addon").unwrap().to_string();
                if let Some(addon) = Addon::locate(&addon)? {
                    template.new_function(&addon, &name)?;
                    Ok(())
                } else {
                    panic!("addon not found");
                }
            }
            ("", _) => Err(HEMTTError::User(String::from(
                "No command was provided, use `template help` to see all commands and options",
            ))),
            _ => Err(HEMTTError::User(String::from(
                "No command was provided, use `template help` to see all commands and options",
            ))),
        }
    }
}
