use std::collections::HashMap;
use std::io::Cursor;
use std::path::PathBuf;

use glob::Pattern;
use linked_hash_map::LinkedHashMap;
use regex::Regex;
use walkdir::WalkDir;

use crate::{Addon, HEMTTError, OkSkip, Project, Stage, Task};

static BINARIZABLE: &[&str] = &["rtm", "p3d"];

#[derive(Clone)]
pub struct Build {
    pub use_bin: bool,
    can_binarize: bool,
}
impl Build {
    pub fn new(use_bin: bool) -> Self {
        let can_binarize = use_bin && cfg!(windows) && Self::find_binarize();

        if !can_binarize {
            if cfg!(windows) {
                warn!("Unable to locate binarize.exe; Files will be packed as is");
            } else {
                warn!("Unable to use binarize.exe on non-windows systems; Files will be packed as is");
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
    fn can_run(&self, _: &Addon, _: &Project, _: &Stage) -> Result<bool, HEMTTError> {
        Ok(true)
    }

    fn parallel(&self, addon: &Addon, p: &Project, _: &Stage) -> Result<OkSkip, HEMTTError> {
        info!("[{}] starting build", addon.name);
        let start_time = std::time::Instant::now();
        let mut pbo = armake2::PBO {
            extension_order: Vec::new(),
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
                trace!("[{}] excluding {}", addon.name, entry.path().display());
                continue;
            }
            trace!("[{}] process file: {}", addon.name, entry.path().display());
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
                        let prefix = line.trim_matches('\\');
                        pbo.header_extensions.insert("prefix".to_string(), prefix.to_string());
                    } else {
                        let header = eq[0].clone();
                        let mut value = eq[1].clone();

                        if header == "prefix" {
                            value = line.trim_matches('\\').to_string();
                        }

                        pbo.header_extensions.insert(header, value);
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
                    debug!("[{}] binarize: {}", addon.name, entry.path().display());
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
        info!(
            "[{}] built in {:?}",
            addon.name,
            std::time::Instant::now().duration_since(start_time)
        );
        Ok((true, false))
    }
}
