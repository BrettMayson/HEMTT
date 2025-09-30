use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{arma::dlc::DLC, config::LaunchOptions};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq, Eq)]
/// Global launch configuration
pub struct LaunchConfig {
    profiles: HashMap<String, LaunchOptions>,

    pointers: HashMap<String, PathBuf>,
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
}

#[allow(clippy::module_name_repetitions)]
#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
/// Global launch configuration file]
pub struct LaunchConfigFile {
    #[serde(default)]
    profiles: HashMap<String, LaunchOptionsFile>,

    #[serde(default)]
    pointers: HashMap<String, PathBuf>,
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
