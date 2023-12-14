use std::{
    fs::{create_dir_all, remove_dir_all},
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU16, Ordering},
};

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use vfs::VfsFileType;

#[allow(unused_imports)] // used in windows only
use self::error::{
    bbe1_tools_not_found::ToolsNotFound, bbe2_pltaform_not_supported::PlatformNotSupported,
    bbe3_binarize_failed::BinarizeFailed,
};
use super::Module;
use crate::{context::Context, error::Error, link::create_link, report::Report};

mod error;

#[derive(Default)]
pub struct Binarize {
    command: Option<String>,
}

impl Module for Binarize {
    fn name(&self) -> &'static str {
        "Binarize"
    }

    #[cfg(windows)]
    fn init(&mut self, _ctx: &Context) -> Result<Report, Error> {
        setup_tmp()?;
        let mut report = Report::new();
        let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
        let Ok(key) = hkcu.open_subkey("Software\\Bohemia Interactive\\binarize") else {
            report.error(ToolsNotFound::code());
            return Ok(report);
        };
        let Ok(path) = key.get_value::<String, _>("path") else {
            report.error(ToolsNotFound::code());
            return Ok(report);
        };
        let path = PathBuf::from(path).join("binarize_x64.exe");
        if path.exists() {
            self.command = Some(path.display().to_string());
        }
        Ok(report)
    }

    #[cfg(not(windows))]
    fn init(&mut self, _ctx: &Context) -> Result<Report, Error> {
        let mut report = Report::new();
        report.error(PlatformNotSupported::code());
        Ok(report)
    }

    #[allow(clippy::too_many_lines)]
    fn pre_build(&self, ctx: &Context) -> Result<Report, Error> {
        let mut targets = Vec::with_capacity(ctx.addons().len());
        let mut report = Report::new();
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

        targets
            .par_iter()
            .map(|target| {
                debug!("binarizing {}", target.entry);
                create_dir_all(&target.output).unwrap();
                let exe = self.command.as_ref().unwrap();
                let mut cmd = Command::new(exe);
                cmd.args([
                    "-norecurse",
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
                if PathBuf::from(&target.output).join(&target.entry).exists() {
                    counter.fetch_add(1, Ordering::Relaxed);
                    None
                } else {
                    Some(BinarizeFailed::code(target.entry.clone()))
                }
            })
            .collect::<Vec<_>>()
            .into_iter()
            .flatten()
            .for_each(|error| {
                report.error(error);
            });

        info!("Binarized {} files", counter.load(Ordering::Relaxed));
        Ok(report)
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

#[allow(dead_code)] // used in windows only
fn setup_tmp(ctx: &Context) -> Result<(), Error> {
    if ctx.tmp().exists() {
        remove_dir_all(ctx.tmp())?;
    }
    create_dir_all(ctx.tmp().join("output"))?;
    let tmp = ctx.tmp().join("source");
    create_dir_all(&tmp)?;
    for addon in ctx.addons() {
        let tmp_addon = tmp.join(addon.prefix().as_pathbuf());
        create_dir_all(tmp_addon.parent().unwrap())?;
        let target = ctx.project_folder().join(
            addon
                .folder()
                .as_str()
                .trim_start_matches('/')
                .replace('/', "\\"),
        );
        create_link(&tmp_addon, &target)?;
    }
    // maybe replace with config or rhai in the future?
    let addons = ctx.project_folder().join("addons");
    for file in std::fs::read_dir(addons)? {
        let file = file?.path();
        if file.is_dir() {
            continue;
        }
        let tmp_file = tmp.join(file.file_name().unwrap());
        if file.metadata()?.len() > 1024 * 1024 * 10 {
            warn!(
                "File `{}` is larger than 10MB, this will slow builds.",
                file.display()
            );
        }
        trace!("copying `{}` to tmp for binarization", file.display());
        std::fs::copy(&file, &tmp_file)?;
    }
    let include = ctx.project_folder().join("include");
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
                    create_link(&tmp_mod, &prefix)?;
                }
            }
        }
    }
    Ok(())
}
