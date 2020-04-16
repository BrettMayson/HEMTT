#[allow(clippy::module_inception)]
pub mod build;
pub mod checks;
pub mod postbuild;
pub mod prebuild;

use crate::{Command, Flow, HEMTTError, Project, Stage, Step};

pub struct Build {}
impl Command for Build {
    fn register(&self) -> clap::App {
        clap::SubCommand::with_name("build")
            .version(*crate::VERSION)
            .about("Build the Project")
            .args(&super::building_args())
    }

    fn run(&self, args: &clap::ArgMatches, mut p: Project) -> Result<(), HEMTTError> {
        let addons = crate::project::addons::get_from_args(args)?;
        let flow = Flow {
            steps: vec![
                Step::single(
                    "♻️",
                    "Clean",
                    Stage::Check,
                    vec![Box::new(crate::build::checks::clear::Clean {})],
                ),
                if args.is_present("force") {
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
                Step::single(
                    "📜",
                    "",
                    Stage::Check,
                    vec![Box::new(crate::flow::Script {
                        release: args.is_present("release"),
                    })],
                ),
                Step::parallel(
                    "🚧",
                    "Prebuild",
                    Stage::PreBuild,
                    vec![Box::new(crate::build::prebuild::preprocess::Preprocess {})],
                ),
                Step::single(
                    "📜",
                    "",
                    Stage::PreBuild,
                    vec![Box::new(crate::flow::Script {
                        release: args.is_present("release"),
                    })],
                ),
                Step::parallel(
                    "📝",
                    "Build",
                    Stage::Build,
                    vec![Box::new(crate::build::build::Build::new(true))],
                ),
                Step::single(
                    "📜",
                    "",
                    Stage::PostBuild,
                    vec![Box::new(crate::flow::Script {
                        release: args.is_present("release"),
                    })],
                ),
                if args.is_present("release") {
                    Step::single(
                        "⭐",
                        "Release",
                        Stage::ReleaseBuild,
                        vec![Box::new(crate::build::postbuild::release::Release {
                            force_release: args.is_present("force-release"),
                        })],
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
                    Step::single(
                        "📜",
                        "",
                        Stage::ReleaseBuild,
                        vec![Box::new(crate::flow::Script {
                            release: args.is_present("release"),
                        })],
                    )
                } else {
                    Step::none()
                },
            ],
        };
        flow.execute(addons, &mut p)?;
        Ok(())
    }
}
