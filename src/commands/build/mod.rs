use std::path::Path;

pub mod addon;
#[allow(clippy::module_inception)]
pub mod build;
pub mod checks;
pub mod postbuild;
pub mod prebuild;

use crate::{Addon, AddonLocation, Command, Flow, HEMTTError, Project, Stage, Step};

pub struct Build {}
impl Command for Build {
    fn register(&self) -> clap::App {
        clap::SubCommand::with_name("build")
            .version(*crate::VERSION)
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
                    .long("force")
                    .short("f"),
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
                Step::single(
                    "♻️",
                    "Clean",
                    Stage::Check,
                    vec![Box::new(crate::build::checks::clear::Clean {})],
                ),
                if args.is_present("rebuild") {
                    Step::parallel(
                        "🗑️",
                        "Clear",
                        Stage::Check,
                        vec![Box::new(crate::build::checks::clear::Clear {})],
                    )
                } else {
                    Step::none()
                },
                Step::parallel(
                    "🔍",
                    "Checks",
                    Stage::Check,
                    vec![
                        Box::new(crate::build::prebuild::render::Render {}),
                        Box::new(crate::build::checks::names::NotEmpty {}),
                        Box::new(crate::build::checks::names::ValidName {}),
                        Box::new(crate::build::checks::modtime::ModTime {}),
                    ],
                ),
                Step::single("📜", "", Stage::Check, vec![Box::new(crate::flow::Script {})]),
                Step::parallel(
                    "🚧",
                    "Prebuild",
                    Stage::PreBuild,
                    vec![Box::new(crate::build::prebuild::preprocess::Preprocess {})],
                ),
                Step::single("📜", "", Stage::PreBuild, vec![Box::new(crate::flow::Script {})]),
                Step::parallel(
                    "📝",
                    "Build",
                    Stage::Build,
                    vec![Box::new(crate::build::build::Build { use_bin: true })],
                ),
                Step::single("📜", "", Stage::PostBuild, vec![Box::new(crate::flow::Script {})]),
                if args.is_present("release") {
                    Step::single(
                        "⭐",
                        "Release",
                        Stage::ReleaseBuild,
                        vec![Box::new(crate::build::postbuild::release::Release {})],
                    )
                } else {
                    Step::none()
                },
                if args.is_present("release") {
                    Step::single(
                        "⭐",
                        "Sign",
                        Stage::ReleaseBuild,
                        vec![Box::new(crate::build::postbuild::sign::Sign {})],
                    )
                } else {
                    Step::none()
                },
                if args.is_present("release") {
                    Step::single("📜", "", Stage::ReleaseBuild, vec![Box::new(crate::flow::Script {})])
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
