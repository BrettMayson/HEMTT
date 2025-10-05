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
                        {
                            let mut mission = ctx
                                .profile()
                                .display()
                                .to_string()
                                .replace('/', "\\")
                                .replace('\n', "\r\n");
                            if !cfg!(windows) {
                                mission = format!("Z:{}", mission.replace('/', "\\"));
                            }
                            mission
                        },
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
