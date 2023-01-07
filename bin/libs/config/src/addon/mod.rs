use std::{
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
}

impl FromStr for Configuration {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s).map_err(Error::from)
    }
}
