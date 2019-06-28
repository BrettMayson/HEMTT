use indicatif::ProgressBar;
use regex::Regex;

use crate::{HEMTTError, Project, Task, Addon, Report};

#[derive(Clone)]
pub struct NotEmpty {}
impl Task for NotEmpty {
    fn can_run(&self, _addon: &Addon, _: &Report, _p: &Project) -> Result<bool, HEMTTError> {
        Ok(true)
    }
    fn run(&self, addon: &Addon, _: &Report, _p: &Project, _pb: &ProgressBar) -> Result<Report, HEMTTError> {
        let mut report = Report::new();
        let empty = std::fs::read_dir(crate::build::folder_name(&addon.location))?.count() == 0;
        if empty {
            report.can_proceed = false;
        }
        Ok(report)
    }
}

#[derive(Clone)]
pub struct ValidName {}
impl Task for ValidName {
    fn can_run(&self, _addon: &Addon, _: &Report, _p: &Project) -> Result<bool, HEMTTError> {
        Ok(true)
    }
    fn run(&self, addon: &Addon, _: &Report, p: &Project, _pb: &ProgressBar) -> Result<Report, HEMTTError> {
        let mut report = Report::new();
        let re = Regex::new(r"^([A-z\-]+)$").unwrap();
        if !re.is_match(&addon.name) {
            report.warnings.push(
                HEMTTError::GENERIC(format!("addon name `{}` is not following standards", &addon.name), format!("try using `{}`", &addon.name.replace(" ", "_")))
            );
        }
        if addon.name.starts_with(&p.prefix) {
            report.warnings.push(
                HEMTTError::GENERIC(format!("Redundant prefix in addon name `{}`", &addon.name),
                    format!("use `{}`, pbos are prefixed automatically", if addon.name.starts_with(&format!("{}_", &p.prefix)) {
                        &addon.name[(p.prefix.len()+1)..]
                    } else {
                        &addon.name[p.prefix.len()..]
                    })
                )
            );
        }
        Ok(report)
    }
}
