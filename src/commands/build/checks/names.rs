#[cfg(not(windows))]
use indicatif::ProgressBar;
#[cfg(windows)]
use indicatif_windows::ProgressBar;

use regex::Regex;

use crate::{Addon, AddonLocation, HEMTTError, Project, Report, Stage, Task};

#[derive(Clone)]
pub struct NotEmpty {}
impl Task for NotEmpty {
    fn can_run(&self, _: &Addon, _: &Report, _: &Project, _: &Stage) -> Result<bool, HEMTTError> {
        Ok(true)
    }

    fn parallel(&self, addon: &Addon, _: &Report, _: &Project, _: &Stage, _: &ProgressBar) -> Result<Report, HEMTTError> {
        let mut report = Report::new();
        let empty = std::fs::read_dir(addon.folder())?.count() == 0;
        if empty {
            report.stop = Some((true, HEMTTError::simple("The addon directory is empty")));
        }
        Ok(report)
    }
}

#[derive(Clone)]
pub struct ValidName {}
impl Task for ValidName {
    fn can_run(&self, _: &Addon, _: &Report, _: &Project, _: &Stage) -> Result<bool, HEMTTError> {
        Ok(true)
    }

    fn parallel(&self, addon: &Addon, _: &Report, p: &Project, _: &Stage, _: &ProgressBar) -> Result<Report, HEMTTError> {
        let mut report = Report::new();
        // WARN: addon name standards
        let re = Regex::new(r"^([A-z0-9\-]+)$").unwrap();
        if !re.is_match(&addon.name) {
            report.warnings.push(HEMTTError::generic(
                format!("addon name `{}` is not following standards", &addon.name),
                format!("try using `{}`", &addon.name.replace(" ", "_")),
            ));
        }
        // WARN: addons shouldn't start with the mod prefix
        if addon.name.starts_with(&p.prefix) {
            report.warnings.push(HEMTTError::generic(
                format!("Redundant prefix in addon name `{}`", &addon.name),
                format!(
                    "use `{}`, pbos are prefixed automatically",
                    if addon.name.starts_with(&format!("{}_", &p.prefix)) {
                        &addon.name[(p.prefix.len() + 1)..]
                    } else {
                        &addon.name[p.prefix.len()..]
                    }
                ),
            ));
        }
        // WARN: compat outside of compat folder
        if addon.name.starts_with("compat") && addon.location != AddonLocation::Compats {
            report.warnings.push(HEMTTError::simple(format!(
                "compatibility addon `{}` should be in `compats/`",
                &addon.name
            )));
        }
        Ok(report)
    }
}
