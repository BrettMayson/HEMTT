use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    str::FromStr,
};

use hemtt_bin_error::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    #[serde(default)]
    /// Whether to preprocess config.cpp
    preprocess: Option<bool>,

    #[serde(default)]
    /// A list of files to skip binarizing
    no_bin: Vec<String>,

    #[serde(default)]
    /// Properties to add to the pbo
    properties: HashMap<String, String>,

    #[serde(default)]
    /// Files to exclude from the pbo
    /// Supports glob patterns
    exclude: Vec<String>,
}

impl Configuration {
    #[must_use]
    pub const fn preprocess(&self) -> bool {
        if let Some(preprocess) = self.preprocess {
            preprocess
        } else {
            true
        }
    }

    /// A list of files to skip binarizing
    ///
    /// # Errors
    ///
    /// [`Error::GlobError`] if a glob pattern is invalid
    pub fn no_bin(&self, root: &str) -> Result<Vec<PathBuf>, Error> {
        self.no_bin
            .iter()
            .map(|f| glob::glob(format!("{root}/{f}").as_str()))
            .collect::<Result<Vec<_>, glob::PatternError>>()?
            .into_iter()
            .flatten()
            .collect::<Result<Vec<_>, _>>()
            .map_err(Error::from)
    }

    /// Load a configuration from a file.
    ///
    /// # Errors
    ///
    /// If the file cannot be read, or if the file is not valid TOML, or if the
    /// file does not contain a valid configuration, an error is returned.
    pub fn from_file(path: &Path) -> Result<Self, Error> {
        let file = std::fs::read_to_string(path)?;
        Self::from_str(&file)
    }

    #[must_use]
    /// Properties to be added to the built PBO
    pub const fn properties(&self) -> &HashMap<String, String> {
        &self.properties
    }

    #[must_use]
    /// Files to be excluded from the built PBO
    pub const fn exclude(&self) -> &Vec<String> {
        &self.exclude
    }
}

impl FromStr for Configuration {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s).map_err(Error::from)
    }
}
