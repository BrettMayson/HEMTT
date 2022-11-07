use std::{collections::HashMap, path::PathBuf};

use hemtt_config::{Config, Parse, Rapify};
use hemtt_pbo::{prefix::FILES, Prefix};
use hemtt_preprocessor::{preprocess_file, Resolver};
use hemtt_tokens::Token;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use vfs::{VfsFileType, VfsPath};

use crate::{context::Context, error::Error};

use super::Module;

pub struct Preprocessor;

impl Preprocessor {
    pub const fn new() -> Self {
        Self
    }
}

impl Module for Preprocessor {
    fn name(&self) -> &'static str {
        "Preprocessor"
    }

    fn pre_build(&self, ctx: &Context) -> Result<(), Error> {
        // TODO map to extra error
        ctx.addons().par_iter().for_each(|addon| {
            // for addon in ctx.addons() {
            // TODO fix error in vfs
            for entry in ctx.fs().join(addon.folder()).unwrap().walk_dir().unwrap() {
                let entry = entry.unwrap();
                if entry.metadata().unwrap().file_type == VfsFileType::File
                    && can_preprocess(entry.as_str())
                {
                    println!("preprocessing {}", entry.as_str());
                    preprocess(entry, ctx).unwrap();
                }
            }
            // }
        });
        Ok(())
    }
}

pub fn preprocess(path: VfsPath, ctx: &Context) -> Result<(), Error> {
    // TODO fix error in vfs
    let mut resolver = VfsResolver::new(ctx)?;
    let tokens = preprocess_file(path.as_str(), &mut resolver)?;
    let rapified = Config::parse(
        ctx.config().hemtt().config(),
        &mut tokens.into_iter().peekable(),
    )?;
    let out = if path.filename() == "config.cpp" {
        path.parent().unwrap().join("config.bin").unwrap()
    } else {
        path
    };
    let mut output = out.create_file().unwrap();
    rapified.rapify(&mut output, 0)?;
    Ok(())
}

pub fn can_preprocess(path: &str) -> bool {
    let path = PathBuf::from(path);
    // if path.display().to_string().contains(".ht.") {
    //     return false;
    // }
    let name = path
        .extension()
        .unwrap_or_else(|| std::ffi::OsStr::new(""))
        .to_str()
        .unwrap();
    ["cpp", "rvmat", "ext"].contains(&name)
}

struct VfsResolver<'a> {
    vfs: &'a VfsPath,
    prefixes: HashMap<String, VfsPath>,
}

impl<'a> VfsResolver<'a> {
    pub fn new(ctx: &'a Context) -> Result<Self, Error> {
        let mut prefixes = HashMap::new();
        for addon in ctx.addons() {
            // TODO fix error in vfs
            for file in FILES {
                let root = ctx.fs().join(addon.folder()).unwrap();
                let path = root.join(file).unwrap();
                if path.exists().unwrap() {
                    prefixes.insert(
                        Prefix::new(
                            &path.read_to_string().unwrap(),
                            ctx.config().hemtt().pbo_prefix_allow_leading_slash(),
                        )?
                        .into_inner(),
                        root,
                    );
                }
            }
        }
        Ok(Self {
            vfs: ctx.fs(),
            prefixes,
        })
    }
}

impl<'a> Resolver for VfsResolver<'a> {
    fn find_include(
        &self,
        _root: &str,
        from: &str,
        to: &str,
        source: Vec<Token>,
    ) -> Result<(PathBuf, String), hemtt_preprocessor::Error> {
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
                    path.push(to.strip_prefix(prefix).unwrap().trim_start_matches('\\'));
                    path
                })
            {
                path
            } else {
                let include = PathBuf::from("include").join(to.trim_start_matches('\\'));
                if include.exists() {
                    include
                } else {
                    return Err(hemtt_preprocessor::Error::IncludeNotFound { target: source });
                }
            }
        } else {
            let mut path = PathBuf::from(from).parent().unwrap().to_path_buf();
            path.push(to);
            path
        };
        if let Ok(mut file) = self
            .vfs
            .join(path.display().to_string().trim_start_matches('/'))
            .unwrap()
            .open_file()
        {
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            Ok((path, content))
        } else {
            Err(hemtt_preprocessor::Error::IncludeNotFound { target: source })
        }
    }
}
