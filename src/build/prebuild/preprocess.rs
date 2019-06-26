use std::path::{Path, PathBuf};
use std::cmp::min;

use indicatif::ProgressBar;
use armake2::preprocess::preprocess;
use walkdir::WalkDir;
use regex::Regex;

use crate::{HEMTTError, FileErrorLineNumber, Task, Project, Addon, Report};

pub static RAPABLE: &[&str] = &["cpp", "rvmat", "ext"];

pub fn can_preprocess(p: &Path) -> (bool, bool) {
    let ext = p.extension().unwrap_or_else(|| std::ffi::OsStr::new("")).to_str().unwrap();
    if RAPABLE.contains(&ext) { return (true, true); }
    //(false, ext == "sqf")
    (false, false)
}

#[derive(Clone)]
pub struct Preprocess {}
impl Task for Preprocess {
    fn pre_can_run(&self, _addon: &Addon, _p: &Project) -> Result<bool, HEMTTError> {
        Ok(true)
    }
    fn pre_run(&self, addon: &Addon, p: &Project, pb: &ProgressBar) -> Result<Report, HEMTTError> {
        let mut report = Report::new();
        for entry in WalkDir::new(&addon.folder()) {
            pb.tick();
            let path = entry.unwrap();
            let (can_rap, can_check) = can_preprocess(&path.path());
            if can_check {
                let (original_path, rendered_path) = crate::RENDERED.lock().unwrap().get_paths(path.path().display().to_string());
                let raw = std::fs::read_to_string(Path::new(&rendered_path))?;
                if raw.len() < 3 { continue; }
                let mut includes = p.include.clone();
                includes.insert(0, PathBuf::from("."));
                pb.set_message(&format!("Preprocess: {}", rendered_path));
                match preprocess(raw.clone(), Some(PathBuf::from(&original_path)), &includes) {
                    Ok((output, info)) => {
                        if can_rap {
                            let mut warnings: Vec<(usize, String, Option<&'static str>)> = Vec::new();
                            armake2::config::config_grammar::config(&output, &mut warnings).unwrap();
                            for w in warnings {
                                pb.tick();
                                let mut line = output[..w.0].chars().filter(|c| c == &'\n').count();
                                let file = info.line_origins[min(line, info.line_origins.len()) - 1].1.as_ref().map(|p| p.to_str().unwrap().to_string());
                                line = info.line_origins[min(line, info.line_origins.len()) - 1].0 as usize + 1;

                                let filename = file.unwrap();
                                report.warnings.push(HEMTTError::LINENO(FileErrorLineNumber {
                                    content: crate::get_line_at(Path::new(&filename), line)?,
                                    col: None,
                                    line: Some(line),
                                    file: filename,
                                    error: w.1,
                                    note: None,
                                }));
                            }
                            crate::CACHED.lock().unwrap().add(rendered_path, output)?;
                        }
                    },
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
    let unexpected_token = Regex::new(r#"(?ms)(?:.+?)In line (.+?):(\d+?):(.+?)Unexpected token "(.+?)", expected: (.+?)$"#).unwrap();
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
    } else if unexpected_token.is_match(&error) {
        let cap = unexpected_token.captures(&error).unwrap();
        let line = cap[2].parse::<usize>().unwrap();
        return Ok(HEMTTError::LINENO(FileErrorLineNumber {
            error: format!("Unexpected token `{}`", &cap[4]),
            col: None,
            line: Some(line),
            file: cap[1].to_string(),
            note: None,
            content: crate::get_line_at(&Path::new(&cap[1]), line)?,
        }))
    }
    eprintln!("unknown armake error `{}`", error);
    unimplemented!()
}
