use std::{collections::HashMap, path::Path, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    #[serde(default)]
    /// Preprocess config
    preprocess: Option<PreprocessCompatibility>,

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

impl Configuration {
    #[must_use]
    pub fn preprocess(&self) -> PreprocessConfig {
        match &self.preprocess {
            Some(PreprocessCompatibility::Deprecated(enabled)) => PreprocessConfig {
                enabled: *enabled,
                exclude: Vec::new(),
            },
            Some(PreprocessCompatibility::New(config)) => config.clone(),
            None => PreprocessConfig {
                enabled: false,
                exclude: Vec::new(),
            },
        }
    }

    #[must_use]
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
        let config = Self::from_str(&file);
        if let Ok(inner) = config.as_ref() {
            if !inner.exclude.is_empty() {
                warn!("`exclude` is deprecated, use `files.exclude` instead. See <https://brettmayson.github.io/HEMTT/configuration/addon> for more information.");
            }
            if !inner.no_bin.is_empty() {
                warn!("`no_bin` is deprecated, use `binarize.exclude` instead. See <https://brettmayson.github.io/HEMTT/configuration/addon> for more information.");
            }
            if let Some(PreprocessCompatibility::Deprecated(_)) = inner.preprocess {
                warn!("`preprocess` as a field is deprecated, use a `preprocess` object instead. See <https://brettmayson.github.io/HEMTT/configuration/addon> for more information.");
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

impl FromStr for Configuration {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s).map_err(Error::from)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PreprocessCompatibility {
    Deprecated(bool),
    New(PreprocessConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreprocessConfig {
    enabled: bool,
    exclude: Vec<String>,
}

impl PreprocessConfig {
    #[must_use]
    pub const fn enabled(&self) -> bool {
        self.enabled
    }

    #[must_use]
    pub const fn exclude(&self) -> &Vec<String> {
        &self.exclude
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BinarizeConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub exclude: Vec<String>,
}

impl BinarizeConfig {
    #[must_use]
    pub const fn enabled(&self) -> bool {
        self.enabled
    }

    #[must_use]
    pub const fn exclude(&self) -> &Vec<String> {
        &self.exclude
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FilesConfig {
    #[serde(default)]
    pub exclude: Vec<String>,
}

impl FilesConfig {
    #[must_use]
    pub const fn exclude(&self) -> &Vec<String> {
        &self.exclude
    }
}
