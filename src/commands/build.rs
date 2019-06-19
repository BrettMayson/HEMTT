use std::fs;
use std::fs::File;
use std::io::Write;

use crate::project::Project;
use crate::error::HEMTTError;
use crate::build;

pub struct Build {}

impl crate::commands::Command for Build {
    fn register(&self) -> (&str, clap::App) {
        ("build",
            clap::SubCommand::with_name("build")
                .about("Build the Project")
                .arg(clap::Arg::with_name("release")
                        .help("Build a release")
                        .long("release")
                        .conflicts_with("dev"))
        )
    }

    fn run(&self, _: &clap::ArgMatches, p: Project) -> Result<(), HEMTTError> {
        let addons = build::get_addons(build::AddonLocation::Addons)?;
        for addon in addons {
            println!("- {}", addon.name);
        }
        Ok(())
    }
}
