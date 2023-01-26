use std::fs::create_dir_all;

use hemtt_bin_error::Error;

use super::Module;

#[derive(Default)]
pub struct Files;

impl Module for Files {
    fn name(&self) -> &'static str {
        "Files"
    }

    fn post_build(&self, ctx: &crate::context::Context) -> Result<(), Error> {
        let glob_options = glob::MatchOptions {
            require_literal_separator: true,
            ..Default::default()
        };
        for file in ctx.config().files().include() {
            for entry in ctx.vfs().walk_dir()? {
                let entry = entry?;
                if entry.is_dir()? {
                    continue;
                }
                if entry.as_str().starts_with("/.hemtt") {
                    continue;
                }
                if !entry.exists()? {
                    continue;
                }

                if !glob::Pattern::new(&file)?.matches_with(entry.as_str(), glob_options) {
                    continue;
                }

                let mut d = ctx.out_folder().clone();

                d.push(entry.as_str().trim_start_matches('/'));
                let folder = d.parent().unwrap();
                if !folder.exists() {
                    std::mem::drop(create_dir_all(folder));
                }
                println!("Copying `{:#?}` => {d:#?}", entry.as_str());
                std::io::copy(&mut entry.open_file()?, &mut std::fs::File::create(&d)?).unwrap();
            }
        }
        Ok(())
    }
}
