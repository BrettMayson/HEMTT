use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use hemtt_handlebars::Variables;
use vfs::VfsFileType;

use crate::{context::AddonContext, HEMTTError, OkSkip, Stage, Task};

pub fn can_render(path: &str) -> bool {
    let path = PathBuf::from(path);
    let name = path
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new(""))
        .to_str()
        .unwrap();
    name.contains(".ht.") || name.ends_with(".ht")
}

pub fn destination(path: &str) -> String {
    let path = PathBuf::from(path);
    path.with_file_name(
        path.file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .replace(".ht.", ".")
            .trim_end_matches(".ht"),
    )
    .display()
    .to_string()
}

pub fn render(path: &str, ctx: &mut AddonContext) -> Result<(), HEMTTError> {
    let dest = destination(&path);
    debug!(
        "[PreBuild] [{:^width$}] [{}] {}",
        "render",
        ctx.addon.name(),
        dest,
        width = ctx.global.task_pad
    );
    let mut source = String::new();
    ctx.global
        .fs()
        .join(path)?
        .open_file()?
        .read_to_string(&mut source)?;
    match hemtt_handlebars::render(&source.replace("\\{", "\\\\{"), &{
        let mut vars = Variables::from(ctx.global.project());
        vars.append(ctx.addon.into());
        vars
    }) {
        Ok(out) => {
            let mut outfile = create_file!(Path::new(&dest))?;
            outfile.write_all(out.as_bytes())?;
            debug!(
                "[PreBuild] [{:^width$}] [{}] `{}` => `{}`",
                "render",
                ctx.addon.name(),
                path,
                dest,
                width = ctx.global.task_pad
            );
            ctx.global
                .fs()
                .join(&dest)?
                .create_file()?
                .write_all(out.as_bytes())?;
            Ok(())
        }
        Err(err) => {
            error!("Render error: {}", err);
            panic!("TODO convert error type")
            // Err(err.into())
        }
    }
}

pub struct Render {}
impl Render {}
impl Task for Render {
    fn name(&self) -> String {
        String::from("render")
    }

    fn hooks(&self) -> &[Stage] {
        &[Stage::Check, Stage::PreBuild]
    }

    fn check(&self, ctx: &mut AddonContext) -> Result<OkSkip, HEMTTError> {
        let mut ok = true;
        for entry in ctx.global.fs().join(ctx.addon.source())?.walk_dir()? {
            let entry = entry?;
            if can_render(entry.as_str()) {
                let dest = destination(entry.as_str());
                if ctx.global.fs().join(&dest)?.exists()? {
                    ok = false;
                    error!(
                        "[Check] [{:^width$}] [{}] target already exists: {}",
                        "render",
                        ctx.addon.name(),
                        dest,
                        width = ctx.global.task_pad
                    );
                }
            }
        }
        Ok((ok, false))
    }

    fn prebuild(&self, ctx: &mut AddonContext) -> Result<OkSkip, HEMTTError> {
        let mut ok = true;
        let mut count = 0;
        for entry in ctx.global.fs().join(ctx.addon.source())?.walk_dir()? {
            let entry = entry?;
            if entry.metadata()?.file_type == VfsFileType::File && can_render(entry.as_str()) {
                ok = ok && render(entry.as_str(), ctx).is_ok();
                count += 1;
            }
        }
        if count > 0 {
            debug!(
                "[PreBuild] [{:^width$}] [{}] rendered {} files",
                "render",
                ctx.addon.name(),
                count,
                width = ctx.global.task_pad
            );
        }
        Ok((ok, false))
    }
}
