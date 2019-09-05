use std::io::Write;
use std::path::Path;

use indicatif::ProgressBar;
use walkdir::WalkDir;

use crate::{Addon, HEMTTError, Project, Report, Task};

pub fn can_render(p: &Path) -> bool {
    let name = p.file_name().unwrap_or_else(|| std::ffi::OsStr::new("")).to_str().unwrap();
    name.contains(".ht.") || name.ends_with(".ht")
}

pub fn render(path: &Path, addon: &Addon, p: &Project) -> Result<Report, HEMTTError> {
    let vars = &addon.get_variables(p);
    let mut report = Report::new();
    match crate::render::run(&std::fs::read_to_string(path)?.replace("\\{", "\\\\{"), vars) {
        Ok(out) => {
            let dest = path
                .display()
                .to_string()
                .replace(".ht.", ".")
                .trim_end_matches(".ht")
                .to_string();
            let mut outfile = create_file!(Path::new(&dest))?;
            outfile.write_all(out.as_bytes())?;
            crate::RENDERED.lock().unwrap().add(path.display().to_string(), dest)?;
        }
        Err(err) => {
            if let HEMTTError::LINENO(mut e) = err {
                e.content = crate::CACHED
                    .lock()
                    .unwrap()
                    .get_line(path.as_os_str().to_str().unwrap(), e.line.unwrap_or(1))?;
                e.file = path.display().to_string();
                report.unique_error(HEMTTError::LINENO(e));
            } else {
                report.errors.push(err);
            }
        }
    }
    Ok(report)
}

#[derive(Clone)]
pub struct Render {}
impl Task for Render {
    fn can_run(&self, _addon: &Addon, _: &Report, _p: &Project) -> Result<bool, HEMTTError> {
        Ok(true)
    }

    fn parallel(&self, addon: &Addon, _: &Report, p: &Project, pb: &ProgressBar) -> Result<Report, HEMTTError> {
        let mut report = Report::new();
        for entry in WalkDir::new(&addon.folder()) {
            let path = entry.unwrap();
            if can_render(&path.path()) {
                pb.set_message(&format!("Render: {}", path.path().display().to_string()));
                report.absorb(render(path.path(), addon, p)?);
            }
        }
        Ok(report)
    }
}
