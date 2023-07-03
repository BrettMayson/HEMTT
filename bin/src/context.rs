use std::{
    collections::HashMap,
    env::temp_dir,
    fs::{create_dir_all, remove_dir_all},
    path::PathBuf,
};

use vfs::{AltrootFS, MemoryFS, OverlayFS, PhysicalFS, VfsPath};

use crate::{addons::Addon, config::project::Configuration, error::Error};

#[derive(Debug, Clone)]
pub struct Context {
    config: Configuration,
    folder: String,
    addons: Vec<Addon>,
    vfs: VfsPath,
    project_folder: PathBuf,
    hemtt_folder: PathBuf,
    out_folder: PathBuf,
    tmp: PathBuf,
}

impl Context {
    pub fn new(root: PathBuf, folder: &str) -> Result<Self, Error> {
        let config = {
            let path = root.join(".hemtt").join("project.toml");
            if !path.exists() {
                return Err(Error::ConfigNotFound);
            }
            let config = Configuration::from_file(&path)?;
            info!(
                "Config loaded for {} {}",
                config.name(),
                config.version().get().expect("Unable to read version")
            );
            config
        };
        let tmp = {
            let mut tmp = temp_dir().join("hemtt");
            // on linux add the user to the path for multiple users
            if !cfg!(target_os = "windows") {
                tmp = tmp.join(whoami::username());
            }
            tmp
        }
        .join(
            root.components()
                .skip(2)
                .collect::<PathBuf>()
                .to_str()
                .unwrap()
                .replace(['\\', '/'], "_"),
        );
        trace!("using temporary folder: {:?}", tmp.display());
        let hemtt_folder = root.join(".hemtt");
        trace!("using project folder: {:?}", root.display());
        let out_folder = root.join(".hemttout");
        trace!("using out folder: {:?}", out_folder.display());
        create_dir_all(&out_folder)?;
        let build_folder = out_folder.join(folder);
        trace!("using build folder: {:?}", build_folder.display());
        if build_folder.exists() {
            remove_dir_all(&build_folder)?;
        }
        create_dir_all(&build_folder)?;
        Ok(Self {
            config,
            folder: folder.to_owned(),
            vfs: OverlayFS::new(&{
                let mut layers = vec![AltrootFS::new(MemoryFS::new().into()).into()];
                if cfg!(target_os = "windows") {
                    trace!("vfs overlay at root: {:?}", tmp.join("output").display());
                    layers.push(AltrootFS::new(PhysicalFS::new(tmp.join("output")).into()).into());
                }
                trace!(
                    "vfs root: {:?}",
                    std::env::current_dir()
                        .expect("Unable to get current dir")
                        .display()
                );
                layers.push(AltrootFS::new(PhysicalFS::new(".").into()).into());
                layers
            })
            .into(),
            project_folder: root,
            hemtt_folder,
            out_folder: build_folder,
            addons: Addon::scan()?,
            tmp,
        })
    }

    #[must_use]
    pub fn filter<F>(self, mut filter: F) -> Self
    where
        F: FnMut(&Addon, &Configuration) -> bool,
    {
        let config = self.config.clone();
        Self {
            addons: self
                .addons
                .into_iter()
                .filter(|a| filter(a, &config))
                .collect(),
            ..self
        }
    }

    #[must_use]
    pub const fn config(&self) -> &Configuration {
        &self.config
    }

    #[must_use]
    pub fn folder(&self) -> &str {
        &self.folder
    }

    #[must_use]
    pub fn addons(&self) -> &[Addon] {
        &self.addons
    }

    #[must_use]
    pub fn prefixes(&self) -> HashMap<String, VfsPath> {
        let mut prefixes = HashMap::new();
        for addon in self.addons() {
            if let Ok(path) = self.vfs().join(addon.folder()) {
                prefixes.insert(format!("\\{}\\", addon.prefix().to_string()), path);
            }
        }
        prefixes
    }

    #[must_use]
    pub const fn vfs(&self) -> &VfsPath {
        &self.vfs
    }

    #[must_use]
    /// The project folder
    pub const fn project_folder(&self) -> &PathBuf {
        &self.project_folder
    }

    #[must_use]
    /// The .hemtt folder
    pub const fn hemtt_folder(&self) -> &PathBuf {
        &self.hemtt_folder
    }

    #[must_use]
    /// The .hemttout folder
    pub const fn out_folder(&self) -> &PathBuf {
        &self.out_folder
    }

    #[must_use]
    /// %temp%/hemtt/project
    pub const fn tmp(&self) -> &PathBuf {
        &self.tmp
    }
}
