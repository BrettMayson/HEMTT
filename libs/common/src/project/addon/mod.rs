use std::{collections::HashMap, path::Path, str::FromStr};

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::error::Error;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Configuration for an addon
pub struct AddonConfig {
    #[serde(default)]
    #[serde(alias = "preprocess")]
    /// Rapifier config
    rapify: RapifyConfig,

    #[serde(default)]
    /// A list of files to skip binarizing
    no_bin: Vec<String>,

    #[serde(default)]
    /// Binarze config
    binarize: BinarizeConfig,

    #[serde(default)]
    /// Properties to add to the pbo
    properties: HashMap<String, String>,

    #[serde(default)]
    /// Files to exclude from the pbo
    /// Supports glob patterns
    exclude: Vec<String>,
    #[serde(default)]
    /// Files to exclude from the pbo
    files: FilesConfig,
}

impl AddonConfig {
    #[must_use]
    /// Rapify config
    pub const fn rapify(&self) -> &RapifyConfig {
        &self.rapify
    }

    #[must_use]
    /// Binirize config
    pub fn binarize(&self) -> BinarizeConfig {
        let mut config = self.binarize.clone();
        config.exclude.append(&mut self.no_bin.clone());
        config
    }

    /// Load a configuration from a file.
    ///
    /// # Errors
    ///
    /// If the file cannot be read, or if the file is not valid TOML, or if the
    /// file does not contain a valid configuration, an error is returned.
    pub fn from_file(path: &Path) -> Result<Self, Error> {
        let file = std::fs::read_to_string(path)?;
        if file.contains("[preprocess]") {
            warn!("`[preprocess]` is deprecated, use `[rapify]` instead. See <https://brettmayson.github.io/HEMTT/configuration/addon> for more information.");
        }
        let config = Self::from_str(&file);
        if let Ok(inner) = config.as_ref() {
            if !inner.exclude.is_empty() {
                warn!("`exclude` is deprecated, use `files.exclude` instead. See <https://brettmayson.github.io/HEMTT/configuration/addon> for more information.");
            }
            if !inner.no_bin.is_empty() {
                warn!("`no_bin` is deprecated, use `binarize.exclude` instead. See <https://brettmayson.github.io/HEMTT/configuration/addon> for more information.");
            }
        }
        config
    }

    #[must_use]
    /// Properties to be added to the built PBO
    pub const fn properties(&self) -> &HashMap<String, String> {
        &self.properties
    }

    #[must_use]
    /// Files to exclude from the PBO
    pub fn files(&self) -> FilesConfig {
        let mut config = self.files.clone();
        config.exclude.append(&mut self.exclude.clone());
        config
    }
}

impl FromStr for AddonConfig {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s).map_err(Error::from)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// Preprocess config
pub struct RapifyConfig {
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    exclude: Vec<String>,
}

impl RapifyConfig {
    #[must_use]
    /// Is preprocess enabled
    pub fn enabled(&self) -> bool {
        self.enabled.unwrap_or(true)
    }

    #[must_use]
    /// Files to exclude from preprocess
    pub const fn exclude(&self) -> &Vec<String> {
        &self.exclude
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// Binarize config
pub struct BinarizeConfig {
    #[serde(default)]
    /// Is binarize enabled
    pub enabled: Option<bool>,
    #[serde(default)]
    /// Files to exclude from binarize
    pub exclude: Vec<String>,
}

impl BinarizeConfig {
    #[must_use]
    /// Is binarize enabled
    pub fn enabled(&self) -> bool {
        self.enabled.unwrap_or(true)
    }

    #[must_use]
    /// Files to exclude from binarize
    pub const fn exclude(&self) -> &Vec<String> {
        &self.exclude
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// Files config
pub struct FilesConfig {
    #[serde(default)]
    /// Files to exclude from the PBO
    pub exclude: Vec<String>,
}

impl FilesConfig {
    #[must_use]
    /// Files to exclude from the PBO
    pub const fn exclude(&self) -> &Vec<String> {
        &self.exclude
    }
}
