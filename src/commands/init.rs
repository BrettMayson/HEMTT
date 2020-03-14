use std::io::{Read, Write};

use crate::{Command, HEMTTError, Project};

pub struct Init {}

impl Command for Init {
    fn register(&self) -> clap::App {
        clap::SubCommand::with_name("init")
            .version(*crate::VERSION)
            .about("Initialize a HEMTT Project")
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

        // Git Ignore
        let git_ignore = std::path::Path::new(".gitignore");
        let mut ignore = crate::GIT_IGNORE.to_vec();
        let mut file = if git_ignore.exists() {
            let mut data = String::new();
            open_file!(git_ignore)?.read_to_string(&mut data)?;
            for l in data.lines() {
                if let Some(index) = ignore.iter().position(|&d| d == l) {
                    ignore.remove(index);
                }
            }
            warn!("Adding recommended paths to `.gitignore`");
            std::fs::OpenOptions::new().append(true).open(git_ignore)?
        } else {
            create_file!(git_ignore)?
        };
        file.write_all(String::from("\n## Added by HEMTT\n").as_bytes())?;
        for i in ignore {
            file.write_all(format!("{}\n", i).as_bytes())?;
        }
        file.write_all(String::from("####\n").as_bytes())?;

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
