use std::fs::create_dir_all;

use hemtt_bin_error::Error;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use crate::utils::create_link;

use super::Module;

pub struct FilePatching;

impl FilePatching {
    pub const fn new() -> Self {
        Self
    }
}

impl Module for FilePatching {
    fn name(&self) -> &'static str {
        "FilePatching"
    }

    fn pre_build(&self, ctx: &crate::context::Context) -> Result<(), Error> {
        create_dir_all(ctx.out_folder().join("addons"))?;
        ctx.addons()
            .par_iter()
            .map(|addon| {
                create_link(
                    &ctx.out_folder()
                        .join("addons")
                        .join(addon.name().replace('/', "\\"))
                        .display()
                        .to_string(),
                    &ctx.project_folder()
                        .join(addon.folder())
                        .display()
                        .to_string(),
                )
            })
            .collect::<Result<(), Error>>()
    }

    fn post_build(&self, _ctx: &crate::context::Context) -> Result<(), Error> {
        println!(
            "You can now use the dev folder at `hemtt/dev` to test your mod with file-patching."
        );
        Ok(())
    }
}
