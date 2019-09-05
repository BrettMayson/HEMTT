use std::path::Path;

use crate::{AddonLocation, Command, Flow, HEMTTError, Project, Step};

pub struct Clean {}
impl Command for Clean {
    fn register(&self) -> clap::App {
        clap::SubCommand::with_name("clean").about("Clean built files")
    }

    fn run(&self, _: &clap::ArgMatches, mut p: Project) -> Result<(), HEMTTError> {
        let mut addons = crate::build::get_addons(AddonLocation::Addons)?;
        if Path::new(&crate::build::addon::folder_name(&AddonLocation::Optionals)).exists() {
            addons.extend(crate::build::get_addons(AddonLocation::Optionals)?);
        }
        if Path::new(&crate::build::addon::folder_name(&AddonLocation::Compats)).exists() {
            addons.extend(crate::build::get_addons(AddonLocation::Compats)?);
        }
        let flow = Flow {
            steps: vec![
                Step::single("♻️", "Clean", vec![Box::new(crate::build::prebuild::clear::Clean {})]),
                Step::parallel("🗑️", "Clear", vec![Box::new(crate::build::prebuild::clear::Clear {})]),
            ],
        };
        flow.execute(addons, &mut p)?;
        Ok(())
    }
}
