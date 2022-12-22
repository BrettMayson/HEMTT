use std::{fs::create_dir_all, path::PathBuf, process::Command};

use hemtt_bin_error::Error;
use hemtt_pbo::{prefix::FILES, Prefix};
use vfs::VfsFileType;

use super::Module;
use crate::utils::create_link;

pub struct Binarize {
    command: Option<String>,
}

impl Binarize {
    pub const fn new() -> Self {
        Self { command: None }
    }
}

impl Module for Binarize {
    fn name(&self) -> &'static str {
        "Binarize"
    }

    #[cfg(windows)]
    fn init(&mut self, ctx: &crate::context::Context) -> Result<(), Error> {
        use std::fs::remove_dir_all;

        let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
        let path: String = hkcu
            .open_subkey("Software\\Bohemia Interactive\\binarize")?
            .get_value("path")?;
        let path = PathBuf::from(path).join("binarize_x64.exe");
        if path.exists() {
            self.command = Some(path.display().to_string());
        }
        // Perhaps in the future we can reuse old binarized output if the source hasn't changed.
        if ctx.tmp().exists() {
            remove_dir_all(ctx.tmp())?;
        }
        create_dir_all(ctx.tmp().join("output"))?;
        let tmp = ctx.tmp().join("source");
        create_dir_all(&tmp)?;
        for addon in ctx.addons() {
            for file in FILES {
                let root = ctx.vfs().join(addon.folder()).unwrap();
                let path = root.join(file).unwrap();
                if path.exists().unwrap() {
                    let prefix = Prefix::new(
                        &path.read_to_string().unwrap(),
                        ctx.config().hemtt().pbo_prefix_allow_leading_slash(),
                    )?
                    .into_inner();
                    let tmp_addon = tmp.join(prefix);
                    create_dir_all(tmp_addon.parent().unwrap())?;
                    let target = std::env::current_dir()?
                        .join(root.as_str().trim_start_matches('/').replace('/', "\\"));
                    create_link(tmp_addon.to_str().unwrap(), target.to_str().unwrap())?;
                }
            }
        }
        let include = std::env::current_dir().unwrap().join("include");
        if !include.exists() {
            return Ok(());
        }
        for outer_prefix in std::fs::read_dir(include)? {
            let outer_prefix = outer_prefix?.path();
            if outer_prefix.is_dir() {
                let tmp_outer_prefix = tmp.join(outer_prefix.file_name().unwrap());
                for prefix in std::fs::read_dir(outer_prefix)? {
                    let prefix = prefix?.path();
                    if prefix.is_dir() {
                        let tmp_mod = tmp_outer_prefix.join(prefix.file_name().unwrap());
                        create_dir_all(tmp_mod.parent().unwrap())?;
                        create_link(tmp_mod.to_str().unwrap(), prefix.to_str().unwrap())?;
                    }
                }
            }
        }
        Ok(())
    }

    #[cfg(not(windows))]
    fn init(&mut self, ctx: &crate::context::Context) -> Result<(), Error> {
        Ok(())
    }

    fn pre_build(&self, ctx: &crate::context::Context) -> Result<(), Error> {
        if self.command.is_none() {
            return Ok(());
        }
        let tmp_source = ctx.tmp().join("source");
        let tmp_out = ctx.tmp().join("output");
        for addon in ctx.addons() {
            for entry in ctx.vfs().join(addon.folder()).unwrap().walk_dir().unwrap() {
                let entry = entry.unwrap();
                if entry.metadata().unwrap().file_type == VfsFileType::File
                    && ["rtm", "p3d"].contains(&entry.extension().unwrap_or_default().as_str())
                {
                    let addon_root = PathBuf::from(entry.as_str())
                        .components()
                        .take(3)
                        .collect::<PathBuf>();
                    let mut tmp_sourced = None;
                    let mut tmp_outed = None;
                    for file in FILES {
                        let prefix_path = ctx
                            .vfs()
                            .join(
                                addon_root
                                    .join(file)
                                    .to_str()
                                    .unwrap()
                                    .replace('\\', "/")
                                    .trim_start_matches('/'),
                            )
                            .unwrap();
                        if prefix_path.exists().unwrap() {
                            let prefix = Prefix::new(
                                &prefix_path.read_to_string().unwrap(),
                                ctx.config().hemtt().pbo_prefix_allow_leading_slash(),
                            )?
                            .into_inner();
                            tmp_sourced = Some(tmp_source.join(prefix.trim_start_matches('\\')));
                            tmp_outed =
                                Some(tmp_out.join(entry.parent().as_str().trim_start_matches('/')));
                            break;
                        }
                    }
                    let tmp_sourced = tmp_sourced.unwrap();
                    let tmp_outed = tmp_outed.unwrap();
                    println!("binarizing {}", entry.filename().trim_start_matches('/'));
                    create_dir_all(&tmp_outed)?;
                    let output = Command::new(self.command.as_ref().unwrap())
                        .args([
                            "-norecurse",
                            "-always",
                            "-silent",
                            "-maxProcesses=0",
                            tmp_sourced.to_str().unwrap().trim_start_matches('/'),
                            tmp_outed.to_str().unwrap(),
                            entry.filename().trim_start_matches('/'),
                        ])
                        .current_dir(&tmp_source)
                        .output()
                        .unwrap();
                    assert!(
                        output.status.success(),
                        "binarize failed with code {:?}",
                        output.status.code().unwrap_or(-1)
                    );
                }
            }
        }
        Ok(())
    }

    fn post_build(&self, _ctx: &crate::context::Context) -> Result<(), Error> {
        Ok(())
    }
}
