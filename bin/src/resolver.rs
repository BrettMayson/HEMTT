use std::{collections::HashMap, path::PathBuf};

use hemtt_preprocessor::{Resolver, Token};
use vfs::VfsPath;

use crate::context::Context;

#[allow(clippy::module_name_repetitions)]
pub struct VfsResolver<'a> {
    vfs: &'a VfsPath,
    prefixes: HashMap<String, VfsPath>,
}

impl<'a> VfsResolver<'a> {
    pub fn new(ctx: &'a Context) -> Self {
        let mut prefixes = HashMap::new();
        for addon in ctx.addons() {
            prefixes.insert(
                addon.prefix().to_string(),
                ctx.vfs().join(addon.folder()).unwrap(),
            );
        }
        Self {
            vfs: ctx.vfs(),
            prefixes,
        }
    }
}

impl<'a> Resolver for VfsResolver<'a> {
    fn find_include(
        &self,
        context: &hemtt_preprocessor::Context<'_>,
        _root: &str,
        from: &str,
        to: &str,
        source: Vec<Token>,
    ) -> Result<(PathBuf, String), hemtt_preprocessor::Error> {
        trace!("find_include: {} {}", from, to);
        let path = if to.starts_with('\\') {
            let to = to.trim_start_matches('\\');
            if let Some(path) = self
                .prefixes
                .iter()
                .find(|(prefix, _)| {
                    let prefix = prefix.trim_start_matches('\\');
                    to.starts_with(&{
                        let mut prefix = prefix.to_string();
                        prefix.push('\\');
                        prefix
                    })
                })
                .map(|(prefix, path)| {
                    let mut path = PathBuf::from(path.as_str());
                    path.push(
                        to.strip_prefix(prefix)
                            .unwrap()
                            .trim_start_matches('\\')
                            .replace('\\', "/"),
                    );
                    path
                })
            {
                path
            } else {
                let include =
                    PathBuf::from("include").join(to.trim_start_matches('\\').replace('\\', "/"));
                if include.exists() {
                    include
                } else {
                    return Err(hemtt_preprocessor::Error::IncludeNotFound {
                        target: source,
                        trace: context.trace(),
                    });
                }
            }
        } else {
            let mut path = PathBuf::from(from).parent().unwrap().to_path_buf();
            path.push(to.replace('\\', "/"));
            path
        };
        if let Ok(mut file) = self
            .vfs
            .join(path.display().to_string().trim_start_matches('/'))
            .unwrap()
            .open_file()
        {
            let mut include_content = String::new();
            file.read_to_string(&mut include_content)?;
            Ok((path, include_content))
        } else {
            Err(hemtt_preprocessor::Error::IncludeNotFound {
                target: source,
                trace: context.trace(),
            })
        }
    }
}
