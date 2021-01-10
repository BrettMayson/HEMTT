use std::path::PathBuf;

use regex::Regex;
use strum::IntoEnumIterator;

use crate::{context::AddonListContext, HEMTTError, Stage, Task};
use hemtt::AddonLocation;

// Clears all pbo files that are not part of the hemtt project
#[derive(Clone)]
pub struct Clear {}
impl Task for Clear {
    fn name(&self) -> String {
        String::from("clear")
    }

    fn hooks(&self) -> &[Stage] {
        &[Stage::Check]
    }

    fn check_single(&self, context: &mut AddonListContext) -> Result<(), HEMTTError> {
        let re = Regex::new(r"(?m)(.+?)\.pbo$").unwrap();
        let mut targets = Vec::new();
        for data in &*context.addons {
            if let Ok(d) = data {
                let (_, _, addon) = d;
                targets.push(addon.pbo(Some(context.global.project().prefix())));
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
        Ok(())
    }
}
