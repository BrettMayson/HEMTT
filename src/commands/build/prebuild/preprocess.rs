use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

#[cfg(not(windows))]
use indicatif::ProgressBar;
#[cfg(windows)]
use indicatif_windows::ProgressBar;

use regex::Regex;
use walkdir::WalkDir;

use crate::{Addon, FileErrorLineNumber, HEMTTError, Project, Report, Task};

pub static RAPABLE: &[&str] = &["cpp", "rvmat", "ext"];
static CMD_GAP: usize = 18;

pub fn can_preprocess(p: &Path) -> bool {
    RAPABLE.contains(&p.extension().unwrap_or_else(|| std::ffi::OsStr::new("")).to_str().unwrap())
}

#[derive(Clone)]
pub struct Preprocess {}
impl Task for Preprocess {
    fn can_run(&self, _: &Addon, _: &Report, _: &Project) -> Result<bool, HEMTTError> {
        Ok(true)
    }

    fn parallel(&self, addon: &Addon, _: &Report, p: &Project, pb: &ProgressBar) -> Result<Report, HEMTTError> {
        let mut report = Report::new();
        for entry in WalkDir::new(&addon.folder()) {
            pb.set_message("Looking for files to preprocess");
            pb.tick();
            let path = entry.unwrap();
            let can_rap = can_preprocess(&path.path());
            if can_rap {
                pb.set_message("Waiting for render lock");
                let (original_path, rendered_path) =
                    crate::RENDERED.lock().unwrap().get_paths(path.path().display().to_string());
                pb.set_message(&format!("{} - {}", &fill_space!(" ", CMD_GAP, "Reading"), rendered_path));
                let raw = crate::CACHED.lock().unwrap().clean_comments(&rendered_path)?.clone();
                if raw.len() < 3 {
                    pb.set_message(&format!("{} - {}", &fill_space!(" ", CMD_GAP, "Skipping"), rendered_path));
                    continue;
                }
                let mut includes = p.include.clone();
                includes.insert(0, PathBuf::from("."));
                pb.set_message(&format!("{} - {}", &fill_space!(" ", CMD_GAP, "Preprocess"), rendered_path));
                // match preprocess(raw.clone(), Some(PathBuf::from(&original_path)), &includes, |path| {
                //     pb.set_message(&format!("{} - {}", &fill_space!(" ", CMD_GAP, "Preprocess"), rendered_path));
                //     crate::CACHED.lock().unwrap().clean_comments(path.to_str().unwrap())
                // }) {
                match armake2::Config::from_string(raw.clone(), Some(PathBuf::from(&original_path)), &includes, |path| {
                    pb.set_message(&format!("{} - {}", &fill_space!(" ", CMD_GAP, "Preprocess"), rendered_path));
                    crate::CACHED.lock().unwrap().clean_comments(path.to_str().unwrap()).unwrap()
                }) {
                    Ok(rapped) => {
                        pb.set_message(&format!("{} - {}", &fill_space!(" ", CMD_GAP, "Rapify"), rendered_path));
                        // let mut warnings: Vec<(usize, String, Option<&'static str>)> = Vec::new();
                        // let rapped = armake2::Config::from_string(&output, Some(PathBuf::from(&original_path)))
                        //     .map_err(|e| HEMTTError::from_armake_parse(e, &rendered_path, Some(output.clone())))?;
                        // let total = warnings.len();
                        // for (i, w) in warnings.into_iter().enumerate() {
                        //     let text = format!("Report {}/{}", i, total);
                        //     pb.set_message(&format!("{} - {}", &fill_space!(" ", CMD_GAP, &text), rendered_path));
                        //     let mut line = output[..w.0].chars().filter(|c| c == &'\n').count();
                        //     let file = info.line_origins[min(line, info.line_origins.len()) - 1]
                        //         .1
                        //         .as_ref()
                        //         .map(|p| p.to_str().unwrap().to_string());
                        //     line = info.line_origins[min(line, info.line_origins.len()) - 1].0 as usize + 1;

                        //     let filename = file.unwrap();
                        //     report.warnings.push(HEMTTError::LINENO(FileErrorLineNumber {
                        //         content: crate::CACHED.lock().unwrap().get_line(&filename, line)?,
                        //         col: None,
                        //         line: Some(line),
                        //         file: filename,
                        //         error: w.1,
                        //         note: None,
                        //     }));
                        // }
                        pb.set_message(&format!("{} - {}", &fill_space!(" ", CMD_GAP, "Caching"), rendered_path));
                        let mut c = Cursor::new(Vec::new());
                        rapped.write_rapified(&mut c)?;
                        c.seek(SeekFrom::Start(0))?;
                        let mut out = Vec::new();
                        c.read_to_end(&mut out)?;
                        pb.set_message("Waiting for cache lock");
                        crate::CACHED
                            .lock()
                            .unwrap()
                            .insert_bytes(&rendered_path.replace("config.cpp", "config.bin"), out)?;
                    }
                    Err(e) => {
                        // Unable to clone HEMTTError
                        //report.unique_error(HEMTTError::from(e));
                        report.stop = Some((true, HEMTTError::from(e)));
                    }
                }
            }
        }
        Ok(report)
    }
}

pub fn convert_preprocess_error(error: String) -> Result<HEMTTError, HEMTTError> {
    let include_error = Regex::new(r#"(?m)File "(.+?)" included from "(.+?)" not found."#).unwrap();
    let unexpected_token =
        Regex::new(r#"(?ms)(?:.+?)In line (.+?):(\d+?):(.+?)Unexpected token "(.+?)", expected: (.+?)$"#).unwrap();
    if include_error.is_match(&error) {
        let cap = include_error.captures(&error).unwrap();
        let contents = crate::CACHED.lock().unwrap().lines(&cap[2])?;
        for (i, content) in contents.into_iter().enumerate() {
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
            content: crate::CACHED.lock().unwrap().get_line(&cap[1], line)?,
        }));
    }
    std::fs::write("armake2.error", &error)?;
    eprintln!("unknown armake error `{}`", error);
    unimplemented!()
}
