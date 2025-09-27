use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::create_dir_all,
    path::PathBuf,
    process::Command,
    sync::{RwLock, atomic::AtomicUsize},
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use hemtt_common::config::PDriveOption;
use hemtt_p3d::SearchCache;
use hemtt_workspace::reporting::Severity;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use vfs::VfsFileType;

#[allow(unused_imports)] // some are Linux only
use self::error::{
    bbe3_binarize_failed::BinarizeFailed, bbw1_tools_not_found::ToolsNotFound,
    bbw2_platform_not_supported::PlatformNotSupported,
};
use self::error::{bbe4_missing_textures::MissingTextures, bbe6_missing_pdrive::MissingPDrive};
use super::Module;
use crate::{
    context::Context, error::Error, link::create_link,
    modules::binarize::error::bbe5_missing_material::MissingMaterials, progress::progress_bar,
    report::Report,
};

mod error;

#[derive(Default)]
pub struct Binarize {
    check_only: bool,
    command: Option<String>,
    compatibility: CompatibiltyTool,
    prechecked: RwLock<Vec<BinarizeTarget>>,
    search_cache: SearchCache,
}

impl Binarize {
    #[must_use]
    pub fn new(check_only: bool) -> Self {
        Self {
            check_only,
            command: None,
            compatibility: CompatibiltyTool::Wine64,
            prechecked: RwLock::new(Vec::new()),
            search_cache: SearchCache::new(),
        }
    }
}

