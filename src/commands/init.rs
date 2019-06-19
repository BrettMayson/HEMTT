use std::fs;
use std::fs::File;
use std::io::Write;

use crate::project::Project;
use crate::error::HEMTTError;

pub struct Init {}

impl crate::commands::Command for Init {
    fn register(&self) -> (&str, clap::App) {
        ("init",
            clap::SubCommand::with_name("init")
                .about("Initialize a HEMTT Project")
        )
    }

    fn require_project(&self) -> bool { false }

    fn run_no_project(&self, _: &clap::ArgMatches) -> Result<(), HEMTTError> {
        let name = ask!("Project Name >");
        let prefix = ask!("Prefix >");
        let author = ask!("Author >");
        let template = ask!("Template >", "cba");

        // Create settings file in TOML
        fs::create_dir_all("./hemtt/")?;
        let project = Project {
            name, prefix, author, template: template.clone()
        };
        let mut out = File::create("./hemtt/dev.toml")?;
        out.write_fmt(format_args!("{}", toml::to_string(&project)?))?;

        // clone template
        match template.as_ref() {
            "none" => {},
            _ => {
                let repo = if template.starts_with("http") {
                    template
                } else {
                    format!("https://github.com/hemtt/{}", template)
                };
                match git2::Repository::clone(&repo, "./hemtt/template") {
                    Ok(_) => println!("Template Cloned"),
                    Err(e) => panic!("Failed to clone: {}", e)
                };
            }
        }
        Ok(())
    }
}