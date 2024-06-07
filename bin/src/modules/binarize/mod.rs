use std::{
    ffi::OsStr,
    fs::create_dir_all,
    path::PathBuf,
    process::Command,
    sync::{
        atomic::{AtomicU16, Ordering},
        RwLock,
    },
};

use hemtt_common::project::hemtt::PDriveOption;
use hemtt_p3d::SearchCache;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use vfs::VfsFileType;

#[allow(unused_imports)] // used in windows only
use self::error::{
    bbe3_binarize_failed::BinarizeFailed, bbw1_tools_not_found::ToolsNotFound,
    bbw2_platform_not_supported::PlatformNotSupported,
};
use self::error::{bbe4_missing_textures::MissingTextures, bbe6_missing_pdrive::MissingPDrive};
use super::Module;
use crate::{
    context::Context, error::Error, link::create_link,
    modules::binarize::error::bbe5_missing_material::MissingMaterials, report::Report,
};

mod error;

#[derive(Default)]
pub struct Binarize {
    check_only: bool,
    command: Option<String>,
    prechecked: RwLock<Vec<BinarizeTarget>>,
}

impl Binarize {
    #[must_use]
    pub fn new(check_only: bool) -> Self {
        Self {
            check_only,
            command: None,
            prechecked: RwLock::new(Vec::new()),
        }
    }
}

