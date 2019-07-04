use indicatif::ProgressBar;
use regex::Regex;

use crate::{HEMTTError, AddonLocation, Project, Task, Addon, Report};

#[derive(Clone)]
pub struct NotEmpty {}
impl Task for NotEmpty {
    fn can_run(&self, _: &Addon, _: &Report, _: &Project) -> Result<bool, HEMTTError> {
        Ok(true)
    }

    fn parallel(&self, addon: &Addon, _: &Report, _p: &Project, _pb: &ProgressBar) -> Result<Report, HEMTTError> {
        let mut report = Report::new();
        let empty = std::fs::read_dir(crate::build::folder_name(&addon.location))?.count() == 0;
        if empty {
            report.stop = Some(HEMTTError::SIMPLE("The addon directory is empty".to_string()));
        }
        Ok(report)
    }
}

#[derive(Clone)]
pub struct ValidName {}
impl Task for ValidName {
    fn can_run(&self, _: &Addon, _: &Report, _: &Project) -> Result<bool, HEMTTError> {
        Ok(true)
    }

    fn parallel(&self, addon: &Addon, _: &Report, p: &Project, _pb: &ProgressBar) -> Result<Report, HEMTTError> {
        let mut report = Report::new();
        // WARN: addon name standards
        let re = Regex::new(r"^([A-z0-9\-]+)$").unwrap();
        if !re.is_match(&addon.name) {
            report.warnings.push(
                HEMTTError::GENERIC(format!("addon name `{}` is not following standards", &addon.name), format!("try using `{}`", &addon.name.replace(" ", "_")))
            );
        }
        // WARN: addons shouldn't start with the mod prefix
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
        // WARN: compat outside of compat folder
        if addon.name.starts_with("compat") && addon.location != AddonLocation::Compats {
            report.warnings.push(
                HEMTTError::SIMPLE(format!("compatibility addon `{}` should be in `compats/`", &addon.name))
            );
        }
        Ok(report)
    }
}
