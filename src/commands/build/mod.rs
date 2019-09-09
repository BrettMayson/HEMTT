use std::path::Path;

pub mod addon;
#[allow(clippy::module_inception)]
pub mod build;
pub mod checks;
pub mod postbuild;
pub mod prebuild;

use crate::{Addon, AddonLocation, Command, Flow, HEMTTError, Project, Step};

pub struct Build {}
impl Command for Build {
    fn register(&self) -> clap::App {
        clap::SubCommand::with_name("build")
            .about("Build the Project")
            .arg(
                clap::Arg::with_name("release")
                    .help("Build a release")
                    .long("release")
                    .conflicts_with("dev"),
            )
            .arg(
                clap::Arg::with_name("rebuild")
                    .help("Rebuild existing files")
                    .long("rebuild")
                    .long("force"),
            )
    }

    fn run(&self, args: &clap::ArgMatches, mut p: Project) -> Result<(), HEMTTError> {
        let mut addons = crate::build::get_addons(AddonLocation::Addons)?;
        if Path::new(&AddonLocation::Optionals.to_string()).exists() {
            addons.extend(crate::build::get_addons(AddonLocation::Optionals)?);
        }
        if Path::new(&AddonLocation::Compats.to_string()).exists() {
            addons.extend(crate::build::get_addons(AddonLocation::Compats)?);
        }
        let flow = Flow {
            steps: vec![
                Step::single("♻️", "Clean", vec![Box::new(crate::build::prebuild::clear::Clean {})]),
                if args.is_present("rebuild") {
                    Step::parallel("🗑️", "Clear", vec![Box::new(crate::build::prebuild::clear::Clear {})])
                } else {
                    Step::none()
                },
                Step::parallel(
                    "🔍",
                    "Checks",
                    vec![
                        Box::new(crate::build::prebuild::render::Render {}),
                        Box::new(crate::build::checks::names::NotEmpty {}),
                        Box::new(crate::build::checks::names::ValidName {}),
                        Box::new(crate::build::checks::modtime::ModTime {}),
                    ],
                ),
                Step::parallel(
                    "🚧",
                    "Prebuild",
                    vec![Box::new(crate::build::prebuild::preprocess::Preprocess {})],
                ),
                Step::parallel("📝", "Build", vec![Box::new(crate::build::build::Build { use_bin: true })]),
                if args.is_present("release") {
                    Step::single("⭐", "Release", vec![Box::new(crate::build::postbuild::release::Release {})])
                } else {
                    Step::none()
                },
                if args.is_present("release") {
                    Step::single("⭐", "Sign", vec![Box::new(crate::build::postbuild::sign::Sign {})])
                } else {
                    Step::none()
                },
            ],
        };
        flow.execute(addons, &mut p)?;
        Ok(())
    }
}

pub fn get_addons(location: AddonLocation) -> Result<Vec<Addon>, HEMTTError> {
    Ok(std::fs::read_dir(&location.to_string())?
        .map(|file| file.unwrap().path())
        .filter(|file_or_dir| file_or_dir.is_dir())
        .map(|file| Addon {
            name: file.file_name().unwrap().to_str().unwrap().to_owned(),
            location: location.clone(),
        })
        .collect())
}
