use std::fs::create_dir_all;

use hemtt_bin_error::Error;

use super::Module;

pub struct Files;
impl Files {
    pub const fn new() -> Self {
        Self
    }
}

impl Module for Files {
    fn name(&self) -> &'static str {
        "Files"
    }

    fn post_build(&self, ctx: &crate::context::Context) -> Result<(), Error> {
        for mut file in ctx.config().files().include() {
            let mirror_structure = file.ends_with('/');
            if mirror_structure {
                file.pop();
            };
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

                if !glob::Pattern::new(&file)?.matches(entry.as_str()) {
                    continue;
                }

                let mut d = ctx.out_folder().clone();

                if mirror_structure {
                    d.push(entry.parent().filename().trim_start_matches('/'));
                    create_dir_all(&d)?;
                }

                d.push(entry.as_str().trim_start_matches('/'));
                println!("Copying `{:#?}` => {:#?}", entry.as_str(), d);
                std::io::copy(&mut entry.open_file()?, &mut std::fs::File::create(&d)?).unwrap();
            }
        }
        Ok(())
    }
}
