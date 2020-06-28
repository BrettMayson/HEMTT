use crate::{Command, Flow, HEMTTError, Project, Stage, Step};

pub struct Pack {}
impl Command for Pack {
    fn register(&self) -> clap::App {
        clap::SubCommand::with_name("pack")
            .version(*crate::VERSION)
            .about("Pack the Project")
            .args(&super::building_args())
    }

    fn run(&self, args: &clap::ArgMatches, mut p: Project) -> Result<(), HEMTTError> {
        let addons = crate::project::addons::get_from_args(args)?;
        let flow = Flow {
            steps: vec![
                Step::single("Clean", Stage::Check, vec![Box::new(crate::build::checks::clear::Clean {})]),
                if args.is_present("force") {
                    Step::parallel("Clear", Stage::Check, vec![Box::new(crate::build::checks::clear::Clear {})])
                } else {
                    Step::none()
                },
                Step::parallel(
                    "Checks",
                    Stage::Check,
                    vec![
                        Box::new(crate::build::prebuild::render::Render {}),
                        Box::new(crate::build::checks::names::NotEmpty {}),
                        Box::new(crate::build::checks::names::ValidName {}),
                        Box::new(crate::build::checks::modtime::ModTime {}),
                    ],
                ),
                Step::parallel("Pack", Stage::Build, vec![Box::new(crate::build::build::Build::new(false))]),
                if args.is_present("release") {
                    Step::single(
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