impl Module for Binarize {
    fn name(&self) -> &'static str {
        "Binarize"
    }

    #[cfg(windows)]
    fn init(&mut self, ctx: &Context) -> Result<Report, Error> {
        let mut report = Report::new();

        if self.check_only {
            return Ok(report);
        }

        let folder = if let Ok(path) = std::env::var("HEMTT_BINARIZE_PATH") {
            trace!("Using Binarize path from HEMTT_BINARIZE_PATH");
            PathBuf::from(path)
        } else {
            trace!("Using Binarize path from registry");
            let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
            let Ok(key) = hkcu.open_subkey("Software\\Bohemia Interactive\\binarize") else {
                report.push(ToolsNotFound::code(Severity::Warning));
                return Ok(report);
            };
            let Ok(path) = key.get_value::<String, _>("path") else {
                report.push(ToolsNotFound::code(Severity::Warning));
                return Ok(report);
            };
            PathBuf::from(path)
        };
        let path = folder.join("binarize_x64.exe");
        if path.exists() {
            self.command = Some(path.display().to_string());
        } else {
            report.push(ToolsNotFound::code(Severity::Warning));
        }
        setup_tmp(ctx)?;
        Ok(report)
    }

    #[cfg(not(windows))]
    fn init(&mut self, ctx: &Context) -> Result<Report, Error> {
        use hemtt_common::steam;

        let mut report = Report::new();

        if self.check_only {
            return Ok(report);
        }

        if cfg!(target_os = "macos") {
            report.push(PlatformNotSupported::code());
            return Ok(report);
        }

        let tools_path = {
            let default = dirs::home_dir()
                .expect("home directory exists")
                .join(".local/share/arma3tools");
            if let Ok(path) = std::env::var("HEMTT_BI_TOOLS") {
                PathBuf::from(path)
            } else if !default.exists() {
                let Some(tools_dir) = steam::find_app(233_800) else {
                    report.push(ToolsNotFound::code(Severity::Warning));
                    return Ok(report);
                };
                tools_dir
            } else {
                default
            }
        };
        let path = tools_path.join("Binarize").join("binarize_x64.exe");
        if path.exists() {
            self.command = Some(path.display().to_string());
            let compatibility = CompatibiltyTool::determine();
            if let Some(tool) = compatibility {
                info!("Using {} for binarization compatibilty", tool.to_string());
                self.compatibility = tool;
            } else {
                debug!("tools found, but not wine64 or proton");
                report.push(ToolsNotFound::code(Severity::Warning));
                self.command = None;
            }
        } else {
            report.push(ToolsNotFound::code(Severity::Warning));
        }
        setup_tmp(ctx)?;
        Ok(report)
    }

    #[allow(clippy::too_many_lines)]
    fn check(&self, ctx: &Context) -> Result<Report, Error> {
        let pdrive_option = if ctx.folder() == Some("check") {
            ctx.config().hemtt().check().pdrive()
        } else {
            ctx.config().hemtt().build().pdrive()
        };

        let mut report = Report::new();
        let tmp_out = ctx.tmp().join("hemtt_binarize_output");
        if let Some(pdrive) = ctx.workspace().pdrive() {
            info!("P Drive at {}", pdrive.link().display());
        } else if pdrive_option == &PDriveOption::Require {
            report.push(MissingPDrive::code());
        }
        for addon in ctx.addons() {
            if let Some(config) = addon.config()
                && !config.binarize().enabled()
            {
                debug!("binarization disabled for {}", addon.name());
                continue;
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
                    if let Some(config) = addon.config()
                        && config
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

                    let mut dependencies = Vec::new();

                    // check mlod for textures
                    if buf == [0x4D, 0x4C, 0x4F, 0x44] {
                        trace!("checking textures & materials for {}", entry.as_str());
                        let p3d = hemtt_p3d::P3D::read(
                            &mut entry.open_file().expect("file should exist from walk_dir"),
                        )
                        .expect("p3d should be able to be read if it is a valid p3d file");
                        dependencies = p3d.dependencies();
                        let (missing_textures, missing_materials) =
                            p3d.missing(ctx.workspace_path(), &self.search_cache)?;
                        if !missing_textures.is_empty() {
                            let diag = MissingTextures::code(
                                entry.as_str().to_string(),
                                missing_textures,
                                *pdrive_option == PDriveOption::Ignore,
                            );
                            report.push(diag);
                        }
                        if !missing_materials.is_empty() {
                            let diag = MissingMaterials::code(
                                entry.as_str().to_string(),
                                missing_materials,
                                *pdrive_option == PDriveOption::Ignore,
                            );
                            report.push(diag);
                        }
                    }

                    let tmp_sourced = ctx.tmp().join(addon.prefix().as_pathbuf()).join(
                        entry
                            .as_str()
                            .trim_start_matches('/')
                            .trim_start_matches(&addon.folder().clone())
                            .trim_start_matches('/')
                            .trim_end_matches(&entry.filename()),
                    );
                    let tmp_outed = tmp_out.join(entry.parent().as_str().trim_start_matches('/'));

                    self.prechecked
                        .write()
                        .expect("can write in check")
                        .push(BinarizeTarget {
                            source: tmp_sourced
                                .to_str()
                                .expect("tmp source path should be valid utf-8")
                                .to_owned(),
                            output: tmp_outed
                                .to_str()
                                .expect("tmp output path should be valid utf-8")
                                .to_owned(),
                            entry: entry.filename().trim_start_matches('/').to_owned(),
                            dependencies,
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
        let cache_path = ctx.out_folder().join("binarize.hcb");
        let cache_dir = ctx.out_folder().join("bincache");
        if !cache_dir.exists() {
            create_dir_all(&cache_dir)?;
        }
        let cache = if !ctx.config().runtime().is_release() && cache_path.exists() {
            let mut cache_file = std::fs::File::open(&cache_path)?;
            debug!("Using existing build cache");
            BuildCache::read(&mut cache_file).expect("should be able to read build cache")
        } else {
            BuildCache::default()
        };
        let mut report = Report::new();
        let file_count = self
            .prechecked
            .read()
            .expect("prechecked should not be poisoned")
            .len();
        let progress = progress_bar(file_count as u64).with_message("Binarizing files");
        let cache_hits = AtomicUsize::new(0);
        self.prechecked
            .read()
            .expect("can read in pre_build")
            .par_iter()
            .map(|target| {
                create_dir_all(&target.output)
                    .expect("should be able to create output dir for target");
                let path = PathBuf::from(&target.output).join(&target.entry);
                let path_hash = hash_filename(path.to_str().expect("path should be valid utf-8"));
                if cache.artifacts.contains_key(&path_hash) {
                    let artifact = cache.artifacts.get(&path_hash).expect("should be able to get artifact from cache");
                    if up_to_date(artifact, &self.search_cache) {
                        debug!("skipping binarization of {} as it is already up-to-date", target.entry);
                        std::fs::copy(cache_dir.join(path_hash).with_extension("hb"), &path).expect("should be able to copy from cache to output");
                        progress.inc(1);
                        cache_hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        return None;
                    }
                }
                debug!("binarizing {}", target.entry);
                let exe = self
                    .command
                    .as_ref()
                    .expect("command should be set if we attempted to binarize");
                let mut cmd = if cfg!(windows) {
                    Command::new(exe)
                } else {
                    match self.compatibility {
                        CompatibiltyTool::Wine64 | CompatibiltyTool::Wine => {
                            let mut cmd = Command::new(self.compatibility.to_string());
                            cmd.arg(exe);
                            cmd.env("WINEPREFIX", "/tmp/hemtt-wine");
                            std::fs::create_dir_all("/tmp/hemtt-wine")
                                .expect("should be able to create wine prefix");
                            cmd
                        }
                        CompatibiltyTool::Proton => {
                            let mut home = dirs::home_dir().expect("home directory exists");
                            if exe.contains("/.var/") {
                                home = home.join(".var/app/com.valvesoftware.Steam");
                            }
                            let mut cmd = Command::new({
                                home.join(".local/share/Steam/steamapps/common/SteamLinuxRuntime_sniper/run")
                            });
                            cmd.env("STEAM_COMPAT_CLIENT_INSTALL_PATH", 
                                home.join(".local/share/Steam")
                            ).env(
                                "STEAM_COMPAT_DATA_PATH",
                                home.join(".local/share/Steam/steamapps/compatdata/233800")
                            ).env("STEAM_COMPAT_INSTALL_PATH", "/tmp/hemtt-scip").arg("--").arg(
                                home.join(".local/share/Steam/steamapps/common/Proton - Experimental/proton")
                            ).arg("run").arg(
                                home.join(".local/share/Steam/steamapps/common/Arma 3 Tools/Binarize/binarize_x64.exe")
                            );
                            cmd
                        }
                    }
                };
                cmd.args([
                    "-norecurse",
                    "-always",
                    "-silent",
                    "-maxProcesses=0",
                    &target
                        .source
                        .trim_start_matches(ctx.tmp().to_str().expect("path is valid utf-8"))
                        .trim_start_matches('/')
                        .trim_start_matches('\\')
                        .replace('/', "\\"),
                    &target
                        .output
                        .trim_start_matches(ctx.tmp().to_str().expect("path is valid utf-8"))
                        .trim_start_matches('/')
                        .trim_start_matches('\\')
                        .replace('/', "\\"),
                    &target.entry.replace('/', "\\"),
                ])
                .current_dir(ctx.tmp());
                trace!("{:?}", cmd);
                let output = cmd.output().expect("should be able to run binarize");
                assert!(
                    output.status.success(),
                    "binarize failed with code {:?}",
                    output.status.code().unwrap_or(-1)
                );
                progress.inc(1);
                if PathBuf::from(&target.output).join(&target.entry).exists() {
                    None
                } else {
                    Some(BinarizeFailed::code(target.entry.clone()))
                }
            })
            .collect::<Vec<_>>()
            .into_iter()
            .flatten()
            .for_each(|error| {
                report.push(error);
            });
        let mut new_cache = HashMap::new();
        for target in self
            .prechecked
            .read()
            .expect("can read in pre_build")
            .iter()
        {
            let path = PathBuf::from(&target.output).join(&target.entry);
            if path.exists() {
                let metadata = path
                    .metadata()
                    .expect("should be able to get metadata for binarized file");
                let modified = metadata
                    .modified()
                    .expect("should be able to get modified time")
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("should be able to get duration since epoch")
                    .as_secs();
                let size = metadata.len();
                let hash = hash_filename(path.to_str().expect("path should be valid utf-8"));
                let cache_path = cache_dir.join(format!("{hash}.hb"));
                std::fs::copy(&path, &cache_path)?;
                new_cache.insert(
                    hash,
                    Artifact {
                        modified,
                        size,
                        dependencies: target
                            .dependencies
                            .iter()
                            .map(|d| {
                                (
                                    d.clone(),
                                    self.search_cache.get_metadata(d).map_or_else(
                                        || Artifact {
                                            modified: 0,
                                            size: 0,
                                            dependencies: HashMap::new(),
                                        },
                                        |(modified, size)| Artifact {
                                            modified,
                                            size,
                                            dependencies: HashMap::new(),
                                        },
                                    ),
                                )
                            })
                            .collect(),
                    },
                );
            }
        }
        if !new_cache.is_empty() {
            let mut cache_file = std::fs::File::create(cache_path)?;
            BuildCache {
                artifacts: new_cache,
            }
            .write(&mut cache_file)?;
        } else if cache_path.exists() {
            std::fs::remove_file(cache_path)?;
        }

        progress.finish_and_clear();
        info!(
            "Binarized {} files{}",
            file_count,
            if cache_hits.load(std::sync::atomic::Ordering::Relaxed) > 0 {
                format!(
                    ", {} from cache",
                    cache_hits.load(std::sync::atomic::Ordering::Relaxed)
                )
            } else {
                String::new()
            }
        );
        Ok(report)
    }
}

fn up_to_date(artifact: &Artifact, search_cache: &SearchCache) -> bool {
    fn check_dependency(name: &str, artifact: &Artifact, search_cache: &SearchCache) -> bool {
        if !artifact.dependencies.iter().all(|(d, children)| {
            if !check_dependency(d, children, search_cache) {
                return false;
            }
            if let Some((modified, size)) = search_cache.get_metadata(d) {
                debug!("checking metadata for {d}: {modified} {size}");
                debug!("against artifact: {} {}", artifact.modified, artifact.size);
                modified == artifact.modified && size == artifact.size
            } else {
                debug!("missing metadata for {d}, assuming out-of-date");
                d.starts_with("a3\\")
            }
        }) {
            return false;
        }
        if let Some((modified, size)) = search_cache.get_metadata(name) {
            debug!("checking metadata for {name}: {modified} {size}");
            debug!("against artifact: {} {}", artifact.modified, artifact.size);
            modified == artifact.modified && size == artifact.size
        } else {
            debug!("missing metadata for {name}, assuming out-of-date");
            name.starts_with("a3\\")
        }
    }
    artifact
        .dependencies
        .iter()
        .all(|(d, children)| check_dependency(d, children, search_cache))
}

struct BinarizeTarget {
    source: String,
    output: String,
    entry: String,
    dependencies: Vec<String>,
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

fn setup_tmp(ctx: &Context) -> Result<(), Error> {
    create_dir_all(ctx.tmp())?;
    create_dir_all(ctx.tmp().join("hemtt_binarize_output"))?;
    for addon in ctx.all_addons() {
        let tmp_addon = ctx.tmp().join(addon.prefix().as_pathbuf());
        create_dir_all(tmp_addon.parent().expect("tmp addon should have a parent"))?;
        let target = ctx.project_folder().join(addon.folder_pathbuf());
        create_link(&tmp_addon, &target)?;
    }
    // maybe replace with config or rhai in the future?
    let addons = ctx.project_folder().join("addons");
    for file in std::fs::read_dir(addons)? {
        let file = file?.path();
        if file.is_dir() {
            continue;
        }
        let tmp_file = ctx
            .tmp()
            .join(file.file_name().expect("file should have a name"));
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
            let tmp_outer_prefix = ctx.tmp().join(
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
    create_link(&ctx.tmp().join("a3"), &pdrive.link())?;
    Ok(())
}

#[derive(Debug, Clone, Default)]
struct BuildCache {
    artifacts: HashMap<String, Artifact>,
}

impl BuildCache {
    #[allow(clippy::cast_possible_truncation)]
    pub fn write<W: std::io::Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        writer.write_u32::<LittleEndian>(1)?;
        writer.write_u32::<LittleEndian>(self.artifacts.len() as u32)?;
        for (name, artifact) in &self.artifacts {
            writer.write_u32::<LittleEndian>(name.len() as u32)?;
            writer.write_all(name.as_bytes())?;
            artifact.write(writer)?;
        }
        Ok(())
    }

    pub fn read<R: std::io::Read>(reader: &mut R) -> Result<Self, std::io::Error> {
        let version = reader.read_u32::<LittleEndian>()?;
        if version != 1 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("unsupported build cache version {version}"),
            ));
        }
        let count = reader.read_u32::<LittleEndian>()? as usize;
        let mut artifacts = HashMap::with_capacity(count);
        for _ in 0..count {
            let len = reader.read_u32::<LittleEndian>()? as usize;
            let mut buf = vec![0; len];
            reader.read_exact(&mut buf)?;
            let name = String::from_utf8(buf).expect("artifact name should be valid utf-8");
            let artifact = Artifact::read(reader)?;
            artifacts.insert(name, artifact);
        }
        Ok(Self { artifacts })
    }
}

#[derive(Debug, Clone, Default)]
struct Artifact {
    modified: u64,
    size: u64,
    dependencies: HashMap<String, Artifact>,
}

impl Artifact {
    #[allow(clippy::cast_possible_truncation)]
    pub fn write<W: std::io::Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        writer.write_u64::<LittleEndian>(self.modified)?;
        writer.write_u64::<LittleEndian>(self.size)?;
        writer.write_u32::<LittleEndian>(self.dependencies.len() as u32)?;
        for dep in &self.dependencies {
            writer.write_u32::<LittleEndian>(dep.0.len() as u32)?;
            writer.write_all(dep.0.as_bytes())?;
            dep.1.write(writer)?;
        }
        Ok(())
    }

    pub fn read<R: std::io::Read>(reader: &mut R) -> Result<Self, std::io::Error> {
        let modified = reader.read_u64::<LittleEndian>()?;
        let size = reader.read_u64::<LittleEndian>()?;
        let dep_count = reader.read_u32::<LittleEndian>()? as usize;
        let mut dependencies = HashMap::with_capacity(dep_count);
        for _ in 0..dep_count {
            let len = reader.read_u32::<LittleEndian>()? as usize;
            let mut buf = vec![0; len];
            reader.read_exact(&mut buf)?;
            let name = String::from_utf8(buf).expect("dependency name should be valid utf-8");
            let dep = Self::read(reader)?;
            dependencies.insert(name, dep);
        }
        Ok(Self {
            modified,
            size,
            dependencies,
        })
    }
}

#[derive(Default)]
pub enum CompatibiltyTool {
    #[default]
    Wine64,
    Wine,
    Proton,
}

impl CompatibiltyTool {
    pub fn determine() -> Option<Self> {
        use dirs::home_dir;
        if cfg!(windows) {
            return None;
        }
        let mut cmd = Command::new("wine64");
        cmd.arg("--version");
        if cmd.output().is_ok() {
            return Some(Self::Wine64);
        }
        let mut cmd = Command::new("wine");
        cmd.arg("--version");
        if cmd.output().is_ok() {
            return Some(Self::Wine);
        }
        if home_dir()
            .expect("home directory exists")
            .join(".local/share/Steam/steamapps/common/SteamLinuxRuntime_sniper/run")
            .exists()
        {
            return Some(Self::Proton);
        }
        None
    }
}

impl std::fmt::Display for CompatibiltyTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Wine64 => write!(f, "wine64"),
            Self::Wine => write!(f, "wine"),
            Self::Proton => write!(f, "proton"),
        }
    }
}

fn hash_filename(filename: &str) -> String {
    use sha1::{Digest, Sha1};
    let mut hasher = Sha1::new();
    hasher.update(filename.as_bytes());
    format!("{:x}", hasher.finalize())
}
