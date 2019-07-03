use std::path::Path;

use crate::{AddonLocation, Command, Project, HEMTTError, Flow, Step};

pub struct Status {}
impl Command for Status {
    fn register(&self) -> (&str, clap::App) {
        ("status",
            clap::SubCommand::with_name("status")
                .about("Get the status of your project")
        )
    }

    fn run(&self, _: &clap::ArgMatches, mut p: Project) -> Result<(), HEMTTError> {
        let mut addons = crate::build::get_addons(AddonLocation::Addons)?;
        if Path::new(&crate::build::folder_name(&AddonLocation::Optionals)).exists() {
            addons.extend(crate::build::get_addons(AddonLocation::Optionals)?);
        }
        if Path::new(&crate::build::folder_name(&AddonLocation::Compats)).exists() {
            addons.extend(crate::build::get_addons(AddonLocation::Compats)?);
        }
        let flow = Flow {
            steps: vec![
                Step::new("🔍", "Checks",
                    vec![
                        Box::new(crate::build::prebuild::render::Render {}),
                        Box::new(crate::build::checks::names::NotEmpty {}),
                        Box::new(crate::build::checks::names::ValidName {}),
                        Box::new(crate::build::prebuild::modtime::ModTime {}),
                    ],
                ),
            ],
        };
        let addons = flow.execute(addons, &mut p)?;
        let mut build = 0;
        for addon in addons {
            let (report, _) = addon?;
            if report.stop.is_none() { build += 1 }
        }
        let template = crate::commands::Template::new();
        println!("Version: {}", template.get_version().unwrap_or("Unable to determine".to_string()));
        println!("Addons to be built: {}", build);
        Ok(())
    }
}
