use crate::Command;
use hemtt::{Addon, AddonLocation, HEMTTError, Project};

pub struct Template;
impl Command for Template {
    fn register(&self) -> clap::App {
        clap::SubCommand::with_name("template")
            .version(*crate::VERSION)
            .about("Manage the project template")
            .subcommand(
                clap::SubCommand::with_name("addon")
                    .about("Create a new addon")
                    .arg(clap::Arg::with_name("name").required(true))
                    .arg(
                        clap::Arg::with_name("location")
                            .required(false)
                            .validator(AddonLocation::validate)
                            .default_value("addons"),
                    )
                    .arg(
                        clap::Arg::with_name("no-handlebars")
                            .long("no-handlebars")
                            .required(false)
                            .takes_value(false),
                    ),
            )
            .subcommand(
                clap::SubCommand::with_name("function")
                    .about("Create a new function")
                    .arg(clap::Arg::with_name("addon").required(true))
                    .arg(clap::Arg::with_name("name").required(true)),
            )
    }

    fn run(&self, a: &clap::ArgMatches, p: Project) -> Result<(), HEMTTError> {
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
