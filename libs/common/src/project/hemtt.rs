//! HEMTT Configuration

use std::collections::HashMap;

use crate::arma::dlc::DLC;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
/// Feature specific configuration
pub struct Features {
    #[serde(default)]
    dev: DevOptions,

    #[serde(default)]
    check: CheckOptions,

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
    /// Check options
    pub const fn check(&self) -> &CheckOptions {
        &self.check
    }

    #[must_use]
    /// Get launch options by key
    pub fn launch(&self, key: &str) -> Option<LaunchOptions> {
        let config = self.launch.get(key);
        config.and_then(|config| {
            config.extends().map_or_else(
                || Some(config.clone()),
                |extends| {
                    let mut base = self.launch(extends).unwrap_or_default();
                    base.workshop.extend(config.workshop.clone());
                    base.dlc.extend(config.dlc.clone());
                    base.presets.extend(config.presets.clone());
                    base.optionals.extend(config.optionals.clone());
                    base.parameters.extend(config.parameters.clone());
                    if let Some(executable) = &config.executable {
                        base.executable = Some(executable.clone());
                    }
                    if let Some(mission) = &config.mission {
                        base.mission = Some(mission.clone());
                    }
                    Some(base)
                },
            )
        })
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
/// Dev specific configuration
pub struct CheckOptions {
    #[serde(default)]
    /// Can includes come from the P drive?
    /// Default: false
    pdrive: PDriveOption,
}

impl CheckOptions {
    #[must_use]
    /// Can includes come from the P drive?
    pub const fn pdrive(&self) -> &PDriveOption {
        &self.pdrive
    }
}

#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
/// Launch specific configuration
pub struct LaunchOptions {
    #[serde(default)]
    extends: Option<String>,

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
    /// Overlay another launch config
    pub fn overlay(&self, other: &Self) -> Self {
        let mut base = self.clone();
        base.workshop.extend(other.workshop.clone());
        base.dlc.extend(other.dlc.clone());
        base.presets.extend(other.presets.clone());
        base.optionals.extend(other.optionals.clone());
        base.parameters.extend(other.parameters.clone());
        if let Some(executable) = &other.executable {
            base.executable = Some(executable.clone());
        }
        if let Some(mission) = &other.mission {
            base.mission = Some(mission.clone());
        }
        base
    }

    pub fn dedup(&mut self) {
        self.workshop.sort();
        self.workshop.dedup();
        self.dlc.sort();
        self.dlc.dedup();
        self.presets.sort();
        self.presets.dedup();
        self.optionals.sort();
        self.optionals.dedup();
        self.parameters.sort();
        self.parameters.dedup();
    }

    #[must_use]
    /// Preset to extend
    pub const fn extends(&self) -> Option<&String> {
        self.extends.as_ref()
    }

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
    pdrive: PDriveOption,
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
    pub const fn pdrive(&self) -> &PDriveOption {
        &self.pdrive
    }
}

#[derive(Default, PartialEq, Eq, Debug, Copy, Clone)]
pub enum PDriveOption {
    Disallow,
    #[default]
    Ignore,
    Require,
}

impl<'de> Deserialize<'de> for PDriveOption {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "disallow" => Ok(Self::Disallow),
            "require" => Ok(Self::Require),
            "ignore" => Ok(Self::Ignore),
            _ => Err(serde::de::Error::custom(
                "valid values are disallow, ignore, require",
            )),
        }
    }
}

impl Serialize for PDriveOption {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Disallow => serializer.serialize_str("disallow"),
            Self::Ignore => serializer.serialize_str("ignore"),
            Self::Require => serializer.serialize_str("required"),
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
