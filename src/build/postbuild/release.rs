use crate::{Task, Addon, Report, Project, HEMTTError};

#[derive(Clone)]
pub struct Release {}
impl Task for Release {
    fn can_run(&self, _: &Addon, _: &Report, _: &Project) -> Result<bool, HEMTTError> {
        Ok(true)
    }

    fn single(&self, addons: Vec<Result<(Report, Addon), HEMTTError>>, p: &Project) -> Result<Vec<Result<(Report, Addon), HEMTTError>>, HEMTTError> {
        let mut can_continue = true;
        /*for addon in &addons {
            if addon.is_err() { can_continue = false; break; }
            let (report, addon) = addon.unwrap();
            if report.stop.is_some() { can_continue = false; break; }
        }*/
        let addons = addons.into_iter().map(|d| {
            if d.is_err() {
                can_continue = false;
                d
            } else {
                let (report, addon) = d.unwrap();
                if let Some((fatal, _)) = report.stop { if fatal { can_continue = false; } }
                Ok((report, addon))
            }
        }).collect();

        if !can_continue {
            return Err(HEMTTError::generic("Unable to release", "One or more addons were not built successfully"));
        }

        // Prepare release directory
        let release_folder = p.release_dir()?;
        println!("{}", release_folder);
        std::fs::create_dir_all(release_folder)?;

        Ok(addons)
    }
}
