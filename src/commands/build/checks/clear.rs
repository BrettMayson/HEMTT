use std::path::PathBuf;

#[cfg(not(windows))]
use indicatif::ProgressBar;
#[cfg(windows)]
use indicatif_windows::ProgressBar;

use regex::Regex;
use strum::IntoEnumIterator;

use crate::{Addon, AddonList, AddonLocation, HEMTTError, Project, Report, Stage, Task};

// Clears existing files that will be rebuilt only
#[derive(Clone)]
pub struct Clear {}
impl Task for Clear {
    fn can_run(&self, _: &Addon, _: &Report, _: &Project, _: &Stage) -> Result<bool, HEMTTError> {
        Ok(true)
    }

    fn parallel(&self, addon: &Addon, _: &Report, p: &Project, _: &Stage, _pb: &ProgressBar) -> Result<Report, HEMTTError> {
        let target = addon.target(p);
        if target.exists() {
            std::fs::remove_file(target)?;
        }
        Ok(Report::new())
    }
}

// Cleans all pbo files
#[derive(Clone)]
pub struct Clean {}
impl Task for Clean {
    fn single(&self, addons: Vec<Result<(Report, Addon), HEMTTError>>, p: &Project, _: &Stage) -> AddonList {
        let re = Regex::new(r"(?m)(.+?)\.pbo$").unwrap();
        let mut targets = Vec::new();
        for data in &addons {
            if let Ok(d) = data {
                let (_, addon) = d;
                targets.push(addon.target(p).display().to_string());
            }
        }
        for dir in AddonLocation::iter() {
            let dir = dir.to_string();
            if !PathBuf::from(&dir).exists() {
                continue;
            }
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                let loc = path.display().to_string();
                if !path.is_dir() && re.is_match(&loc) && !targets.contains(&loc) {
                    std::fs::remove_file(&loc)?;
                }
            }
        }
        Ok(addons)
    }
}
