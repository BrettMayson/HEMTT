use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use crate::{context::AddonContext, HEMTTError, OkSkip, Stage, Task};

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
    fn name(&self) -> String {
        String::from("modtime")
    }

    fn hooks(&self) -> &[Stage] {
        &[Stage::Check]
    }

    fn check(&self, ctx: &mut AddonContext) -> Result<OkSkip, HEMTTError> {
        let modified = modtime(&ctx.addon.source())?;
        let target = ctx.addon.destination(
            &hemtt::Project::find_root()?,
            Some(&ctx.global.project.prefix),
            None,
        );
        let mut skip = false;
        if target.exists() {
            if let Ok(time) = std::fs::metadata(&target).unwrap().modified() {
                debug!(
                    "[Check] [{:^width$}] [{}] modtime: {:?} | lastbuild: {:?}",
                    "modtime",
                    ctx.addon.name,
                    modified,
                    time,
                    width = ctx.global.task_pad,
                );
                if time >= modified {
                    skip = true;
                    info!(
                        "[Check] [{:^width$}] [{}] The PBO is up to date: {}",
                        "modtime",
                        ctx.addon.name,
                        target.display(),
                        width = ctx.global.task_pad,
                    );
                }
            }
        } else {
            debug!(
                "[Check] [{:^width$}] [{}] no pbo exists at {}",
                "modtime",
                ctx.addon.name,
                target.display(),
                width = ctx.global.task_pad,
            );
        }
        Ok((true, skip))
    }
}
