use std::path::{Path, PathBuf};

use armake2::preprocess::preprocess;
use walkdir::WalkDir;
use regex::Regex;

use crate::flow::{Task, Report};
use crate::error::{HEMTTError, FileErrorLineNumber};
use crate::project::Project;

pub fn can_preprocess(p: &Path) -> bool {
    if p.file_name().unwrap_or_else(|| std::ffi::OsStr::new("")).to_str().unwrap() == "config.cpp" { return true; }
    p.extension().unwrap_or_else(|| std::ffi::OsStr::new("")).to_str().unwrap() == "sqf"
}

pub struct Preprocess {}
impl Task for Preprocess {
    fn pre_can_run(&self, _addon: &crate::build::Addon, _p: &Project) -> Result<bool, HEMTTError> {
        Ok(true)
    }
    fn pre_run(&self, addon: &crate::build::Addon, p: &mut Project) -> Result<Report, HEMTTError> {
        let mut report = Report::new();
        for entry in WalkDir::new(&addon.folder()) {
            let path = entry.unwrap();
            if can_preprocess(&path.path()) {
                let (original_path, rendered_path) = p.rendered_files.get_paths(path.path().display().to_string());
                let raw = std::fs::read_to_string(Path::new(&rendered_path))?;
                if raw.len() < 3 { return Ok(report) }
                let mut includes = p.include.clone();
                includes.insert(0, PathBuf::from("."));
                match preprocess(raw, Some(PathBuf::from(original_path)), &includes) {
                    Ok((output, _info)) => {},
                    Err(e) => {
                        report.unique_error(convert_preprocess_error(e.to_string())?);
                        report.can_proceed = false;
                    }
                }
            }
        }
        Ok(report)
    }
}

pub fn convert_preprocess_error(error: String) -> Result<HEMTTError, HEMTTError> {
    let include_error = Regex::new(r#"(?m)File "(.+?)" included from "(.+?)" not found."#).unwrap();
    if include_error.is_match(&error) {
        let cap = include_error.captures(&error).unwrap();
        let contents = std::fs::read_to_string(&cap[2])?;
        for (i, content) in contents.lines().enumerate() {
            if content.contains(&format!("#include \"{}\"", &cap[1])) {
                return Ok(HEMTTError::LINENO(FileErrorLineNumber {
                    error: format!("Included file `{}` could not be found", &cap[1]),
                    content: content.to_string(),
                    line: Some(i + 1),
                    col: None,
                    file: cap[2].to_string(),
                    note: None,
                }));
            }
        }
    }
    unimplemented!()
}
