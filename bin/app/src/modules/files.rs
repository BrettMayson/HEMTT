use std::fs::create_dir_all;

use crate::error::Error;

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
        for mut file in ctx.config().project().files() {
            let mirror_structure = file.ends_with('/');
            if mirror_structure {
                file.pop();
            };
            for path in (glob::glob(&file)?).flatten() {
                let mut d = ctx.hemtt_folder().clone();
                if mirror_structure {
                    d.push(path.parent().unwrap());
                    create_dir_all(&d)?;
                }

                if std::fs::metadata(&path).unwrap().is_dir() {
                    println!("Copying dir `{path:#?}` => {d:#?}");
                    fs_extra::dir::copy(&path, &d, &fs_extra::dir::CopyOptions::new()).unwrap();
                } else {
                    d.push(path.file_name().unwrap().to_str().unwrap());
                    println!("Copying file `{path:#?}` => {d:#?}");
                    fs_extra::file::copy(&path, &d, &fs_extra::file::CopyOptions::new()).unwrap();
                }
            }
        }
        Ok(())
    }
}
