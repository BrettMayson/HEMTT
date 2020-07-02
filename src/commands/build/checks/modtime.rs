use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use crate::{Addon, HEMTTError, OkSkip, Project, Stage, Task};

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
    fn can_run(&self, _: &Addon, _: &Project, _: &Stage) -> Result<bool, HEMTTError> {
        Ok(true)
    }

    fn parallel(&self, addon: &Addon, p: &Project, _: &Stage) -> Result<OkSkip, HEMTTError> {
        let modified = modtime(&addon.folder())?;
        let target = addon.target(p);
        let mut skip = false;
        if target.exists() {
            if let Ok(time) = std::fs::metadata(&target).unwrap().modified() {
                if time >= modified {
                    skip = true;
                    info!("The PBO already exists: {}", target.display());
                }
            }
        }
        Ok((true, skip))
    }
}
