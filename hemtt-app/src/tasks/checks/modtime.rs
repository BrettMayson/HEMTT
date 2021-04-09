use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use crate::{context::AddonContext, HEMTTError, Stage, Task};

pub fn modtime<P: Into<PathBuf>>(addon: P) -> Result<SystemTime, HEMTTError> {
    let mut recent: SystemTime = SystemTime::now() - Duration::new(60 * 60 * 24 * 365 * 10, 0);
    for entry in walkdir::WalkDir::new(addon.into()) {
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

    fn check(&self, ctx: &mut AddonContext) -> Result<(), HEMTTError> {
        let modified = modtime(&ctx.addon().source())?;
        let target = ctx.addon().destination(
            &hemtt::Project::find_root()?,
            Some(ctx.global().project().prefix()),
            None,
        );
        if target.exists() {
            if let Ok(time) = std::fs::metadata(&target).unwrap().modified() {
                ctx.debug(&format!("modtime: {:?} | lastbuild: {:?}", modified, time));
                if time >= modified {
                    ctx.set_skip(true);
                    ctx.info(&format!("The PBO is up to date: {}", target.display()));
                }
            }
        } else {
            ctx.info(&format!("no pbo exists at {}", target.display()));
        }
        Ok(())
    }
}
