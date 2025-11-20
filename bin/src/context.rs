use std::{env::temp_dir, path::PathBuf, sync::Arc};

use hemtt_common::config::{GlobalConfig, ProjectConfig};
use hemtt_rhai::libraries::project::RhaiProject;
use hemtt_workspace::{LayerType, Workspace, WorkspacePath, addons::Addon};

use crate::error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Should the current contents of .hemttout\{} be preserved
pub enum PreservePrevious {
    /// The contents should be removed
    Remove,
    /// The contents should be preserved
    Keep,
}

pub type State = state::TypeMap![Send + Sync];

#[derive(Debug, Clone)]
pub struct Context {
    global: GlobalConfig,
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
    profile: PathBuf,
    state: Arc<State>,
}

impl Context {
    /// Read the global config from the standard location
    ///
    /// # Errors
    /// [`hemtt_common::Error::Io`] if the file cannot be read
    /// [`hemtt_common::Error::Toml`] if the file is not valid tom
    ///
    /// # Panics
    /// If the platform does not support a config directory
    pub fn read_global() -> Result<GlobalConfig, hemtt_common::Error> {
        let config_dir = dirs::config_dir()
            .expect("Running on an unspported platform? No config directory found.")
            .join("hemtt");
        debug!("using global config directory: {:?}", config_dir.display());
        let config_file = config_dir.join("config.toml");
        if config_file.exists() {
            GlobalConfig::from_file(&config_file)
        } else {
            Ok(GlobalConfig::default())
        }
    }

