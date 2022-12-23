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
        create_dir_all(ctx.hemtt_folder().join("addons"))?;
        ctx.addons()
            .par_iter()
            .map(|addon| {
                create_link(
                    ctx.hemtt_folder()
                        .join("addons")
                        .join(addon.name().replace('/', "\\"))
                        .to_str()
                        .unwrap(),
                    &addon.folder().replace('/', "\\"),
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
