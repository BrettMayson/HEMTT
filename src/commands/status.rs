use std::path::Path;

use crate::{AddonLocation, Command, Flow, HEMTTError, Project, Step};

pub struct Status {}
impl Command for Status {
    fn register(&self) -> clap::App {
        clap::SubCommand::with_name("status").about("Get the status of your project")
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
            steps: vec![Step::parallel(
                "🔍",
                "Checks",
                vec![
                    Box::new(crate::build::prebuild::render::Render {}),
                    Box::new(crate::build::checks::names::NotEmpty {}),
                    Box::new(crate::build::checks::names::ValidName {}),
                    Box::new(crate::build::checks::modtime::ModTime {}),
                ],
            )],
        };
        let addons = flow.execute(addons, &mut p)?;
        let mut build = 0;
        for addon in addons {
            let (report, _) = addon?;
            if report.stop.is_none() {
                build += 1
            }
        }
        println!("CI Environment: {}", crate::is_ci());
        println!(
            "Version: {}",
            p.version().unwrap_or_else(|_| "Unable to determine".to_string())
        );
        println!("Addons to be built: {}", build);
        Ok(())
    }
}
