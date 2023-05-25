use std::{
    collections::HashMap,
    path::PathBuf,
    sync::atomic::{AtomicI16, Ordering},
};

use hemtt_config::{parse::Parse, rapify::Rapify, Config};
use hemtt_preprocessor::{preprocess_file, Resolver};
use hemtt_tokens::Token;
use peekmore::PeekMore;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use vfs::{VfsFileType, VfsPath};

use crate::{context::Context, error::Error};

use super::Module;

#[derive(Default)]
pub struct Rapifier;

impl Module for Rapifier {
    fn name(&self) -> &'static str {
        "Rapifier"
    }

    fn pre_build(&self, ctx: &Context) -> Result<(), Error> {
        let resolver = VfsResolver::new(ctx);
        let counter = AtomicI16::new(0);
        ctx.addons()
            .par_iter()
            .map(|addon| {
                for entry in ctx.vfs().join(addon.folder())?.walk_dir()? {
                    let entry = entry?;
                    if entry.metadata()?.file_type == VfsFileType::File
                        && can_preprocess(entry.as_str())
                    {
                        if entry.filename() == "config.cpp" {
                            if let Some(config) = addon.config() {
                                if !config.preprocess() {
                                    debug!("skiping {}", entry.as_str());
                                    continue;
                                }
                            }
                        }
                        debug!("rapifying {}", entry.as_str());
                        rapify(entry.clone(), ctx, &resolver)?;
                        counter.fetch_add(1, Ordering::Relaxed);
                    }
                }
                Ok(())
            })
            .collect::<Result<(), Error>>()?;
        info!("Rapified {} addon configs", counter.load(Ordering::Relaxed));
        Ok(())
    }
}

pub fn rapify(path: VfsPath, ctx: &Context, resolver: &VfsResolver) -> Result<(), Error> {
    let tokens = preprocess_file(path.as_str(), resolver)?;
    let rapified = Config::parse(
        ctx.config().hemtt().config(),
        &mut tokens.into_iter().peekmore(),
        &Token::builtin(None),
    )?;
    let out = if path.filename() == "config.cpp" {
        path.parent().join("config.bin").unwrap()
    } else {
        path
    };
    let mut output = out.create_file()?;
    rapified.rapify(&mut output, 0)?;
    Ok(())
}

pub fn can_preprocess(path: &str) -> bool {
    let path = PathBuf::from(path);
    let name = path
        .extension()
        .unwrap_or_else(|| std::ffi::OsStr::new(""))
        .to_str()
        .unwrap();
    ["cpp", "rvmat", "ext"].contains(&name)
}

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
        context: &hemtt_preprocessor::Context,
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
