use std::{
    env::temp_dir,
    fs::{create_dir_all, remove_dir_all},
    path::PathBuf,
};

use hemtt_common::project::ProjectConfig;
use hemtt_workspace::{addons::Addon, LayerType, Workspace, WorkspacePath};

use crate::error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Should the current contents of .hemttout\{} be preserved
pub enum PreservePrevious {
    /// The contents should be removed
    Remove,
    /// The contents should be preserved
    Keep,
}

#[derive(Debug, Clone)]
pub struct Context {
    config: ProjectConfig,
    folder: Option<String>,
    addons: Vec<Addon>,
    all_addons: Vec<Addon>,
    workspace: WorkspacePath,
    project_folder: PathBuf,
    hemtt_folder: PathBuf,
    out_folder: PathBuf,
    build_folder: Option<PathBuf>,
    tmp: PathBuf,
}

impl Context {
    /// Create a new context
    ///
    /// # Errors
    /// [`Error::ConfigNotFound`] if the project.toml is not found
    /// [`Error::Io`] if the temporary folder fails to be created
    /// [`Error::Git`] if the git hash is invalid
    /// [`Error::Version`] if the version is invalid
    ///
    /// # Panics
    /// If the project folder is not a valid [`OsStr`] (UTF-8)
    pub fn new(
        folder: Option<&str>,
        preserve_previous: PreservePrevious,
        print_info: bool,
    ) -> Result<Self, Error> {
        let root = std::env::current_dir()?;
        let config = {
            let path = root.join(".hemtt").join("project.toml");
            if !path.exists() {
                return Err(Error::ConfigNotFound);
            }
            ProjectConfig::from_file(&path)?
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
                .expect("valid utf-8")
                .replace(['\\', '/'], "_"),
        );
        if tmp.exists() {
            remove_dir_all(&tmp)?;
        }
        trace!("using temporary folder: {:?}", tmp.display());
        let hemtt_folder = root.join(".hemtt");
        trace!("using project folder: {:?}", root.display());
        let out_folder = root.join(".hemttout");
        trace!("using out folder: {:?}", out_folder.display());
        create_dir_all(&out_folder)?;
        std::fs::File::create(out_folder.join("ci_annotations.txt"))?;
        let mut builder = Workspace::builder().physical(&root, LayerType::Source);
        let mut maybe_build_folder = None;
        if let Some(folder) = folder {
            let build_folder = out_folder.join(folder);
            trace!("using build folder: {:?}", build_folder.display());
            if preserve_previous == PreservePrevious::Remove && build_folder.exists() {
                remove_dir_all(&build_folder)?;
            }
            create_dir_all(&build_folder)?;
            if cfg!(target_os = "windows") {
                builder = builder.physical(&tmp.join("output"), LayerType::Build);
            }
            let include = root.join("include");
            if include.is_dir() {
                builder = builder.physical(&include, LayerType::Include);
            }
            maybe_build_folder = Some(build_folder);
        };
        let workspace = builder.memory().finish(
            Some(config.clone()),
            folder.is_some(),
            config.hemtt().build().pdrive(),
        )?;
        {
            let version = config.version().get(workspace.vfs());
            if let Err(hemtt_common::project::Error::Git(_)) = version {
                error!("Failed to find a git repository with at least one commit, if you are not using git add the following to your project.toml");
                println!("\n[version]\ngit_hash = 0\n");
                std::process::exit(1);
            };
            if print_info {
                info!("Config loaded for {} {}", config.name(), version?);
            }
        }
        let addons = Addon::scan(&root)?;
        Ok(Self {
            config,
            folder: folder.map(std::borrow::ToOwned::to_owned),
            workspace,
            all_addons: addons.clone(),
            addons,
            project_folder: root,
            hemtt_folder,
            out_folder,
            build_folder: maybe_build_folder,
            tmp,
        })
    }

    #[must_use]
    pub fn filter<F>(self, mut filter: F) -> Self
    where
        F: FnMut(&Addon, &ProjectConfig) -> bool,
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
    pub const fn config(&self) -> &ProjectConfig {
        &self.config
    }

    #[must_use]
    pub const fn folder(&self) -> Option<&String> {
        self.folder.as_ref()
    }

    #[must_use]
    pub fn addons(&self) -> &[Addon] {
        &self.addons
    }

    #[must_use]
    pub fn all_addons(&self) -> &[Addon] {
        &self.all_addons
    }

    #[must_use]
    pub fn addon(&self, name: &str) -> Option<&Addon> {
        self.addons.iter().find(|a| a.name() == name)
    }

    #[must_use]
    pub const fn workspace(&self) -> &WorkspacePath {
        &self.workspace
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
    /// The .hemttout/{command} folder
    pub const fn build_folder(&self) -> Option<&PathBuf> {
        self.build_folder.as_ref()
    }

    #[must_use]
    /// %temp%/hemtt/project
    pub const fn tmp(&self) -> &PathBuf {
        &self.tmp
    }
}
