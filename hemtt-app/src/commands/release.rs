use crate::{flow::Stage, Command, Flow, HEMTTError, Project};

pub struct Release {}
impl Command for Release {
    fn register(&self) -> clap::App {
        clap::SubCommand::with_name("release")
            .version(*crate::VERSION)
            .about("Release the Project")
    }

    fn run(&self, args: &clap::ArgMatches, p: Project) -> Result<(), HEMTTError> {
        let addons = crate::get_addons_from_args(args)?;
        let flow = Flow {
            tasks: {
                vec![
                    // Box::new(crate::tasks::Clean {}),
                    // Box::new(crate::tasks::Clear {}),
                    Box::new(crate::tasks::NotEmpty {}),
                    Box::new(crate::tasks::ValidName {}),
                    // Box::new(crate::tasks::ModTime {}),
                    Box::new(crate::tasks::Populate {}),
                    Box::new(crate::tasks::Prefix::new()),
                    Box::new(crate::tasks::Preprocess {}),
                    Box::new(crate::tasks::Rapify {}),
                    Box::new(crate::tasks::Pack {}),
                    Box::new(crate::tasks::Release {}),
                    Box::new(crate::tasks::Sign {}),
                ]
            },
        };
        flow.execute(addons, Stage::release(), &p)?;
        Ok(())
    }
}
