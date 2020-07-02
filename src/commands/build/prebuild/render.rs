use std::io::Write;
use std::path::Path;

use walkdir::WalkDir;

use crate::{Addon, HEMTTError, OkSkip, Project, Stage, Task};

pub fn can_render(p: &Path) -> bool {
    let name = p.file_name().unwrap_or_else(|| std::ffi::OsStr::new("")).to_str().unwrap();
    name.contains(".ht.") || name.ends_with(".ht")
}

pub fn render(path: &Path, addon: &Addon, p: &Project) -> Result<(), HEMTTError> {
    debug!("render file: {:?}", path);
    let vars = &addon.get_variables(p);
    match crate::render::run(
        &std::fs::read_to_string(path)?.replace("\\{", "\\\\{"),
        Some(path.to_str().unwrap()),
        vars,
    ) {
        Ok(out) => {
            let dest = path
                .display()
                .to_string()
                .replace(".ht.", ".")
                .trim_end_matches(".ht")
                .to_string();
            let mut outfile = create_file!(Path::new(&dest))?;
            outfile.write_all(out.as_bytes())?;
            debug!("rendered `{}` to `{}`", path.display(), dest);
            crate::RENDERED
                .lock()
                .unwrap()
                .add(path.display().to_string(), dest.clone())?;
            crate::CACHED.lock().unwrap().insert(&dest, out)?;
            Ok(())
        }
        Err(err) => {
            if let HEMTTError::LINENO(mut e) = err {
                e.content = crate::CACHED
                    .lock()
                    .unwrap()
                    .get_line(path.as_os_str().to_str().unwrap(), e.line.unwrap_or(1))?;
                e.file = path.display().to_string();
                error!("Render error: {:?}", e);
                Err(HEMTTError::LINENO(e))
            } else {
                error!("Render error: {}", err);
                Err(err)
            }
        }
    }
}

#[derive(Clone)]
pub struct Render {}
impl Task for Render {
    fn can_run(&self, _addon: &Addon, _p: &Project, _: &Stage) -> Result<bool, HEMTTError> {
        Ok(true)
    }

    fn parallel(&self, addon: &Addon, p: &Project, _: &Stage) -> Result<OkSkip, HEMTTError> {
        let mut ok = true;
        for entry in WalkDir::new(&addon.folder()) {
            let path = entry.unwrap();
            if can_render(path.path()) {
                ok = ok && render(path.path(), addon, p).is_ok();
            }
        }
        Ok((ok, false))
    }
}
