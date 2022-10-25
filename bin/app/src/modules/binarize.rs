use std::{path::{Path, PathBuf}, process::Command, env::temp_dir, fs::create_dir_all};

use vfs::VfsFileType;

use crate::error::Error;

use super::Module;

pub struct Binarize {
    command: Option<String>,
}

impl Binarize {
    pub fn new() -> Self {
        Self { command: None }
    }
}

impl Module for Binarize {
    fn name(&self) -> &'static str {
        "binarize"
    }

    #[cfg(windows)]
    fn init(&mut self, ctx: &crate::context::Context) -> Result<(), crate::error::Error> {
        let hkcu = winreg::RegKey::predef(winreg::enums::KEY_CURRENT_USER);
        let path = PathBuf::from(
            hkcu.open_subkey("Software\\Bohemia Interactive\\binarize")?
                .get_value("path")?,
        )
        .join("binarize_x64.exe");
        if path.exists() {
            self.command = path.to_string();
        }
        Ok(())
    }

    #[cfg(not(windows))]
    fn init(&mut self, ctx: &crate::context::Context) -> Result<(), crate::error::Error> {
        Ok(())
    }

    fn pre_build(&self, ctx: &crate::context::Context) -> Result<(), crate::error::Error> {
        if self.command.is_none() {
            return Ok(());
        }
        for addon in ctx.addons() {
            for entry in ctx.fs().join(addon.folder()).unwrap().walk_dir().unwrap() {
                let entry = entry.unwrap();
                if entry.metadata().unwrap().file_type == VfsFileType::File
                    && ["rtm", "p3d"].contains(&entry.extension().unwrap_or_default().as_str())
                {
                    let tmp = create_temp_directory(entry.filename().as_str())?;
                    let output = Command::new(self.command.as_ref().unwrap())
                        .args(&[
                            "-norecurse",
                            "-always",
                            "-silent",
                            "-maxProcessor=0",
                            entry.parent().unwrap().as_str(),
                            tmp.to_str().unwrap(),
                            entry.as_str(),
                        ])
                        .output()
                        .unwrap();
                }
            }
        }
        Ok(())
    }

    fn post_build(&self, ctx: &crate::context::Context) -> Result<(), crate::error::Error> {
        Ok(())
    }
}

fn create_temp_directory(name: &str) -> Result<PathBuf, Error> {
    let dir = temp_dir();
    let mut i = 0;

    let mut path;
    loop {
        path = dir.join(format!("hemtt_{}_{}", name, i));
        if !path.exists() { break; }

        i += 1;
    }

    create_dir_all(&path)?;

    Ok(path)
}
