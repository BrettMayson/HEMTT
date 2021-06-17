use std::io::{Read, Write};
use std::path::PathBuf;

use hemtt_handlebars::Variables;
use vfs::{VfsFileType, VfsPath};

use crate::{context::AddonContext, HEMTTError, Stage, Task};

pub fn can_populate(path: &str) -> bool {
    let path = PathBuf::from(path);
    let name = path
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new(""))
        .to_str()
        .unwrap();
    name.contains(".ht.") || name.ends_with(".ht") || name == "$PBOPREFIX$"
}

pub fn destination(path: VfsPath) -> Result<VfsPath, HEMTTError> {
    path.parent()
        .unwrap()
        .join(path.filename().replace(".ht.", ".").trim_end_matches(".ht"))
        .map_err(|e| e.into())
}

pub fn populate(source: VfsPath, dest: VfsPath, ctx: &mut AddonContext) -> Result<(), HEMTTError> {
    ctx.debug(&format!("`{}` => `{}`", source.as_str(), dest.as_str()));
    let mut buf = String::new();
    source.open_file()?.read_to_string(&mut buf)?;
    match hemtt_handlebars::render(&buf.replace("\\{", "\\\\{"), &{
        let mut vars = Variables::from(ctx.global().project());
        vars.append(ctx.addon().into());
        vars
    }) {
        Ok(out) => {
            dest.create_file()?.write_all(out.as_bytes())?;
            Ok(())
        }
        Err(err) => {
            error!("Populate error: {}", err);
            panic!("TODO convert error type")
        }
    }
}

pub struct Populate {}
impl Populate {}
impl Task for Populate {
    fn name(&self) -> String {
        String::from("populate")
    }

    fn hooks(&self) -> &[Stage] {
        &[Stage::Check]
    }

    fn check(&self, ctx: &mut AddonContext) -> Result<(), HEMTTError> {
        for entry in ctx.global().fs().join(ctx.addon().source())?.walk_dir()? {
            let entry = entry?;
            ctx.trace(&format!("checking file: {:?}", entry));
            if can_populate(entry.as_str()) {
                let dest = destination(entry)?;
                if dest.filename() != "$PBOPREFIX$" && dest.exists()? {
                    ctx.warn(&format!("target already exists: {}", dest.as_str()));
                }
            }
        }
        for entry in ctx.global().fs().join(ctx.addon().source())?.walk_dir()? {
            let entry = entry?;
            if entry.metadata()?.file_type == VfsFileType::File && can_populate(entry.as_str()) {
                populate(entry.clone(), destination(entry)?, ctx)?;
            }
        }
        Ok(())
    }
}
