use std::fs::create_dir_all;

use crate::{context::Context, error::Error, report::Report};

use super::Module;

#[derive(Default)]
pub struct Files;

impl Module for Files {
    fn name(&self) -> &'static str {
        "Files"
    }

    fn post_build(&self, ctx: &Context) -> Result<Report, Error> {
        let glob_options = glob::MatchOptions {
            require_literal_separator: true,
            ..Default::default()
        };
        let mut copied = 0;
        let mut globs = Vec::new();
        for file in ctx.config().files().include() {
            globs.push(glob::Pattern::new(&file)?);
        }
        for entry in ctx.workspace_path().walk_dir()? {
            if entry.as_str().starts_with("/.hemtt") {
                continue;
            }
            if entry.metadata()?.file_type == vfs::VfsFileType::Directory {
                continue;
            }
            if !entry.exists()? {
                continue;
            }

            if !globs
                .iter()
                .any(|pat| pat.matches_with(entry.as_str(), glob_options))
            {
                continue;
            }

            let mut d = ctx.build_folder().expect("build folder exists").clone();

            d.push(entry.as_str().trim_start_matches('/'));
            let folder = d.parent().expect("must have parent, just joined");
            if !folder.exists() {
                std::mem::drop(create_dir_all(folder));
            }
            debug!("copying {:?} => {:?}", entry.as_str(), d.display());
            std::io::copy(&mut entry.open_file()?, &mut std::fs::File::create(&d)?)?;
            copied += 1;
        }
        info!("Copied {} files", copied);
        Ok(Report::new())
    }
}
