use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use walkdir::WalkDir;

use hemtt_cache::Temporary;
use hemtt_handlebars::Variables;

use crate::{
    context::{AddonContext, AddonListContext},
    HEMTTError, OkSkip, Stage, Task,
};

pub fn can_render(p: &Path) -> bool {
    let name = p
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new(""))
        .to_str()
        .unwrap();
    name.contains(".ht.") || name.ends_with(".ht")
}

pub fn destination(path: &Path) -> PathBuf {
    path.with_file_name(
        path.file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .replace(".ht.", ".")
            .trim_end_matches(".ht"),
    )
}

pub fn render(path: &Path, ctx: &mut AddonContext) -> Result<(), HEMTTError> {
    let dest = destination(path);
    debug!(
        "[PreBuild] [{:^width$}] [{}] {}",
        "render",
        ctx.addon.name,
        dest.display(),
        width = ctx.global.task_pad
    );
    let mut source = String::new();
    ctx.global.cache.read(path)?.read_to_string(&mut source)?;
    match hemtt_handlebars::render(&source.replace("\\{", "\\\\{"), &{
        let mut vars = Variables::from(ctx.global.project);
        vars.append(ctx.addon.into());
        vars
    }) {
        Ok(out) => {
            let mut outfile = create_file!(Path::new(&dest))?;
            outfile.write_all(out.as_bytes())?;
            debug!(
                "[PreBuild] [{:^width$}] [{}] `{}` => `{}`",
                "render",
                ctx.addon.name,
                path.display(),
                dest.display(),
                width = ctx.global.task_pad
            );
            // crate::RENDERED
            //     .lock()
            //     .unwrap()
            //     .add(path.display().to_string(), dest.clone())?;
            ctx.global.cache.insert(dest, Temporary::from_string(&out)?);
            Ok(())
        }
        Err(err) => {
            error!("Render error: {}", err);
            panic!("TODO convert error type")
            // Err(err.into())
        }
    }
}

pub struct Render {
    rendered: RwLock<Vec<PathBuf>>,
}
impl Render {
    pub fn new() -> Self {
        Self {
            rendered: RwLock::new(Vec::new()),
        }
    }
}
impl Task for Render {
    fn name(&self) -> String {
        String::from("render")
    }

    fn hooks(&self) -> &[Stage] {
        &[Stage::Check, Stage::PreBuild, Stage::PostBuild]
    }

    fn check(&self, ctx: &mut AddonContext) -> Result<OkSkip, HEMTTError> {
        let mut ok = true;
        for entry in WalkDir::new(&ctx.addon.source()) {
            let path = entry.unwrap();
            if can_render(path.path()) {
                let dest = destination(path.path());
                if dest.exists() {
                    ok = false;
                    error!(
                        "[Check] [{:^width$}] [{}] target already exists: {}",
                        "render",
                        ctx.addon.name,
                        dest.display(),
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
        for entry in WalkDir::new(&ctx.addon.source()) {
            let path = entry.unwrap();
            if can_render(path.path()) {
                let dest = destination(path.path());
                ok = ok && render(path.path(), ctx).is_ok();
                self.rendered.write().unwrap().push(dest);
                count += 1;
            }
        }
        if count > 0 {
            debug!(
                "[PreBuild] [{:^width$}] [{}] rendered {} files",
                "render",
                ctx.addon.name,
                count,
                width = ctx.global.task_pad
            );
        }
        Ok((ok, false))
    }

    fn postbuild_single(&self, ctx: &mut AddonListContext) -> Result<(), HEMTTError> {
        let mut err = None;
        self.rendered.read().unwrap().iter().for_each(|f| {
            if f.exists() {
                trace!("removing rendered file: {}", f.display(),);
                if let Err(e) = remove_file!(f) {
                    error!(
                        "[PostBuild] [{:^width$}] failed to delete rendered file: {}",
                        "render",
                        f.display(),
                        width = ctx.global.task_pad
                    );
                    err = Some(e);
                }
            } else {
                warn!(
                    "[PostBuild] [{:^width$}] expected rendered file was missing: {}",
                    "render",
                    f.display(),
                    width = ctx.global.task_pad
                );
            }
        });
        Ok(())
    }
}
