use std::{
    io::{Read, Seek},
    path::PathBuf,
    sync::Arc,
};

use hemtt_workspace::reporting::{Code, Severity};

use crate::{context::Context, report::Report};

use super::Module;

pub const TEXT_EXTENSIONS: [&str; 6] = ["sqf", "txt", "hpp", "cpp", "rvmat", "ext", "inc"];

#[derive(Default)]
pub struct FineNewLineCheck {}

impl Module for FineNewLineCheck {
    fn name(&self) -> &'static str {
        "Fine New Line Check"
    }

    fn check(&self, ctx: &Context) -> Result<Report, crate::Error> {
        fn files_to_check(root: &PathBuf) -> Vec<PathBuf> {
            walkdir::WalkDir::new(root)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| e.file_type().is_file())
                .filter(|e| !e.path().display().to_string().contains(".hemttout"))
                .filter(|e| {
                    e.path()
                        .extension()
                        .is_some_and(|e| TEXT_EXTENSIONS.contains(&e.to_str().unwrap_or_default()))
                })
                .map(|e| e.path().to_path_buf())
                .collect::<Vec<_>>()
        }
        let mut report = Report::new();
        for folder in ["addons", "optionals"] {
            let folder = ctx.project_folder().join(folder);
            if !folder.exists() {
                continue;
            }
            let files = files_to_check(&folder);
            for path in files {
                // Seek to end instead of reading the whole file
                let mut file = std::fs::File::open(&path)?;
                let display_path = path
                    .display()
                    .to_string()
                    .trim_start_matches(ctx.project_folder().to_str().unwrap_or_default())
                    .to_string();
                if file.seek(std::io::SeekFrom::End(-1)).is_err() {
                    report.push(Arc::new(FNLError { file: display_path }));
                    continue;
                }
                let mut last_byte = [0; 1];
                if file.read_exact(&mut last_byte).is_err() {
                    report.push(Arc::new(FNLError { file: display_path }));
                    continue;
                }
                if last_byte[0] != b'\n' {
                    report.push(Arc::new(FNLError { file: display_path }));
                }
            }
        }
        Ok(report)
    }
}

struct FNLError {
    file: String,
}

impl Code for FNLError {
    fn ident(&self) -> &'static str {
        "FNL"
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn message(&self) -> String {
        format!("File `{}` is missing a newline at the end", self.file)
    }

    fn help(&self) -> Option<String> {
        Some("Run `hemtt utils fnl` to insert newlines at the end of files".to_string())
    }
}
