use crate::{Command, Flow, HEMTTError, Project, Task};

pub struct Build {}
impl Command for Build {
    fn register(&self) -> clap::App {
        clap::SubCommand::with_name("build")
            .version(*crate::VERSION)
            .about("Build the Project")
        // .args(&super::building_args())
    }

    fn run(&self, args: &clap::ArgMatches, mut p: Project) -> Result<(), HEMTTError> {
        let addons = crate::get_addons_from_args(args)?;
        let flow = Flow {
            tasks: {
                let mut tasks: Vec<Box<dyn Task>> = vec![
                    Box::new(crate::tasks::Clear {}),
                    Box::new(crate::tasks::NotEmpty {}),
                    Box::new(crate::tasks::ValidName {}),
                    Box::new(crate::tasks::ModTime {}),
                    Box::new(crate::tasks::Render::new()),
                    // Step::single(
                    //     "",
                    //     vec![Box::new(crate::flow::Script {
                    //         release: args.is_present("release"),
                    //     })],
                    // ),
                    // Step::parallel(
                    //     "Prebuild",
                    //     vec![
                    //         // Box::new(crate::build::prebuild::preprocess::Preprocess {}),
                    //     ],
                    // ),
                    // Step::single(
                    //     "",
                    //     vec![Box::new(crate::flow::Script {
                    //         release: args.is_present("release"),
                    //     })],
                    // ),
                    // Step::parallel(
                    //     "Build",
                    //     Stage::Build,
                    //     vec![Box::new(crate::build::build::Build::new(true))],
                    // ),
                    // Step::single(
                    //     "",
                    //     vec![Box::new(crate::flow::Script {
                    //         release: args.is_present("release"),
                    //     })],
                    // ),
                    // if args.is_present("release") {
                    //     Step::single(
                    //         "Release",
                    //         vec![Box::new(crate::build::postbuild::release::Release {
                    //             force_release: args.is_present("force-release"),
                    //         })],
                    //     )
                    // } else {
                    //     Step::none()
                    // },
                    // if args.is_present("release") {
                    //     Step::single(
                    //         "Sign",
                    //         vec![Box::new(crate::build::postbuild::sign::Sign {})],
                    //     )
                    // } else {
                    //     Step::none()
                    // },
                    // if args.is_present("release") {
                    //     Step::single(
                    //         "",
                    //         vec![Box::new(crate::flow::Script {
                    //             release: args.is_present("release"),
                    //         })],
                    //     )
                    // } else {
                    //     Step::none()
                    // },
                ];
                if args.is_present("force") {
                    tasks.push(Box::new(crate::tasks::Clean {}));
                }
                tasks
            },
        };
        flow.execute(addons, &mut p)?;
        Ok(())
    }
}
