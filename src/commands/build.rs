use std::path::Path;

use crate::{AddonLocation, Command, Project, HEMTTError, Flow, Step};

pub struct Build {}
impl Command for Build {
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
        let mut addons = crate::build::get_addons(AddonLocation::Addons)?;
        if Path::new(&crate::build::folder_name(&AddonLocation::Optionals)).exists() {
            addons.extend(crate::build::get_addons(AddonLocation::Optionals)?);
        }
        if Path::new(&crate::build::folder_name(&AddonLocation::Compats)).exists() {
            addons.extend(crate::build::get_addons(AddonLocation::Compats)?);
        }
        let flow = Flow {
            steps: vec![
                Step::new("🔍", "Checks",
                    vec![
                        Box::new(crate::build::prebuild::render::Render {}),
                        Box::new(crate::build::checks::names::NotEmpty {}),
                        Box::new(crate::build::checks::names::ValidName {}),
                        Box::new(crate::build::prebuild::modtime::ModTime {}),
                    ],
                ),
                Step::new("🚧", "Prebuild",
                    vec![
                        Box::new(crate::build::prebuild::preprocess::Preprocess {}),
                    ],
                ),
                Step::new("📝", "Build",
                    vec![
                        Box::new(crate::build::build::Build { use_bin: true }),
                    ],
                ),
            ],
        };
        flow.execute(addons, &mut p)?;
        Ok(())
    }
}
