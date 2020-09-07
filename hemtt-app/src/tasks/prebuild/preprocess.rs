use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::{context::AddonContext, Addon, HEMTTError, OkSkip, Project, Task};

#[derive(Clone)]
pub struct Preprocess {}
impl Task for Preprocess {
    fn parallel(&self, ctx: &mut AddonContext) -> Result<OkSkip, HEMTTError> {
        let mut ok = true;
        for entry in WalkDir::new(&ctx.addon.source()) {
            let path = entry.unwrap();
            let can_rap = hemtt::preprocess::can_preprocess_file(path.path());
            if can_rap {
                let (original_path, rendered_path) = crate::RENDERED
                    .lock()
                    .unwrap()
                    .get_paths(path.path().display().to_string());
                let raw = ctx
                    .global
                    .cache
                    .read()
                    .unwrap()
                    .clean_comments(&rendered_path)?
                    .clone();
                if raw.len() < 3 {
                    continue;
                }
                let mut includes = ctx.global.project.include.clone();
                includes.insert(0, PathBuf::from("."));
                // match preprocess(raw.clone(), Some(PathBuf::from(&original_path)), &includes, |path| {
                //     pb.set_message(&format!("{} - {}", &fill_space!(" ", CMD_GAP, "Preprocess"), rendered_path));
                //     crate::CACHED.lock().unwrap().clean_comments(path.to_str().unwrap())
                // }) {
                match hemtt::Config::from_string(
                    raw.clone(),
                    Some(PathBuf::from(&original_path)),
                    &includes,
                    |path| {
                        crate::CACHED
                            .lock()
                            .unwrap()
                            .clean_comments(path.to_str().unwrap())
                            .unwrap()
                    },
                ) {
                    Ok(rapped) => {
                        let mut c = Cursor::new(Vec::new());
                        rapped.write_rapified(&mut c)?;
                        c.seek(SeekFrom::Start(0))?;
                        let mut out = Vec::new();
                        c.read_to_end(&mut out)?;
                        crate::CACHED.lock().unwrap().insert_bytes(
                            &rendered_path.replace("config.cpp", "config.bin"),
                            out,
                        )?;
                    }
                    Err(e) => {
                        // Unable to clone HEMTTError
                        //report.unique_error(HEMTTError::from(e));
                        // report.stop = Some((true, HEMTTError::from(e)));
                        error!("PreProcess error: {}", e);
                        ok = false;
                    }
                }
            }
        }
        Ok((ok, false))
    }
}
