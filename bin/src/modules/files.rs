use std::fs::create_dir_all;

use crate::{context::Context, error::Error, progress::progress_bar, report::Report};

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
        let mut to_copy = Vec::new();
        let mut globs = Vec::new();
        for file in ctx.config().files().include() {
            globs.push(glob::Pattern::new(file)?);
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
            to_copy.push((entry, d));
        }

        let mut copied = 0;
        let progress = progress_bar(to_copy.len() as u64).with_message("Copying files");
        for (source, dest) in to_copy {
            debug!("copying {:?} => {:?}", source.as_str(), dest.display());
            progress.set_message(format!("Copying {}", source.as_str()));
            std::io::copy(&mut source.open_file()?, &mut std::fs::File::create(&dest)?)?;
            copied += 1;
            progress.inc(1);
        }
        progress.finish_and_clear();
        info!("Copied {} files", copied);
        Ok(Report::new())
    }
}
