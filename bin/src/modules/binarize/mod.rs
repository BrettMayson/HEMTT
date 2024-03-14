use std::{
    collections::{HashMap, HashSet},
    fs::{create_dir_all, remove_dir_all},
    path::PathBuf,
    process::Command,
    sync::{
        atomic::{AtomicU16, Ordering},
        RwLock,
    },
};

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use vfs::VfsFileType;

use self::error::bbe4_missing_textures::MissingTextures;
#[allow(unused_imports)] // used in windows only
use self::error::{
    bbe3_binarize_failed::BinarizeFailed, bbw1_tools_not_found::ToolsNotFound,
    bbw2_platform_not_supported::PlatformNotSupported,
};
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
    fn init(&mut self, ctx: &Context) -> Result<Report, Error> {
        setup_tmp(ctx)?;
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
    #[allow(clippy::cognitive_complexity)]
    fn check(&self, ctx: &Context) -> Result<Report, Error> {
        let mut report = Report::new();
        let tmp_source = ctx.tmp().join("source");
        let tmp_out = ctx.tmp().join("output");
        let mut textures_cache: HashMap<String, bool> = HashMap::new();
        let mut materials_cache: HashMap<String, bool> = HashMap::new();
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

                    // check mlod for textures
                    if buf == [0x4D, 0x4C, 0x4F, 0x44] {
                        trace!("checking textures & materials for {}", entry.as_str());
                        let p3d = hemtt_p3d::P3D::read(&mut entry.open_file().unwrap()).unwrap();
                        let mut textures = HashSet::new();
                        for lod in &p3d.lods {
                            for face in &lod.faces {
                                textures.insert(face.texture.clone());
                            }
                        }
                        let mut missing = Vec::new();
                        for texture in textures {
                            if texture.is_empty() || texture.starts_with('#') {
                                continue;
                            }
                            let texture = if texture.starts_with('\\') {
                                texture
                            } else {
                                format!("\\{texture}")
                            };
                            let texture = texture.to_lowercase();
                            if let Some(exists) = textures_cache.get(&texture) {
                                if !exists {
                                    missing.push(texture);
                                }
                            } else if ctx.workspace().locate(&texture).unwrap().is_none() {
                                #[allow(clippy::case_sensitive_file_extension_comparisons)]
                                // working on lowercase paths
                                let (replaced, ext) = if texture.ends_with(".paa") {
                                    (texture.replace(".paa", ".tga"), "tga")
                                } else if texture.ends_with(".tga") {
                                    (texture.replace(".tga", ".paa"), "paa")
                                } else if texture.ends_with(".png") {
                                    (texture.replace(".png", ".paa"), "paa")
                                } else {
                                    (texture.clone(), "")
                                };
                                if ext.is_empty()
                                    || ctx.workspace().locate(&replaced).unwrap().is_none()
                                {
                                    textures_cache.insert(texture.clone(), false);
                                    missing.push(texture);
                                } else {
                                    textures_cache.insert(texture.clone(), true);
                                }
                            } else {
                                textures_cache.insert(texture.clone(), true);
                            }
                        }
                        if !missing.is_empty() {
                            report
                                .error(MissingTextures::code(entry.as_str().to_string(), missing));
                        }
                        let mut materials = HashSet::new();
                        for lod in &p3d.lods {
                            for face in &lod.faces {
                                materials.insert(face.material.clone());
                            }
                        }
                        let mut missing = Vec::new();
                        for material in materials {
                            if material.is_empty() || material.starts_with('#') {
                                continue;
                            }
                            let material = if material.starts_with('\\') {
                                material
                            } else {
                                format!("\\{material}")
                            };
                            if let Some(exists) = materials_cache.get(&material) {
                                if !exists {
                                    missing.push(material);
                                }
                            } else if ctx.workspace().locate(&material).unwrap().is_none() {
                                materials_cache.insert(material.clone(), false);
                                missing.push(material);
                            } else {
                                materials_cache.insert(material.clone(), true);
                            }
                        }
                        if !missing.is_empty() {
                            report
                                .error(MissingMaterials::code(entry.as_str().to_string(), missing));
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
        Ok(report)
    }

    #[allow(clippy::too_many_lines)]
    fn pre_build(&self, ctx: &Context) -> Result<Report, Error> {
        if self.command.is_none() || self.check_only {
            return Ok(Report::new());
        }
        let mut report = Report::new();
        let counter = AtomicU16::new(0);
        let tmp_source = ctx.tmp().join("source");
        self.prechecked
            .read()
            .expect("can read in pre_build")
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
    for addon in ctx.all_addons() {
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
