use std::io::Write;

use crate::{Command, HEMTTError, Project};

pub struct Init {}

impl Command for Init {
    fn register(&self) -> clap::App {
        clap::SubCommand::with_name("init").about("Initialize a HEMTT Project")
    }

    fn require_project(&self) -> bool {
        false
    }

    fn run_no_project(&self, _: &clap::ArgMatches) -> Result<(), HEMTTError> {
        let name = ask!("Project Name >");
        let prefix = ask!("Prefix >");
        let author = ask!("Author >");
        let template = ask!("Template >", "cba");

        // Create settings file in TOML
        create_dir!("./.hemtt/")?;
        let project = Project::new(name, prefix, author, template.clone());
        let mut out = create_file!("./.hemtt/base.toml")?;
        out.write_fmt(format_args!("{}", toml::to_string(&project)?))?;

        // clone template
        match template.as_ref() {
            "" => {}
            _ => {
                let repo = if template.starts_with("http") {
                    template
                } else {
                    format!("https://github.com/hemtt/{}", template)
                };
                match git2::Repository::clone(&repo, "./.hemtt/template") {
                    Ok(_) => println!("Template Cloned"),
                    Err(e) => panic!("Failed to clone: {}", e),
                };
            }
        }
        Ok(())
    }
}
