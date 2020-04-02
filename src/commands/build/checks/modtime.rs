use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use indicatif::ProgressBar;

use crate::{Addon, HEMTTError, Project, Report, Stage, Task};

pub fn modtime(addon: &PathBuf) -> Result<SystemTime, HEMTTError> {
    let mut recent: SystemTime = SystemTime::now() - Duration::new(60 * 60 * 24 * 365 * 10, 0);
    for entry in walkdir::WalkDir::new(addon) {
        let metadata = std::fs::metadata(entry.unwrap().path())?;
        if let Ok(time) = metadata.modified() {
            if time > recent {
                recent = time;
            }
        }
    }
    Ok(recent)
}

#[derive(Clone)]
pub struct ModTime {}
impl Task for ModTime {
    fn can_run(&self, _: &Addon, _: &Report, _: &Project, _: &Stage) -> Result<bool, HEMTTError> {
        Ok(true)
    }

    fn parallel(&self, addon: &Addon, _: &Report, p: &Project, _: &Stage, _pb: &ProgressBar) -> Result<Report, HEMTTError> {
        let mut report = Report::new();
        let modified = modtime(&addon.folder())?;
        let target = addon.target(p);
        if target.exists() {
            if let Ok(time) = std::fs::metadata(&target).unwrap().modified() {
                if time >= modified {
                    report.stop = Some((
                        false,
                        HEMTTError::GENERIC("The PBO already exists".to_owned(), target.display().to_string()),
                    ));
                }
            }
        }
        Ok(report)
    }
}
