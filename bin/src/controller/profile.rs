#![allow(clippy::unwrap_used)] // Experimental feature

use std::{fs::File, io::Write};

use crate::{context::Context, error::Error};

#[allow(clippy::disallowed_methods)]
mod embedded {
    use rust_embed::RustEmbed;

    #[derive(RustEmbed)]
    #[folder = "dist/profile"]
    pub struct Distributables;
}

pub fn setup(ctx: &Context) -> Result<(), Error> {
    if ctx.profile().exists() {
        fs_err::remove_dir_all(ctx.profile())?;
    }
    fs_err::create_dir_all(ctx.profile())?;
    for file in embedded::Distributables::iter() {
        let file = file.to_string();
        trace!("unpacking {:?}", file);
        let path = ctx.profile().join(&file);
        fs_err::create_dir_all(path.parent().unwrap())?;
        let mut f = File::create(&path)?;
        f.write_all(&embedded::Distributables::get(&file).unwrap().data)?;
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub enum AutotestMission {
    Internal(String),
    Custom(String),
}

impl AutotestMission {
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Custom(s) | Self::Internal(s) => s,
        }
    }
}

pub fn autotest(ctx: &Context, missions: &[(String, AutotestMission)]) -> Result<(), Error> {
    let mut autotest = File::create(ctx.profile().join("Users/hemtt/autotest.cfg"))?;
    autotest.write_all(b"class TestMissions {")?;
    for (name, file) in missions {
        autotest.write_all(
            format!(
                r#"class {} {{campaign = "";mission = "{}";}};"#,
                name,
                match file {
                    AutotestMission::Internal(s) => format!(
                        r"{}\autotest\{}",
                        ctx.profile()
                            .display()
                            .to_string()
                            .replace('/', "\\")
                            .replace('\n', "\r\n"),
                        s
                    ),
                    AutotestMission::Custom(s) => s.clone(),
                }
            )
            .as_bytes(),
        )?;
    }
    autotest.write_all(b"};")?;
    Ok(())
}
