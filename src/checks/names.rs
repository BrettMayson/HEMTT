use regex::Regex;

use crate::flow::{Task, Report};
use crate::error::HEMTTError;
use crate::project::Project;

pub struct NotEmpty {}
impl Task for NotEmpty {
    fn chk_can_run(&self, _addon: &crate::build::Addon, _p: &Project) -> Result<bool, HEMTTError> {
        Ok(true)
    }
    fn chk_run(&self, addon: &crate::build::Addon, _p: &Project) -> Result<Report, HEMTTError> {
        let empty = std::fs::read_dir(crate::build::folder_name(&addon.location))?.collect::<Vec<_>>().len() == 0;
        let mut report = Report::new();
        if empty {
            report.can_proceed = false;
        }
        Ok(report)
    }
}

pub struct ValidName {}
impl Task for ValidName {
    fn chk_can_run(&self, _addon: &crate::build::Addon, _p: &Project) -> Result<bool, HEMTTError> {
        Ok(true)
    }
    fn chk_run(&self, addon: &crate::build::Addon, p: &Project) -> Result<Report, HEMTTError> {
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
