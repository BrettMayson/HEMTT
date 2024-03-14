//! HEMTT Configuration

use std::{borrow::Cow, collections::HashMap};

use crate::arma::dlc::DLC;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
/// Feature specific configuration
pub struct Features {
    #[serde(default)]
    dev: DevOptions,

    #[serde(default)]
    launch: HashMap<String, LaunchOptions>,

    #[serde(default)]
    build: BuildOptions,

    #[serde(default)]
    release: ReleaseOptions,
}

impl Features {
    #[must_use]
    /// Dev options
    pub const fn dev(&self) -> &DevOptions {
        &self.dev
    }

    #[must_use]
    /// Get launch options by key
    pub fn launch(&self, key: &str) -> Option<Cow<LaunchOptions>> {
        self.launch.get(key).map(Cow::Borrowed)
    }

    #[must_use]
    /// Get all launch keys
    pub fn launch_keys(&self) -> Vec<String> {
        self.launch.keys().cloned().collect()
    }

    #[must_use]
    /// Build options
    pub const fn build(&self) -> &BuildOptions {
        &self.build
    }

    #[must_use]
    /// Release options
    pub const fn release(&self) -> &ReleaseOptions {
        &self.release
    }
}

#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
/// Dev specific configuration
pub struct DevOptions {
    #[serde(default)]
    exclude: Vec<String>,
}

impl DevOptions {
    #[must_use]
    /// Addons to exclude from dev
    pub fn exclude(&self) -> &[String] {
        &self.exclude
    }
}

#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
/// Launch specific configuration
pub struct LaunchOptions {
    #[serde(default)]
    /// Workshop mods that should be launched with the mod
    workshop: Vec<String>,

    #[serde(default)]
    /// DLCs that should be launched with the mod
    dlc: Vec<DLC>,

    #[serde(default)]
    /// HTML presets that should be launched with the mod
    presets: Vec<String>,

    #[serde(default)]
    /// Optional addons that should be built into the mod
    optionals: Vec<String>,

    #[serde(default)]
    /// Mission to launch directly into the editor with
    mission: Option<String>,

    #[serde(default)]
    /// Extra launch parameters
    parameters: Vec<String>,

    #[serde(default)]
    /// Binary to launch, defaults to `arma3_x64.exe`
    executable: Option<String>,
}

impl LaunchOptions {
    #[must_use]
    /// Workshop mods that should be launched with the mod
    pub fn workshop(&self) -> &[String] {
        &self.workshop
    }

    #[must_use]
    /// DLCs that should be launched with the mod
    pub fn dlc(&self) -> &[DLC] {
        &self.dlc
    }

    #[must_use]
    /// HTML presets that should be launched with the mod
    pub fn presets(&self) -> &[String] {
        &self.presets
    }

    #[must_use]
    /// Optional addons that should be built into the mod
    pub fn optionals(&self) -> &[String] {
        &self.optionals
    }

    #[must_use]
    /// Mission to launch directly into the editor with
    pub const fn mission(&self) -> Option<&String> {
        self.mission.as_ref()
    }

    #[must_use]
    /// Extra launch parameters
    pub fn parameters(&self) -> &[String] {
        &self.parameters
    }

    #[must_use]
    /// Binary to launch, defaults to `arma3_x64.exe`
    pub fn executable(&self) -> String {
        let executable = self
            .executable
            .clone()
            .unwrap_or_else(|| "arma3_x64".to_owned());
        if cfg!(target_os = "windows") {
            format!("{executable}.exe")
        } else {
            executable
        }
    }
}

#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
/// Build specific configuration
pub struct BuildOptions {
    #[serde(default)]
    /// Should optionals be built into their own mod?
    /// Default: true
    optional_mod_folders: Option<bool>,
    #[serde(default)]
    /// Can includes come from the P drive?
    /// Default: false
    allow_pdrive: Option<bool>,
}

impl BuildOptions {
    #[must_use]
    /// Should optionals be built into their own mod?
    pub const fn optional_mod_folders(&self) -> bool {
        if let Some(optional) = self.optional_mod_folders {
            optional
        } else {
            true
        }
    }

    #[must_use]
    /// Can includes come from the P drive?
    pub const fn allow_pdrive(&self) -> bool {
        if let Some(allow_pdirve) = self.allow_pdrive {
            allow_pdirve
        } else {
            false
        }
    }
}

#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
/// Release specific configuration
pub struct ReleaseOptions {
    #[serde(default)]
    /// The folder name of the project
    /// Default: `prefix`
    folder: Option<String>,
    #[serde(default)]
    /// Should the PBOs be signed?
    /// Default: true
    sign: Option<bool>,
    #[serde(default)]
    /// Create an archive of the release
    /// Default: true
    archive: Option<bool>,
}

impl ReleaseOptions {
    #[must_use]
    /// The folder name of the project
    pub fn folder(&self) -> Option<String> {
        self.folder.clone()
    }

    #[must_use]
    /// Should the PBOs be signed?
    pub const fn sign(&self) -> bool {
        if let Some(sign) = self.sign {
            sign
        } else {
            true
        }
    }

    #[must_use]
    /// Create an archive of the release
    pub const fn archive(&self) -> bool {
        if let Some(archive) = self.archive {
            archive
        } else {
            true
        }
    }
}
