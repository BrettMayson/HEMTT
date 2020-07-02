use std::path::PathBuf;

use regex::Regex;
use strum::IntoEnumIterator;

use crate::{Addon, AddonList, AddonLocation, HEMTTError, OkSkip, Project, Stage, Task};

// Clears existing files that are part of the hemtt project
#[derive(Clone)]
pub struct Clear {}
impl Task for Clear {
    fn can_run(&self, _: &Addon, _: &Project, _: &Stage) -> Result<bool, HEMTTError> {
        Ok(true)
    }

    fn parallel(&self, addon: &Addon, p: &Project, _: &Stage) -> Result<OkSkip, HEMTTError> {
        let target = addon.target(p);
        if target.exists() {
            remove_file!(target)?;
        }
        Ok((true, false))
    }
}

// Cleans all pbo files that are not part of the hemtt project
#[derive(Clone)]
pub struct Clean {}
impl Task for Clean {
    fn single(&self, addons: AddonList, p: &Project, _: &Stage) -> Result<AddonList, HEMTTError> {
        let re = Regex::new(r"(?m)(.+?)\.pbo$").unwrap();
        let mut targets = Vec::new();
        for data in &addons {
            if let Ok(d) = data {
                let (_, _, addon) = d;
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
                    remove_file!(&loc)?;
                }
            }
        }
        Ok(addons)
    }
}
