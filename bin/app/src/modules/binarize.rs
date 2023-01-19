use std::{fs::create_dir_all, path::PathBuf, process::Command};

use hemtt_bin_error::Error;
use hemtt_pbo::{prefix::FILES, Prefix};
use vfs::VfsFileType;

use super::Module;

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
    fn init(&mut self, _ctx: &crate::context::Context) -> Result<(), Error> {
        let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
        let path: String = hkcu
            .open_subkey("Software\\Bohemia Interactive\\binarize")?
            .get_value("path")?;
        let path = PathBuf::from(path).join("binarize_x64.exe");
        if path.exists() {
            self.command = Some(path.display().to_string());
        }
        Ok(())
    }

    #[cfg(not(windows))]
    fn init(&mut self, _ctx: &crate::context::Context) -> Result<(), Error> {
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
                    if let Some(config) = addon.config() {
                        if config
                            .no_bin(&addon.folder())?
                            .contains(&PathBuf::from(entry.as_str().trim_start_matches('/')))
                        {
                            println!("skipping binarization of {}", entry.as_str());
                            continue;
                        }
                    }

                    // skip OLOD & BMTR files as they are already binarized
                    let mut buf = [0; 4];
                    entry.open_file().unwrap().read_exact(&mut buf).unwrap();
                    if buf == [0x4F, 0x4C, 0x4F, 0x44] || buf == [0x42, 0x4D, 0x54, 0x52] {
                        continue;
                    }

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