impl Module for Binarize {
    fn name(&self) -> &'static str {
        "Binarize"
    }

    #[cfg(windows)]
    fn init(&mut self, _ctx: &Context) -> Result<Report, Error> {
        let mut report = Report::new();
        let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
        let Ok(key) = hkcu.open_subkey("Software\\Bohemia Interactive\\binarize") else {
            report.warn(ToolsNotFound::code());
            return Ok(report);
        };
        let Ok(path) = key.get_value::<String, _>("path") else {
            report.warn(ToolsNotFound::code());
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
        report.warn(PlatformNotSupported::code());
        Ok(report)
    }

    #[allow(clippy::too_many_lines)]
    fn check(&self, ctx: &Context) -> Result<Report, Error> {
        let pdrive_option = if ctx.folder() == Some(&"check".to_string()) {
            ctx.config().hemtt().check().pdrive()
        } else {
            ctx.config().hemtt().build().pdrive()
        };

        let mut report = Report::new();
        let tmp_source = ctx.tmp().join("source");
        let tmp_out = ctx.tmp().join("output");
        let search_cache = SearchCache::new();
        if let Some(pdrive) = ctx.workspace().pdrive() {
            info!("P Drive at {}", pdrive.link().display());
        } else if pdrive_option == &PDriveOption::Require {
            report.error(MissingPDrive::code());
        }
        for addon in ctx.addons() {
            if let Some(config) = addon.config() {
                if !config.binarize().enabled() {
                    debug!("binarization disabled for {}", addon.name());
                    continue;
                }
            }
            for entry in ctx
                .workspace_path()
                .join(addon.folder())
                .expect("workspace should be able to join the addon folder")
                .walk_dir()
                .expect("should be able to walk the vfs addon directory")
            {
                if entry
                    .metadata()
                    .expect("should be able to get metadata for the vfs entry")
                    .file_type
                    == VfsFileType::File
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
                    entry
                        .open_file()
                        .expect("file should exist if it came from walk_dir")
                        .read_exact(&mut buf)
                        .expect("p3ds should be at least 4 bytes");
                    if check_signature(buf) {
                        debug!(
                            "skipping binarization of already binarized {}",
                            entry.as_str()
                        );
                        continue;
                    }

                    // check mlod for textures
                    if buf == [0x4D, 0x4C, 0x4F, 0x44] {
                        trace!("checking textures & materials for {}", entry.as_str());
                        let p3d = hemtt_p3d::P3D::read(
                            &mut entry.open_file().expect("file should exist from walk_dir"),
                        )
                        .expect("p3d should be able to be read if it is a valid p3d file");
                        let (missing_textures, missing_materials) =
                            p3d.missing(ctx.workspace_path(), &search_cache)?;
                        if !missing_textures.is_empty() {
                            let warn = *pdrive_option == PDriveOption::Ignore;
                            let diag = MissingTextures::code(
                                entry.as_str().to_string(),
                                missing_textures,
                                warn,
                            );
                            if warn {
                                report.warn(diag);
                            } else {
                                report.error(diag);
                            }
                        }
                        if !missing_materials.is_empty() {
                            let warn = *pdrive_option == PDriveOption::Ignore;
                            let diag = MissingMaterials::code(
                                entry.as_str().to_string(),
                                missing_materials,
                                warn,
                            );
                            if warn {
                                report.warn(diag);
                            } else {
                                report.error(diag);
                            }
                        }
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

                    self.prechecked
                        .write()
                        .expect("can write in check")
                        .push(BinarizeTarget {
                            source: tmp_sourced
                                .to_str()
                                .expect("tmp source path should be valid utf-8")
                                .trim_start_matches('/')
                                .trim_start_matches(&addon.folder())
                                .to_owned(),
                            output: tmp_outed
                                .to_str()
                                .expect("tmp output path should be valid utf-8")
                                .to_owned(),
                            entry: entry.filename().trim_start_matches('/').to_owned(),
                        });
                }
            }
        }
        info!(
            "Validated {} files for binarization",
            self.prechecked
                .read()
                .expect("prechecked should not be poisoned")
                .len()
        );
        Ok(report)
    }

    #[allow(clippy::too_many_lines)]
    fn pre_build(&self, ctx: &Context) -> Result<Report, Error> {
        if self.command.is_none() || self.check_only {
            return Ok(Report::new());
        }
        setup_tmp(ctx)?;
        let mut report = Report::new();
        let counter = AtomicU16::new(0);
        let tmp_source = ctx.tmp().join("source");
        self.prechecked
            .read()
            .expect("can read in pre_build")
            .par_iter()
            .map(|target| {
                debug!("binarizing {}", target.entry);
                create_dir_all(&target.output)
                    .expect("should be able to create output dir for target");
                let exe = self
                    .command
                    .as_ref()
                    .expect("command should be set if we attempted to binarize");
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
                let output = cmd.output().expect("should be able to run binarize");
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
    create_dir_all(ctx.tmp().join("output"))?;
    let tmp = ctx.tmp().join("source");
    create_dir_all(&tmp)?;
    for addon in ctx.all_addons() {
        let tmp_addon = tmp.join(addon.prefix().as_pathbuf());
        create_dir_all(tmp_addon.parent().expect("tmp addon should have a parent"))?;
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
        let tmp_file = tmp.join(file.file_name().expect("file should have a name"));
        if file.metadata()?.len() > 1024 * 1024 * 10 {
            warn!(
                "File `{}` is larger than 10MB, this will slow builds.",
                file.display()
            );
        }
        trace!("copying `{}` to tmp for binarization", file.display());
        std::fs::copy(&file, &tmp_file)?;
    }

    // link include folders
    let include = ctx.project_folder().join("include");
    if !include.exists() {
        return Ok(());
    }
    let has_pdrive = ctx.workspace().pdrive().is_some();
    let mut warned_a3_include = false;
    for outer_prefix in std::fs::read_dir(include)? {
        let outer_prefix = outer_prefix?.path();
        if has_pdrive && outer_prefix.file_name() == Some(OsStr::new("a3")) {
            if !warned_a3_include {
                info!("binarize ignores include/a3 when a P Drive is used");
                warned_a3_include = true;
            }
            continue;
        }
        if outer_prefix.is_dir() {
            let tmp_outer_prefix = tmp.join(
                outer_prefix
                    .file_name()
                    .expect("outer prefix should have a name"),
            );
            for prefix in std::fs::read_dir(outer_prefix)? {
                let prefix = prefix?.path();
                if prefix.is_dir() {
                    let tmp_mod = tmp_outer_prefix
                        .join(prefix.file_name().expect("prefix should have a name"));
                    create_dir_all(tmp_mod.parent().expect("tmp mod should have a parent"))?;
                    create_link(&tmp_mod, &prefix)?;
                }
            }
        }
    }

    // link the pdrive, if it is required
    if ctx.config().hemtt().build().pdrive() != &PDriveOption::Require {
        return Ok(());
    }
    let Some(pdrive) = ctx.workspace().pdrive() else {
        return Ok(());
    };
    create_link(&tmp.join("a3"), &pdrive.link())?;
    Ok(())
}
