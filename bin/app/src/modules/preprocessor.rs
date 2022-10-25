use std::path::PathBuf;

use hemtt_config::{Config, Parse, Rapify};
use hemtt_preprocessor::{preprocess_file, Resolver};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use vfs::{VfsFileType, VfsPath};

use crate::error::Error;

use super::Module;

pub struct Preprocessor;

impl Preprocessor {
    pub fn new() -> Self {
        Self
    }
}

impl Module for Preprocessor {
    fn name(&self) -> &'static str {
        "Preprocessor"
    }

    fn pre_build(&self, ctx: &crate::context::Context) -> Result<(), Error> {
        // TODO map to extra error
        ctx.addons().par_iter().for_each(|addon| {
            println!("{}", addon.name());
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
        });
        Ok(())
    }
}

pub fn preprocess(path: VfsPath, ctx: &crate::context::Context) -> Result<(), Error> {
    // TODO fix error in vfs
    let mut resolver = VfsResolver::new(ctx.fs());
    let tokens = preprocess_file(path.as_str(), &mut resolver)?;
    let rapified = Config::parse(&mut tokens.into_iter().peekable())?;
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
}

impl<'a> VfsResolver<'a> {
    pub fn new(vfs: &'a VfsPath) -> Self {
        Self { vfs }
    }
}

impl<'a> Resolver for VfsResolver<'a> {
    fn find_include(
        &self,
        _root: &str,
        from: &str,
        to: &str,
    ) -> Result<(PathBuf, String), hemtt_preprocessor::Error> {
        let mut path = PathBuf::from(from).parent().unwrap().to_path_buf();
        path.push(to);
        let mut file = self
            .vfs
            .join(path.display().to_string().trim_start_matches('/'))
            .unwrap()
            .open_file()
            .unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Ok((path, content))
    }
}
