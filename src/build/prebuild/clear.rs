use indicatif::ProgressBar;

use crate::{Task, Report, Addon, Project, HEMTTError};

#[derive(Clone)]
pub struct Clear {}
impl Task for Clear {
    fn can_run(&self, _: &Addon, _: &Report, _: &Project) -> Result<bool, HEMTTError> {
        Ok(true)
    }

    fn run(&self, addon: &Addon, _: &Report, p: &Project, _pb: &ProgressBar) -> Result<Report, HEMTTError> {
        let target = addon.target(p);
        if target.exists() {
            std::fs::remove_file(target)?;
        }
        Ok(Report::new())
    }
}
