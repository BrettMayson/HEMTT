use std::path::Path;

use crate::{AddonLocation, Command, Flow, HEMTTError, Project, Step};

pub struct Pack {}
impl Command for Pack {
    fn register(&self) -> clap::App {
        clap::SubCommand::with_name("pack")
            .about("Pack the Project")
            .arg(clap::Arg::with_name("release")
                    .help("Pack a release")
                    .long("release")
                    .conflicts_with("dev"))
            .arg(clap::Arg::with_name("clear")
                    .help("Clears existing built files")
                    .long("clear")
                    .long("force"))
    }

    fn run(&self, args: &clap::ArgMatches, mut p: Project) -> Result<(), HEMTTError> {
        let mut addons = crate::build::get_addons(AddonLocation::Addons)?;
        if Path::new(&crate::build::addon::folder_name(&AddonLocation::Optionals)).exists() {
            addons.extend(crate::build::get_addons(AddonLocation::Optionals)?);
        }
        if Path::new(&crate::build::addon::folder_name(&AddonLocation::Compats)).exists() {
            addons.extend(crate::build::get_addons(AddonLocation::Compats)?);
        }
        let flow = Flow {
            steps: vec![
                if args.is_present("clear") {
                    Step::parallel("🗑️", "Clear",
                    vec![
                        Box::new(crate::build::prebuild::clear::Clear {}),
                    ])
                } else {
                    Step::single("♻️", "Clean",
                    vec![
                        Box::new(crate::build::prebuild::clear::Clean {}),
                    ])
                },
                Step::parallel("🔍", "Checks",
                    vec![
                        Box::new(crate::build::prebuild::render::Render {}),
                        Box::new(crate::build::checks::names::NotEmpty {}),
                        Box::new(crate::build::checks::names::ValidName {}),
                        Box::new(crate::build::checks::modtime::ModTime {}),
                    ],
                ),
                Step::parallel("📦", "Pack",
                    vec![
                        Box::new(crate::build::build::Build { use_bin: false }),
                    ],
                ),
            ],
        };
        flow.execute(addons, &mut p)?;
        Ok(())
    }
}
