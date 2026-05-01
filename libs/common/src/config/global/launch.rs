use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{arma::dlc::DLC, config::LaunchOptions};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq, Eq)]
/// Global launch configuration
pub struct LaunchConfig {
    profiles: HashMap<String, LaunchOptions>,

    pointers: HashMap<String, PathBuf>,

    remove_links: bool,
}

impl LaunchConfig {
    #[must_use]
    /// Launch profiles
    pub const fn profiles(&self) -> &HashMap<String, LaunchOptions> {
        &self.profiles
    }

    #[must_use]
    /// Pointers to mod locations
    pub const fn pointers(&self) -> &HashMap<String, PathBuf> {
        &self.pointers
    }

    #[must_use]
    /// Whether to remove symbolic links
    pub const fn remove_links(&self) -> bool {
        self.remove_links
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
/// Global launch configuration file
pub struct LaunchConfigFile {
    #[serde(default)]
    profiles: HashMap<String, LaunchOptionsFile>,

    #[serde(default)]
    pointers: HashMap<String, PathBuf>,

    #[serde(default)]
    remove_links: bool,
}

impl From<LaunchConfigFile> for LaunchConfig {
    fn from(file: LaunchConfigFile) -> Self {
        Self {
            profiles: file
                .profiles
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
            pointers: file.pointers,
            remove_links: file.remove_links,
        }
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
/// Global launch configuration file
pub struct LaunchOptionsFile {
    #[serde(default)]
    workshop: Vec<String>,

    #[serde(default)]
    dlc: Vec<DLC>,

    #[serde(default)]
    parameters: Vec<String>,

    #[serde(default)]
    executable: Option<String>,
}

impl From<LaunchOptionsFile> for LaunchOptions {
    fn from(file: LaunchOptionsFile) -> Self {
        Self {
            workshop: file.workshop,
            dlc: file.dlc,
            parameters: file.parameters,
            executable: file.executable,
            ..Default::default()
        }
    }
}
