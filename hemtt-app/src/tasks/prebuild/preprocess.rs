use std::path::PathBuf;

use vfs::VfsFileType;

use crate::{
    context::AddonContext,
    HEMTTError, OkSkip, Stage, Task,
};

pub fn can_preprocess(path: &str) -> bool {
    let path = PathBuf::from(path);
    let name = path
        .extension()
        .unwrap_or_else(|| std::ffi::OsStr::new(""))
        .to_str()
        .unwrap();
    ["cpp", "rvmat", "ext"].contains(&name)
}

pub fn preprocess(path: &str, ctx: &mut AddonContext) -> Result<(), HEMTTError> {
    debug!("Preprocessing: {}", path);
    let mut buf = String::new();
    ctx.global.fs().join(path)?.open_file()?.read_to_string(&mut buf)?;
    let processed = hemtt_arma_config::preprocess(hemtt_arma_config::tokenize(&buf).unwrap(), |include| {
        let mut buf = String::new();
        ctx.global.fs().join(include).unwrap().open_file().unwrap().read_to_string(&mut buf).unwrap();
        buf
    });
    let mut f = ctx.global.fs().join(path)?.create_file()?;
    f.write_all(hemtt_arma_config::render(processed?).as_bytes())?;
    Ok(())
}

pub struct Preprocess {}

impl Task for Preprocess {
    fn name(&self) -> String {
        String::from("preprocess")
    }

    fn hooks(&self) -> &[Stage] {
        &[Stage::Check, Stage::PreBuild, Stage::PostBuild]
    }

    fn prebuild(&self, ctx: &mut AddonContext) -> Result<OkSkip, HEMTTError> {
        let mut ok = true;
        let mut count = 0;
        for entry in ctx.global.fs().join(ctx.addon.source())?.walk_dir()? {
            let entry = entry?;
            trace!("Entry: {:?}", entry);
            if ok && entry.metadata()?.file_type == VfsFileType::File && can_preprocess(entry.as_str()) {
                let res = preprocess(entry.as_str(), ctx);
                if let Err(e) = res {
                    ok = false;
                    error!("{}", e);
                }
                count += 1;
            }
        }
        if count > 0 {
            debug!(
                "[PreBuild] [{:^width$}] [{}] preprocessed {} files",
                "preprocess",
                ctx.addon.name(),
                count,
                width = ctx.global.task_pad
            );
        }
        Ok((ok, false))
    }
}
