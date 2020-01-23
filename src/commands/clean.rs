use crate::{Command, Flow, HEMTTError, Project, Stage, Step};

pub struct Clean {}
impl Command for Clean {
    fn register(&self) -> clap::App {
        clap::SubCommand::with_name("clean")
            .version(*crate::VERSION)
            .about("Clean built files")
    }

    fn run(&self, _: &clap::ArgMatches, mut p: Project) -> Result<(), HEMTTError> {
        let mut addons = crate::project::addons::get_all()?;
        let flow = Flow {
            steps: vec![
                Step::parallel(
                    "♻️",
                    "Clean",
                    Stage::Check,
                    vec![Box::new(crate::build::checks::clear::Clean {})],
                ),
                Step::single(
                    "🗑️",
                    "Clear",
                    Stage::Check,
                    vec![Box::new(crate::build::checks::clear::Clear {})],
                ),
            ],
        };
        flow.execute(addons, &mut p)?;
        Ok(())
    }
}
