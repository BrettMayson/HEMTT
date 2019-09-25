use std::path::Path;

use crate::{AddonLocation, Command, Flow, HEMTTError, Project, Stage, Step};

pub struct Clean {}
impl Command for Clean {
    fn register(&self) -> clap::App {
        clap::SubCommand::with_name("clean").about("Clean built files")
    }

    fn run(&self, _: &clap::ArgMatches, mut p: Project) -> Result<(), HEMTTError> {
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
                Step::parallel(
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
