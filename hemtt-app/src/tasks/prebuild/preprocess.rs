use std::{path::PathBuf, sync::Arc};

use hemtt_arma_config::resolver::{ResolvedFile, Resolver};
use vfs::{VfsFileType, VfsPath};

use crate::{context::AddonContext, HEMTTError, Stage, Task};

use super::prefix::PrefixMap;

pub fn can_preprocess(path: &str) -> bool {
    let path = PathBuf::from(path);
    let name = path
        .extension()
        .unwrap_or_else(|| std::ffi::OsStr::new(""))
        .to_str()
        .unwrap();
    ["cpp", "rvmat", "ext"].contains(&name)
}

pub fn preprocess(path: VfsPath, ctx: &mut AddonContext) -> Result<(), HEMTTError> {
    ctx.debug(&format!("Preprocessing: {}", path.as_str()));
    let mut buf = String::new();
    path.open_file()?.read_to_string(&mut buf)?;
    let processed = hemtt_arma_config::preprocess(
        hemtt_arma_config::tokenize(&buf, path.as_str()).unwrap(),
        ctx.addon().source(),
        VfsResolver::new(
            ctx.global().fs().clone(),
            ctx.global().container.get::<PrefixMap>(),
        ),
    );
    let mut f = path.create_file()?;
    f.write_all(hemtt_arma_config::render(processed?).export().as_bytes())?;
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

    fn prebuild(&self, ctx: &mut AddonContext) -> Result<(), HEMTTError> {
        let mut ok = true;
        let mut count = 0;
        for entry in ctx.global().fs().join(ctx.addon().source())?.walk_dir()? {
            let entry = entry?;
            if ok
                && entry.metadata()?.file_type == VfsFileType::File
                && can_preprocess(entry.as_str())
            {
                let res = preprocess(entry, ctx);
                if let Err(e) = res {
                    ok = false;
                    ctx.error(&format!("{}", e));
                    ctx.set_failed(e);
                }
                count += 1;
            }
        }
        if count > 0 {
            ctx.debug(&format!("preprocessed {} files", count,));
        }
        Ok(())
    }
}

#[derive(Clone)]
struct VfsResolver<'a>(Arc<VfsPath>, &'a PrefixMap);
impl<'a> VfsResolver<'a> {
    pub fn new(path: VfsPath, prefixes: &'a PrefixMap) -> Self {
        Self(Arc::new(path), prefixes)
    }
}
impl<'a> Resolver for VfsResolver<'a> {
    fn resolve(&self, _root: &str, from: &str, to: &str) -> Result<ResolvedFile, HEMTTError> {
        let to = to.trim_start_matches('/');
        let from = from.trim_start_matches('/');
        trace!("Resolving from {} to {} on {:?}", from, to, self.0);
        trace!("root {:?}", self.0);
        let mut buf = String::new();
        let new_path = self
            .0
            .join(from)
            .unwrap()
            .parent()
            .unwrap()
            .join(&to)
            .unwrap();
        match new_path.open_file() {
            Ok(mut f) => {
                f.read_to_string(&mut buf).unwrap();
                Ok(ResolvedFile::new(new_path.as_str(), buf))
            }
            Err(e) => {
                let to = to.replace("\\", "/");
                trace!("using prefix map: {:?}", self.1.inner());
                // Check for prefix
                if let Some((prefix, path)) = self.1.inner().iter().find(|(prefix, _)| {
                    debug!("prefix `{}` vs to `{}`", prefix.replace("\\", "/"), to);
                    to.starts_with(&format!("/{}", prefix.replace("\\", "/")))
                }) {
                    debug!("found a match: {:?} with {}", path, prefix);
                    debug!(
                        "{:#?}",
                        self.0
                            .join("addons/main")
                            .unwrap()
                            .read_dir()
                            .unwrap()
                            .for_each(|f| {
                                f.open_file().unwrap();
                                debug!("{}", f.as_str());
                            })
                    );
                    debug!(
                        "joining {:?}",
                        to.trim_start_matches(&format!("/{}/", prefix.replace("\\", "/")))
                    );
                    let new_path = self
                        .0
                        .join(&path.trim_start_matches('/'))
                        .unwrap()
                        .join(to.trim_start_matches(&format!("/{}/", prefix.replace("\\", "/"))))
                        .unwrap();
                    debug!("newpath {:?}", new_path);
                    new_path
                        .open_file()
                        .unwrap()
                        .read_to_string(&mut buf)
                        .unwrap();
                    Ok(ResolvedFile::new(new_path.as_str(), buf))
                } else {
                    // TODO use the project's includes vec
                    if PathBuf::from("include").exists() {
                        let new_path = self.0.join(&format!("include{}", to)).unwrap();
                        new_path
                            .open_file()
                            .unwrap()
                            .read_to_string(&mut buf)
                            .unwrap();
                        Ok(ResolvedFile::new(new_path.as_str(), buf))
                    } else {
                        Err(e.into())
                    }
                }
            }
        }
    }
}
