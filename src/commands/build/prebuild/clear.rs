use std::path::PathBuf;

use indicatif::ProgressBar;
use regex::Regex;
use strum::IntoEnumIterator;

use crate::{AddonLocation, Task, Report, Addon, Project, HEMTTError};

#[derive(Clone)]
pub struct Clear {}
impl Task for Clear {
    fn can_run(&self, _: &Addon, _: &Report, _: &Project) -> Result<bool, HEMTTError> {
        Ok(true)
    }

    fn parallel(&self, addon: &Addon, _: &Report, p: &Project, _pb: &ProgressBar) -> Result<Report, HEMTTError> {
        let target = addon.target(p);
        if target.exists() {
            std::fs::remove_file(target)?;
        }
        Ok(Report::new())
    }
}

#[derive(Clone)]
pub struct Clean {}
impl Task for Clean {
    fn can_run(&self, _: &Addon, _: &Report, _: &Project) -> Result<bool, HEMTTError> {
        Ok(true)
    }

    fn single(&self, addons: Vec<Result<(Report, Addon), HEMTTError>>, p: &Project) -> Result<Vec<Result<(Report, Addon), HEMTTError>>, HEMTTError> {
        let re = Regex::new(r"(?m)(.+?)\.pbo$").unwrap();
        let mut targets = Vec::new();
        for data in &addons {
            if let Ok(d) = data {
                let (_, addon) = d;
                targets.push(addon.target(p).display().to_string());
            }
        }
        for dir in AddonLocation::iter() {
            let dir = crate::build::addon::folder_name(&dir);
            if !PathBuf::from(&dir).exists() { continue; }
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                let loc = path.display().to_string();
                if !path.is_dir() && re.is_match(&loc) && !targets.contains(&loc) {
                    std::fs::remove_file(&loc)?;
                    println!("Removing {}", loc);
                }
            }
        }
        Ok(addons)
    }
}
