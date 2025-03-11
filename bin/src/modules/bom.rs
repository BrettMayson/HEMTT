use std::{io::Read, path::PathBuf, sync::Arc};

use hemtt_workspace::reporting::Code;

use crate::{context::Context, report::Report};

use super::Module;

#[derive(Default)]
pub struct BOMCheck {}

impl Module for BOMCheck {
    fn name(&self) -> &'static str {
        "BOM Check"
    }

    fn check(&self, ctx: &Context) -> Result<Report, crate::Error> {
        fn files_to_check(root: &PathBuf) -> Vec<PathBuf> {
            const IGNORED_EXTENSIONS: [&str; 4] = ["p3d", "rtm", "bin", "paa"];
            walkdir::WalkDir::new(root)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| e.file_type().is_file())
                .filter(|e| !e.path().display().to_string().contains(".hemttout"))
                .filter(|e| {
                    e.path().extension().is_some_and(|e| {
                        !IGNORED_EXTENSIONS.contains(&e.to_str().unwrap_or_default())
                    })
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
                let mut buffer = [0; 3];
                let mut file = std::fs::File::open(&path)?;
                if file.read_exact(&mut buffer).is_err() {
                    continue;
                }
                if buffer == [0xEF, 0xBB, 0xBF] {
                    report.push(Arc::new(BOMError {
                        file: {
                            path.display()
                                .to_string()
                                .strip_prefix(&ctx.project_folder().display().to_string())
                                .unwrap_or_default()
                                .to_string()
                        },
                    }));
                }
            }
        }
        Ok(report)
    }
}

struct BOMError {
    file: String,
}

impl Code for BOMError {
    fn ident(&self) -> &'static str {
        "BOM"
    }

    fn message(&self) -> String {
        format!("File `{}` has a BOM marker", self.file)
    }

    fn help(&self) -> Option<String> {
        Some("Run `hemtt utils bom` to remove BOM markers".to_string())
    }
}