    /// Read the project config from .hemtt/project.toml
    ///
    /// # Errors
    /// [`Error::ConfigNotFound`] if the project.toml is not found
    /// [`Error::Io`] if the file cannot be read
    pub fn read_project() -> Result<ProjectConfig, Error> {
        let root = std::env::current_dir()?;
        let path = root.join(".hemtt").join("project.toml");
        if !path.exists() {
            return Err(Error::ConfigNotFound);
        }
        Ok(ProjectConfig::from_file(&path)?)
    }

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
        let config = Self::read_project()?;
        let tmp = {
            let mut tmp = temp_dir().join("hemtt");
            // on linux add the user to the path for multiple users
            if !cfg!(target_os = "windows") {
                tmp = tmp.join(whoami::username());
            }
            tmp
        };
        let profile = tmp.join("profile");
        let tmp = tmp.join(
            root.components()
                .skip(2)
                .collect::<PathBuf>()
                .to_str()
                .expect("valid utf-8")
                .replace(['\\', '/'], "_"),
        );
        if tmp.exists() {
            fs_err::remove_dir_all(&tmp)?;
        }
        trace!("using temporary folder: {:?}", tmp.display());
        let hemtt_folder = root.join(".hemtt");
        trace!("using project folder: {:?}", root.display());
        if !hemtt_folder.exists() {
            return Err(Error::ConfigNotFound);
        }
        let out_folder = root.join(".hemttout");
        trace!("using out folder: {:?}", out_folder.display());
        fs_err::create_dir_all(&out_folder)?;
        fs_err::File::create(out_folder.join("ci_annotations.txt"))?;
        let mut builder = Workspace::builder().physical(&root, LayerType::Source);
        let maybe_build_folder = if let Some(folder) = folder {
            let build_folder = out_folder.join(folder);
            trace!("using build folder: {:?}", build_folder.display());
            if preserve_previous == PreservePrevious::Remove && build_folder.exists() {
                fs_err::remove_dir_all(&build_folder)?;
            }
            fs_err::create_dir_all(&build_folder)?;
            builder = builder.physical(&tmp.join("hemtt_binarize_output"), LayerType::Build);
            let include = root.join("include");
            if include.is_dir() {
                builder = builder.physical(&include, LayerType::Include);
            }
            Some(build_folder)
        } else {
            None
        };
        let workspace = builder.memory().finish(
            Some(config.clone()),
            folder.is_some(),
            if folder == Some("check") {
                config.hemtt().check().pdrive()
            } else {
                config.hemtt().build().pdrive()
            },
        )?;
        version_check(&config, &workspace, print_info)?;
        let addons = Addon::scan(&root)?;
        Ok(Self {
            global: Self::read_global()?,
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
            profile,
            state: Arc::new(State::default()),
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
    pub const fn global(&self) -> &GlobalConfig {
        &self.global
    }

    #[must_use]
    pub const fn config(&self) -> &ProjectConfig {
        &self.config
    }

    #[must_use]
    pub fn folder(&self) -> Option<&str> {
        self.folder.as_deref()
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
    pub const fn workspace_path(&self) -> &WorkspacePath {
        &self.workspace
    }

    #[must_use]
    pub fn workspace(&self) -> &Workspace {
        self.workspace.workspace()
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

    #[must_use]
    /// %temp%/hemtt/profile
    pub const fn profile(&self) -> &PathBuf {
        &self.profile
    }

    #[must_use]
    /// The state of the context
    pub fn state(&self) -> Arc<State> {
        self.state.clone()
    }

    #[must_use]
    pub fn with_config(self, config: ProjectConfig) -> Self {
        Self { config, ..self }
    }

    /// Convert the context to a [`RhaiProject`]
    ///
    /// # Panics
    /// If the version config is invalid
    pub fn as_rhai_project(&self) -> RhaiProject {
        RhaiProject {
            name: self.config().name().to_string(),
            author: self
                .config()
                .author()
                .map_or_else(String::new, std::string::ToString::to_string),
            prefix: self.config().prefix().to_string(),
            mainprefix: self
                .config()
                .mainprefix()
                .map_or_else(String::new, std::string::ToString::to_string),
            version: self
                .config()
                .version()
                .get(self.workspace_path().vfs())
                .expect("version config is valid to get to rhai module"),
            // addons: self.addons().to_vec(),
        }
    }
}

fn version_check(
    config: &ProjectConfig,
    workspace: &WorkspacePath,
    print_info: bool,
) -> Result<(), Error> {
    let version = config.version().get(workspace.vfs());
    if let Err(hemtt_common::Error::Git(_)) = version {
        error!(
            "Failed to find a git repository with at least one commit, if you are not using git add the following to your project.toml"
        );
        println!("\n[version]\ngit_hash = 0\n");
        std::process::exit(1);
    }
    if let Err(hemtt_common::Error::Version(e)) = version {
        match e {
            hemtt_common::version::Error::UnknownVersion => {
                error!("HEMTT was not able to determine the version of your project.");
                println!("\nThere are two ways to define the version of your project");
                println!(
                    "\n1. Macros inside `addons/main/script_version.hpp`, or a specified file"
                );
                println!("[version]\npath = \"addons/not_main/script_version.hpp\"");
                println!("\n2. A `version` table in your project.toml");
                println!("[version]\nmajor = 1\nminor = 0\npatch = 0\nbuild = 0\n");
                if supports_hyperlinks::on(supports_hyperlinks::Stream::Stdout) {
                    let link = terminal_link::Link::new(
                        "The HEMTT Book",
                        "https://hemtt.dev/configuration/version.html",
                    );
                    println!("\nRead more about Version Configuration in {link}");
                } else {
                    println!(
                        "\nRead more about Version Configuration at https://hemtt.dev/configuration/version.html"
                    );
                }
            }
            hemtt_common::version::Error::ExpectedMajor => {
                error!("HEMTT is not able to determine the MAJOR version of your project.");
            }
            hemtt_common::version::Error::ExpectedMinor => {
                error!("HEMTT is not able to determine the MINOR version of your project.");
            }
            hemtt_common::version::Error::ExpectedPatch => {
                error!("HEMTT is not able to determine the PATCH version of your project.");
            }
            hemtt_common::version::Error::ExpectedBuild => {
                error!("HEMTT is not able to determine the BUILD version of your project.");
            }
            hemtt_common::version::Error::InvalidComponent(s) => {
                error!("HEMTT is not able to determine the version of your project.");
                println!("Encountered an invalid version component: {s}");
            }
            hemtt_common::version::Error::VersionPathConflict => {
                // The version path is defined, and a version table is defined
                error!("HEMTT is not able to determine the source of the version.");
                println!("\nThere are two ways to define the version of your project");
                println!("\n1. The `path` field in the `version` table in your project.toml");
                println!(
                    "\n2. The `major`, `minor`, `patch`, and `build` fields in the `version` table in your project.toml"
                );
                println!("Currently both are defined, only one can be used.");
            }
        }
        std::process::exit(1);
    }
    if print_info {
        info!("Config loaded for {} {}", config.name(), version?);
    }
    Ok(())
}
