use std::collections::HashMap;
use std::io::Cursor;
use std::path::PathBuf;

#[cfg(not(windows))]
use indicatif::ProgressBar;
#[cfg(windows)]
use indicatif_windows::ProgressBar;

use linked_hash_map::LinkedHashMap;
use regex::Regex;
use walkdir::WalkDir;

use crate::{Addon, HEMTTError, Project, Report, Stage, Task};

static BINARIZABLE: &[&str] = &["rtm", "p3d"];

#[derive(Clone)]
pub struct Build {
    pub use_bin: bool,
}
impl Task for Build {
    fn can_run(&self, _: &Addon, _: &Report, _: &Project, _: &Stage) -> Result<bool, HEMTTError> {
        Ok(true)
    }

    fn parallel(&self, addon: &Addon, _r: &Report, p: &Project, _: &Stage, pb: &ProgressBar) -> Result<Report, HEMTTError> {
        let mut report = Report::new();
        let mut pbo = armake2::PBO {
            files: LinkedHashMap::new(),
            header_extensions: HashMap::new(),
            headers: Vec::new(),
            checksum: None,
        };
        let directory = addon.folder();
        let binarize = self.use_bin
            && cfg!(windows)
            && !(directory.join("$NOBIN$").exists() || directory.join("$NOBIN-NOTEST$").exists())
            && if match armake2::find_binarize_exe() {
                Ok(p) => p.exists(),
                Err(_) => false,
            } {
                true
            } else {
                report.warnings.push(HEMTTError::generic(
                    "Unable to locate binarize.exe",
                    "Files will be packed as is",
                ));
                false
            };

        for entry in WalkDir::new(&addon.folder()) {
            let entry = entry.unwrap();
            if entry.path().is_dir() {
                continue;
            }
            if crate::build::prebuild::render::can_render(entry.path()) {
                continue;
            }
            pb.set_message(&entry.path().display().to_string());
            let name = entry
                .path()
                .display()
                .to_string()
                .trim_start_matches(&format!("{}{}", addon.folder().to_str().unwrap(), std::path::MAIN_SEPARATOR))
                .to_string();
            let ext = entry
                .path()
                .extension()
                .unwrap_or_else(|| std::ffi::OsStr::new(""))
                .to_str()
                .unwrap()
                .to_string();
            let is_binarizable = binarize && BINARIZABLE.contains(&ext.as_str());

            if name == "$PBOPREFIX$" {
                let content = crate::CACHED
                    .lock()
                    .unwrap()
                    .lines(&entry.path().display().to_string())?
                    .clone();
                for line in content {
                    if line.is_empty() {
                        break;
                    }

                    let eq: Vec<String> = line.split('=').map(|s| s.to_string()).collect();
                    if eq.len() == 1 {
                        pbo.header_extensions.insert("prefix".to_string(), line.to_string());
                    } else {
                        pbo.header_extensions.insert(eq[0].clone(), eq[1].clone());
                    }
                }
            } else {
                let content = crate::CACHED
                    .lock()
                    .unwrap()
                    .read(&entry.path().display().to_string())?
                    .clone();
                if crate::build::prebuild::preprocess::RAPABLE.contains(&ext.as_ref()) {
                    if self.use_bin {
                        pbo.files.insert(
                            name.replace("config.cpp", "config.bin"),
                            Cursor::new(
                                crate::CACHED
                                    .lock()
                                    .unwrap()
                                    .read(&entry.path().display().to_string().replace("config.cpp", "config.bin"))?
                                    .into_boxed_slice(),
                            ),
                        );
                    } else {
                        pbo.files.insert(
                            name,
                            Cursor::new(
                                crate::CACHED
                                    .lock()
                                    .unwrap()
                                    .read(&entry.path().display().to_string())?
                                    .into_boxed_slice(),
                            ),
                        );
                    }
                } else if cfg!(windows) && is_binarizable {
                    let cursor = armake2::binarize(&PathBuf::from(entry.path()))?;
                    pbo.files.insert(name, cursor);
                } else {
                    if is_binarizable && !cfg!(windows) {
                        report.warnings.push(HEMTTError::generic(
                            format!("Unable to binarize `{}`", entry.path().display().to_string()),
                            "On non-windows systems binarize.exe cannot be used; file will packed as is",
                        ));
                    }

                    pbo.files.insert(
                        Regex::new(".p3do$").unwrap().replace_all(&name, ".p3d").to_string(),
                        Cursor::new(content.into_boxed_slice()),
                    );
                }
            }
        }
        let mut outf = create_file!(addon.target(p))?;
        pbo.write(&mut outf)?;
        Ok(report)
    }
}
