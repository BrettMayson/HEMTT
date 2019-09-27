use std::path::Path;

use crate::{AddonLocation, Command, Flow, HEMTTError, Project, Stage, Step};

pub struct Pack {}
impl Command for Pack {
    fn register(&self) -> clap::App {
        clap::SubCommand::with_name("pack")
            .version(*crate::VERSION)
            .about("Pack the Project")
            .arg(
                clap::Arg::with_name("release")
                    .help("Pack a release")
                    .long("release")
                    .conflicts_with("dev"),
            )
            .arg(
                clap::Arg::with_name("clear")
                    .help("Clears existing built files")
                    .long("clear")
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
                if args.is_present("clear") {
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
                Step::parallel(
                    "📦",
                    "Pack",
                    Stage::Build,
                    vec![Box::new(crate::build::build::Build { use_bin: false })],
                ),
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
