#![allow(clippy::unwrap_used)] // Experimental feature

use std::{fs::File, io::Write};

use rust_embed::RustEmbed;

use crate::{context::Context, error::Error};

#[derive(RustEmbed)]
#[folder = "dist/profile"]
struct Distributables;

pub fn setup(ctx: &Context) -> Result<(), Error> {
    if ctx.profile().exists() {
        std::fs::remove_dir_all(ctx.profile())?;
    }
    std::fs::create_dir_all(ctx.profile())?;
    for file in Distributables::iter() {
        let file = file.to_string();
        trace!("unpacking {:?}", file);
        let path = ctx.profile().join(&file);
        std::fs::create_dir_all(path.parent().unwrap())?;
        let mut f = File::create(&path)?;
        f.write_all(&Distributables::get(&file).unwrap().data)?;
    }
    Ok(())
}

pub fn autotest(ctx: &Context, missions: &[(String, String)]) -> Result<(), Error> {
    let mut autotest = File::create(ctx.profile().join("Users/hemtt/autotest.cfg"))?;
    autotest.write_all(b"class TestMissions {")?;
    for (name, file) in missions {
        autotest.write_all(
            format!(
                r#"class {} {{campaign = "";mission = "{}\autotest\{}";}};"#,
                name,
                ctx.profile()
                    .display()
                    .to_string()
                    .replace('/', "\\")
                    .replace('\n', "\r\n"),
                file
            )
            .as_bytes(),
        )?;
    }
    autotest.write_all(b"};")?;
    Ok(())
}
