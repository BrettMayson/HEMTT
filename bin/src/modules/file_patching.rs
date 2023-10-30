use std::fs::create_dir_all;

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use crate::{context::Context, error::Error, link::create_link};

use super::Module;

#[derive(Default)]
pub struct FilePatching;

impl Module for FilePatching {
    fn name(&self) -> &'static str {
        "FilePatching"
    }

    fn pre_build(&self, ctx: &Context) -> Result<(), Error> {
        create_dir_all(ctx.build_folder().join("addons"))?;
        ctx.addons()
            .par_iter()
            .map(|addon| {
                create_link(
                    &ctx.build_folder()
                        .join("addons")
                        .join(addon.name().replace('/', "\\")),
                    &ctx.project_folder().join(addon.folder().replace('/', "\\")),
                )
            })
            .collect::<Result<(), Error>>()
    }

    fn post_build(&self, _ctx: &Context) -> Result<(), Error> {
        info!(
            "You can now use the dev folder at `.hemttout/dev` to test your mod with file-patching."
        );
        Ok(())
    }
}
