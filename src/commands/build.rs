use std::path::Path;

use crate::project::Project;
use crate::HEMTTError;
use crate::build;
use crate::flow::Flow;

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

    fn run(&self, _: &clap::ArgMatches, mut p: Project) -> Result<(), HEMTTError> {
        let mut addons = build::get_addons(build::AddonLocation::Addons)?;
        if Path::new(&build::folder_name(&build::AddonLocation::Optionals)).exists() {
            addons.extend(build::get_addons(build::AddonLocation::Optionals)?);
        }
        if Path::new(&build::folder_name(&build::AddonLocation::Compats)).exists() {
            addons.extend(build::get_addons(build::AddonLocation::Compats)?);
        }
        for addon in &addons {
            println!("- {} {:?}", addon.name, addon.location);
        }
        let flow = Flow {
            checks: vec![
                Box::new(crate::build::prebuild::render::Render {}),
                Box::new(crate::build::checks::names::NotEmpty {}),
                Box::new(crate::build::checks::names::ValidName {}),
            ],
            pre_build: vec![
                Box::new(crate::build::prebuild::preprocess::Preprocess {}),
            ],
            post_build: vec![],
            release: vec![],
        };
        flow.execute(addons, &mut p)?;
        Ok(())
    }
}
