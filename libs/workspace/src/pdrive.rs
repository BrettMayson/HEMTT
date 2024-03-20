use std::{
    collections::HashMap,
    env::temp_dir,
    fs::File,
    path::{Path, PathBuf},
};

use hemtt_common::steam;
use hemtt_pbo::ReadablePbo;
use tracing::{error, info};
use vfs::{AltrootFS, PhysicalFS, VfsPath};

use super::Error;

const DIRS: [&str; 14] = [
    "Addons",
    "AoW\\Addons",
    "Argo\\Addons",
    "Contact\\Addons",
    "Curator\\Addons",
    "Enoch\\Addons",
    "Expansion\\Addons",
    "Heli\\Addons",
    "Jets\\Addons",
    "Kart\\Addons",
    "Mark\\Addons",
    "Orange\\Addons",
    "Tacops\\Addons",
    "Tank\\Addons",
];

#[derive(Debug, PartialEq, Eq)]
pub enum PDrive {
    /// A P drive exists with the a3 folder
    Tools(VfsPath, PathBuf),
    /// A P drive does not exist, but Arma 3 is installed and will be used on demand
    OnDemand(PDriveOnDemand),
}

impl PDrive {
    #[must_use]
    /// Get the path to the a3 folder
    ///
    /// # Panics
    /// Panics if the vfs fails to manage paths
    pub fn path_to(&self, path: &str) -> Option<VfsPath> {
        match self {
            Self::Tools(vfs, _) => {
                let path = vfs.join(path).ok()?;
                if path
                    .exists()
                    .expect("vfs should be able to check if path exists")
                {
                    Some(path)
                } else {
                    None
                }
            }
            Self::OnDemand(p) => p.path_to(path),
        }
    }

    #[must_use]
    /// Get the path to the a3 folder for linking
    pub fn link(&self) -> PathBuf {
        match self {
            Self::Tools(_, p) => p.clone(),
            Self::OnDemand(p) => p.real_root.clone().join("a3"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct PDriveOnDemand {
    real_root: PathBuf,
    root: VfsPath,
    cache: HashMap<String, PathBuf>,
}

impl PDriveOnDemand {
    fn new(p: &Path) -> Result<Self, Error> {
        let path = temp_dir().join("hemtt_pdrive");
        if !path.exists() {
            std::fs::create_dir_all(&path)?;
        }
        Ok(Self {
            real_root: path.clone(),
            root: AltrootFS::new(PhysicalFS::new(path).into()).into(),
            cache: {
                let mut cache = HashMap::new();
                for dir in &DIRS {
                    for addon in std::fs::read_dir(p.join(dir))? {
                        let addon = addon?;
                        let path = addon.path();
                        if path.is_file() {
                            if path.extension().unwrap_or_default() != "pbo" {
                                continue;
                            }
                            cache.insert(
                                path.file_stem()
                                    .expect("file stem should be able to be extracted from path")
                                    .to_string_lossy()
                                    .to_string(),
                                path,
                            );
                        }
                    }
                }
                cache
            },
        })
    }

    fn path_to(&self, path: &str) -> Option<VfsPath> {
        if self
            .root
            .join(path)
            .expect("path can be joined")
            .exists()
            .expect("path exists")
        {
            return Some(self.root.join(path).expect("path can be joined"));
        }
        let parts = path.split('/').collect::<Vec<_>>();
        let pbo_name = parts[2];
        if let Some(p) = self.cache.get(pbo_name) {
            let source = parts[3..].join("/");
            let dest = parts.join("/");
            let dest_path = self.root.join(dest).expect("path can be joined");
            if dest_path.exists().expect("path exists") {
                return Some(dest_path);
            }
            let mut pbo = ReadablePbo::from(File::open(p).ok()?).ok()?;
            let Some(mut file) = pbo.file(&source).ok()? else {
                return None;
            };
            let _ = dest_path.parent().create_dir_all();
            if let Err(e) = std::io::copy(
                &mut file,
                &mut dest_path.create_file().expect("file can be created"),
            ) {
                error!("Failed to extract file from PBO: {}", e);
                return None;
            }
            return Some(dest_path);
        }
        None
    }
}

/// Search for the P drive, returns a `VfsPath` and the path to the a3 folder if found
///
/// # Panics
/// Panics if the vfs fails to manage paths
pub fn search() -> Option<PDrive> {
    // Check if a P drive exists with the a3 folder
    let path = Path::new("P:\\a3");
    if path.is_dir() {
        return Some(PDrive::Tools(
            AltrootFS::new(PhysicalFS::new("P:/").into()).into(),
            path.to_path_buf(),
        ));
    }

    // Check if the P drive exists, unmapped, at ~/Documents/Arma 3 Projects
    let user_documents = dirs::document_dir()?;
    let path = user_documents.join("Arma 3 Projects");
    let path_a3 = path.join("a3");
    if path_a3.is_dir() {
        return Some(PDrive::Tools(
            AltrootFS::new(PhysicalFS::new(path).into()).into(),
            path_a3,
        ));
    }

    // Loop up the arma 3 tools, check for a custom P drive mapping
    if let Some(tools_path) = steam::find_app(233_800) {
        let settings = tools_path.join("settings.ini");
        let settings = std::fs::read_to_string(settings).ok()?;
        let mut using_user_path = false;
        let mut pdrive_path = { dirs::document_dir()?.join("Arma 3 Projects") };
        for line in settings.lines() {
            if line == "P_DriveUser=1" {
                using_user_path = true;
            } else if line.starts_with("P_DrivePath=") && using_user_path {
                let path = line.split('=').nth(1)?;
                pdrive_path = PathBuf::from(path);
            }
        }
        let path = tools_path.join(pdrive_path);
        let path_a3 = path.join("a3");
        if path_a3.is_dir() {
            return Some(PDrive::Tools(
                AltrootFS::new(PhysicalFS::new(path).into()).into(),
                path_a3,
            ));
        }
    }

    // Check if Arma 3 is installed and use it on demand
    if let Some(arma3_path) = steam::find_app(107_410) {
        info!("Using Arma 3 on demand as P drive");
        return Some(PDrive::OnDemand(
            PDriveOnDemand::new(&arma3_path).expect("on demand pdrive"),
        ));
    }

    None
}
