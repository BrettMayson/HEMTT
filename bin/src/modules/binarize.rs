use std::{
    fs::create_dir_all,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU16, Ordering},
};

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use vfs::VfsFileType;

use super::Module;
use crate::{context::Context, error::Error};

#[derive(Default)]
pub struct Binarize {
    command: Option<String>,
}

impl Module for Binarize {
    fn name(&self) -> &'static str {
        "Binarize"
    }

    #[cfg(windows)]
    fn init(&mut self, _ctx: &Context) -> Result<(), Error> {
        let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
        let Ok(key) = hkcu.open_subkey("Software\\Bohemia Interactive\\binarize") else {
            return Ok(());
        };
        let Ok(path) = key.get_value::<String, _>("path") else {
            return Ok(());
        };
        let path = PathBuf::from(path).join("binarize_x64.exe");
        if path.exists() {
            self.command = Some(path.display().to_string());
        }
        Ok(())
    }

    #[cfg(not(windows))]
    fn init(&mut self, _ctx: &Context) -> Result<(), Error> {
        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    fn pre_build(&self, ctx: &Context) -> Result<(), Error> {
        if self.command.is_none() {
            if cfg!(target_os = "windows") {
                warn!("Binarize was not found in the system registery.");
            } else {
                warn!("Binarize is not available on non-Windows platforms.");
            }
            return Ok(());
        }

        let mut targets = Vec::with_capacity(ctx.addons().len());

        let counter = AtomicU16::new(0);
        let tmp_source = ctx.tmp().join("source");
        let tmp_out = ctx.tmp().join("output");
        for addon in ctx.addons() {
            if let Some(config) = addon.config() {
                if !config.binarize().enabled() {
                    debug!("binarization disabled for {}", addon.name());
                    continue;
                }
            }
            for entry in ctx
                .workspace()
                .join(addon.folder())
                .unwrap()
                .walk_dir()
                .unwrap()
            {
                if entry.metadata().unwrap().file_type == VfsFileType::File
                    && ["rtm", "p3d", "wrp"]
                        .contains(&entry.extension().unwrap_or_default().as_str())
                {
                    if let Some(config) = addon.config() {
                        if config
                            .binarize()
                            .exclude()
                            .iter()
                            .map(|file| glob::Pattern::new(file))
                            .collect::<Result<Vec<_>, glob::PatternError>>()?
                            .iter()
                            .any(|pat| {
                                pat.matches(
                                    entry
                                        .as_str()
                                        .trim_start_matches(&format!("/{}/", addon.folder())),
                                )
                            })
                        {
                            debug!("skipping binarization of {}", entry.as_str());
                            continue;
                        }
                    }

                    // skip OLOD & BMTR files as they are already binarized
                    let mut buf = [0; 4];
                    entry.open_file().unwrap().read_exact(&mut buf).unwrap();
                    if check_signature(buf) {
                        debug!(
                            "skipping binarization of already binarized {}",
                            entry.as_str()
                        );
                        continue;
                    }

                    let tmp_sourced = tmp_source.join(addon.prefix().as_pathbuf()).join(
                        entry
                            .as_str()
                            .trim_start_matches('/')
                            .trim_start_matches(&addon.folder().to_string())
                            .trim_start_matches('/')
                            .trim_end_matches(&entry.filename())
                            .replace('/', "\\"),
                    );
                    let tmp_outed = tmp_out.join(entry.parent().as_str().trim_start_matches('/'));

                    targets.push(BinarizeTarget {
                        source: tmp_sourced
                            .to_str()
                            .unwrap()
                            .trim_start_matches('/')
                            .trim_start_matches(&addon.folder())
                            .to_owned(),
                        output: tmp_outed.to_str().unwrap().to_owned(),
                        entry: entry.filename().trim_start_matches('/').to_owned(),
                    });
                }
            }
        }

        targets.par_iter().for_each(|target| {
            debug!("binarizing {}", target.entry);
            create_dir_all(&target.output).unwrap();
            let exe = self.command.as_ref().unwrap();
            let mut cmd = Command::new(exe);
            cmd.args([
                "-always",
                "-silent",
                "-maxProcesses=0",
                &target.source,
                &target.output,
                &target.entry,
            ])
            .current_dir(&tmp_source);
            trace!("{:?}", cmd);
            let output = cmd.output().unwrap();
            assert!(
                output.status.success(),
                "binarize failed with code {:?}",
                output.status.code().unwrap_or(-1)
            );
            if !PathBuf::from(&target.output).join(&target.entry).exists() {
                error!(
                    "No output file for {}, it likely failed to binarize",
                    target.entry
                );
            }
            counter.fetch_add(1, Ordering::Relaxed);
        });

        info!("Binarized {} files", counter.load(Ordering::Relaxed));
        Ok(())
    }
}

struct BinarizeTarget {
    source: String,
    output: String,
    entry: String,
}

/// Check if the file signature indicates that it is already binarized
fn check_signature(buf: [u8; 4]) -> bool {
    // OLOD
    buf == [0x4F, 0x44, 0x4F, 0x4C] ||
    // BMTR
    buf == [0x42, 0x4D, 0x54, 0x52] ||
    // OPRW
    buf == [0x4F, 0x50, 0x52, 0x57]
}
