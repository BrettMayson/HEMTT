use std::collections::{HashMap};
use std::fs::File;
use std::path::{Path};
use std::io::Write;

use walkdir::WalkDir;

use crate::flow::{Task, Report};
use crate::error::{HEMTTError};
use crate::project::Project;

pub fn can_render(p: &Path) -> bool {
    let name = p.file_name().unwrap_or_else(|| std::ffi::OsStr::new("")).to_str().unwrap();
    name.contains(".ht.") || name.ends_with(".ht")
}

pub fn render(path: &Path, report: &mut Report, addon: &crate::build::Addon, p: &mut Project) -> Result<(), HEMTTError> {
    let vars = &addon.get_variables(p);
    match crate::render::run(&std::fs::read_to_string(path)?.replace("\\{", "\\\\{"), vars) {
        Ok(out) => {
            let dest = path.display().to_string().replace(".ht.", ".").trim_end_matches(".ht").to_string();
            let mut outfile = File::create(Path::new(&dest))?;
            outfile.write_all(out.as_bytes())?;
            p.rendered_files.add(path.display().to_string(), dest)?;
        },
        Err(err) => {
            if let HEMTTError::LINENO(mut e) = err {
                e.content = crate::get_line_at(&path, e.line.unwrap_or(1))?;
                e.file = path.display().to_string();
                report.unique_error(HEMTTError::LINENO(e));
            } else {
                report.errors.push(err);
            }
        },
    }
    Ok(())
}

pub struct Render {}
impl Task for Render {
    fn pre_can_run(&self, _addon: &crate::build::Addon, _p: &Project) -> Result<bool, HEMTTError> {
        Ok(true)
    }
    fn pre_run(&self, addon: &crate::build::Addon, p: &mut Project) -> Result<Report, HEMTTError> {
        let mut report = Report::new();
        for entry in WalkDir::new(&addon.folder()) {
            let path = entry.unwrap();
            if can_render(&path.path()) {
                render(path.path(), &mut report, addon, p)?;
            }
        }
        Ok(report)
    }
}

pub struct RenderedFiles {
    redirects: HashMap<String, String>,
}

impl RenderedFiles {
    pub fn new() -> Self {
        Self {
            redirects: HashMap::new(),
        }
    }

    pub fn add(&mut self, original: String, tmp: String) -> Result<(), HEMTTError> {
        self.redirects.insert(original.clone(), tmp.clone());
        Ok(())
    }

    pub fn get_path(&self, original: String) -> Option<&String> {
        self.redirects.get(&original)
    }

    pub fn get_paths(&self, original: String) -> (String, String) {
        let rendered = can_render(&Path::new(&original));
        if rendered {
            (original.replace(".ht.", ".").trim_end_matches(".ht").to_string(), self.redirects.get(&original).unwrap().to_string())
        } else {
            (original.clone(), original)
        }
    }
}

impl Drop for RenderedFiles {
    fn drop(&mut self) {
        for (_, tmp) in self.redirects.iter() {
            std::fs::remove_file(tmp);
        }
    }
}
