use std::collections::HashMap;
use std::io::Cursor;
use std::path::PathBuf;

#[cfg(not(windows))]
use indicatif::ProgressBar;
#[cfg(windows)]
use indicatif_windows::ProgressBar;

use glob::Pattern;
use linked_hash_map::LinkedHashMap;
use regex::Regex;
use walkdir::WalkDir;

use crate::{Addon, HEMTTError, Project, Report, Stage, Task};

static BINARIZABLE: &[&str] = &["rtm", "p3d"];

#[derive(Clone)]
pub struct Build {
    pub use_bin: bool,
    can_binarize: bool,
}
impl Build {
    pub fn new(use_bin: bool) -> Self {
        let can_binarize = use_bin && cfg!(windows) && Build::find_binarize();

        if !can_binarize {
            if cfg!(windows) {
                warnmessage!("Unable to locate binarize.exe", "Files will be packed as is");
            } else {
                warnmessage!(
                    "Unable to use binarize.exe on non-windows systems",
                    "Files will be packed as is"
                );
            }
        };

        Self { use_bin, can_binarize }
    }

    fn find_binarize() -> bool {
        match armake2::find_binarize_exe() {
            Ok(p) => {
                debug!("binarize found at {:?}", p);
                p.exists()
            }
            Err(_) => false,
        }
    }
}
impl Task for Build {
    fn can_run(&self, _: &Addon, _: &Report, _: &Project, _: &Stage) -> Result<bool, HEMTTError> {
        Ok(true)
    }

    fn parallel(&self, addon: &Addon, _r: &Report, p: &Project, _: &Stage, pb: &ProgressBar) -> Result<Report, HEMTTError> {
        let mut pbo = armake2::PBO {
            files: LinkedHashMap::new(),
            header_extensions: HashMap::new(),
            headers: Vec::new(),
            checksum: None,
        };
        let directory = addon.folder();
        let binarize =
            self.can_binarize && !(directory.join("$NOBIN$").exists() || directory.join("$NOBIN-NOTEST$").exists());

        let exclude_patterns: Vec<Pattern> = p.exclude.iter().map(|i| Pattern::new(i)).map(|e| e.unwrap()).collect();

        for entry in WalkDir::new(&addon.folder()) {
            let entry = entry.unwrap();
            if entry.path().is_dir() {
                continue;
            }
            if crate::build::prebuild::render::can_render(entry.path()) {
                continue;
            }
            if exclude_patterns.iter().any(|x| x.matches(entry.path().to_str().unwrap())) {
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
                    pbo.files.insert(
                        Regex::new(".p3do$").unwrap().replace_all(&name, ".p3d").to_string(),
                        Cursor::new(content.into_boxed_slice()),
                    );
                }
            }
        }

        // Add projects header extensions
        for header_ext in &p.header_exts {
            pbo.header_extensions.insert(
                header_ext.0.to_string(),
                crate::render::run(
                    header_ext.1,
                    Some(&format!("project:header_ext:{}", header_ext.0)),
                    &addon.get_variables(p),
                )?,
            );
        }

        let mut outf = create_file!(addon.target(p))?;
        pbo.write(&mut outf)?;
        Ok(Report::new())
    }
}
