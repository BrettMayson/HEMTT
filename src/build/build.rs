use std::fs::File;
use std::path::PathBuf;
use std::io::{BufRead, BufReader, Read, Cursor};
use std::collections::HashMap;

use armake2::pbo::PBO;
use walkdir::WalkDir;
use linked_hash_map::{LinkedHashMap};
use regex::{Regex};
use indicatif::ProgressBar;

use crate::{Addon, HEMTTError, IOPathError, Report};

static BINARIZABLE: &[&str] = &["rtm", "p3d"];

pub fn dir(addon: &Addon, pb: &ProgressBar) -> Result<(PBO, Report), HEMTTError> {
    let mut report = Report::new();
    let mut pbo = armake2::pbo::PBO {
        files: LinkedHashMap::new(),
        header_extensions: HashMap::new(),
        headers: Vec::new(),
        checksum: None,
    };
    let directory = addon.folder();
    let binarize = !(directory.join("$NOBIN$").exists() || directory.join("$NOBIN-NOTEST$").exists());

    for entry in WalkDir::new(&addon.folder()) {
        let entry = entry.unwrap();
        if crate::build::prebuild::render::can_render(entry.path()) { continue; }
        pb.set_message(&entry.path().display().to_string());
        let name = entry.path().file_name().unwrap().to_str().unwrap().to_string();
        let ext = entry.path().extension().unwrap_or_else(|| std::ffi::OsStr::new("")).to_str().unwrap().to_string();
        let is_binarizable = binarize && BINARIZABLE.contains(&ext.as_str());

        if entry.path().is_dir() { continue; }
        let file = File::open(&entry.path()).map_err(|e| HEMTTError::PATH(IOPathError{
            source: e,
            path: PathBuf::from(entry.path()),
        }))?;



        if name == "$PBOPREFIX$" {
            let reader = BufReader::new(&file);
            for line in reader.lines() {
                let line = line?;
                if line.is_empty() { break; }

                let eq: Vec<String> = line.split('=').map(|s| s.to_string()).collect();
                if eq.len() == 1 {
                    pbo.header_extensions.insert("prefix".to_string(), line.to_string());
                } else {
                    pbo.header_extensions.insert(eq[0].clone(), eq[1].clone());
                }
            }
        } else if crate::build::prebuild::preprocess::RAPABLE.contains(&ext.as_ref()) {
            match crate::CACHED.lock().unwrap().get_path(entry.path().display().to_string()) {
                Some(v) => {
                    pbo.files.insert(name, Cursor::new(v.as_bytes().to_vec().into_boxed_slice()));
                },
                None => {
                    report.errors.push(HEMTTError::SIMPLE(format!("`{}` was not found in the cache", entry.path().display().to_string())))
                }
            }
        } else if cfg!(windows) && is_binarizable {
            let cursor = armake2::binarize::binarize(&PathBuf::from(entry.path()))?;
            pbo.files.insert(name, cursor);
        } else {
            if is_binarizable && !cfg!(windows) {
                report.warnings.push(HEMTTError::GENERIC(
                    format!("Unable to binarize `{}`", entry.path().display().to_string()),
                    "On non-windows systems binarize.exe cannot be used; file will packed as is".to_string()
                ));
            }

            let mut buffer: Vec<u8> = Vec::new();
            let mut reader = BufReader::new(&file);
            reader.read_to_end(&mut buffer).map_err(|e| HEMTTError::PATH(IOPathError{
                source: e,
                path: PathBuf::from(entry.path()),
            }))?;
            pbo.files.insert(
                Regex::new(".p3do$").unwrap().replace_all(&name, ".p3d").to_string(),
                Cursor::new(buffer.into_boxed_slice()),
            );
        }
    }
    Ok((pbo, report))
}
