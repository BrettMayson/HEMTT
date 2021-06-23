use crate::{flow::Stage, Command, Flow, HEMTTError, Project};

pub struct Clean {}
impl Command for Clean {
    fn register(&self) -> clap::App {
        clap::SubCommand::with_name("clean")
            .version(*crate::VERSION)
            .about("Clean built files")
    }

    fn run(&self, _: &clap::ArgMatches, p: Project) -> Result<(), HEMTTError> {
        let addons = hemtt::get_all_addons()?;
        let flow = Flow {
            tasks: vec![
                Box::new(crate::tasks::Clear {}),
                Box::new(crate::tasks::Clean {}),
            ],
        };
        flow.execute(addons, Stage::check(), &p)?;
        Ok(())
    }
}
