use std::fs::create_dir_all;

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use crate::{context::Context, error::Error, link::create_link, report::Report};

use super::Module;

#[derive(Default)]
pub struct FilePatching;

impl Module for FilePatching {
    fn name(&self) -> &'static str {
        "FilePatching"
    }

    fn pre_build(&self, ctx: &Context) -> Result<Report, Error> {
        create_dir_all(ctx.build_folder().join("addons"))?;
        ctx.addons()
            .par_iter()
            .map(|addon| {
                create_link(
                    &ctx.build_folder()
                        .join("addons")
                        .join(addon.name().replace('/', "\\")),
                    &ctx.project_folder().join(if cfg!(windows) {
                        addon.folder().replace('/', "\\")
                    } else {
                        addon.folder()
                    }),
                )
            })
            .collect::<Result<(), Error>>()?;
        Ok(Report::new())
    }

    fn post_build(&self, _ctx: &Context) -> Result<Report, Error> {
        info!("You can use the dev folder at `.hemttout/dev` to test your mod with file-patching.");
        Ok(Report::new())
    }
}
